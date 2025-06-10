use async_trait::async_trait;
use std::time::{Duration, Instant};
use tokio::time;
use tracing::{debug, error, info, instrument, warn};

use crate::api::canister::book_room_details::call_update_book_room_details_backend;
use crate::api::canister::get_user_booking::get_user_booking_backend;
use crate::api::payments::ports::{GetPaymentStatusRequest, GetPaymentStatusResponse};
use crate::api::payments::NowPayments;
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

    // // 2. use those room details to book room from booking provider
    // // Build BookRoomRequest using backend_booking
    // let passenger_details = user_details_to_passenger_details(&backend_booking.guests);

    // let frontend_room_details = vec![RoomDetail { passenger_details }];

    // info!("Frontend room details: {frontend_room_details:?}");

    // let book_room_request = BookRoomRequest {
    //     result_token: backend_booking
    //         .user_selected_hotel_room_details
    //         .hotel_details
    //         .hotel_token
    //         .clone(),
    //     block_room_id: backend_booking
    //         .user_selected_hotel_room_details
    //         .hotel_details
    //         .block_room_id
    //         .clone(),
    //     app_reference: app_ref.clone(),
    //     room_details: frontend_room_details,
    // };

    // info!("Book room request: {book_room_request:?}");

    // let request_json = serde_json::to_string(&book_room_request)
    //     .map_err(|e| format!("Failed to serialize BookRoomRequest: {e:?}"))?;

    // info!("Request JSON: {request_json}");

    // let book_room_response_str = book_room_api(request_json)
    //     .await
    //     .map_err(|e| format!("book_room API call failed: {e:?}"))?;
    // info!("Book room response: {book_room_response_str}");

    // let book_room_response: BookRoomResponse = serde_json::from_str(&book_room_response_str)
    //     .map_err(|e| format!("Failed to deserialize BookRoomResponse: {e:?}"))?;

    // // 3. store back the results in backend
    // let book_room_backend = create_backend_book_room_response(
    //     (event.user_email.clone(), app_ref.clone()),
    //     book_room_response.clone(),
    // );

    // info!("Book room backend response: {book_room_backend:?}");

    // let book_room_backend_saved_status =
    //     call_update_book_room_details_backend(booking_id.into(), book_room_backend)
    //         .await
    //         .ok();

    // info!("Book room and backend update complete: {book_room_backend_saved_status:?}");

    // // todo update the event with backend booking status and backend payment status
    // // let mut updated_event = event;
    // // updated_event.backend_booking_status = Some(backend_response);
    // // Ok(updated_event)

    // // todo (booking_hold) check for the backend booking status -- if BookingOnHold - then keep calling the booking provider for the final status.
    // // let hotel_booking_detail_response = get_hotel_booking_detail_from_travel_provider_v2(HotelBookingDetailRequest { app_reference: app_ref.clone() })
    // //     .await
    // //     .map_err(|e| format!("Failed in get_hotel_booking_detail_from_travel_provider_v2 for BookingOnHold: {e}")).ok();

    Ok(event)
}

#[instrument(name = "book_room_hotel_details_looped", skip(event), err(Debug))]
async fn book_room_hotel_details_looped(
    event: ServerSideBookingEvent,
) -> Result<ServerSideBookingEvent, String> {
    // get app_reference from event.order_id
    // 1. get the blocked room from backend (from event or by fetching booking details)
    // For this example, assume booking details are already fetched and available in event or context

    // let app_ref =
    //     PaymentIdentifiers::app_reference_from_order_id(&event.order_id).ok_or_else(|| {
    //         format!(
    //             "Failed to extract app_reference from order_id: {}",
    //             event.order_id
    //         )
    //     })?;

    // let booking_id = BookingId {
    //     app_reference: app_ref.clone(),
    //     email: event.user_email.clone(),
    // };

    // let request = HotelBookingDetailRequest {
    //     app_reference: app_ref.clone(),
    // };

    // let hotel_details_response = get_hotel_booking_detail_from_travel_provider_v2(request)
    //     .await
    //     .map_err(|e| format!("Failed to get hotel booking detail: {e}"))?;

    // if !hotel_details_response.status {
    //     return Err(format!(
    //         "Failed to get hotel booking detail: {}",
    //         hotel_details_response.message
    //     ));
    // }

    // // update_hold_booking vector length is zero => return error
    // if hotel_details_response.update_hold_booking.is_empty() {
    //     return Err("Failed to get hotel booking detail: update_hold_booking is empty".to_string());
    // }

    // todo (booking_hold) details are not present in the api

    Ok(event)
}

// ---------------------
// PIPELINE INTEGRATION for backend provider as a step in pipeline
// ---------------------

#[derive(Debug, Clone)]
pub struct MakeBookingFromBookingProvider;

impl MakeBookingFromBookingProvider {
    /// Verifies that the payment status is 'Paid'
    #[instrument(name = "verify_payment_status", skip(payment_status), err(Debug))]
    fn verify_payment_status(payment_status: &backend::BackendPaymentStatus) -> Result<(), String> {
        match payment_status {
            backend::BackendPaymentStatus::Paid(paid_status) => {
                info!("paid_status: {}", paid_status);
                Ok(())
            }
            unpaid_or_failed_status => Err(format!(
                "Payment status is not finished/paid - {:?}",
                unpaid_or_failed_status
            )),
        }
    }

    /// Processes the booking based on its current status
    #[instrument(name = "process_booking_status", skip(event, booking), err(Debug))]
    async fn process_booking_status(
        event: ServerSideBookingEvent,
        booking: backend::Booking,
    ) -> Result<ServerSideBookingEvent, String> {
        let booking_clone = booking.clone();
        match booking.book_room_status {
            Some(book_room_status) => {
                let booking_status = &book_room_status.commit_booking.resolved_booking_status;

                match booking_status {
                    backend::ResolvedBookingStatus::BookingConfirmed => {
                        info!("Booking already confirmed, returning result.");
                        Ok(event)
                    }
                    backend::ResolvedBookingStatus::Unknown => {
                        info!("Payment confirmed, proceeding with booking provider call");
                        book_room_and_update_backend(event, booking_clone).await
                    }
                    backend::ResolvedBookingStatus::BookingOnHold => {
                        info!("Booking is on hold, proceeding with booking provider call");
                        book_room_hotel_details_looped(event).await
                    }
                    invalid_status @ (backend::ResolvedBookingStatus::BookingCancelled
                    | backend::ResolvedBookingStatus::BookingFailed) => {
                        info!("Cannot proceed - booking status is {:?}", invalid_status);
                        Err(format!(
                            "Cannot proceed with booking - current status is {:?}",
                            invalid_status
                        ))
                    }
                }
            }

            None => {
                info!(
                    "booking.book_room_status.is_none() => proceeding with booking provider call"
                );
                book_room_and_update_backend(event, booking_clone).await
            }
        }
    }

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

        let booking_clone = booking.clone();
        // 1d. Verify payment status
        Self::verify_payment_status(&booking.payment_details.payment_status)?;

        Self::process_booking_status(event, booking_clone).await
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
