use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use estate_fe::view_state_layer::AppState;
use estate_fe::{
    api::{
        canister::get_user_booking::get_booking_by_id_backend,
        payments::service::PaymentServiceImpl,
    },
    canister::backend,
    ssr_booking::{
        booking_handler::MakeBookingFromBookingProvider,
        email_handler::SendEmailAfterSuccessfullBooking,
        get_booking_from_backend::GetBookingFromBackend,
        payment_handler::{
            GetPaymentStatusFromPaymentProvider, GetPaymentStatusFromPaymentProviderV2,
        },
        pipeline::process_pipeline,
        SSRBookingPipelineStep, ServerSideBookingEvent,
    },
    utils::app_reference::BookingId,
};
use serde_json::json;

use super::{parse_json_request, ConfirmationProcessRequest};

/// Detect payment provider from available information
async fn detect_payment_provider(payment_id: &Option<String>, booking_id: &BookingId) -> String {
    // Method 1: Detect from payment_id if available
    if let Some(ref pid) = payment_id {
        if let Ok(provider) = PaymentServiceImpl::detect_provider_from_payment_id(pid) {
            return provider.as_str().to_string();
        }
    }

    // Method 2: Try to extract from backend booking data
    let backend_booking_id = backend::BookingId {
        app_reference: booking_id.app_reference.clone(),
        email: booking_id.email.clone(),
    };

    if let Ok(Some(booking)) = get_booking_by_id_backend(backend_booking_id).await {
        return booking
            .payment_details
            .payment_api_response
            .provider
            .clone();
    }

    // Method 3: Unknown provider - will use AllProviders fallback
    "unknown".to_string()
}

/// Fetch booking from backend and return serializable booking data
/// Returns None if booking not found, logs error if fetch fails
async fn fetch_booking_data(booking_id: &BookingId) -> Option<serde_json::Value> {
    let backend_booking_id = backend::BookingId {
        app_reference: booking_id.app_reference.clone(),
        email: booking_id.email.clone(),
    };

    match get_booking_by_id_backend(backend_booking_id).await {
        Ok(Some(booking)) => {
            tracing::info!("Successfully fetched booking data from backend");
            // Convert booking to serializable format
            match serde_json::to_value(&booking) {
                Ok(booking_json) => Some(booking_json),
                Err(e) => {
                    tracing::error!("Failed to serialize booking data: {}", e);
                    None
                }
            }
        }
        Ok(None) => {
            tracing::warn!(
                "No booking found in backend for booking_id: {:?}",
                booking_id
            );
            None
        }
        Err(e) => {
            tracing::error!("Failed to fetch booking from backend: {}", e);
            None
        }
    }
}

#[axum::debug_handler]
#[cfg_attr(feature = "debug_log", tracing::instrument(skip(state)))]
pub async fn process_confirmation_api_server_fn_route(
    State(state): State<AppState>,
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

    // Validate required parameters - app_reference is required, payment_id is optional
    let app_reference = match request.app_reference {
        Some(app_ref) => app_ref,
        None => {
            let error_msg = "Missing required parameter: app_reference is required";
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

    // payment_id is optional - extract if available
    let payment_id = request.payment_id;

    // Extract app_reference and order_id using existing utilities
    // The order_id from payment provider needs to be converted to get the actual booking order_id
    // payment_id is what we got from payment provider, but we need the order_id from app_reference

    // Extract email and order_id - support both flows
    let (order_id, user_email) = if let Some(explicit_email) = request.email {
        // New flow: email provided explicitly, use app_reference as order_id
        (app_reference.clone(), explicit_email)
    } else if let Some(booking_id) = BookingId::from_order_id(&app_reference) {
        // Existing flow: app_reference is actually the order_id in proper format
        (app_reference.clone(), booking_id.email)
    } else {
        // Neither explicit email nor valid order_id format
        let error_msg = format!(
            "Failed to extract email: either provide 'email' field or ensure app_reference is a valid order_id format: {}",
            app_reference
        );
        tracing::error!("{}", error_msg);
        let error_response = json!({
            "success": false,
            "message": error_msg,
            "order_id": null,
            "user_email": null
        });
        return (StatusCode::BAD_REQUEST, error_response.to_string()).into_response();
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
        payment_id: payment_id.clone(), // Now optional
        order_id: order_id.clone(),
        provider: detect_payment_provider(&payment_id, &booking_id).await,
        user_email: user_email.clone(),
        payment_status: None,
        backend_payment_status: Some("confirmation_page_initiated".to_string()),
        backend_booking_status: None,
        backend_booking_struct: None,
    };

    // Create pipeline steps - using V2 for unified payment provider support
    let payment_status_step =
        SSRBookingPipelineStep::PaymentStatusV2(GetPaymentStatusFromPaymentProviderV2);
    let book_room_step = SSRBookingPipelineStep::BookRoom(MakeBookingFromBookingProvider);
    let get_booking_step = SSRBookingPipelineStep::GetBookingFromBackend(GetBookingFromBackend);
    let send_email_step = SSRBookingPipelineStep::SendEmail(SendEmailAfterSuccessfullBooking);

    let steps = vec![
        get_booking_step,
        payment_status_step,
        book_room_step,
        send_email_step,
    ];

    tracing::info!(
        "Executing booking pipeline for order_id: {}, payment_id: {:?}",
        order_id,
        payment_id
    );

    // Execute the pipeline - this will publish events to the eventbus
    let notifier = &state.notifier_for_pipeline;
    let pipeline_result = process_pipeline(event, &steps, Some(notifier)).await;

    // Fetch booking data from backend regardless of pipeline success/failure
    let booking_data = fetch_booking_data(&booking_id).await;

    match pipeline_result {
        Ok(final_event) => {
            tracing::info!(
                "Booking pipeline completed successfully for payment_id: {:?}",
                payment_id
            );

            let success_response = json!({
                "success": true,
                "message": "Booking process initiated successfully. Check eventbus for updates.",
                "order_id": order_id,
                "user_email": user_email,
                "booking_data": booking_data
            });

            (StatusCode::OK, success_response.to_string()).into_response()
        }
        Err(e) => {
            tracing::error!(
                "Booking pipeline failed: {} - but still returning booking data if available",
                e
            );

            let error_response = json!({
                "success": false,
                "message": format!("Booking processing failed: {}", e),
                "order_id": Some(order_id),
                "user_email": Some(user_email),
                "booking_data": booking_data
            });

            (StatusCode::OK, error_response.to_string()).into_response()
        }
    }
}
