use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use estate_fe::{
    api::{
        client_side_api::{
            CheckPaymentStatusRequest, GetBackendBookingRequest, UpdatePaymentRequest,
        },
        payments::ports::{GetPaymentStatusRequest, GetPaymentStatusResponse},
    },
    canister::backend::*,
    ssr_booking::payment_handler::nowpayments_get_payment_status,
    utils::admin::{admin_canister, AdminCanisters},
    view_state_layer::AppState,
};
use serde_json::json;
use tracing::{error, info, instrument};

use crate::basic_auth::validate_basic_auth_from_headers;

#[instrument(name = "check_payment_status_handler", skip(state))]
pub async fn check_payment_status(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CheckPaymentStatusRequest>,
) -> Response {
    // Validate authentication
    if let Err(status_code) = validate_basic_auth_from_headers(&headers, &state) {
        return (
            status_code,
            Json(json!({
                "error": "Authentication required",
                "details": "Invalid or missing basic authentication credentials"
            })),
        )
            .into_response();
    }
    info!(
        "Checking payment status for payment_id: {}",
        request.payment_id
    );

    let payment_request = GetPaymentStatusRequest {
        payment_id: request.payment_id,
    };

    match nowpayments_get_payment_status(payment_request).await {
        Ok(response) => {
            info!("Payment status check successful");
            Json(response).into_response()
        }
        Err(e) => {
            error!("Failed to check payment status: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to check payment status",
                    "details": e.to_string()
                })),
            )
                .into_response()
        }
    }
}

#[instrument(name = "get_backend_booking_handler", skip(state, headers))]
pub async fn get_backend_booking(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<GetBackendBookingRequest>,
) -> Response {
    // Validate authentication
    if let Err(status_code) = validate_basic_auth_from_headers(&headers, &state) {
        return (
            status_code,
            Json(json!({
                "error": "Authentication required",
                "details": "Invalid or missing basic authentication credentials"
            })),
        )
            .into_response();
    }
    info!(
        "Getting backend booking for email: {}, app_reference: {}",
        request.email, request.app_reference
    );

    let booking_id = BookingId {
        email: request.email,
        app_reference: request.app_reference,
    };

    let admin_canister = AdminCanisters::from_env();
    let backend = admin_canister.backend_canister().await;

    match backend.get_booking_by_id(booking_id.clone()).await {
        Ok(booking_option) => {
            info!(
                "Backend booking retrieval successful with booking: {:#?}",
                booking_option
            );
            Json(booking_option).into_response()
        }
        Err(e) => {
            error!(
                "Failed to get backend booking: for booking_id: {:#?}, error: {:#?}",
                &booking_id, e
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to get backend booking",
                    "details": e.to_string()
                })),
            )
                .into_response()
        }
    }
}

#[instrument(name = "update_payment_details_handler", skip(state))]
pub async fn update_payment_details(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<UpdatePaymentRequest>,
) -> Response {
    // Validate authentication
    if let Err(status_code) = validate_basic_auth_from_headers(&headers, &state) {
        return (
            status_code,
            Json(json!({
                "error": "Authentication required",
                "details": "Invalid or missing basic authentication credentials"
            })),
        )
            .into_response();
    }
    info!(
        "Updating payment details for email: {}, app_reference: {}",
        request.email, request.app_reference
    );

    let booking_id = BookingId {
        email: request.email,
        app_reference: request.app_reference,
    };

    let admin_canister = AdminCanisters::from_env();
    let backend = admin_canister.backend_canister().await;

    match backend
        .update_payment_details(booking_id, request.payment_details)
        .await
    {
        Ok(Result4::Ok(_booking)) => {
            info!("Payment details updated successfully");
            Json("Payment details updated successfully".to_string()).into_response()
        }
        Ok(Result4::Err(e)) => {
            error!("Backend error updating payment details: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "Backend error",
                    "details": e
                })),
            )
                .into_response()
        }
        Err(e) => {
            error!("Failed to update payment details: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to update payment details",
                    "details": e.to_string()
                })),
            )
                .into_response()
        }
    }
}
