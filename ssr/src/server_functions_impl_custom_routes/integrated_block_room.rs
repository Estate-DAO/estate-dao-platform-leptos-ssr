use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use estate_fe::view_state_layer::AppState;
use estate_fe::{
    api::canister::add_booking::call_add_booking_backend,
    domain::BookingError,
    utils::{app_reference::BookingId, booking_backend_conversions::BookingBackendConversions},
};
use serde_json::json;

use super::{call_block_room_api, parse_json_request, IntegratedBlockRoomRequest};

#[axum::debug_handler]
#[tracing::instrument(skip(state))]
pub async fn integrated_block_room_api_server_fn_route(
    State(state): State<AppState>,
    body: String,
) -> Response {
    tracing::info!(
        "Starting integrated block room API with body: {}",
        &body[0..100.min(body.len())]
    );

    // Parse the JSON request
    let request: IntegratedBlockRoomRequest = match parse_json_request(&body) {
        Ok(req) => {
            tracing::info!("Successfully parsed integrated block room request");
            req
        }
        Err(_) => {
            // Custom error for this endpoint with specific response format
            let error_response = json!({
                "success": false,
                "message": "Invalid JSON request format",
                "block_room_response": null,
                "booking_id": ""
            });
            return (StatusCode::BAD_REQUEST, error_response.to_string()).into_response();
        }
    };

    // Extract data from request
    let block_room_request = request.block_room_request;
    let booking_id = request.booking_id;
    let email = request.email;
    let hotel_token = request.hotel_token;

    tracing::info!(
        "Processing integrated block room for booking_id: {}, email: {}",
        booking_id,
        email
    );

    // Step 1: Call block room API using shared helper
    let block_result = match call_block_room_api(&state, block_room_request.clone()).await {
        Ok(response) => {
            tracing::info!(
                "Block room API call successful: block_id = {}",
                response.block_id
            );
            response
        }
        Err(e) => {
            tracing::error!("Block room API call failed: {:?}", e);
            let error_response = json!({
                "success": false,
                "message": format!("Block room failed: {}", e),
                "block_room_response": null,
                "booking_id": booking_id
            });
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                error_response.to_string(),
            )
                .into_response();
        }
    };

    // Step 2: Create backend booking using BookingBackendConversions
    let backend_booking_result = async {
        // Parse booking_id to get BookingId
        let booking_id_struct = BookingId::from_order_id(&booking_id).ok_or_else(|| {
            BookingError::ValidationError("Invalid booking_id format".to_string())
        })?;

        // Create backend booking from block room data
        let destination = block_room_request
            .hotel_info_criteria
            .search_criteria
            .destination_city_name;
        let destination_domain = estate_fe::domain::DomainDestination {
            city_id: block_room_request
                .hotel_info_criteria
                .search_criteria
                .destination_city_id,
            city_name: destination.clone(),
            country_code: block_room_request
                .hotel_info_criteria
                .search_criteria
                .destination_country_code,
            country_name: block_room_request
                .hotel_info_criteria
                .search_criteria
                .destination_country_name,
        };

        let date_range = estate_fe::domain::DomainSelectedDateRange {
            start: block_room_request
                .hotel_info_criteria
                .search_criteria
                .check_in_date,
            end: block_room_request
                .hotel_info_criteria
                .search_criteria
                .check_out_date,
        };

        // Extract room details from selected rooms
        let room_details: Vec<estate_fe::domain::DomainRoomData> = block_room_request
            .selected_rooms
            .into_iter()
            .map(|room| room.room_data)
            .collect();

        // Create temporary SelectedDateRange for date formatting
        let formatted_dates = estate_fe::component::SelectedDateRange {
            start: date_range.start,
            end: date_range.end,
        };

        // Extract user details from block room request
        let user_details = block_room_request.user_details;

        // Build hotel details
        let hotel_details = estate_fe::domain::DomainHotelDetails {
            checkin: formatted_dates.dd_month_yyyy_start(),
            checkout: formatted_dates.dd_month_yyyy_end(),
            hotel_name: "Hotel".to_string(), // Could be extracted from request if available
            hotel_code: block_room_request.hotel_info_criteria.token.clone(),
            star_rating: 4,
            description: "Hotel description".to_string(),
            hotel_facilities: vec![],
            address: "Hotel address".to_string(),
            images: vec![],
            all_rooms: room_details
                .iter()
                .map(|room_data| estate_fe::domain::DomainRoomOption {
                    price: block_result.total_price.clone(),
                    room_data: room_data.clone(),
                    meal_plan: None,
                    occupancy_info: None,
                })
                .collect(),
            amenities: vec![],
        };

        let payment_amount = block_result.total_price.room_price;
        let payment_currency = "USD".to_string();
        let block_room_id = Some(block_result.block_id.clone());

        // Create backend booking
        let backend_booking = BookingBackendConversions::create_backend_booking(
            Some(destination_domain),
            date_range,
            room_details,
            hotel_details,
            user_details,
            booking_id_struct,
            payment_amount,
            payment_currency,
            block_room_id,
            hotel_token,
        )?;

        // Validate the booking
        BookingBackendConversions::validate_backend_booking(&backend_booking)?;

        Ok::<_, BookingError>(backend_booking)
    }
    .await;

    let backend_booking = match backend_booking_result {
        Ok(booking) => booking,
        Err(e) => {
            tracing::error!("Failed to create backend booking: {:?}", e);
            let error_response = json!({
                "success": false,
                "message": format!("Failed to create backend booking: {}", e),
                "block_room_response": serde_json::to_value(&block_result).ok(),
                "booking_id": booking_id
            });
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                error_response.to_string(),
            )
                .into_response();
        }
    };

    // Step 3: Save to backend canister
    let canister_result = call_add_booking_backend(email.clone(), backend_booking).await;

    match canister_result {
        Ok(_) => {
            tracing::info!(
                "Successfully saved booking to backend canister for booking_id: {}",
                booking_id
            );

            let success_response = json!({
                "success": true,
                "message": "Room blocked and booking saved successfully",
                "block_room_response": serde_json::to_value(&block_result).ok(),
                "booking_id": booking_id
            });

            (StatusCode::OK, success_response.to_string()).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to save booking to canister: {}", e);

            // Return partial success - block room succeeded but canister save failed
            let error_response = json!({
                "success": false,
                "message": format!("Room blocked successfully, but failed to save to backend: {}", e),
                "block_room_response": serde_json::to_value(&block_result).ok(),
                "booking_id": booking_id
            });

            (StatusCode::PARTIAL_CONTENT, error_response.to_string()).into_response()
        }
    }
}
