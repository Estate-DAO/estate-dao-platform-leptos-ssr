use axum::{extract::State, http::StatusCode, response::IntoResponse};
use estate_fe::view_state_layer::AppState;
use estate_fe::{
    application_services::HotelService, domain::DomainBookRoomRequest, init::get_liteapi_driver,
};
use serde_json::json;

#[axum::debug_handler]
pub async fn book_room_api_server_fn_route(
    State(_state): State<AppState>,
    body: String,
) -> Result<axum::response::Response, StatusCode> {
    estate_fe::log!(
        "Starting book_room_api_server_fn_route with body: {}",
        &body[0..100.min(body.len())]
    );

    // Parse the JSON request
    let book_request: DomainBookRoomRequest = match serde_json::from_str(&body) {
        Ok(req) => {
            estate_fe::log!("Successfully parsed book room request");
            req
        }
        Err(e) => {
            estate_fe::log!("Failed to parse JSON request: {:?}", e);
            return Ok(axum::Json(json!({
                "error": format!("Invalid JSON request: {}", e)
            }))
            .into_response());
        }
    };

    // Create hotel service with provider from global client
    let provider = get_liteapi_driver();
    let hotel_service = HotelService::new(provider);

    estate_fe::log!(
        "Calling hotel service book_room with block_id: {}",
        book_request.block_id
    );

    // Call the hotel service
    match hotel_service.book_room(book_request).await {
        Ok(book_response) => {
            estate_fe::log!(
                "Successfully booked room. Booking ID: {}",
                book_response.booking_id
            );

            // Return the domain response as JSON
            match serde_json::to_value(&book_response) {
                Ok(json_response) => Ok(axum::Json(json_response).into_response()),
                Err(e) => {
                    estate_fe::log!("Failed to serialize book response: {:?}", e);
                    Ok(axum::Json(json!({
                        "error": format!("Failed to serialize response: {}", e)
                    }))
                    .into_response())
                }
            }
        }
        Err(e) => {
            estate_fe::log!("Book room failed: {:?}", e);

            // Return error response
            Ok(axum::Json(json!({
                "error": format!("Book room failed: {}", e)
            }))
            .into_response())
        }
    }
}
