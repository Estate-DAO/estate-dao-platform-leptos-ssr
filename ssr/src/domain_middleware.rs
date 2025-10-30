use axum::{
    extract::Request,
    http::{header, StatusCode, Uri},
    middleware::Next,
    response::{IntoResponse, Redirect},
};

/// **Domain Normalization Middleware**
///
/// **Purpose**: Ensures consistent subdomain usage by redirecting www.domain.com to domain.com
/// **Benefits**:
/// - Fixes cookie domain issues between subdomains
/// - Ensures consistent payment redirect flow
/// - Preserves all query parameters and paths
/// - Works for all routes automatically
/// - Environment-aware (production, staging, localhost)

pub async fn domain_normalization_middleware(request: Request, next: Next) -> impl IntoResponse {
    let host = request
        .headers()
        .get("host")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    // Extract canonical domain and check if we need to redirect
    if let Some(canonical_url) = should_redirect_to_canonical(host, request.uri()) {
        tracing::info!(
            "Domain normalization: Redirecting {} to {}",
            format!(
                "https://{}{}",
                host,
                request
                    .uri()
                    .path_and_query()
                    .map(|pq| pq.as_str())
                    .unwrap_or("/")
            ),
            canonical_url
        );

        // Use 301 permanent redirect to indicate canonical domain
        return (
            StatusCode::MOVED_PERMANENTLY,
            [(header::LOCATION, canonical_url)],
        )
            .into_response();
    }

    // Continue with normal processing for canonical domain
    next.run(request).await
}

/// Check if the current host should be redirected to canonical domain
/// Returns Some(canonical_url) if redirect is needed, None otherwise
fn should_redirect_to_canonical(host: &str, uri: &Uri) -> Option<String> {
    // Skip redirection for localhost and IP addresses
    if host.starts_with("localhost") || host.starts_with("127.0.0.1") || host.starts_with("0.0.0.0")
    {
        return None;
    }

    // Check if host starts with "www."
    if let Some(canonical_domain) = host.strip_prefix("www.") {
        let path_and_query = uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("/");

        // Determine protocol (assume https for non-localhost)
        let protocol = if canonical_domain.contains("localhost") {
            "http"
        } else {
            "https"
        };

        return Some(format!(
            "{}://{}{}",
            protocol, canonical_domain, path_and_query
        ));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_www_redirect_production() {
        let app = axum::Router::new()
            .route("/", axum::routing::get(|| async { "OK" }))
            .layer(axum::middleware::from_fn(domain_normalization_middleware));

        let request = Request::builder()
            .uri("https://www.nofeebooking.com/confirmation?session_id=test123")
            .header("host", "www.nofeebooking.com")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::MOVED_PERMANENTLY);
        assert_eq!(
            response.headers().get("location").unwrap(),
            "https://nofeebooking.com/confirmation?session_id=test123"
        );
    }

    #[tokio::test]
    async fn test_www_redirect_staging() {
        let app = axum::Router::new()
            .route("/", axum::routing::get(|| async { "OK" }))
            .layer(axum::middleware::from_fn(domain_normalization_middleware));

        let request = Request::builder()
            .uri("https://www.staging.example.com/test?param=value")
            .header("host", "www.staging.example.com")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::MOVED_PERMANENTLY);
        assert_eq!(
            response.headers().get("location").unwrap(),
            "https://staging.example.com/test?param=value"
        );
    }

    #[tokio::test]
    async fn test_canonical_domain_passthrough() {
        let app = axum::Router::new()
            .route("/", axum::routing::get(|| async { "OK" }))
            .layer(axum::middleware::from_fn(domain_normalization_middleware));

        let request = Request::builder()
            .uri("https://nofeebooking.com/confirmation?session_id=test123")
            .header("host", "nofeebooking.com")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_localhost_no_redirect() {
        let app = axum::Router::new()
            .route("/", axum::routing::get(|| async { "OK" }))
            .layer(axum::middleware::from_fn(domain_normalization_middleware));

        let request = Request::builder()
            .uri("http://localhost:3002/test")
            .header("host", "localhost:3002")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Localhost should not be redirected
        assert_eq!(response.status(), StatusCode::OK);
    }
}
