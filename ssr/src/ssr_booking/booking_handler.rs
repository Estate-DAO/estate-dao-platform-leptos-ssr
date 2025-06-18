use async_trait::async_trait;
use std::time::{Duration, Instant};
use tokio::time;
use tracing::{debug, error, info, instrument, warn};

use crate::api::canister::book_room_details::call_update_book_room_details_backend;
use crate::api::canister::get_user_booking::{get_booking_by_id_backend, get_user_booking_backend};
use crate::api::payments::ports::{GetPaymentStatusRequest, GetPaymentStatusResponse};
use crate::api::payments::NowPayments;
use crate::canister::backend::{self, BeBookRoomResponse, Booking};
use crate::ssr_booking::pipeline::{PipelineExecutor, PipelineValidator};
use crate::ssr_booking::{PipelineDecision, ServerSideBookingEvent};
use crate::utils::app_reference::BookingId;
use crate::utils::booking_id::PaymentIdentifiers;

// New imports for v1 implementation
use crate::adapters::liteapi_adapter::LiteApiAdapter;
use crate::application_services::hotel_service::HotelService;
use crate::domain::{
    BookingError, DomainBookRoomRequest, DomainBookRoomResponse, DomainBookingContext,
    DomainBookingGuest, DomainBookingHolder, DomainBookingStatus, DomainOriginalSearchInfo,
    DomainPaymentInfo, DomainPaymentMethod, DomainRoomOccupancyForBooking,
};

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
// TRANSFORMATION FUNCTIONS for v1 implementation
// ---------------------

/// Guest-to-room assignment strategy (can be swapped in future)
///
/// Current strategy:
/// 1. At least one adult in each room
/// 2. All children go in the first room
///
/// Input: Total adults, total children, number of rooms
/// Output: Room occupancy distribution
#[instrument(name = "assign_guests_to_rooms")]
fn assign_guests_to_rooms(
    total_adults: u32,
    total_children: u32,
    children_ages: Vec<u8>,
    number_of_rooms: u32,
) -> Result<Vec<DomainRoomOccupancyForBooking>, BookingError> {
    if number_of_rooms == 0 {
        return Err(BookingError::ValidationError(
            "Number of rooms must be at least 1".to_string(),
        ));
    }

    if total_adults < number_of_rooms {
        return Err(BookingError::ValidationError(format!(
            "Not enough adults ({}) for {} rooms (need at least 1 adult per room)",
            total_adults, number_of_rooms
        )));
    }

    let mut room_occupancies = Vec::new();

    // Strategy: Distribute adults evenly, all children in room 1
    let base_adults_per_room = total_adults / number_of_rooms;
    let extra_adults = total_adults % number_of_rooms;

    for room_number in 1..=number_of_rooms {
        // Calculate adults for this room
        let adults_in_room = if room_number <= extra_adults {
            base_adults_per_room + 1 // First few rooms get extra adult
        } else {
            base_adults_per_room
        };

        // All children go in room 1
        let (children_in_room, children_ages_in_room) = if room_number == 1 {
            (total_children, children_ages.clone())
        } else {
            (0, Vec::new())
        };

        room_occupancies.push(DomainRoomOccupancyForBooking {
            room_number,
            adults: adults_in_room,
            children: children_in_room,
            children_ages: children_ages_in_room,
        });

        info!(
            "Room {}: {} adults, {} children",
            room_number, adults_in_room, children_in_room
        );
    }

    Ok(room_occupancies)
}

/// Infer number of rooms from backend booking data
///
/// Strategy:
/// 1. Use room_details count as proxy for number of rooms
/// 2. Default to 1 room if no room details
/// 3. Log inference for debugging
#[instrument(name = "infer_number_of_rooms")]
fn infer_number_of_rooms(backend_booking: &backend::Booking) -> u32 {
    let room_details_count = backend_booking
        .user_selected_hotel_room_details
        .room_details
        .len() as u32;

    let inferred_rooms = if room_details_count > 0 {
        room_details_count
    } else {
        1
    };

    info!(
        "Inferred {} rooms from {} room details (LIMITATION: not from actual search criteria)",
        inferred_rooms, room_details_count
    );

    inferred_rooms
}

/// Format date tuple from backend to string
/// Backend stores dates as (year, month, day) tuples
#[instrument(name = "format_date_from_backend_tuple")]
fn format_date_from_backend_tuple(date_tuple: (u32, u32, u32)) -> String {
    format!(
        "{:04}-{:02}-{:02}",
        date_tuple.0, date_tuple.1, date_tuple.2
    )
}

