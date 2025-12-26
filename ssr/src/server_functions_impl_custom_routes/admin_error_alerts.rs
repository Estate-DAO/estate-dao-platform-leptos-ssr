//! Admin endpoints for error alert management
//!
//! Provides endpoints to:
//! - Send a test error email
//! - Flush pending errors immediately

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use estate_fe::utils::error_alerts::{CriticalError, ErrorType};
use estate_fe::utils::geoip_service::{extract_client_ip, extract_user_agent, lookup_ip};
use estate_fe::view_state_layer::AppState;
use serde_json::json;

/// Send a test error email to verify the alert system is working
#[axum::debug_handler]
pub async fn send_test_error_alert(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, Response> {
    tracing::info!("Admin triggered test error alert");

    // Extract client info from headers
    let client_ip = extract_client_ip(&headers);
    let user_agent = extract_user_agent(&headers);
    let location = client_ip
        .as_ref()
        .and_then(|ip| lookup_ip(ip))
        .map(|loc| loc.to_string());

    // Create a test error with client info
    let test_error = CriticalError::new(
        ErrorType::BookingProviderFailure {
            provider: "liteapi".to_string(),
            hotel_id: Some("HTL-DEMO-456".to_string()),
            operation: "block_room".to_string(),
        },
        "Room blocking failed: Rate no longer available",
    )
    .with_request("POST", "/api/book_room")
    .with_user("customer@test.com")
    .with_client_info(client_ip, user_agent, location)
    .with_source(
        "ssr/src/application_services/booking_service.rs",
        89,
        "block_room",
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
