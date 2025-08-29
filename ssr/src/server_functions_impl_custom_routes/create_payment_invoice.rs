use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use estate_fe::api::payments::{
    domain::DomainCreateInvoiceRequest, service::PaymentServiceImpl, PaymentService,
};
use estate_fe::view_state_layer::AppState;
use serde_json::json;

use super::parse_json_request;

#[axum::debug_handler]
#[tracing::instrument(skip(_state))]
pub async fn create_payment_invoice_api_server_fn_route(
    State(_state): State<AppState>,
    body: String,
) -> Result<Response, Response> {
    tracing::info!(
        "Starting create payment invoice API with body: {}",
        &body[0..100.min(body.len())]
    );

    // Parse the JSON request
    let request: DomainCreateInvoiceRequest = {
        let req = parse_json_request(&body)?;
        tracing::info!("Successfully parsed payment invoice request");
        req
    };

    tracing::info!(
        "Processing payment request - Provider: {:?}, Order ID: {}, Amount: {}",
        request.provider,
        request.order_id,
        request.price_amount
    );

    // Initialize the payment service
    let payment_service = PaymentServiceImpl::new();

    // Create the invoice using the abstracted service
    let response = payment_service
        .create_invoice(request)
        .await
        .map_err(|payment_error| {
            tracing::error!("Payment invoice creation failed: {}", payment_error);
            let error_response = json!({
                "error": format!("Payment creation failed: {}", payment_error),
            });
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                error_response.to_string(),
            )
                .into_response()
        })?;

    tracing::info!(
        "Payment invoice created successfully - Invoice ID: {}, Payment URL: {}",
        response.invoice_id,
        response.payment_url
    );

    // Return the successful response
    let success_response = json!(response);
    Ok((StatusCode::OK, success_response.to_string()).into_response())
}
