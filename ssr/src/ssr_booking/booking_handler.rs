use async_trait::async_trait;
use std::time::{Duration, Instant};
use tokio::time;
use tracing::{debug, error, info, warn};

use crate::api::payments::ports::{GetPaymentStatusRequest, GetPaymentStatusResponse};
use crate::api::payments::NowPayments;
use crate::ssr_booking::pipeline::{PipelineExecutor, PipelineValidator};
use crate::ssr_booking::{PipelineDecision, ServerSideBookingEvent};

// ---------------------
// external api calls
// ---------------------

async fn book_room(event: ServerSideBookingEvent) -> Result<ServerSideBookingEvent, String> {
    info!("Booking room");
    // 1. get the blocked room from backend
    // 2. use those room details to book room from booking provider
    // 3. store back the reseults in backend

    Ok(event)
}
// ---------------------
// PIPELINE INTEGRATION for backend provider as a step in pipeline
// ---------------------

#[derive(Debug, Clone)]
pub struct MakeBookingFromBookingProvider;

impl MakeBookingFromBookingProvider {
    pub async fn execute(event: ServerSideBookingEvent) -> Result<ServerSideBookingEvent, String> {
        info!("Executing MakeBookingFromBookingProvider");
        // step 1: update the backend that book_room api will be called now.
        // let booking_id = BookingId {
        //     app_reference: event.order_id.clone(),
        //     email: event.user_email.clone(),
        // };

        // // Update the backend with a message indicating that we're about to call book_room API
        // let backend = event.backend.as_ref().ok_or("Backend not initialized")?;
        // backend.update_booking_message(booking_id, "Initiating room booking process...".to_string())
        //     .await
        //     .map_err(|e| format!("Failed to update booking message: {}", e))?;

        // step 2: call the book_room API from file a04_book_room.rs
        // step 3: check the response from API,

        // if success, update the backend with relevant data
        //
        // #[derive(CandidType, Deserialize, Default, Serialize, Clone, Debug)]
        // pub struct BEBookRoomResponse {
        //     pub status: String,
        //     pub message: String,
        //     pub commit_booking: BookingDetails,
        // }

        // if failure in book_room status, also update the above.
        // if API failure happens (API response.status != 200 -- see a04_book_room.rs and client.rs for more details) - return error from pipeline

        // step 4: if backend is updated with BEBookRoomResponse, return updated event

        // make the task list and keep updating scratchpad.md accordingly.
        //

        Ok(event)
    }
}

#[async_trait]
impl PipelineValidator for MakeBookingFromBookingProvider {
    async fn validate(&self, event: &ServerSideBookingEvent) -> Result<PipelineDecision, String> {
        // Check if all required fields are present
        if event.order_id.is_empty() {
            return Err("Order ID is missing".to_string());
        }

        if event.payment_id.is_none() {
            return Err("Payment ID is missing".to_string());
        }

        if event.user_email.is_empty() {
            return Err("User email is missing".to_string());
        }

        if event.payment_status.is_none() {
            return Err("Payment status is missing".to_string());
        }

        if event.backend_payment_status.is_none() {
            return Err("Backend payment status is missing".to_string());
        }

        // Check payment status conditions
        let payment_status = event.payment_status.as_ref().unwrap();
        let backend_payment_status = event.backend_payment_status.as_ref().unwrap();

        if payment_status != "finished" {
            return Err(format!(
                "Payment status is not finished: {}",
                payment_status
            ));
        }

        if backend_payment_status != "PAID" {
            return Err(format!(
                "Backend payment status is not PAID: {}",
                backend_payment_status
            ));
        }

        // step : do the backend API call with the booking_id to check book_room details
        // if the backend shows that the room is booked, throw error indicating the BookingStatus

        Ok(PipelineDecision::Run)
    }
}

#[async_trait]
impl PipelineExecutor for MakeBookingFromBookingProvider {
    async fn execute(event: ServerSideBookingEvent) -> Result<ServerSideBookingEvent, String> {
        MakeBookingFromBookingProvider::execute(event).await
    }
}
