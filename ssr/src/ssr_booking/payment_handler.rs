use async_trait::async_trait;
use std::time::{Duration, Instant};
use tokio::time;
use tracing::{debug, error, info, instrument, warn};

use crate::api::payments::ports::{GetPaymentStatusRequest, GetPaymentStatusResponse};
use crate::api::payments::NowPayments;
use crate::canister::backend::{
    BackendPaymentStatus, BePaymentApiResponse, BookingId, PaymentDetails, Result2,
};
use crate::ssr_booking::pipeline::{PipelineExecutor, PipelineValidator};
use crate::ssr_booking::{PipelineDecision, ServerSideBookingEvent};
use crate::utils::admin::{admin_canister, AdminCanisters};
use crate::utils::booking_id::PaymentIdentifiers;

// ---------------------
// external api calls
// ---------------------

pub async fn nowpayments_get_payment_status(
    request: GetPaymentStatusRequest,
) -> Result<GetPaymentStatusResponse, String> {
    let nowpayments = NowPayments::default();
    info!("{:#?}", request);
    match nowpayments.send(request).await {
        Ok(response) => Ok(response),
        Err(e) => Err(e.to_string()),
    }
}

/// Wrapper function with retry logic:
/// - Retries every 5 seconds
/// - Cancels (returns error) after 10 minutes if no successful response
pub async fn get_payment_status_with_retry(
    request: GetPaymentStatusRequest,
) -> Result<GetPaymentStatusResponse, String>
where
    GetPaymentStatusRequest: Clone,
{
    let retry_interval = Duration::from_secs(5);
    let max_timeout = Duration::from_secs(20 * 60); // 20 minutes
    let start_time = Instant::now();

    loop {
        // Attempt to get the payment status
        match nowpayments_get_payment_status(request.clone()).await {
            Ok(response) => return Ok(response),
            Err(e) => {
                // Check if we've exceeded the timeout
                if start_time.elapsed() >= max_timeout {
                    error!(
                        "Timed out waiting for payment status: payment_id = {:?}, error: {}",
                        request.payment_id,
                        e.to_string()
                    );
                    return Err(format!("Timed out waiting for payment status: {}", e));
                }
                info!("Payment status : {}. Retrying in 5 seconds...", e);
                time::sleep(retry_interval).await;
            }
        }
    }
}

// ---------------------
// PIPELINE INTEGRATION for payment provider as a step in pipeline
// ---------------------

#[derive(Debug, Clone)]
pub struct GetPaymentStatusFromPaymentProvider;

#[async_trait]
impl PipelineValidator for GetPaymentStatusFromPaymentProvider {
    #[instrument(name = "validate_get_payment_status", skip(self, event), err(Debug))]
    async fn validate(&self, event: &ServerSideBookingEvent) -> Result<PipelineDecision, String> {
        if event.payment_id.is_some() {
            // what is relation between booking id and order id ??? (see: memories/booking_id.md)
            // TODO: check if payment is_paid == false, (from backend) then RUN
            // if is_paid == true, PipelineDecision::Skip

            // ensure that order_id exists in ServerSideBookingEvent and booking_id can be derived from order_id - log the error use crate::log;

            Ok(PipelineDecision::Run)
        } else {
            Ok(PipelineDecision::Skip)
        }
    }
}

#[async_trait]
impl PipelineExecutor for GetPaymentStatusFromPaymentProvider {
    #[instrument(name = "execute_get_payment_status", skip(event), err(Debug))]
    async fn execute(event: ServerSideBookingEvent) -> Result<ServerSideBookingEvent, String> {
        // step 1:  Retrieves the payment status  from API

        // Get payment ID from event
        let payment_id = event
            .payment_id
            .clone()
            .and_then(|id| id.parse::<u64>().ok())
            .ok_or_else(|| "Payment ID not found in event".to_string())?;

        // Create request for payment status
        let request = GetPaymentStatusRequest { payment_id };

        // Get payment status with retry
        let response = get_payment_status_with_retry(request).await?;

        // Get payment status string from response
        let payment_status = response.get_payment_status();

        // Create updated event with payment status
        let mut updated_event = event;
        updated_event.payment_status = Some(payment_status.clone());
        info!("Updated event: {:?}", updated_event);

        // step 2: write code to update payment status in backend using the function details in file memories/update_payment.md
        let order_id = updated_event.order_id.clone();
        let user_email = updated_event.user_email.clone();

        // Extract app_reference from order_id using our new utility
        let app_reference =
            PaymentIdentifiers::app_reference_from_order_id(&order_id).ok_or_else(|| {
                format!(
                    "Failed to extract app_reference from order_id: {}",
                    order_id
                )
            })?;

        let booking_id = BookingId {
            app_reference,
            email: user_email.clone(),
        };
        let booking_id_clone = booking_id.clone();

        // Create BePaymentApiResponse from the payment status response
        let payment_api_response =
            BePaymentApiResponse::from((response.clone(), "nowpayments".to_string()));

        // Create PaymentDetails with the appropriate payment status
        let payment_details = PaymentDetails {
            payment_status: if payment_status == "finished" {
                BackendPaymentStatus::Paid(payment_status.clone())
            } else {
                BackendPaymentStatus::Unpaid(Some(payment_status.clone()))
            },
            booking_id: booking_id.clone(),
            payment_api_response,
        };

        let admin_canister = AdminCanisters::from_env();
        let backend = admin_canister.backend_canister().await;
        // .map_err(|e| format!("Failed to get backend canister: {}", e))?;

        // Update payment details in backend and verify the response
        let updated_booking = backend
            .update_payment_details(booking_id, payment_details)
            .await
            .map_err(|e| format!("call backend canister to update payment details: {}", e))?;

        // Verify that payment status is correctly set to Paid
        match updated_booking {
            Result2::Ok(booking) => {
                info!(
                    "Payment details updated successfully. Payment status: {:?}",
                    booking.payment_details.payment_status
                );
            }
            Result2::Err(e) => {
                return Err(format!("Failed to update payment details: {}", e));
            }
        }

        // step 3: if the backend update is successful, return updated_event
        // Update the event with the latest payment status and ensure email is set
        updated_event.user_email = booking_id_clone.email;
        updated_event.payment_status = Some(payment_status.clone());

        // Log the successful update
        info!(
            "Successfully updated payment details in backend. Payment status: {}",
            payment_status
        );

        Ok(updated_event)
    }
}
