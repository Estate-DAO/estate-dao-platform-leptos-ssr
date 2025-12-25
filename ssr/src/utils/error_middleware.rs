use crate::view_state_layer::AppState;
use axum::{body::Body, extract::State, http::Request, middleware::Next, response::Response};

/// Middleware to intercept responses and report 5xx errors
pub async fn error_alert_middleware(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();

    // Run the handler
    let response = next.run(request).await;

    // Check for server errors (5xx) or rate limiting (429)
    let status = response.status();
    let should_alert = status.is_server_error() || status.as_u16() == 429;

    if should_alert {
        // Capture backtrace at this point
        // NOTE: This captures the middleware stack, not the original error.
        // Full stack traces require RUST_BACKTRACE=1 environment variable.
        let backtrace = std::backtrace::Backtrace::capture();
        let stack_trace = format!("{}", backtrace);

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

            let mut error = CriticalError::new(
                ErrorType::Http500 {
                    status_code: status.as_u16(),
                    response_body: None, // Can't read body without consuming it
                },
                &error_msg,
            )
            .with_request(&method.to_string(), &uri.to_string());

            // Add stack trace if available (requires RUST_BACKTRACE=1)
            if !stack_trace.is_empty() && !stack_trace.contains("disabled backtrace") {
                error = error.with_stack_trace(&stack_trace);
            }

            service.report(error).await;
        });
    }

    response
}
