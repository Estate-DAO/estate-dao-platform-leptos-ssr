use async_trait::async_trait;
use std::time::{Duration, Instant};
use tokio::time;
use tracing::{debug, error, info, instrument, warn};

use crate::utils::notifier::{self, Notifier};
use crate::utils::notifier_event::{NotifierEvent, NotifierEventType};
use crate::utils::uuidv7;
use chrono::Utc;

use crate::api::payments::ports::{GetPaymentStatusRequest, GetPaymentStatusResponse};
use crate::api::payments::NowPayments;
use crate::canister::backend::{
    BackendPaymentStatus, BePaymentApiResponse, BookingId, PaymentDetails, Result2, Result3,
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
    let nowpayments = NowPayments::try_from_env();
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
    let max_timeout = Duration::from_secs(30);
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

/// Checks if payment status is 'finished' with retry and exponential backoff
/// - Retries with exponential backoff (starting at 5 seconds)
/// - Cancels (returns error) after the specified timeout duration if payment is not finished
///
/// # Arguments
/// * `request` - The payment status request to check
/// * `timeout_duration` - Maximum time to wait for payment to be finished (default: 10 minutes)
///
/// # Returns
/// * `Ok(())` - If payment status is 'finished'
/// * `Err(String)` - If payment status is not 'finished' after timeout or other errors occur
#[instrument(
    name = "check_payment_status_finished",
    skip(request),
    fields(
        payment_id = ?request.payment_id,
        timeout_secs = ?timeout_duration.map(|d| d.as_secs()).unwrap_or(20 * 60)
    ),
    err
)]
pub async fn check_if_payment_status_finished_with_backoff(
    request: GetPaymentStatusRequest,
    timeout_duration: Option<Duration>,
) -> Result<GetPaymentStatusResponse, String> {
    let timeout = timeout_duration.unwrap_or(Duration::from_secs(20 * 60)); // Default 20 minutes
    let start_time = Instant::now();
    let mut retry_interval = Duration::from_secs(5); // Start with 5 seconds
    let max_retry_interval = Duration::from_secs(20); // Cap at 20 seconds
    let backoff_factor = 1.5; // Exponential backoff multiplier

    info!(
        "Starting payment status check with timeout of {:?}",
        timeout
    );

    loop {
        // Get payment status with retry
        match get_payment_status_with_retry(request.clone()).await {
            Ok(response) => {
                // Check if payment is finished
                if response.is_finished() {
                    info!("Payment is finished: payment_id = {:?}", request.payment_id);
                    return Ok(response);
                }

                // Payment is not finished, log current status and retry
                let status = response.get_payment_status();
                info!(
                    "Payment not finished yet: payment_id = {:?}, status = {}",
                    request.payment_id, status
                );
            }
            Err(e) => {
                warn!(
                    "Error checking payment status: payment_id = {:?}, error: {}",
                    request.payment_id, e
                );
            }
        }

        // Check if we've exceeded the timeout
        if start_time.elapsed() >= timeout {
            error!(
                "Timed out waiting for payment to finish: payment_id = {:?}",
                request.payment_id
            );
            return Err(format!(
                "Timed out waiting for payment to finish after {:?}",
                timeout
            ));
        }

        // Apply exponential backoff with a cap
        info!(
            "Payment not finished. Retrying in {:?} seconds...",
            retry_interval.as_secs()
        );
        time::sleep(retry_interval).await;

        // Increase retry interval with exponential backoff
        retry_interval = std::cmp::min(
            Duration::from_secs((retry_interval.as_secs() as f64 * backoff_factor) as u64),
            max_retry_interval,
        );
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
        // Ensure that order_id exists in ServerSideBookingEvent and booking_id can be derived from order_id
        if event.order_id.is_empty() {
            error!("order_id is empty in ServerSideBookingEvent");
            return Err("order_id is required but empty".to_string());
        }

        // Verify that app_reference can be derived from order_id
        if PaymentIdentifiers::app_reference_from_order_id(&event.order_id).is_none() {
            error!(
                "Failed to extract app_reference from order_id: {}",
                event.order_id
            );
            return Err(format!(
                "Failed to extract app_reference from order_id: {}",
                event.order_id
            ));
        }

        if event.payment_id.is_some() {
            // Payment ID provided - check external provider flow
            if let Some(ref backend_booking) = event.backend_booking_struct {
                match &backend_booking.payment_details.payment_status {
                    BackendPaymentStatus::Paid(_) => {
                        info!("Payment already paid in backend, skipping external payment status check");
                        return Ok(PipelineDecision::Skip);
                    }
                    BackendPaymentStatus::Unpaid(_) => {
                        info!("Payment not yet paid in backend, proceeding with external payment status check");
                    }
                }
            }
            Ok(PipelineDecision::Run)
        } else {
            // No payment ID - we'll extract status from backend_booking_struct
            info!("No payment_id provided, will extract payment status from backend booking data");
            Ok(PipelineDecision::Run)
        }
    }
}