/// Transform backend::Booking to DomainBookRoomRequest for hotel service
#[instrument(
    name = "backend_booking_to_domain_book_room_request",
    skip(backend_booking),
    err(Debug)
)]
fn backend_booking_to_domain_book_room_request(
    backend_booking: &backend::Booking,
    app_reference: String,
) -> Result<DomainBookRoomRequest, BookingError> {
    // Extract block_room_id as block_id
    let block_id = backend_booking
        .user_selected_hotel_room_details
        .hotel_details
        .block_room_id
        .clone();

    if block_id.is_empty() {
        return Err(BookingError::ValidationError(
            "Block room ID is required for booking".to_string(),
        ));
    }

    // Create booking holder from first adult
    let first_adult = backend_booking.guests.adults.first().ok_or_else(|| {
        BookingError::ValidationError("At least one adult is required".to_string())
    })?;

    let holder = DomainBookingHolder {
        first_name: first_adult.first_name.clone(),
        last_name: first_adult.last_name.clone().unwrap_or_default(),
        email: first_adult.email.clone().unwrap_or_default(),
        phone: first_adult.phone.clone().unwrap_or_default(),
    };

    // Infer number of rooms BEFORE creating guests (LiteAPI requirement)
    let number_of_rooms = infer_number_of_rooms(backend_booking);

    // Validate: Must have at least one adult per room
    if (backend_booking.guests.adults.len() as u32) < number_of_rooms {
        return Err(BookingError::ValidationError(format!(
            "[SERVER VALIDATION ERROR] Need at least {} adults for {} rooms, but only {} provided. Each room requires a primary contact.",
            number_of_rooms, number_of_rooms, backend_booking.guests.adults.len()
        )));
    }

    // FIXED: Create one PRIMARY CONTACT per room (not per adult)
    // LiteAPI Rule: Need exactly one guest per room as the primary contact/room manager
    let guests: Vec<DomainBookingGuest> = backend_booking
        .guests
        .adults
        .iter()
        .take(number_of_rooms as usize) // 🔑 KEY FIX: Limit to room count, not adult count
        .enumerate()
        .map(|(index, adult)| DomainBookingGuest {
            occupancy_number: (index + 1) as u32, // Room number (1, 2, 3...)
            first_name: adult.first_name.clone(),
            last_name: adult.last_name.clone().unwrap_or_default(),
            email: adult.email.clone().unwrap_or_default(),
            phone: adult.phone.clone().unwrap_or_default(),
            remarks: None,
        })
        .collect();

    // Determine guest distribution
    let total_adults = backend_booking.guests.adults.len() as u32;
    let total_children = backend_booking.guests.children.len() as u32;
    let children_ages: Vec<u8> = backend_booking
        .guests
        .children
        .iter()
        .map(|child| child.age)
        .collect();

    // Assign guests to rooms using strategy
    let room_occupancies =
        assign_guests_to_rooms(total_adults, total_children, children_ages, number_of_rooms)?;

    // Construct original search criteria from available backend data
    let original_search_criteria = Some(DomainOriginalSearchInfo {
        hotel_id: backend_booking
            .user_selected_hotel_room_details
            .hotel_details
            .hotel_code
            .clone(),
        checkin_date: format_date_from_backend_tuple(
            backend_booking
                .user_selected_hotel_room_details
                .date_range
                .start,
        ),
        checkout_date: format_date_from_backend_tuple(
            backend_booking
                .user_selected_hotel_room_details
                .date_range
                .end,
        ),
        guest_nationality: None, // Not available in backend
    });

    let booking_context = DomainBookingContext {
        number_of_rooms,
        room_occupancies,
        total_guests: total_adults + total_children,
        original_search_criteria,
    };

    // Set payment method to Wallet for crypto payments
    let payment = DomainPaymentInfo {
        method: DomainPaymentMethod::Wallet,
    };

    Ok(DomainBookRoomRequest {
        block_id,
        holder,
        guests,
        payment,
        guest_payment: None,
        special_requests: None,
        booking_context,
        client_reference: Some(app_reference),
    })
}

