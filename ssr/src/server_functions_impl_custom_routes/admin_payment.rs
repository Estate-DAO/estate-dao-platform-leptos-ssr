use axum::{
    extract::State,
    http::StatusCode,
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
    canister::backend::{Booking, BookingId, PaymentDetails, Result3},
    ssr_booking::payment_handler::nowpayments_get_payment_status,
    utils::admin::{admin_canister, AdminCanisters},
    view_state_layer::AppState,
};
use serde_json::json;
use tracing::{error, info, instrument};

#[instrument(name = "check_payment_status_handler", skip(state))]
pub async fn check_payment_status(
    State(state): State<AppState>,
    Json(request): Json<CheckPaymentStatusRequest>,
) -> Response {
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

#[instrument(name = "get_backend_booking_handler", skip(state))]
pub async fn get_backend_booking(
    State(state): State<AppState>,
    Json(request): Json<GetBackendBookingRequest>,
) -> Response {
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

    match backend.get_booking_by_id(booking_id).await {
        Ok(booking_option) => {
            info!("Backend booking retrieval successful");
            Json(booking_option).into_response()
        }
        Err(e) => {
            error!("Failed to get backend booking: {}", e);
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
    Json(request): Json<UpdatePaymentRequest>,
) -> Response {
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
        Ok(Result3::Ok(_booking)) => {
            info!("Payment details updated successfully");
            Json("Payment details updated successfully".to_string()).into_response()
        }
        Ok(Result3::Err(e)) => {
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
