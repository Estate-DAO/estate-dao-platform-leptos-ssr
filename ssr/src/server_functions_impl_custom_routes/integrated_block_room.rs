use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use estate_fe::view_state_layer::AppState;
use estate_fe::{
    api::canister::add_booking::call_add_booking_backend,
    domain::{
        BookingError, DomainCurrencyAmount, DomainDestination, DomainHotelDetails, DomainRoomData,
        DomainRoomGroup, DomainRoomVariant, DomainSelectedDateRange,
    },
    utils::{app_reference::BookingId, booking_backend_conversions::BookingBackendConversions},
};
use hotel_providers::liteapi::models::booking::LiteApiPrebookResponse;
use serde_json::json;

use super::{
    call_block_room_api, parse_json_request, IntegratedBlockRoomRequest,
    IntegratedBlockRoomResponse,
};

/// HTTP status code for partial success (room blocked but backend save failed)
const PARTIAL_SUCCESS_STATUS: StatusCode = StatusCode::PARTIAL_CONTENT;

/// Default values for hotel details
const DEFAULT_HOTEL_NAME: &str = "Hotel";
const DEFAULT_STAR_RATING: i32 = 4;
const DEFAULT_HOTEL_DESCRIPTION: &str = "Hotel description";
const DEFAULT_HOTEL_ADDRESS: &str = "Hotel address";
const DEFAULT_PAYMENT_CURRENCY: &str = "USD";

