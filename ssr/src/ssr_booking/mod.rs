pub mod booking_handler;
pub mod mock_handler;
pub mod payment_handler;
pub mod pipeline;
pub mod pipeline_lock;
pub use pipeline_lock::PipelineLockManager;

mod pipeline_integration_test;

use crate::ssr_booking::pipeline::PipelineExecutor;
use crate::ssr_booking::pipeline::PipelineValidator;
use mock_handler::MockStep;
use payment_handler::GetPaymentStatusFromPaymentProvider;
use pipeline::PipelineDecision;
use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};

// --------------------------
// Data Structures & Enums
// --------------------------

#[derive(Debug, Clone)]
pub struct ServerSideBookingEvent {
    pub payment_id: Option<String>,
    pub provider: String,
    // pub booking_id: String,
    /// order id received from payment provider -> can be used to derive app_reference
    pub order_id: String,
    pub user_email: String,
    pub payment_status: Option<String>,
    pub backend_payment_status: Option<String>,
}

// --------------------------
// PipelineStep Enum Wrapper
// --------------------------

#[derive(Debug, Clone)]
pub enum SSRBookingPipelineStep {
    PaymentStatus(GetPaymentStatusFromPaymentProvider),
    // BookingCall(CreateBookingCallForTravelProvider),
    /// for testing purposes
    Mock(MockStep),
}

impl SSRBookingPipelineStep {
    /// Delegates validation to the inner type.
    pub async fn validate(
        &self,
        event: &ServerSideBookingEvent,
    ) -> Result<PipelineDecision, String> {
        match self {
            SSRBookingPipelineStep::PaymentStatus(step) => step.validate(event).await,
            // SSRBookingPipelineStep::BookingCall(step) => step.validate(event).await,
            SSRBookingPipelineStep::Mock(step) => step.validate(event).await,
        }
    }

    /// For execution, we call the static execute function (ignoring any internal state)
    /// except for the Mock step where we want to record that execution was attempted.
    pub async fn execute(
        &self,
        event: ServerSideBookingEvent,
    ) -> Result<ServerSideBookingEvent, String> {
        match self {
            SSRBookingPipelineStep::PaymentStatus(_) => {
                GetPaymentStatusFromPaymentProvider::execute(event).await
            }
            // PipelineStep::BookingCall(_) => {
            //     CreateBookingCallForTravelProvider::execute(event).await
            // }
            SSRBookingPipelineStep::Mock(step) => {
                step.executed.store(true, Ordering::SeqCst);
                Ok(event)
            }
        }
    }
}

impl fmt::Display for &SSRBookingPipelineStep {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SSRBookingPipelineStep::PaymentStatus(_) => {
                write!(f, "GetPaymentStatusFromPaymentProvider")
            }
            // SSRBookingPipelineStep::BookingCall(_) => write!(f, "CreateBookingCallForTravelProvider"),
            SSRBookingPipelineStep::Mock(_) => write!(f, "MockStep"),
        }
    }
}