/// Transform DomainBookRoomResponse to BeBookRoomResponse for backend storage
#[instrument(
    name = "domain_book_room_response_to_backend_response",
    skip(domain_response),
    err(Debug)
)]
fn domain_book_room_response_to_backend_response(
    domain_response: DomainBookRoomResponse,
    booking_id: backend::BookingId,
) -> Result<BeBookRoomResponse, BookingError> {
    use crate::canister::backend::{BookingDetails, BookingStatus, ResolvedBookingStatus};

    // Map domain status to backend status
    let (api_status, resolved_status) = match domain_response.status {
        crate::domain::DomainBookingStatus::Confirmed => (
            BookingStatus::Confirmed,
            ResolvedBookingStatus::BookingConfirmed,
        ),
        crate::domain::DomainBookingStatus::Pending => (
            BookingStatus::Confirmed,
            ResolvedBookingStatus::BookingOnHold,
        ),
        crate::domain::DomainBookingStatus::Failed => (
            BookingStatus::BookFailed,
            ResolvedBookingStatus::BookingFailed,
        ),
        crate::domain::DomainBookingStatus::Cancelled => (
            BookingStatus::BookFailed,
            ResolvedBookingStatus::BookingCancelled,
        ),
    };

    let status = if matches!(api_status, BookingStatus::Confirmed) {
        "success".to_string()
    } else {
        "failed".to_string()
    };

    let booking_details = BookingDetails {
        api_status,
        booking_ref_no: domain_response.booking_id.clone(),
        booking_status: format!("{:?}", resolved_status),
        confirmation_no: domain_response.hotel_confirmation_code.clone(),
        resolved_booking_status: resolved_status,
        booking_id,
        travelomatrix_id: domain_response.supplier_booking_id.clone(),
    };

    let message = format!(
        "Booking {} - Status: {:?}",
        domain_response.booking_id, domain_response.status
    );

    Ok(BeBookRoomResponse {
        status,
        commit_booking: booking_details,
        message,
    })
}

/// Initialize hotel service with liteapi adapter
#[instrument(name = "create_hotel_service_with_liteapi")]
fn create_hotel_service_with_liteapi() -> HotelService<LiteApiAdapter> {
    let liteapi_client = crate::api::liteapi::client::LiteApiHTTPClient::default();
    let liteapi_adapter = LiteApiAdapter::new(liteapi_client);
    HotelService::init(liteapi_adapter)
}

/// New version of book_room_and_update_backend with full hotel service integration
#[instrument(
    name = "book_room_and_update_backend_v1",
    skip(event, backend_booking),
    err(Debug)
)]
async fn book_room_and_update_backend_v1(
    event: ServerSideBookingEvent,
    backend_booking: backend::Booking,
) -> Result<ServerSideBookingEvent, String> {
    info!("Starting book_room_and_update_backend_v1");

    // Extract app_reference from order_id
    let app_ref =
        PaymentIdentifiers::app_reference_from_order_id(&event.order_id).ok_or_else(|| {
            format!(
                "Failed to extract app_reference from order_id: {}",
                event.order_id
            )
        })?;

    let booking_id = backend::BookingId {
        app_reference: app_ref.clone(),
        email: event.user_email.clone(),
    };

    info!("Processing booking with ID: {booking_id:?}");

    // Step 1: Transform backend booking to domain request
    let domain_request =
        backend_booking_to_domain_book_room_request(&backend_booking, app_ref.clone())
            .map_err(|e| format!("Failed to transform backend booking to domain request: {e:?}"))?;

    info!("Transformed to domain request: {domain_request:?}");

    // Step 2: Initialize hotel service and call book_room
    let hotel_service = create_hotel_service_with_liteapi();
    let domain_response = hotel_service
        .book_room(domain_request)
        .await
        .map_err(|e| format!("Hotel service book_room failed: {e:?}"))?;

    info!("Received domain response: {domain_response:?}");

    // Step 3: Transform domain response to backend response
    let backend_response =
        domain_book_room_response_to_backend_response(domain_response, booking_id.clone())
            .map_err(|e| {
                format!("Failed to transform domain response to backend response: {e:?}")
            })?;

    info!("Transformed to backend response: {backend_response:?}");

    // Step 4: Save results to backend
    let save_result = call_update_book_room_details_backend(booking_id, backend_response)
        .await
        .map_err(|e| format!("Failed to save booking results to backend: {e}"))?;

    info!("Backend save result: {save_result}");

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
                        info!("Payment confirmed, proceeding with booking provider call v1");
                        book_room_and_update_backend_v1(event, booking_clone).await
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
                    "booking.book_room_status.is_none() => proceeding with booking provider call v1"
                );
                book_room_and_update_backend_v1(event, booking_clone).await
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

        // 1b. Fetch booking by ID directly from backend
        let booking = get_booking_by_id_backend(booking_id.clone())
            .await
            .map_err(|e| format!("Failed to fetch booking: ServerFnError = {}", e))?
            .ok_or_else(|| "No booking found with the specified booking ID".to_string())?;

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
