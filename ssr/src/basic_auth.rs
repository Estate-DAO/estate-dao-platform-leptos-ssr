use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use base64::{engine::general_purpose, Engine as _};
use tracing::warn;

use crate::AppState; // Adjust import path as needed

/// Basic authentication middleware
///
/// Usage:
/// ```rust
/// let protected_routes = Router::new()
///     .route("/admin", get(admin_handler))
///     .layer(middleware::from_fn_with_state(state.clone(), basic_auth_middleware));
/// ```
pub async fn basic_auth_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let headers = request.headers();

    // Check if Authorization header exists
    let auth_header = match headers.get(AUTHORIZATION) {
        Some(header) => header,
        None => {
            warn!("Missing Authorization header");
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    // Convert header to string
    let auth_str = match auth_header.to_str() {
        Ok(s) => s,
        Err(_) => {
            warn!("Invalid Authorization header format");
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    // Check if it starts with "Basic "
    if !auth_str.starts_with("Basic ") {
        warn!("Authorization header is not Basic auth");
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Extract and decode the base64 part
    let encoded_credentials = &auth_str[6..]; // Skip "Basic "
    let decoded_bytes = match general_purpose::STANDARD.decode(encoded_credentials) {
        Ok(bytes) => bytes,
        Err(_) => {
            warn!("Failed to decode base64 credentials");
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    // Convert to string
    let credentials_str = match String::from_utf8(decoded_bytes) {
        Ok(s) => s,
        Err(_) => {
            warn!("Invalid UTF-8 in credentials");
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    // Split username:password
    let mut parts = credentials_str.splitn(2, ':');
    let username = match parts.next() {
        Some(u) => u,
        None => {
            warn!("Missing username in credentials");
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    let password = match parts.next() {
        Some(p) => p,
        None => {
            warn!("Missing password in credentials");
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    // Validate credentials against environment config
    if username != state.env_var_config.basic_auth_username
        || password != state.env_var_config.basic_auth_password
    {
        warn!("Invalid credentials provided");
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Authentication successful, continue to next middleware/handler
    let response = next.run(request).await;
    Ok(response)
}

/// Helper function to create WWW-Authenticate header for 401 responses
/// Use this if you want to prompt the browser for basic auth dialog
pub async fn basic_auth_middleware_with_challenge(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    match basic_auth_middleware(State(state), request, next).await {
        Ok(response) => response,
        Err(status_code) => {
            let mut response = Response::new("Unauthorized".into());
            *response.status_mut() = status_code;
            response.headers_mut().insert(
                "WWW-Authenticate",
                "Basic realm=\"Protected Area\"".parse().unwrap(),
            );
            response
        }
    }
}

// Custom middleware that only protects /admin routes
pub async fn selective_auth_middleware(
    State(state): State<AppState>,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let path = request.uri().path();

    // Only apply auth to admin routes
    if path.starts_with("/admin") {
        basic_auth_middleware_with_challenge(State(state), request, next).await
    } else {
        // Let other routes pass through
        next.run(request).await
    }
}
