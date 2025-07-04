use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use estate_fe::view_state_layer::AppState;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{debug, error, info, instrument, warn};

use estate_fe::api::consts::{ConfigLoader, EnvVarConfig};
use estate_fe::ssr_booking::email_handler::EmailClient;
use estate_fe::utils::{app_reference::BookingId, otp_storage::GLOBAL_OTP_STORAGE};

use super::parse_json_request;

#[derive(Serialize, Deserialize, Debug)]
pub struct SendOtpRequest {
    pub email: String,
    pub booking_id: String, // BookingId.to_order_id() format
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SendOtpResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VerifyOtpRequest {
    pub booking_id: String,
    pub otp: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VerifyOtpResponse {
    pub success: bool,
    pub message: String,
}

#[axum::debug_handler]
#[instrument(skip(_state))]
pub async fn send_otp_email_api_server_fn_route(
    State(_state): State<AppState>,
    body: String,
) -> Result<Response, Response> {
    info!(
        "Starting send OTP email API with body: {}",
        &body[0..100.min(body.len())]
    );

    // Parse the JSON request
    let request: SendOtpRequest = parse_json_request(&body)?;

    info!(
        "Processing OTP send request - Email: {}, Booking ID: {}",
        request.email, request.booking_id
    );

    // Validate booking_id format
    if let None = BookingId::from_order_id(&request.booking_id) {
        error!("Invalid booking_id format: {}", request.booking_id);
        let error_response = json!({
            "success": false,
            "message": "Invalid booking ID format",
        });
        return Ok((StatusCode::BAD_REQUEST, error_response.to_string()).into_response());
    }

    // Validate email format (basic validation)
    if !request.email.contains('@') || request.email.is_empty() {
        error!("Invalid email format: {}", request.email);
        let error_response = json!({
            "success": false,
            "message": "Invalid email format",
        });
        return Ok((StatusCode::BAD_REQUEST, error_response.to_string()).into_response());
    }

    // Generate 6-digit OTP
    let otp = estate_fe::utils::otp_storage::OtpStorage::generate_6_digit_otp();
    debug!(
        "Generated OTP: {} for booking_id: {}",
        otp, request.booking_id
    );

    // Store OTP in global storage
    match GLOBAL_OTP_STORAGE.store_otp(&request.booking_id, otp.clone()) {
        Ok(_) => {
            info!(
                "OTP stored successfully for booking_id: {}",
                request.booking_id
            );
        }
        Err(e) => {
            error!("Failed to store OTP: {}", e);
            let error_response = json!({
                "success": false,
                "message": "Failed to store verification code",
            });
            return Ok((
                StatusCode::INTERNAL_SERVER_ERROR,
                error_response.to_string(),
            )
                .into_response());
        }
    }

    // Send OTP email
    let config = EnvVarConfig::try_from_env();
    let email_client = EmailClient::new(config.email_client_config);

    match email_client
        .send_otp_email(&request.email, &otp, &request.booking_id)
        .await
    {
        Ok(_) => {
            info!("OTP email sent successfully to: {}", request.email);
            let success_response = SendOtpResponse {
                success: true,
                message: "Verification code sent to your email".to_string(),
            };
            Ok((
                StatusCode::OK,
                serde_json::to_string(&success_response).unwrap(),
            )
                .into_response())
        }
        Err(e) => {
            error!("Failed to send OTP email: {}", e);
            // Remove the stored OTP since email sending failed
            if let Err(remove_err) =
                GLOBAL_OTP_STORAGE.verify_otp(&request.booking_id, "invalid_otp_to_remove")
            {
                warn!(
                    "Failed to remove OTP after email send failure: {}",
                    remove_err
                );
            }

            let error_response = SendOtpResponse {
                success: false,
                message: "Failed to send verification email. Please try again.".to_string(),
            };
            Ok((
                StatusCode::INTERNAL_SERVER_ERROR,
                serde_json::to_string(&error_response).unwrap(),
            )
                .into_response())
        }
    }
}

#[axum::debug_handler]
#[instrument(skip(_state))]
pub async fn verify_otp_api_server_fn_route(
    State(_state): State<AppState>,
    body: String,
) -> Result<Response, Response> {
    info!(
        "Starting verify OTP API with body: {}",
        &body[0..50.min(body.len())] // Limited for security (don't log full OTP)
    );

    // Parse the JSON request
    let request: VerifyOtpRequest = parse_json_request(&body)?;

    info!(
        "Processing OTP verification - Booking ID: {}, OTP length: {}",
        request.booking_id,
        request.otp.len()
    );

    // Validate OTP format (6 digits)
    if request.otp.len() != 6 || !request.otp.chars().all(|c| c.is_ascii_digit()) {
        warn!(
            "Invalid OTP format provided for booking_id: {}",
            request.booking_id
        );
        let error_response = VerifyOtpResponse {
            success: false,
            message: "Invalid verification code format. Please enter 6 digits.".to_string(),
        };
        return Ok((
            StatusCode::BAD_REQUEST,
            serde_json::to_string(&error_response).unwrap(),
        )
            .into_response());
    }

    // Validate booking_id format
    if let None = BookingId::from_order_id(&request.booking_id) {
        error!("Invalid booking_id format: {}", request.booking_id);
        let error_response = VerifyOtpResponse {
            success: false,
            message: "Invalid booking ID format".to_string(),
        };
        return Ok((
            StatusCode::BAD_REQUEST,
            serde_json::to_string(&error_response).unwrap(),
        )
            .into_response());
    }

    // Verify OTP
    match GLOBAL_OTP_STORAGE.verify_otp(&request.booking_id, &request.otp) {
        Ok(is_valid) => {
            if is_valid {
                info!(
                    "OTP verification successful for booking_id: {}",
                    request.booking_id
                );
                let success_response = VerifyOtpResponse {
                    success: true,
                    message: "Email verified successfully".to_string(),
                };
                Ok((
                    StatusCode::OK,
                    serde_json::to_string(&success_response).unwrap(),
                )
                    .into_response())
            } else {
                warn!(
                    "OTP verification failed for booking_id: {}",
                    request.booking_id
                );
                let error_response = VerifyOtpResponse {
                    success: false,
                    message: "Invalid or expired verification code".to_string(),
                };
                Ok((
                    StatusCode::BAD_REQUEST,
                    serde_json::to_string(&error_response).unwrap(),
                )
                    .into_response())
            }
        }
        Err(e) => {
            error!("OTP verification error: {}", e);
            let error_response = VerifyOtpResponse {
                success: false,
                message: "Verification failed. Please try again.".to_string(),
            };
            Ok((
                StatusCode::INTERNAL_SERVER_ERROR,
                serde_json::to_string(&error_response).unwrap(),
            )
                .into_response())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::app_reference::BookingId;

    fn create_test_booking_id() -> String {
        let booking = BookingId {
            app_reference: "HB-123".to_string(),
            email: "test@example.com".to_string(),
        };
        booking.to_order_id()
    }

    #[test]
    fn test_send_otp_request_serialization() {
        let request = SendOtpRequest {
            email: "test@example.com".to_string(),
            booking_id: create_test_booking_id(),
        };

        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: SendOtpRequest = serde_json::from_str(&serialized).unwrap();

        assert_eq!(request.email, deserialized.email);
        assert_eq!(request.booking_id, deserialized.booking_id);
    }

    #[test]
    fn test_verify_otp_request_serialization() {
        let request = VerifyOtpRequest {
            booking_id: create_test_booking_id(),
            otp: "123456".to_string(),
        };

        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: VerifyOtpRequest = serde_json::from_str(&serialized).unwrap();

        assert_eq!(request.booking_id, deserialized.booking_id);
        assert_eq!(request.otp, deserialized.otp);
    }

    #[test]
    fn test_send_otp_response_serialization() {
        let response = SendOtpResponse {
            success: true,
            message: "Code sent".to_string(),
        };

        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: SendOtpResponse = serde_json::from_str(&serialized).unwrap();

        assert_eq!(response.success, deserialized.success);
        assert_eq!(response.message, deserialized.message);
    }

    #[test]
    fn test_verify_otp_response_serialization() {
        let response = VerifyOtpResponse {
            success: false,
            message: "Invalid code".to_string(),
        };

        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: VerifyOtpResponse = serde_json::from_str(&serialized).unwrap();

        assert_eq!(response.success, deserialized.success);
        assert_eq!(response.message, deserialized.message);
    }
}
