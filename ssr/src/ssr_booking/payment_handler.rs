use async_trait::async_trait;
use std::time::{Duration, Instant};
use tokio::time;
use tracing::{debug, error, info, instrument, warn};

use crate::utils::notifier::{self, Notifier};
use crate::utils::notifier_event::{NotifierEvent, NotifierEventType};
use crate::utils::uuidv7;
use chrono::Utc;

use crate::api::canister::get_user_booking::get_booking_by_id_backend;
use crate::api::payments::domain::PaymentService;
use crate::api::payments::domain::{DomainGetPaymentStatusResponse, PaymentStatus};
use crate::api::payments::ports::{GetPaymentStatusRequest, GetPaymentStatusResponse};
use crate::api::payments::service::PaymentServiceImpl;
use crate::api::payments::NowPayments;
use crate::canister::backend::Booking;
use crate::canister::backend::{
    BackendPaymentStatus, BePaymentApiResponse, BookingId, PaymentDetails, Result2, Result3,
};
use crate::ssr_booking::pipeline::{PipelineExecutor, PipelineValidator};
use crate::ssr_booking::{PipelineDecision, ServerSideBookingEvent};
use crate::utils::admin::{admin_canister, AdminCanisters};
use crate::utils::booking_id::PaymentIdentifiers;

use crate::utils::app_reference::BookingId as FrontendBookingId;

// ---------------------
// HELPER FUNCTIONS
// ---------------------

/// Helper function to check if payment is completed using Domain PaymentStatus
fn is_payment_completed_domain(status: &PaymentStatus) -> bool {
    matches!(status, PaymentStatus::Completed)
}

/// Helper function to create BePaymentApiResponse with enriched data from booking struct
/// Returns Error if payment_id_v2 is not available (strict requirement)
pub fn create_payment_api_response_from_booking(
    domain_response: Option<&DomainGetPaymentStatusResponse>,
    backend_booking: &Booking,
    updated_event: &ServerSideBookingEvent,
    payment_status: String,
    source: String,
) -> Result<BePaymentApiResponse, String> {
    // STRICT REQUIREMENT: payment_id_v2 must be present
    let payment_id_v2 = domain_response
        .map(|r| r.payment_id.clone())
        .or_else(|| updated_event.payment_id.as_ref().cloned())
        .filter(|id| !id.is_empty())
        .ok_or_else(|| {
            "payment_id_v2 is required but not available in domain response or event".to_string()
        })?;

    // Extract hotel and room details for better descriptions
    let hotel_details = &backend_booking
        .user_selected_hotel_room_details
        .hotel_details;
    let room_details = &backend_booking
        .user_selected_hotel_room_details
        .room_details;
    let requested_amount = backend_booking
        .user_selected_hotel_room_details
        .requested_payment_amount;
    let date_range = &backend_booking.user_selected_hotel_room_details.date_range;

    // Create rich order description from booking details
    let order_description = if !room_details.is_empty() {
        format!(
            "Hotel: {} | Room: {} | Dates: {}-{}-{} to {}-{}-{} | Amount: ${:.2}",
            hotel_details.hotel_name,
            room_details[0].room_type_name,
            date_range.start.0,
            date_range.start.1,
            date_range.start.2,
            date_range.end.0,
            date_range.end.1,
            date_range.end.2,
            requested_amount
        )
    } else {
        format!(
            "Hotel: {} | Dates: {}-{}-{} to {}-{}-{} | Amount: ${:.2}",
            hotel_details.hotel_name,
            date_range.start.0,
            date_range.start.1,
            date_range.start.2,
            date_range.end.0,
            date_range.end.1,
            date_range.end.2,
            requested_amount
        )
    };

    // Calculate total room price
    let total_room_price: f32 = room_details.iter().map(|room| room.room_price).sum();

    // Use existing payment data if available from backend
    let existing_payment_response = &backend_booking.payment_details.payment_api_response;

    Ok(BePaymentApiResponse {
        // Payment status from current flow
        payment_status,

        // Payment IDs - payment_id_v2 is mandatory
        payment_id: domain_response
            .and_then(|r| r.payment_id.parse::<u64>().ok())
            .or_else(|| {
                (existing_payment_response.payment_id != 0)
                    .then_some(existing_payment_response.payment_id)
            })
            .unwrap_or(0), // deprecated field
        payment_id_v2, // This is validated above to be present

        // Provider information - prioritize domain response
        provider: domain_response
            .map(|r| r.provider.as_str().to_string())
            .unwrap_or(source),

        // Timestamps - use existing if available, otherwise current time
        created_at: if !existing_payment_response.created_at.is_empty() {
            existing_payment_response.created_at.clone()
        } else {
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
        },
        updated_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),

        // Payment amounts - prioritize domain response, then existing, then booking data
        pay_amount: domain_response
            .and_then(|r| r.amount_total.map(|amt| amt as f64 / 100.0)) // Convert cents to dollars
            .or_else(|| {
                (existing_payment_response.pay_amount != 0.0)
                    .then_some(existing_payment_response.pay_amount)
            })
            .unwrap_or(requested_amount),

        actually_paid: domain_response
            .and_then(|r| r.amount_total.map(|amt| amt as f64 / 100.0))
            .or_else(|| {
                (existing_payment_response.actually_paid != 0.0)
                    .then_some(existing_payment_response.actually_paid)
            })
            .unwrap_or(0.0),

        price_amount: domain_response
            .and_then(|r| r.amount_total)
            .or_else(|| {
                (existing_payment_response.price_amount != 0)
                    .then_some(existing_payment_response.price_amount)
            })
            .unwrap_or((requested_amount * 100.0) as u64), // Convert to cents

        // Currency information - prioritize domain response, then existing, then defaults
        pay_currency: domain_response
            .and_then(|r| r.currency.clone())
            .or_else(|| {
                (!existing_payment_response.pay_currency.is_empty())
                    .then_some(existing_payment_response.pay_currency.clone())
            })
            .unwrap_or_else(|| "USDC".to_string()),

        price_currency: domain_response
            .and_then(|r| r.currency.clone())
            .or_else(|| {
                (!existing_payment_response.price_currency.is_empty())
                    .then_some(existing_payment_response.price_currency.clone())
            })
            .unwrap_or_else(|| "USD".to_string()),

        // Order details
        order_id: domain_response
            .and_then(|r| r.order_id.clone())
            .unwrap_or_else(|| updated_event.order_id.clone()),
        order_description,

        // Invoice and purchase IDs - use existing if available
        invoice_id: (existing_payment_response.invoice_id != 0)
            .then_some(existing_payment_response.invoice_id)
            .unwrap_or(0),
        purchase_id: (existing_payment_response.purchase_id != 0)
            .then_some(existing_payment_response.purchase_id)
            .unwrap_or(0),
    })
}

