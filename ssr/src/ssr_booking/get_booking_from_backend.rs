use super::{
    pipeline::{PipelineDecision, PipelineExecutor, PipelineValidator},
    ServerSideBookingEvent,
};
use crate::{
    api::canister::get_user_booking::get_user_booking_backend, canister::backend,
    utils::booking_id::PaymentIdentifiers,
};
use tracing::instrument;

#[derive(Debug, Clone)]
pub struct GetBookingFromBackend;

impl GetBookingFromBackend {
    #[instrument(name = "get_booking_from_backend_run", skip(event), err(Debug))]
    pub async fn run(event: ServerSideBookingEvent) -> Result<ServerSideBookingEvent, String> {
        // extract user email
        let user_email = event.user_email.clone();
        // extract booking id
        // let booking_id = event.booking_id.clone();

        // ---------------------------
        // 1a. Derive BookingId from order_id and user_email
        let app_reference = PaymentIdentifiers::app_reference_from_order_id(&event.order_id)
            .ok_or_else(|| {
                format!(
                    "Failed to extract app_reference from order_id: {}",
                    event.order_id
                )
            })?;
        let booking_id = backend::BookingId {
            app_reference,
            email: event.user_email.clone(),
        };

        // get booking from backend
        let bookings_opt = get_user_booking_backend(user_email.clone())
            .await
            .map_err(|e| format!("Failed to fetch booking: ServerFnError =  {}", e))?;

        let bookings = bookings_opt.ok_or_else(|| "No bookings found for user".to_string())?;

        // 1c. Find the booking with the correct BookingId
        let booking = bookings
            .into_iter()
            .find(|b| {
                b.booking_id.app_reference == booking_id.app_reference
                    && b.booking_id.email == booking_id.email
            })
            .ok_or_else(|| "No matching booking found for user".to_string())?;

        // return event with booking
        let mut updated_event = event;
        updated_event.backend_booking_struct = Some(booking);
        Ok(updated_event)
    }
}

#[async_trait::async_trait]
impl PipelineValidator for GetBookingFromBackend {
    #[instrument(
        name = "validate_get_booking_from_backend",
        skip(self, event),
        err(Debug)
    )]
    async fn validate(&self, event: &ServerSideBookingEvent) -> Result<PipelineDecision, String> {
        // ensure user email is non empty

        if event.payment_id.is_none() {
            return Err("Payment ID is missing".to_string());
        }

        if event.user_email.is_empty() {
            return Err("User email is missing".to_string());
        }

        if event.payment_status.is_none() {
            return Err("Payment status is missing".to_string());
        }

        Ok(PipelineDecision::Run)
    }
}

#[async_trait::async_trait]
impl PipelineExecutor for GetBookingFromBackend {
    #[instrument(name = "execute_get_booking_from_backend", skip(event), err(Debug))]
    async fn execute(event: ServerSideBookingEvent) -> Result<ServerSideBookingEvent, String> {
        GetBookingFromBackend::run(event).await
    }
}
