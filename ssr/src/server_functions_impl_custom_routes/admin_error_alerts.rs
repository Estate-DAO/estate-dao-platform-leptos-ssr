//! Admin endpoints for error alert management
//!
//! Provides endpoints to:
//! - Send a test error email
//! - Flush pending errors immediately

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use estate_fe::utils::error_alerts::{CriticalError, ErrorType};
use estate_fe::view_state_layer::AppState;
use serde_json::json;

/// Send a test error email to verify the alert system is working
#[axum::debug_handler]
pub async fn send_test_error_alert(State(state): State<AppState>) -> Result<Response, Response> {
    tracing::info!("Admin triggered test error alert");

    // Create a test error
    let test_error = CriticalError::new(
        ErrorType::BookingProviderFailure {
            provider: "test".to_string(),
            hotel_id: Some("TEST-HOTEL-123".to_string()),
            operation: "admin_test".to_string(),
        },
        "This is a TEST error triggered from the admin panel to verify the error alert system is working correctly.",
    )
    .with_request("POST", "/server_fn_api/admin/test_error_alert")
    .with_source(
        "ssr/src/server_functions_impl_custom_routes/admin_error_alerts.rs",
        24,
        "send_test_error_alert",
    );

    // Report the error
    state.error_alert_service.report(test_error).await;

    // Also flush immediately so the test email goes out now
    match state.error_alert_service.flush().await {
        Ok(_) => {
            tracing::info!("Test error alert sent and flushed successfully");
            let response = json!({
                "success": true,
                "message": "Test error reported and email sent immediately"
            });
            Ok((StatusCode::OK, response.to_string()).into_response())
        }
        Err(e) => {
            tracing::error!("Failed to flush test error alert: {}", e);
            let response = json!({
                "success": false,
                "message": format!("Test error reported but flush failed: {}", e)
            });
            Ok((StatusCode::INTERNAL_SERVER_ERROR, response.to_string()).into_response())
        }
    }
}

/// Flush all pending errors immediately (without waiting for 5-min interval)
#[axum::debug_handler]
pub async fn flush_pending_errors(State(state): State<AppState>) -> Result<Response, Response> {
    tracing::info!("Admin triggered flush of pending errors");

    match state.error_alert_service.flush().await {
        Ok(_) => {
            tracing::info!("Pending errors flushed successfully");
            let response = json!({
                "success": true,
                "message": "Pending errors flushed and email sent (if any errors were pending)"
            });
            Ok((StatusCode::OK, response.to_string()).into_response())
        }
        Err(e) => {
            tracing::error!("Failed to flush pending errors: {}", e);
            let response = json!({
                "success": false,
                "message": format!("Failed to flush pending errors: {}", e)
            });
            Ok((StatusCode::INTERNAL_SERVER_ERROR, response.to_string()).into_response())
        }
    }
}