#[axum::debug_handler]
#[tracing::instrument(skip(state), fields(booking_id = %"", email = %""))]
pub async fn integrated_block_room_api_server_fn_route(
    State(state): State<AppState>,
    body: String,
) -> Response {
    match process_integrated_block_room_request(state, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

/// Process the integrated block room request with proper error handling
async fn process_integrated_block_room_request(
    state: AppState,
    body: String,
) -> Result<Response, Response> {
    tracing::info!("Processing integrated block room request");

    let request = parse_request(&body)?;

    // Update tracing span with request details
    tracing::Span::current().record("booking_id", &request.booking_id);
    tracing::Span::current().record("email", &request.email);

    let block_result = execute_block_room_operation(&state, &request).await?;
    let backend_booking = create_backend_booking(&state, &request, &block_result).await?;
    let final_response = save_booking_to_backend(&request, backend_booking, &block_result).await?;

    Ok(final_response)
}

/// Parse and validate the incoming request
fn parse_request(body: &str) -> Result<IntegratedBlockRoomRequest, Response> {
    tracing::debug!("Parsing request body: {}", &body[0..100.min(body.len())]);

    parse_json_request(body).map_err(|_| {
        let error_response = IntegratedBlockRoomResponse {
            success: false,
            message: "Invalid JSON request format".to_string(),
            block_room_response: None,
            booking_id: String::new(),
        };
        create_error_response(StatusCode::BAD_REQUEST, error_response)
    })
}

/// Execute the block room operation
async fn execute_block_room_operation(
    state: &AppState,
    request: &IntegratedBlockRoomRequest,
) -> Result<estate_fe::domain::DomainBlockRoomResponse, Response> {
    tracing::info!(
        "Executing block room operation for booking_id: {}",
        request.booking_id
    );

    call_block_room_api(state, request.block_room_request.clone())
        .await
        .map_err(|e| {
            tracing::error!("Block room API call failed: {}", e);
            let error_response = IntegratedBlockRoomResponse {
                success: false,
                message: format!("Block room failed: {}", e),
                block_room_response: None,
                booking_id: request.booking_id.clone(),
            };
            create_error_response(StatusCode::INTERNAL_SERVER_ERROR, error_response)
        })
        .map(|response| {
            tracing::info!(
                "Block room operation successful: block_id = {}",
                response.block_id
            );
            response
        })
}

/// Create backend booking from request and block room response
async fn create_backend_booking(
    state: &estate_fe::view_state_layer::AppState,
    request: &IntegratedBlockRoomRequest,
    block_result: &estate_fe::domain::DomainBlockRoomResponse,
) -> Result<estate_fe::canister::backend::Booking, Response> {
    tracing::info!(
        "Creating backend booking for booking_id: {}",
        request.booking_id
    );

    let booking_id_struct = parse_booking_id(&request.booking_id)?;
    let destination = extract_destination_from_request(request);
    let date_range = extract_date_range_from_request(request);
    let mut room_details = extract_room_details_from_request(request);
    enrich_room_details_with_provider_data(&mut room_details, block_result);
    align_room_details_with_blocked_rooms(&mut room_details, block_result);
    let hotel_details =
        build_hotel_details(state, request, block_result, &date_range, &room_details).await;

    let backend_booking = BookingBackendConversions::create_backend_booking(
        Some(destination),
        date_range,
        room_details,
        hotel_details,
        request.block_room_request.user_details.clone(),
        booking_id_struct,
        block_result.total_price.room_price,
        DEFAULT_PAYMENT_CURRENCY.to_string(),
        Some(block_result.block_id.clone()),
        request.hotel_token.clone(),
    )
    .map_err(|e| {
        tracing::error!("Failed to create backend booking: {}", e);
        let error_response = IntegratedBlockRoomResponse {
            success: false,
            message: format!("Failed to create backend booking: {}", e),
            block_room_response: Some(block_result.clone()),
            booking_id: request.booking_id.clone(),
        };
        create_error_response(StatusCode::INTERNAL_SERVER_ERROR, error_response)
    })?;

    // Validate the booking
    BookingBackendConversions::validate_backend_booking(&backend_booking).map_err(|e| {
        tracing::error!("Backend booking validation failed: {}", e);
        let error_response = IntegratedBlockRoomResponse {
            success: false,
            message: format!("Booking validation failed: {}", e),
            block_room_response: Some(block_result.clone()),
            booking_id: request.booking_id.clone(),
        };
        create_error_response(StatusCode::INTERNAL_SERVER_ERROR, error_response)
    })?;

    Ok(backend_booking)
}

/// Save booking to backend canister
async fn save_booking_to_backend(
    request: &IntegratedBlockRoomRequest,
    backend_booking: estate_fe::canister::backend::Booking,
    block_result: &estate_fe::domain::DomainBlockRoomResponse,
) -> Result<Response, Response> {
    tracing::info!("Saving booking to backend canister");

    match call_add_booking_backend(request.email.clone(), backend_booking).await {
        Ok(_) => {
            tracing::info!("Successfully saved booking to backend canister");
            let success_response = IntegratedBlockRoomResponse {
                success: true,
                message: "Room blocked and booking saved successfully".to_string(),
                block_room_response: Some(block_result.clone()),
                booking_id: request.booking_id.clone(),
            };
            Ok(create_success_response(success_response))
        }
        Err(e) => {
            tracing::error!("Failed to save booking to canister: {}", e);
            let error_response = IntegratedBlockRoomResponse {
                success: false,
                message: format!(
                    "Room blocked successfully, but failed to save to backend: {}",
                    e
                ),
                block_room_response: Some(block_result.clone()),
                booking_id: request.booking_id.clone(),
            };
            Err(create_error_response(
                PARTIAL_SUCCESS_STATUS,
                error_response,
            ))
        }
    }
}

/// Parse booking ID with proper error handling
fn parse_booking_id(booking_id: &str) -> Result<BookingId, Response> {
    BookingId::from_order_id(booking_id).ok_or_else(|| {
        let error = BookingError::ValidationError("Invalid booking_id format".to_string());
        tracing::error!("Booking ID parsing failed: {}", error);
        let error_response = IntegratedBlockRoomResponse {
            success: false,
            message: format!("Invalid booking ID format: {}", error),
            block_room_response: None,
            booking_id: booking_id.to_string(),
        };
        create_error_response(StatusCode::BAD_REQUEST, error_response)
    })
}

/// Extract destination information from request
fn extract_destination_from_request(request: &IntegratedBlockRoomRequest) -> DomainDestination {
    let search_criteria = &request
        .block_room_request
        .hotel_info_criteria
        .search_criteria;

    DomainDestination {
        place_id: search_criteria.place_id.clone(),
        // city_id: search_criteria.destination_city_id,
        // city_name: search_criteria.destination_city_name.clone(),
        // country_code: search_criteria.destination_country_code.clone(),
        // country_name: search_criteria.destination_country_name.clone(),
    }
}

/// Extract date range from request
fn extract_date_range_from_request(
    request: &IntegratedBlockRoomRequest,
) -> DomainSelectedDateRange {
    let search_criteria = &request
        .block_room_request
        .hotel_info_criteria
        .search_criteria;

    DomainSelectedDateRange {
        start: search_criteria.check_in_date,
        end: search_criteria.check_out_date,
    }
}

/// Extract room details from request
fn extract_room_details_from_request(request: &IntegratedBlockRoomRequest) -> Vec<DomainRoomData> {
    request
        .block_room_request
        .selected_rooms
        .iter()
        .flat_map(|room| {
            // Repeat the room_data according to quantity so we track each room instance
            let qty = room.quantity.max(1);
            std::iter::repeat(room.room_data.clone()).take(qty as usize)
        })
        .collect()
}

/// Use provider data from LiteAPI to attach occupancy numbers to selected rooms
fn enrich_room_details_with_provider_data(
    room_details: &mut Vec<DomainRoomData>,
    block_result: &estate_fe::domain::DomainBlockRoomResponse,
) {
    if room_details.is_empty() {
        return;
    }

    let provider_data = match &block_result.provider_data {
        Some(data) => data,
        None => return,
    };

    let prebook_response: LiteApiPrebookResponse = match serde_json::from_str(provider_data) {
        Ok(resp) => resp,
        Err(e) => {
            tracing::warn!("Failed to parse LiteAPI prebook provider data: {}", e);
            return;
        }
    };

    let data = match prebook_response.data {
        Some(data) => data,
        None => return,
    };

    use std::collections::{HashMap, HashSet};
    let mut occupancy_by_rate: HashMap<String, u32> = HashMap::new();
    for room_type in data.room_types {
        for rate in room_type.rates {
            let occupancy = rate.occupancy_number.max(0) as u32;
            occupancy_by_rate.insert(rate.rate_id, occupancy);
        }
    }

    let mut fallback_needed = false;
    let mut used_occupancies: HashSet<u32> = HashSet::new();
    for room in room_details.iter_mut() {
        if let Some(rate_key) = occupancy_by_rate.get(&room.rate_key) {
            room.occupancy_number = Some(*rate_key);
            used_occupancies.insert(*rate_key);
        } else if room.occupancy_number.is_none() {
            fallback_needed = true;
        }
    }

    if fallback_needed {
        for (idx, room) in room_details.iter_mut().enumerate() {
            room.occupancy_number.get_or_insert((idx + 1) as u32);
        }
    }

    // If we have more occupancies from provider than room_details, add clones to match counts
    let missing: Vec<u32> = occupancy_by_rate
        .values()
        .copied()
        .filter(|occ| !used_occupancies.contains(occ))
        .collect();

    if !missing.is_empty() {
        if let Some(template) = room_details.first().cloned() {
            for occ in missing {
                let mut new_room = template.clone();
                new_room.occupancy_number = Some(occ);
                // encode occupancy in room_unique_id to keep entries distinct
                new_room.room_unique_id =
                    estate_fe::utils::backend_default_impl::encode_room_id_with_occupancy(
                        &template.room_unique_id,
                        Some(occ),
                    );
                room_details.push(new_room);
            }
        }
    }
}

/// Ensure we only keep room details for rooms that were actually blocked by the provider.
fn align_room_details_with_blocked_rooms(
    room_details: &mut Vec<DomainRoomData>,
    block_result: &estate_fe::domain::DomainBlockRoomResponse,
) {
    if room_details.is_empty() || block_result.blocked_rooms.is_empty() {
        return;
    }

    use std::collections::HashSet;
    let blocked_codes: HashSet<_> = block_result
        .blocked_rooms
        .iter()
        .map(|room| room.room_code.clone())
        .collect();

    let original_rooms = room_details.clone();
    room_details.retain(|room| blocked_codes.contains(&room.room_unique_id));

    if room_details.is_empty() {
        for blocked in &block_result.blocked_rooms {
            if let Some(original) = original_rooms
                .iter()
                .find(|room| room.room_unique_id == blocked.room_code)
            {
                room_details.push(original.clone());
            } else {
                room_details.push(DomainRoomData {
                    room_name: blocked.room_name.clone(),
                    room_unique_id: blocked.room_code.clone(),
                    mapped_room_id: String::new(),
                    occupancy_number: Some(1),
                    rate_key: String::new(),
                    offer_id: String::new(),
                });
            }
        }
    }
}

/// Fetch actual hotel details from hotel service
async fn fetch_actual_hotel_details(
    _state: &estate_fe::view_state_layer::AppState,
    hotel_criteria: &estate_fe::domain::DomainHotelInfoCriteria,
) -> Result<DomainHotelDetails, String> {
    use estate_fe::{application_services::HotelService, init::get_liteapi_driver};

    tracing::info!("Fetching hotel details for token: {}", hotel_criteria.token);

    // Create the hotel service with LiteApiDriver from global client
    let liteapi_driver = get_liteapi_driver();
    let hotel_service = HotelService::new(liteapi_driver);

    // Get hotel information
    hotel_service
        .get_hotel_details(hotel_criteria.clone())
        .await
        .map_err(|e| format!("Failed to fetch hotel details: {}", e))
}

/// Build hotel details with actual hotel information
async fn build_hotel_details(
    state: &estate_fe::view_state_layer::AppState,
    request: &IntegratedBlockRoomRequest,
    block_result: &estate_fe::domain::DomainBlockRoomResponse,
    date_range: &DomainSelectedDateRange,
    room_details: &[DomainRoomData],
) -> DomainHotelDetails {
    // Create temporary SelectedDateRange for date formatting
    let formatted_dates = estate_fe::component::SelectedDateRange {
        start: date_range.start,
        end: date_range.end,
    };

    let all_rooms: Vec<DomainRoomGroup> = room_details
        .iter()
        .map(|room_data| {
            // Find the corresponding blocked room to get cancellation policy and meal plan
            let blocked_room = block_result
                .blocked_rooms
                .iter()
                .find(|blocked| blocked.room_code == room_data.room_unique_id);

            let variant = DomainRoomVariant {
                offer_id: room_data.offer_id.clone(),
                rate_key: room_data.rate_key.clone(),
                room_name: room_data.room_name.clone(),
                mapped_room_id: room_data.mapped_room_id.clone(),
                room_count: 1,
                room_unique_id: room_data.room_unique_id.clone(),
                occupancy_number: room_data.occupancy_number,
                meal_plan: blocked_room.and_then(|br| br.meal_plan.clone()),
                total_price_for_all_rooms: block_result.total_price.room_price,
                total_price_for_one_room: block_result.total_price.room_price,
                price_per_room_excluding_taxes: block_result.total_price.room_price,
                currency_code: block_result.total_price.currency_code.clone(),
                tax_breakdown: vec![],
                occupancy_info: None,
                cancellation_info: None,
                // NEW: Enhanced rate information (empty defaults for block room context)
                perks: vec![],
                original_price: None,
                board_type_code: None,
                remarks: None,
            };

            DomainRoomGroup {
                mapped_room_id: Some(room_data.mapped_room_id.clone()),
                name: room_data.room_name.clone(),
                min_price: block_result.total_price.room_price,
                currency_code: block_result.total_price.currency_code.clone(),
                images: vec![],
                amenities: vec![],
                bed_types: vec![],
                room_types: vec![variant],
            }
        })
        .collect();

    // Try to fetch actual hotel details
    let (
        actual_hotel_name,
        actual_address,
        actual_description,
        actual_facilities,
        actual_amenities,
        actual_images,
        actual_star_rating,
    ) = match fetch_actual_hotel_details(state, &request.block_room_request.hotel_info_criteria)
        .await
    {
        Ok(hotel_details) => {
            tracing::info!(
                "Successfully fetched hotel details: {} at {}",
                hotel_details.hotel_name,
                hotel_details.address
            );
            (
                hotel_details.hotel_name,
                hotel_details.address,
                hotel_details.description,
                hotel_details.hotel_facilities,
                hotel_details.amenities,
                hotel_details.images,
                hotel_details.star_rating,
            )
        }
        Err(e) => {
            tracing::warn!("Failed to fetch hotel details, using defaults: {}", e);
            (
                DEFAULT_HOTEL_NAME.to_string(),
                DEFAULT_HOTEL_ADDRESS.to_string(),
                DEFAULT_HOTEL_DESCRIPTION.to_string(),
                vec![],
                vec![],
                vec![],
                DEFAULT_STAR_RATING,
            )
        }
    };

    DomainHotelDetails {
        checkin: formatted_dates.dd_month_yyyy_start(),
        checkout: formatted_dates.dd_month_yyyy_end(),
        hotel_name: actual_hotel_name,
        hotel_code: request.block_room_request.hotel_info_criteria.token.clone(),
        star_rating: actual_star_rating,
        rating: None,
        review_count: None,
        categories: Vec::new(),
        description: actual_description,
        hotel_facilities: actual_facilities,
        address: actual_address,
        images: actual_images,
        all_rooms,
        amenities: actual_amenities,
        search_criteria: None,
        search_info: None,
    }
}

/// Create a success response with proper formatting
fn create_success_response(response: IntegratedBlockRoomResponse) -> Response {
    match serde_json::to_string(&response) {
        Ok(json_string) => (StatusCode::OK, json_string).into_response(),
        Err(e) => {
            tracing::error!("Failed to serialize success response: {}", e);
            let fallback_response = json!({
                "success": true,
                "message": "Operation completed but response serialization failed",
                "booking_id": response.booking_id
            });
            (StatusCode::OK, fallback_response.to_string()).into_response()
        }
    }
}

/// Create an error response with proper formatting
fn create_error_response(status: StatusCode, response: IntegratedBlockRoomResponse) -> Response {
    match serde_json::to_string(&response) {
        Ok(json_string) => (status, json_string).into_response(),
        Err(e) => {
            tracing::error!("Failed to serialize error response: {}", e);
            let fallback_response = json!({
                "success": false,
                "message": "An error occurred and response serialization failed",
                "booking_id": response.booking_id
            });
            (status, fallback_response.to_string()).into_response()
        }
    }
}
