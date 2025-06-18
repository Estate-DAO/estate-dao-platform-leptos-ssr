use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use estate_fe::view_state_layer::AppState;
use estate_fe::{
    ssr_booking::{
        booking_handler::MakeBookingFromBookingProvider,
        email_handler::SendEmailAfterSuccessfullBooking,
        payment_handler::GetPaymentStatusFromPaymentProvider, pipeline::process_pipeline,
        SSRBookingPipelineStep, ServerSideBookingEvent,
    },
    utils::app_reference::BookingId,
};
use serde_json::json;

use super::{parse_json_request, ConfirmationProcessRequest};

#[axum::debug_handler]
pub async fn process_confirmation_api_server_fn_route(
    State(_state): State<AppState>,
    body: String,
) -> Response {
    tracing::info!(
        "Starting confirmation processing with body: {}",
        &body[0..100.min(body.len())]
    );

    // Parse the JSON request
    let request: ConfirmationProcessRequest = match parse_json_request(&body) {
        Ok(req) => {
            tracing::info!("Successfully parsed confirmation process request");
            req
        }
        Err(_) => {
            // Custom error for this endpoint with specific response format
            let error_response = json!({
                "success": false,
                "message": "Invalid JSON request format",
                "order_id": null,
                "user_email": null
            });
            return (StatusCode::BAD_REQUEST, error_response.to_string()).into_response();
        }
    };

    // Validate required parameters
    let (payment_id, app_reference) = match (request.payment_id, request.app_reference) {
        (Some(pay_id), Some(app_ref)) => (pay_id, app_ref),
        _ => {
            let error_msg =
                "Missing required parameters: payment_id and app_reference are required";
            tracing::error!("{}", error_msg);
            let error_response = json!({
                "success": false,
                "message": error_msg,
                "order_id": null,
                "user_email": null
            });
            return (StatusCode::BAD_REQUEST, error_response.to_string()).into_response();
        }
    };

    // Extract app_reference and order_id using existing utilities
    // The order_id from payment provider needs to be converted to get the actual booking order_id
    // payment_id is what we got from payment provider, but we need the order_id from app_reference

    // First, try to extract email from order_id if it's the proper format
    let (order_id, user_email) = if let Some(booking_id) = BookingId::from_order_id(&app_reference)
    {
        // app_reference is actually the order_id in proper format
        (app_reference.clone(), booking_id.email)
    } else {
        // app_reference might be the simple app reference, need to build order_id
        // For now, assume app_reference contains the order_id format
        return {
            let error_msg = format!(
                "Failed to parse BookingId from app_reference: {}",
                app_reference
            );
            tracing::error!("{}", error_msg);
            let error_response = json!({
                "success": false,
                "message": error_msg,
                "order_id": null,
                "user_email": null
            });
            (StatusCode::BAD_REQUEST, error_response.to_string()).into_response()
        };
    };

    if user_email.is_empty() {
        let error_msg = "Failed to extract user email from app_reference/order_id";
        tracing::error!("{}", error_msg);
        let error_response = json!({
            "success": false,
            "message": error_msg,
            "order_id": Some(order_id),
            "user_email": null
        });
        return (StatusCode::BAD_REQUEST, error_response.to_string()).into_response();
    }

    // Validate that we can create BookingId from order_id (double-check)
    let booking_id = match BookingId::from_order_id(&order_id) {
        Some(bid) => bid,
        None => {
            let error_msg = format!("Failed to create BookingId from order_id: {}", order_id);
            tracing::error!("{}", error_msg);
            let error_response = json!({
                "success": false,
                "message": error_msg,
                "order_id": Some(order_id),
                "user_email": Some(user_email)
            });
            return (StatusCode::BAD_REQUEST, error_response.to_string()).into_response();
        }
    };

    tracing::info!(
        "Extracted booking_id: app_reference = {}, email = {}",
        booking_id.app_reference,
        booking_id.email
    );

    // Create ServerSideBookingEvent
    let event = ServerSideBookingEvent {
        payment_id: Some(payment_id.clone()),
        order_id: order_id.clone(),
        provider: "nowpayments".to_string(),
        user_email: user_email.clone(),
        payment_status: None,
        backend_payment_status: Some("confirmation_page_initiated".to_string()),
        backend_booking_status: None,
        backend_booking_struct: None,
    };

    // Create pipeline steps
    let payment_status_step =
        SSRBookingPipelineStep::PaymentStatus(GetPaymentStatusFromPaymentProvider);
    let book_room_step = SSRBookingPipelineStep::BookRoom(MakeBookingFromBookingProvider);
    let send_email_step = SSRBookingPipelineStep::SendEmail(SendEmailAfterSuccessfullBooking);

    let steps = vec![payment_status_step, book_room_step, send_email_step];

    tracing::info!(
        "Executing booking pipeline for order_id: {}, payment_id: {}",
        order_id,
        payment_id
    );

    // Execute the pipeline - this will publish events to the eventbus
    match process_pipeline(event, &steps, None).await {
        Ok(final_event) => {
            tracing::info!(
                "Booking pipeline completed successfully for payment_id: {}",
                payment_id
            );

            let success_response = json!({
                "success": true,
                "message": "Booking process initiated successfully. Check eventbus for updates.",
                "order_id": order_id,
                "user_email": user_email
            });

            (StatusCode::OK, success_response.to_string()).into_response()
        }
        Err(e) => {
            tracing::error!("Booking pipeline failed: {}", e);

            let error_response = json!({
                "success": false,
                "message": format!("Booking processing failed: {}", e),
                "order_id": Some(order_id),
                "user_email": Some(user_email)
            });

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                error_response.to_string(),
            )
                .into_response()
        }
    }
}