// /// Alternative helper when only backend booking is available (no domain response)
// /// Still requires payment_id_v2 to be present in the event
// pub fn create_payment_api_response_from_backend_only(
//     backend_booking: &Booking,
//     updated_event: &ServerSideBookingEvent,
//     payment_status: String,
//     source: String,
// ) -> Result<BePaymentApiResponse, String> {
//     create_payment_api_response_from_booking(
//         None, // No domain response
//         backend_booking,
//         updated_event,
//         payment_status,
//         source,
//     )
// }

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
                // deprecated field
                payment_id: 0,
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
                // deprecated field
                payment_id: 0,
                payment_id_v2: updated_event
                    .payment_id
                    .as_ref()
                    .cloned()
                    .unwrap_or("".to_string()),
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

// ---------------------
// V2 PAYMENT HANDLER - Unified provider support with retry logic
// ---------------------

#[derive(Debug, Clone)]
pub struct GetPaymentStatusFromPaymentProviderV2;

impl GetPaymentStatusFromPaymentProviderV2 {
    /// Helper function to check payment status and determine pipeline decision
    fn check_payment_status_decision(payment_status: &BackendPaymentStatus) -> PipelineDecision {
        match payment_status {
            BackendPaymentStatus::Paid(_) => {
                info!("Payment already paid in backend, skipping external payment status check");
                PipelineDecision::Skip
            }
            BackendPaymentStatus::Unpaid(_) => {
                info!("Payment not yet paid in backend, proceeding with external payment status check");
                PipelineDecision::Run
            }
        }
    }

