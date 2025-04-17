use async_trait::async_trait;
use std::time::{Duration, Instant};
use tokio::time;
use tracing::{debug, error, info, instrument, warn};

use crate::api::canister::book_room_details::call_update_book_room_details_backend;
use crate::api::canister::get_user_booking::get_user_booking_backend;
use crate::api::payments::ports::{GetPaymentStatusRequest, GetPaymentStatusResponse};
use crate::api::payments::NowPayments;
use crate::api::{
    book_room as book_room_api, create_backend_book_room_response,
    user_details_to_passenger_details, BookRoomRequest, BookRoomResponse, BookingDetails,
    BookingStatus, RoomDetail,
};
use crate::canister::backend::{self, BeBookRoomResponse, Booking};
use crate::ssr_booking::pipeline::{PipelineExecutor, PipelineValidator};
use crate::ssr_booking::{PipelineDecision, ServerSideBookingEvent};
use crate::utils::app_reference::BookingId;
use crate::utils::booking_id::PaymentIdentifiers;

// ---------------------
// external api calls
// ---------------------

#[instrument(
    name = "book_room_and_update_backend",
    skip(event, backend_booking),
    err(Debug)
)]
async fn book_room_and_update_backend(
    event: ServerSideBookingEvent,
    backend_booking: backend::Booking,
) -> Result<ServerSideBookingEvent, String> {
    info!("Booking room");
    // 1. get the blocked room from backend (from event or by fetching booking details)
    // For this example, assume booking details are already fetched and available in event or context

    let app_ref =
        PaymentIdentifiers::app_reference_from_order_id(&event.order_id).ok_or_else(|| {
            format!(
                "Failed to extract app_reference from order_id: {}",
                event.order_id
            )
        })?;

    let booking_id = BookingId {
        app_reference: app_ref.clone(),
        email: event.user_email.clone(),
    };

    info!("Booking ID: {booking_id:?}");

    // 2. use those room details to book room from booking provider
    // Build BookRoomRequest using backend_booking
    let passenger_details = user_details_to_passenger_details(&backend_booking.guests);

    let frontend_room_details = vec![RoomDetail { passenger_details }];

    info!("Frontend room details: {frontend_room_details:?}");

    let book_room_request = BookRoomRequest {
        result_token: backend_booking
            .user_selected_hotel_room_details
            .hotel_details
            .hotel_token
            .clone(),
        block_room_id: backend_booking
            .user_selected_hotel_room_details
            .hotel_details
            .block_room_id
            .clone(),
        app_reference: app_ref.clone(),
        room_details: frontend_room_details,
    };

    info!("Book room request: {book_room_request:?}");

    let request_json = serde_json::to_string(&book_room_request)
        .map_err(|e| format!("Failed to serialize BookRoomRequest: {e:?}"))?;

    info!("Request JSON: {request_json}");

    let book_room_response_str = book_room_api(request_json)
        .await
        .map_err(|e| format!("book_room API call failed: {e:?}"))?;
    info!("Book room response: {book_room_response_str}");

    let book_room_response: BookRoomResponse = serde_json::from_str(&book_room_response_str)
        .map_err(|e| format!("Failed to deserialize BookRoomResponse: {e:?}"))?;

    // 3. store back the results in backend
    let book_room_backend = create_backend_book_room_response(
        (event.user_email.clone(), app_ref.clone()),
        book_room_response.clone(),
    );

    info!("Book room backend response: {book_room_backend:?}");

    let book_room_backend_saved_status =
        call_update_book_room_details_backend(booking_id.into(), book_room_backend)
            .await
            .ok();

    info!("Book room and backend update complete: {book_room_backend_saved_status:?}");

    // todo update the event with backend booking status and backend payment status
    // let mut updated_event = event;
    // updated_event.backend_booking_status = Some(backend_response);
    // Ok(updated_event)

    // todo (booking_hold) check for the backend booking status -- if BookingOnHold - then keep calling the booking provider for the final status.

    Ok(event)
}

// ---------------------
// PIPELINE INTEGRATION for backend provider as a step in pipeline
// ---------------------

#[derive(Debug, Clone)]
pub struct MakeBookingFromBookingProvider;