#[async_trait]
impl PipelineExecutor for GetPaymentStatusFromPaymentProvider {
    #[instrument(name = "execute_get_payment_status", skip(event, notifier), err(Debug))]
    async fn execute(
        event: ServerSideBookingEvent,
        notifier: Option<&Notifier>,
    ) -> Result<ServerSideBookingEvent, String> {
        let mut updated_event = event;
        let (payment_status, is_finished, source) = if let Some(ref payment_id_str) =
            updated_event.payment_id
        {
            // Flow 1: External payment provider check
            info!(
                "Checking payment status from external provider for payment_id: {}",
                payment_id_str
            );

            let payment_id = payment_id_str
                .parse::<u64>()
                .map_err(|e| format!("Invalid payment_id format: {}", e))?;

            let request = GetPaymentStatusRequest { payment_id };
            let response = check_if_payment_status_finished_with_backoff(request, None).await?;
            let status = response.get_payment_status();
            let finished = response.is_finished();

            updated_event.payment_status = Some(status.clone());
            (status, finished, "external_provider".to_string())
        } else {
            // Flow 2: Extract from backend booking data
            info!("No payment_id provided, extracting payment status from backend booking data");

            if let Some(ref backend_booking) = updated_event.backend_booking_struct {
                let (status, finished) = match &backend_booking.payment_details.payment_status {
                    BackendPaymentStatus::Paid(_) => ("finished".to_string(), true),
                    BackendPaymentStatus::Unpaid(_) => ("waiting".to_string(), false),
                };
                info!("Extracted payment status from backend: {}", status);
                updated_event.payment_status = Some(status.clone());
                (status, finished, "backend_booking".to_string())
            } else {
                return Err(
                    "No backend booking data available to extract payment status".to_string(),
                );
            }
        };

        info!(
            "Payment status determined: {} (finished: {}, source: {})",
            payment_status, is_finished, source
        );

        // --- EMIT CUSTOM EVENT: PaymentStatusChecked ---
        if let Some(n) = notifier {
            let correlation_id = tracing::Span::current()
                .field("correlation_id")
                .map(|f| f.to_string())
                .unwrap_or_else(|| "unknown_correlation_id".to_string());

            let custom_event = NotifierEvent {
                event_id: uuidv7::create(),
                correlation_id,
                timestamp: Utc::now(),
                order_id: updated_event.order_id.clone(),
                step_name: Some("GetPaymentStatusFromPaymentProvider".to_string()),
                event_type: NotifierEventType::PaymentStatusChecked {
                    status: payment_status.clone(),
                    is_finished,
                },
                email: updated_event.user_email.clone(),
            };
            info!("Emitting PaymentStatusChecked event: {custom_event:#?}");
            n.notify(custom_event).await;
        }
        // --- END EMIT CUSTOM EVENT ---

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

        // Create BePaymentApiResponse based on the source of payment status
        let payment_api_response = if source == "external_provider"
            && updated_event.payment_id.is_some()
        {
            // For external provider, we already have the response from the API call above
            // We need to reconstruct the GetPaymentStatusResponse for the conversion
            let success_response = crate::api::payments::ports::SuccessGetPaymentStatusResponse {
                payment_status: payment_status.clone(),
                payment_id: updated_event
                    .payment_id
                    .as_ref()
                    .unwrap()
                    .parse::<u64>()
                    .unwrap_or(0),
                invoice_id: 0,   // We don't have this from the current flow
                price_amount: 0, // Default value
                price_currency: "USD".to_string(), // Default value
                pay_amount: 0.0, // Default value
                actually_paid: 0.0, // Default value
                pay_currency: "USDC".to_string(), // Default value
                order_id: order_id.clone(),
                order_description: "Payment confirmation".to_string(),
                purchase_id: 0, // Default value
                created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                updated_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            };
            let get_payment_response = GetPaymentStatusResponse::Success(success_response);
            BePaymentApiResponse::from((get_payment_response, "nowpayments".to_string()))
        } else {
            // Backend-extracted status - create a minimal response
            BePaymentApiResponse {
                payment_status: payment_status.clone(),
                payment_id: updated_event
                    .payment_id
                    .as_ref()
                    .and_then(|p| p.parse::<u64>().ok())
                    .unwrap_or(0),
                provider: "backend_extracted".to_string(),
                created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                updated_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                actually_paid: 0.0,
                invoice_id: 0,
                order_description: "Payment from backend data".to_string(),
                pay_amount: 0.0,
                pay_currency: "USDC".to_string(),
                price_amount: 0,
                purchase_id: 0,
                order_id: order_id.clone(),
                price_currency: "USD".to_string(),
            }
        };

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
            Result3::Ok(booking) => {
                info!(
                    "Payment details updated successfully. Payment status: {:?}",
                    booking.payment_details.payment_status
                );
            }
            Result3::Err(e) => {
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