    /// Helper function to load booking from backend
    async fn load_booking_from_backend(
        order_id: &str,
        user_email: &str,
    ) -> Result<crate::canister::backend::Booking, String> {
        // let app_reference =
        //     PaymentIdentifiers::app_reference_from_order_id(order_id).ok_or_else(|| {
        //         format!(
        //             "Failed to extract app_reference from order_id: {}",
        //             order_id
        //         )
        //     })?;

        // let booking_id = BookingId {
        //     app_reference,
        //     email: user_email.to_string(),
        // };

        let booking_id = PaymentIdentifiers::booking_id_from_order_id(order_id, user_email)?;
        info!(
            "load_booking_from_backend - start - Booking ID: {:?}",
            booking_id
        );
        // Fetch booking by ID directly from backend
        get_booking_by_id_backend(booking_id.clone().into())
            .await
            .map_err(|e| format!("Failed to fetch booking: ServerFnError = {}", e))?
            .ok_or_else(|| {
                format!(
                    "load_booking_from_backend - finished - No booking found with the specified booking ID: {:?}",
                    booking_id
                )
            })
    }

    #[instrument(
        name = "check_payment_provider_if_unpaid",
        skip(payment_id, backend_booking, order_id),
        err(Debug)
    )]
    /// Helper function to check payment provider if backend status is unpaid
    async fn check_payment_provider_if_unpaid(
        payment_id: Option<&str>,
        backend_booking: &crate::canister::backend::Booking,
        order_id: &str,
    ) -> Result<Option<crate::api::payments::domain::DomainGetPaymentStatusResponse>, String> {
        use crate::api::payments::domain::{DomainGetPaymentStatusRequest, PaymentStatus};

        // Only check payment provider if payment_id exists and backend shows unpaid
        let payment_id_str = match payment_id {
            Some(id) => id,
            None => {
                info!("No payment_id provided, skipping external payment provider check");
                return Ok(None);
            }
        };

        // Check if we should call external provider using existing decision logic
        let decision =
            Self::check_payment_status_decision(&backend_booking.payment_details.payment_status);

        match decision {
            crate::ssr_booking::PipelineDecision::Skip => {
                info!("Backend payment status is Paid, skipping external payment provider check");
                Ok(None)
            }
            crate::ssr_booking::PipelineDecision::Run => {
                info!(
                    "Backend payment status is Unpaid, checking external payment provider for payment_id: {}",
                    payment_id_str
                );

                // Use the unified payment service with 10-minute retry logic
                let payment_service = PaymentServiceImpl::new();
                let domain_request: DomainGetPaymentStatusRequest = DomainGetPaymentStatusRequest {
                    payment_id: payment_id_str.to_string(),
                    provider: None, // Let the service auto-detect the provider
                };

                let domain_response = check_payment_status_with_retry_v2(
                    payment_service,
                    domain_request,
                    order_id.to_string(),
                    Duration::from_secs(600), // 10 minutes
                )
                .await?;

                info!(
                    "External payment provider response: status={:?}, provider={:?}",
                    domain_response.status, domain_response.provider
                );

                Ok(Some(domain_response))
            }
            crate::ssr_booking::PipelineDecision::Abort(reason) => {
                info!(
                    "Pipeline decision is Abort: {}, skipping external payment provider check",
                    reason
                );
                Ok(None)
            }
        }
    }
}

#[async_trait]
impl PipelineValidator for GetPaymentStatusFromPaymentProviderV2 {
    #[instrument(name = "validate_get_payment_status_v2", skip(self, event), err(Debug))]
    async fn validate(&self, event: &ServerSideBookingEvent) -> Result<PipelineDecision, String> {
        // Ensure that order_id exists in ServerSideBookingEvent and booking_id can be derived from order_id
        // if event.order_id.is_empty() {
        //     error!("order_id is empty in ServerSideBookingEvent");
        //     return Err("order_id is required but empty".to_string());
        // }

        // // Verify that app_reference can be derived from order_id
        // if PaymentIdentifiers::app_reference_from_order_id(&event.order_id).is_none() {
        //     error!(
        //         "Failed to extract app_reference from order_id: {}",
        //         event.order_id
        //     );
        //     return Err(format!(
        //         "Failed to extract app_reference from order_id: {}",
        //         event.order_id
        //     ));
        // }

        if event.payment_id.is_some() {
            // Payment ID provided - check external provider flow
            if let Some(ref backend_booking) = event.backend_booking_struct {
                // Booking already loaded, check payment status
                Ok(Self::check_payment_status_decision(
                    &backend_booking.payment_details.payment_status,
                ))
            } else {
                // Booking not loaded yet, but we need to check payment status
                // // Load booking to determine if we should skip or run
                // let booking =
                //     Self::load_booking_from_backend(&event.order_id, &event.user_email).await?;
                // Ok(Self::check_payment_status_decision(
                //     &booking.payment_details.payment_status,
                // ))
                Ok(PipelineDecision::Run)
            }
        } else {
            // No payment ID - we'll extract status from backend_booking_struct
            info!("No payment_id provided, will extract payment status from backend booking data");
            Ok(PipelineDecision::Run)
        }
    }
}