impl MakeBookingFromBookingProvider {
    #[instrument(
        name = "make_booking_from_booking_provider_run",
        skip(event),
        err(Debug)
    )]
    pub async fn run(event: ServerSideBookingEvent) -> Result<ServerSideBookingEvent, String> {
        info!("Executing MakeBookingFromBookingProvider");

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

        // 1b. Fetch bookings from backend
        let bookings_opt = get_user_booking_backend(event.user_email.clone())
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

        // 1d. Check payment status must be 'finished' (Paid)
        match &booking.payment_details.payment_status {
            backend::BackendPaymentStatus::Paid(paid_status) => {
                info!("paid_status: {}", paid_status);
                // Continue to next steps
            }
            unpaid_or_failed_status => {
                return Err(format!(
                    "Payment status is not finished/paid - {:?}",
                    unpaid_or_failed_status
                ))
            }
        }

        // 2. Check booking status and handle accordingly
        match &booking.book_room_status {
            Some(book_room_status) => {
                match &book_room_status.commit_booking.resolved_booking_status {
                    backend::ResolvedBookingStatus::BookingConfirmed => {
                        info!("Booking already confirmed, returning result.");
                        return Ok(event);
                    }
                    // todo - fix this later
                    backend::ResolvedBookingStatus::Unknown => {
                        // backend::BookingStatus::PaymentConfirmed | backend::BookingStatus::PaymentPolling => {
                        // backend::BookingStatus::PaymentConfirmed => {
                        info!("Payment confirmed, proceeding with booking provider call");
                        book_room_and_update_backend(event, booking).await
                    }
                    invalid_status @ (backend::ResolvedBookingStatus::BookingCancelled
                    | backend::ResolvedBookingStatus::BookingFailed) => {
                        // invalid_status @ (backend::BookingStatus::PaymentPolling | backend::BookingStatus::BookingCancelled | backend::BookingStatus::BookFailed) => {
                        info!("Cannot proceed - booking status is {:?}", invalid_status);
                        return Err(format!(
                            "Cannot proceed with booking - current status is {:?}",
                            invalid_status
                        ));
                    }
                    backend::ResolvedBookingStatus::BookingOnHold => {
                        info!("Booking is on hold, proceeding with booking provider call");
                        // todo (booking_hold) fix this by doing a periodic call
                        return Err(format!(
                            "Booking is on hold, cannot proceed with booking provider call"
                        ));
                    }
                }
            }
            None => {
                warn!("No book_room_status found for booking");
                return Err("No book_room_status found for booking".to_string());
            }
        }

        // -----------------------------

        // step 1: update the backend that book_room api will be called now.
        // let booking_id = BookingId {
        //     app_reference: event.order_id.clone(),
        //     email: event.user_email.clone(),
        // };
        // -----------------------------

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
    }
}

#[async_trait]
impl PipelineValidator for MakeBookingFromBookingProvider {
    #[instrument(name = "validate_make_booking", skip(self, event), err(Debug))]
    async fn validate(&self, event: &ServerSideBookingEvent) -> Result<PipelineDecision, String> {
        // Check if all required fields are present
        // if event.order_id.is_empty() {
        //     return Err("Order ID is missing".to_string());
        // }

        if event.payment_id.is_none() {
            return Err("Payment ID is missing".to_string());
        }

        if event.user_email.is_empty() {
            return Err("User email is missing".to_string());
        }

        if event.payment_status.is_none() {
            return Err("Payment status is missing".to_string());
        }

        // if event.backend_payment_status.is_none() {
        //     return Err("Backend payment status is missing".to_string());
        // }

        // Check payment status conditions
        let payment_status = event.payment_status.as_ref().unwrap();
        // let backend_payment_status = event.backend_payment_status.as_ref().unwrap();

        if payment_status != "finished" {
            return Err(format!(
                "Payment status is not finished: {}",
                payment_status
            ));
        }

        // if backend_payment_status != "PAID" {
        //     return Err(format!(
        //         "Backend payment status is not PAID: {}",
        //         backend_payment_status
        //     ));
        // }

        // step : do the backend API call with the booking_id to check book_room details
        // if the backend shows that the room is booked, throw error indicating the BookingStatus

        Ok(PipelineDecision::Run)
    }
}

#[async_trait]
impl PipelineExecutor for MakeBookingFromBookingProvider {
    #[instrument(name = "execute_make_booking", skip(event), err(Debug))]
    async fn execute(event: ServerSideBookingEvent) -> Result<ServerSideBookingEvent, String> {
        MakeBookingFromBookingProvider::run(event).await
    }
}

// pub fn create_backend_book_room_response(
//     (email, app_reference): (String, String),
//     book_room_response: BookRoomResponse,
// ) -> BeBookRoomResponse {
//     match book_room_response {
//         BookRoomResponse::Failure(fe_booking_details_fail) => BeBookRoomResponse {
//             commit_booking: backend::BookingDetails::default(),
//             status: fe_booking_details_fail.status.to_string(),
//             message: fe_booking_details_fail.message,
//         },
//         BookRoomResponse::Success(fe_booking_details_success) => {
//             let booking_details = fe_booking_details_success
//                 .commit_booking
//                 .booking_details
//                 .clone();
//             let fe_booking_details: BookingDetails =
//                 fe_booking_details_success.commit_booking.into();

//             let be_booking_details = backend::BookingDetails {
//                 booking_id: backend::BookingId {
//                     email,
//                     app_reference,
//                 },
//                 travelomatrix_id: fe_booking_details.travelomatrix_id,
//                 booking_ref_no: fe_booking_details.booking_ref_no,
//                 booking_status: fe_booking_details.booking_status,
//                 confirmation_no: fe_booking_details.confirmation_no,
//                 api_status: fe_booking_details_success.status.clone().into(),
//                 resolved_booking_status: booking_details
//                     .parse_resolved_booking_status_from_api_response()
//                     .into(),
//             };
//             BeBookRoomResponse {
//                 commit_booking: be_booking_details,
//                 status: fe_booking_details_success.status.to_string(),
//                 message: fe_booking_details_success.message,
//             }
//         }
//     }
// }
