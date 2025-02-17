pub mod mock_handler;
pub mod payment_handler;
pub mod pipeline;

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
    pub booking_id: String,
    pub user_email: String,
}

// --------------------------
// PipelineStep Enum Wrapper
// --------------------------

#[derive(Debug, Clone)]
pub enum PipelineStep {
    PaymentStatus(GetPaymentStatusFromPaymentProvider),
    // BookingCall(CreateBookingCallForTravelProvider),
    /// for testing purposes
    Mock(MockStep),
}

impl PipelineStep {
    /// Delegates validation to the inner type.
    pub async fn validate(
        &self,
        event: &ServerSideBookingEvent,
    ) -> Result<PipelineDecision, String> {
        match self {
            PipelineStep::PaymentStatus(step) => step.validate(event).await,
            // PipelineStep::BookingCall(step) => step.validate(event).await,
            PipelineStep::Mock(step) => step.validate(event).await,
        }
    }

    /// For execution, we call the static execute function (ignoring any internal state)
    /// except for the Mock step where we want to record that execution was attempted.
    pub async fn execute(
        &self,
        event: ServerSideBookingEvent,
    ) -> Result<ServerSideBookingEvent, String> {
        match self {
            PipelineStep::PaymentStatus(_) => {
                GetPaymentStatusFromPaymentProvider::execute(event).await
            }
            // PipelineStep::BookingCall(_) => {
            //     CreateBookingCallForTravelProvider::execute(event).await
            // }
            PipelineStep::Mock(step) => {
                step.executed.store(true, Ordering::SeqCst);
                Ok(event)
            }
        }
    }
}

impl fmt::Display for &PipelineStep {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PipelineStep::PaymentStatus(_) => write!(f, "GetPaymentStatusFromPaymentProvider"),
            // PipelineStep::BookingCall(_) => write!(f, "CreateBookingCallForTravelProvider"),
            PipelineStep::Mock(_) => write!(f, "MockStep"),
        }
    }
}