#[async_trait]
impl PipelineExecutor for GetPaymentStatusFromPaymentProviderV2 {
    #[instrument(
        name = "execute_get_payment_status_v2",
        skip(event, notifier),
        err(Debug)
    )]
    async fn execute(
        event: ServerSideBookingEvent,
        notifier: Option<&Notifier>,
    ) -> Result<ServerSideBookingEvent, String> {
        use crate::api::payments::domain::{DomainGetPaymentStatusRequest, PaymentStatus};

        let mut updated_event = event;

        // Determine the flow based on payment_id and order_id state
        let (payment_status, is_finished, source, domain_response_opt) = if updated_event
            .payment_id
            .is_some()
            && updated_event.order_id.is_empty()
        {
            // Flow 1a: payment_id exists but order_id is empty - get order_id from payment provider
            let payment_id_str = updated_event.payment_id.as_ref().unwrap();
            info!(
                    "Flow 1a: order_id is empty, checking payment status from external provider for payment_id: {} to extract order_id",
                    payment_id_str
                );

            // Use the unified payment service with 10-minute retry logic
            let payment_service = PaymentServiceImpl::new();
            let domain_request: DomainGetPaymentStatusRequest = DomainGetPaymentStatusRequest {
                payment_id: payment_id_str.clone(),
                provider: None, // Let the service auto-detect the provider
            };

            let domain_response = check_payment_status_with_retry_v2(
                payment_service,
                domain_request,
                updated_event.order_id.clone(),
                Duration::from_secs(600), // 10 minutes
            )
            .await?;

            // Extract order_id from domain response
            let extracted_order_id = domain_response
                .order_id
                .as_ref()
                .ok_or_else(|| "order_id is None in domain response, cannot proceed".to_string())?;

            info!(
                "Extracted order_id from domain response: {}",
                extracted_order_id
            );

            let booking_id = FrontendBookingId::from_order_id(&extracted_order_id)
                .ok_or_else(|| "Failed to extract booking_id from order_id".to_string())?;

            // Update event with extracted order_id
            updated_event.order_id = extracted_order_id.clone();
            if updated_event.user_email.is_empty() {
                updated_event.user_email = booking_id.email.clone();
            }

            info!(
                "updated_event: {:?}, extraced_booking_id: {:?}",
                updated_event, booking_id
            );

            // Now load booking from backend using extracted order_id
            let booking =
                Self::load_booking_from_backend(&extracted_order_id, &booking_id.email).await?;

            info!(
                "Loaded booking from backend using extracted order_id: {}",
                updated_event.order_id
            );
            updated_event.backend_booking_struct = Some(booking);

            let finished = matches!(domain_response.status, PaymentStatus::Completed);

            info!(
                    "Domain response received (order_id was empty): status={:?}, provider={:?}, finished={}",
                    domain_response.status, domain_response.provider, finished
                );

            updated_event.payment_status = Some(domain_response.status.as_str().to_string());
            (
                domain_response.status.as_str().to_string(),
                finished,
                domain_response.provider.as_str().to_string(),
                Some(domain_response),
            )
        } else {
            // Flow 1b & 2: order_id exists OR no payment_id - load booking from backend and conditionally check payment provider
            info!(
                "Flow 1b/2: loading booking from backend for order_id: {} and payment_id: {:?}",
                updated_event.order_id, updated_event.payment_id
            );

            // Ensure order_id exists for backend loading
            if updated_event.order_id.is_empty() {
                return Err(
                    "order_id is required but empty for backend booking loading".to_string()
                );
            }

            // Load booking from backend if not already loaded
            if updated_event.backend_booking_struct.is_none() {
                info!(
                    "updated_event.backend_booking_struct is none: updated_event: {:?}",
                    updated_event
                );
                let booking = Self::load_booking_from_backend(
                    &updated_event.order_id,
                    &updated_event.user_email,
                )
                .await?;

                info!(
                    "Loaded booking from backend for order_id: {}, backend_booking_struct: {:?}",
                    updated_event.order_id, booking
                );
                updated_event.backend_booking_struct = Some(booking);
            }

            // Get reference to backend booking
            let backend_booking = updated_event
                .backend_booking_struct
                .as_ref()
                .ok_or_else(|| "Backend booking not loaded after loading attempt".to_string())?;

            // Check if we should call external payment provider based on backend status
            let domain_response_opt = Self::check_payment_provider_if_unpaid(
                updated_event.payment_id.as_deref(),
                backend_booking,
                &updated_event.order_id,
            )
            .await?;

            // Determine final payment status based on domain response or backend booking
            if let Some(domain_response) = domain_response_opt {
                // External provider returned a response - use it
                let finished = matches!(domain_response.status, PaymentStatus::Completed);

                info!(
                        "Using external payment provider response: status={:?}, provider={:?}, finished={}",
                        domain_response.status, domain_response.provider, finished
                    );

                updated_event.payment_status = Some(domain_response.status.as_str().to_string());
                (
                    domain_response.status.as_str().to_string(),
                    finished,
                    domain_response.provider.as_str().to_string(),
                    Some(domain_response),
                )
            } else {
                // No external provider check - use backend booking status
                let (status, finished) = match &backend_booking.payment_details.payment_status {
                    BackendPaymentStatus::Paid(_) => ("finished".to_string(), true),
                    BackendPaymentStatus::Unpaid(_) => ("waiting".to_string(), false),
                };

                info!("Using backend payment status: {}", status);

                let source = backend_booking
                    .payment_details
                    .payment_api_response
                    .provider
                    .clone();

                updated_event.payment_status = Some(status.clone());
                (status, finished, source, None)
            }
        };

        info!(
            "Payment status determined (V2): {} (finished: {}, source: {})",
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
                step_name: Some("GetPaymentStatusFromPaymentProviderV2".to_string()),
                event_type: NotifierEventType::PaymentStatusChecked {
                    status: payment_status.clone(),
                    is_finished,
                },
                email: updated_event.user_email.clone(),
            };
            info!("Emitting PaymentStatusChecked event (V2): {custom_event:#?}");
            n.notify(custom_event).await;
        }
        // --- END EMIT CUSTOM EVENT ---

        // Update payment status in backend (same logic as V1)
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

        // Create BePaymentApiResponse using helper function with enriched data
        let payment_api_response = create_payment_api_response_from_booking(
            domain_response_opt.as_ref(),
            updated_event.backend_booking_struct.as_ref().unwrap(),
            &updated_event,
            payment_status.clone(),
            source.clone(),
        )
        .map_err(|e| format!("Failed to create payment API response: {}", e))?;

        // Create PaymentDetails with the appropriate payment status using Domain response
        let backend_payment_status = if let Some(ref domain_resp) = domain_response_opt {
            info!("Domain response status: {:?}", domain_resp.status);
            if is_payment_completed_domain(&domain_resp.status) {
                info!("Payment is completed according to domain response, setting to Paid");
                BackendPaymentStatus::Paid(payment_status.clone())
            } else {
                info!("Payment is not completed according to domain response, setting to Unpaid");
                BackendPaymentStatus::Unpaid(Some(payment_status.clone()))
            }
        } else {
            // This should not happen in V2 since we always have domain response from external providers
            // Log warning and default to Unpaid
            warn!("No domain response available for payment status determination");
            BackendPaymentStatus::Unpaid(Some(payment_status.clone()))
        };

        info!("Final backend payment status: {:?}", backend_payment_status);

        let payment_details = PaymentDetails {
            payment_status: backend_payment_status,
            booking_id: booking_id.clone(),
            payment_api_response,
        };

        // Validate payment details before sending to backend
        if let BackendPaymentStatus::Paid(_) = payment_details.payment_status {
            info!(
                "✅ Sending PAID status to backend: {:?}",
                payment_details.payment_status
            );
        } else {
            warn!(
                "⚠️ Sending non-PAID status to backend: {:?}",
                payment_details.payment_status
            );
        }

        let admin_canister = AdminCanisters::from_env();
        let backend = admin_canister.backend_canister().await;

        // Log what we're sending to backend
        info!(
            "Calling backend update_payment_details with booking_id: {:?}, payment_details: {:?}",
            booking_id, payment_details
        );

        // Update payment details in backend and verify the response
        let updated_booking = backend
            .update_payment_details(booking_id, payment_details)
            .await
            .map_err(|e| format!("call backend canister to update payment details: {}", e))?;

        // Verify that payment status is correctly set and log entire booking
        match updated_booking {
            Result3::Ok(booking) => {
                info!(
                    "Backend update SUCCESS - returned booking payment status: {:?}",
                    booking.payment_details.payment_status
                );
                info!(
                    "Backend update SUCCESS - returned booking ID: {:?}",
                    booking.payment_details.booking_id
                );
                info!(
                    "Backend update SUCCESS - ENTIRE BOOKING STRUCT: {:#?}",
                    booking
                );

                // Additional validation
                if matches!(
                    booking.payment_details.payment_status,
                    BackendPaymentStatus::Paid(_)
                ) {
                    info!("✅ Backend correctly updated payment status to PAID");
                } else {
                    error!(
                        "❌ Backend failed to update payment status - still shows: {:?}",
                        booking.payment_details.payment_status
                    );
                }
            }
            Result3::Err(e) => {
                error!("Backend update FAILED: {}", e);
                return Err(format!("Failed to update payment details: {}", e));
            }
        }

        // Update the event with the latest payment status and ensure email is set
        updated_event.user_email = booking_id_clone.email;
        updated_event.payment_status = Some(payment_status.clone());

        // Log the successful update
        info!(
            "Successfully updated payment details in backend (V2). Payment status: {}",
            payment_status
        );

        Ok(updated_event)
    }
}

