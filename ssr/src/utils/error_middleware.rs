use crate::api::auth::types::OidcUser;
use crate::utils::geoip_service::{extract_client_ip, extract_user_agent, lookup_ip};
use crate::view_state_layer::AppState;
use axum::{
    body::{to_bytes, Body, Bytes},
    extract::{ConnectInfo, State},
    http::Request,
    middleware::Next,
    response::Response,
};
use axum_extra::extract::cookie::Key;
use std::net::SocketAddr;
use tower_cookies::cookie::{Cookie as RawCookie, CookieJar};

const SESSION_COOKIE: &str = "session";

/// Extract user email from session cookie
fn extract_user_email_from_request(headers: &http::HeaderMap, key: &Key) -> Option<String> {
    // Get the Cookie header
    let cookie_header = headers.get(http::header::COOKIE)?;
    let cookie_str = cookie_header.to_str().ok()?;

    // Parse cookies and reconstruct the jar
    let mut jar = CookieJar::new();
    for cookie_str in cookie_str.split(';') {
        if let Ok(cookie) = RawCookie::parse(cookie_str.trim().to_string()) {
            jar.add_original(cookie);
        }
    }

    // Get the signed session cookie (key is compatible with cookie crate's Key)
    // Note: axum_extra::extract::cookie::Key re-exports cookie::Key
    if let Some(signed_cookie) = jar.signed(key).get(SESSION_COOKIE) {
        // Parse the JSON value to get user info
        if let Ok(user) = serde_json::from_str::<OidcUser>(signed_cookie.value()) {
            return user.email;
        }
    }

    None
}

/// Middleware to intercept responses and report 5xx/429 errors with request/response data
pub async fn error_alert_middleware(
    State(state): State<AppState>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let headers = request.headers().clone();

    // Extract user email from session cookie
    let user_email = extract_user_email_from_request(&headers, &state.cookie_key);

    // Extract client info from headers first (proxy headers take precedence)
    // Fall back to direct connection IP if no proxy headers
    let client_ip =
        extract_client_ip(&headers).or_else(|| connect_info.map(|ci| ci.0.ip().to_string()));
    let user_agent = extract_user_agent(&headers);

    // Look up location from IP (won't work for localhost/private IPs)
    let location = client_ip
        .as_ref()
        .and_then(|ip| lookup_ip(ip))
        .map(|loc| loc.to_string());

    // Capture request body (limited to 10KB to avoid memory issues)
    let (parts, body) = request.into_parts();
    let request_body_bytes = match to_bytes(body, 10 * 1024).await {
        Ok(bytes) => bytes,
        Err(_) => Bytes::new(),
    };
    let request_body_string = String::from_utf8_lossy(&request_body_bytes).to_string();

    // Reconstruct the request with the body
    let request = Request::from_parts(parts, Body::from(request_body_bytes));

    // Run the handler
    let response = next.run(request).await;

    // Check for server errors (5xx) or rate limiting (429)
    let status = response.status();
    let should_alert = status.is_server_error() || status.as_u16() == 429;

    if should_alert {
        // Capture response body (limited to 10KB)
        let (parts, body) = response.into_parts();
        let response_body_bytes = match to_bytes(body, 10 * 1024).await {
            Ok(bytes) => bytes,
            Err(_) => Bytes::new(),
        };
        let response_body_string = String::from_utf8_lossy(&response_body_bytes).to_string();

        // Reconstruct the response
        let response = Response::from_parts(parts, Body::from(response_body_bytes));

        // Capture backtrace
        let backtrace = std::backtrace::Backtrace::capture();
        let stack_trace = format!("{}", backtrace);

        // Clone data for the async task
        let request_body = request_body_string.clone();
        let response_body = response_body_string.clone();

        // We spawn a task to avoid blocking the response
        tokio::spawn(async move {
            use crate::utils::error_alerts::{CriticalError, ErrorType};

            let service = state.error_alert_service;

            let error_type_label = if status.as_u16() == 429 {
                "Rate Limited (429)"
            } else {
                "Server Error"
            };
            let error_msg = format!(
                "{}: HTTP {} for {} {}",
                error_type_label, status, method, uri
            );

            // Truncate bodies for display
            let req_body_display = if request_body.len() > 500 {
                format!("{}...[truncated]", &request_body[..500])
            } else {
                request_body.clone()
            };

            let resp_body_display = if response_body.len() > 500 {
                format!("{}...[truncated]", &response_body[..500])
            } else {
                response_body.clone()
            };

            let mut error = CriticalError::new(
                ErrorType::Http500 {
                    status_code: status.as_u16(),
                    response_body: Some(resp_body_display),
                },
                &error_msg,
            )
            .with_request(&method.to_string(), &uri.to_string())
            .with_request_body(&req_body_display)
            .with_client_info(client_ip, user_agent, location);

            // Add user context if available (from session cookie)
            if let Some(ref email) = user_email {
                error = error.with_user(email);
            }

            // Add stack trace if available (requires RUST_BACKTRACE=1)
            if !stack_trace.is_empty() && !stack_trace.contains("disabled backtrace") {
                error = error.with_stack_trace(&stack_trace);
            }

            service.report(error).await;
        });

        return response;
    }

    response
}
