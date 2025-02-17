use async_trait::async_trait;
use std::time::{Duration, Instant};
use tokio::time;

use crate::api::payments::ports::{GetPaymentStatusRequest, GetPaymentStatusResponse};
use crate::api::payments::NowPayments;
use crate::ssr_booking::pipeline::{PipelineExecutor, PipelineValidator};
use crate::ssr_booking::{PipelineDecision, ServerSideBookingEvent};

// ---------------------
// external api calls
// ---------------------

pub async fn nowpayments_get_payment_status(
    request: GetPaymentStatusRequest,
) -> Result<GetPaymentStatusResponse, String> {
    let nowpayments = NowPayments::default();
    println!("{:#?}", request);
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
    let max_timeout = Duration::from_secs(10 * 60); // 10 minutes
    let start_time = Instant::now();

    loop {
        // Attempt to get the payment status
        match nowpayments_get_payment_status(request.clone()).await {
            Ok(response) => return Ok(response),
            Err(e) => {
                // Check if we've exceeded the timeout
                if start_time.elapsed() >= max_timeout {
                    return Err(format!("Timed out waiting for payment status: {}", e));
                }
                println!("Payment status : {}. Retrying in 5 seconds...", e);
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
    async fn validate(&self, event: &ServerSideBookingEvent) -> Result<PipelineDecision, String> {
        if event.payment_id.is_some() {
            Ok(PipelineDecision::Run)
        } else {
            Ok(PipelineDecision::Skip)
        }
    }
}

#[async_trait]
impl PipelineExecutor for GetPaymentStatusFromPaymentProvider {
    async fn execute(event: ServerSideBookingEvent) -> Result<ServerSideBookingEvent, String> {
        println!("Executing GetPaymentStatusFromPaymentProvider");
        Ok(event)
    }
}