/// V2 retry logic using unified PaymentServiceImpl with 10-minute timeout and exponential backoff
async fn check_payment_status_with_retry_v2(
    payment_service: PaymentServiceImpl,
    request: crate::api::payments::domain::DomainGetPaymentStatusRequest,
    order_id: String,
    max_timeout: Duration,
) -> Result<crate::api::payments::domain::DomainGetPaymentStatusResponse, String> {
    let start_time = Instant::now();
    let mut retry_interval = Duration::from_secs(5); // Start with 5 seconds
    let max_retry_interval = Duration::from_secs(30); // Cap at 30 seconds
    let backoff_factor = 1.5;

    info!(
        "Starting V2 payment status check with timeout of {:?} for order_id: {}",
        max_timeout, order_id
    );

    loop {
        // Check if we've exceeded the timeout
        if start_time.elapsed() > max_timeout {
            error!(
                "Payment verification timeout exceeded for order_id: {}, payment_id: {}",
                order_id, request.payment_id
            );
            return Err("Payment verification timeout exceeded".to_string());
        }

        info!(
            "Checking payment status (V2) for order_id: {}, payment_id: {}, elapsed: {:?}",
            order_id,
            request.payment_id,
            start_time.elapsed()
        );

        match payment_service.get_payment_status(request.clone()).await {
            Ok(domain_response) => {
                info!(
                    "Payment status (V2) for order_id {}: {:?} (provider: {:?})",
                    order_id, domain_response.status, domain_response.provider
                );
                info!("Full domain response: {:?}", domain_response);

                // Check if payment is in a final state
                match domain_response.status {
                    crate::api::payments::domain::PaymentStatus::Completed
                    | crate::api::payments::domain::PaymentStatus::Failed
                    | crate::api::payments::domain::PaymentStatus::Refunded
                    | crate::api::payments::domain::PaymentStatus::Expired
                    | crate::api::payments::domain::PaymentStatus::Cancelled => {
                        info!("Payment reached final state: {:?}", domain_response.status);
                        return Ok(domain_response);
                    }
                    crate::api::payments::domain::PaymentStatus::Pending
                    | crate::api::payments::domain::PaymentStatus::Unknown(_) => {
                        info!(
                            "Payment still pending for order_id: {}, retrying in {:?}",
                            order_id, retry_interval
                        );
                    }
                }
            }
            Err(e) => {
                warn!(
                    "Payment status check failed (V2) for order_id: {}, error: {}, retrying in {:?}",
                    order_id, e, retry_interval
                );
            }
        }

        // Sleep before next retry
        tokio::time::sleep(retry_interval).await;

        // Increase retry interval with exponential backoff
        retry_interval = std::cmp::min(
            Duration::from_secs((retry_interval.as_secs() as f64 * backoff_factor) as u64),
            max_retry_interval,
        );
    }
}
