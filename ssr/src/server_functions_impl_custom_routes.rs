use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use estate_fe::view_state_layer::AppState;
use estate_fe::{
    adapters::LiteApiAdapter,
    adapters::ProvabAdapter,
    api::liteapi::LiteApiHTTPClient,
    application_services::HotelService,
    domain::{
        DomainBlockRoomRequest, DomainBlockRoomResponse, DomainBookRoomRequest,
        DomainBookRoomResponse, DomainHotelDetails, DomainHotelInfoCriteria,
        DomainHotelListAfterSearch, DomainHotelSearchCriteria,
    },
    ports::traits::HotelProviderPort,
};
use serde_json::json;

// Helper function to filter hotels with valid pricing
fn filter_hotels_with_valid_pricing(
    mut search_result: DomainHotelListAfterSearch,
) -> DomainHotelListAfterSearch {
    let original_count = search_result.hotel_results.len();
    let hotels_without_pricing = search_result
        .hotel_results
        .iter()
        .filter(|hotel| hotel.price.room_price <= 0.0)
        .count();

    search_result
        .hotel_results
        .retain(|hotel| hotel.price.room_price > 0.0);

    let final_count = search_result.hotel_results.len();

    if hotels_without_pricing > 0 {
        tracing::info!(
            "API search filtering: Found {} hotels total, {} without pricing ({}%), {} with valid pricing returned to client",
            original_count,
            hotels_without_pricing,
            if original_count > 0 { (hotels_without_pricing * 100) / original_count } else { 0 },
            final_count
        );
    } else if original_count > 0 {
        tracing::info!(
            "API search filtering: All {} hotels had valid pricing",
            original_count
        );
    }

    search_result
}

pub fn api_routes() -> Router<AppState> {
    Router::new()
        .route("/search_hotel_api", post(search_hotel_api_server_fn_route))
        .route("/block_room_api", post(block_room_api_server_fn_route))
        .route("/book_room_api", post(book_room_api_server_fn_route))
        .route(
            "/get_hotel_info_api",
            post(get_hotel_info_api_server_fn_route),
        )
        .route(
            "/get_hotel_rates_api",
            post(get_hotel_rates_api_server_fn_route),
        )
}

#[axum::debug_handler]
pub async fn search_hotel_api_server_fn_route(
    State(state): State<AppState>,
    body: String,
) -> Response {
    // <!-- Parse input string to struct -->
    let request: DomainHotelSearchCriteria = match serde_json::from_str(&body) {
        Ok(req) => req,
        Err(e) => {
            tracing::error!("Failed to parse search request: {:?}", e);
            let error_response = json!({
                "error": format!("Invalid input format: {}", e)
            });
            return (StatusCode::BAD_REQUEST, error_response.to_string()).into_response();
        }
    };

    // <!-- Create the hotel service with LiteApiAdapter -->
    let liteapi_adapter = LiteApiAdapter::new(state.liteapi_client.clone());
    let hotel_service = HotelService::new(liteapi_adapter);

    // <!-- Perform the hotel search -->
    match hotel_service.search_hotels(request).await {
        Ok(result) => {
            // <!-- Filter out hotels with zero pricing -->
            let filtered_result = filter_hotels_with_valid_pricing(result);

            // <!-- Serialize response to string -->
            match serde_json::to_string(&filtered_result) {
                Ok(json_string) => (StatusCode::OK, json_string).into_response(),
                Err(e) => {
                    tracing::error!("Failed to serialize response: {:?}", e);
                    let error_response = json!({
                        "error": format!("Failed to serialize response: {}", e)
                    });
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        error_response.to_string(),
                    )
                        .into_response()
                }
            }
        }
        Err(e) => {
            // <!-- Log the error -->
            tracing::error!("Hotel search failed: {:?}", e);

            // <!-- Return error response -->
            let error_response = json!({
                "error": format!("Hotel search failed: {}", e)
            });

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                error_response.to_string(),
            )
                .into_response()
        }
    }
}

#[axum::debug_handler]
pub async fn get_hotel_info_api_server_fn_route(
    State(state): State<AppState>,
    body: String,
) -> Response {
    // <!-- Parse input string to struct -->
    let request: DomainHotelInfoCriteria = match serde_json::from_str(&body) {
        Ok(req) => req,
        Err(e) => {
            tracing::error!("Failed to parse hotel info request: {:?}", e);
            let error_response = json!({
                "error": format!("Invalid input format: {}", e)
            });
            return (StatusCode::BAD_REQUEST, error_response.to_string()).into_response();
        }
    };

    // <!-- Create the hotel service with LiteApiAdapter -->
    let liteapi_adapter = LiteApiAdapter::new(state.liteapi_client.clone());
    let hotel_service = HotelService::new(liteapi_adapter);

    // <!-- Get hotel information -->
    match hotel_service.get_hotel_details(request).await {
        Ok(result) => {
            // <!-- Serialize response to string -->
            match serde_json::to_string(&result) {
                Ok(json_string) => (StatusCode::OK, json_string).into_response(),
                Err(e) => {
                    tracing::error!("Failed to serialize response: {:?}", e);
                    let error_response = json!({
                        "error": format!("Failed to serialize response: {}", e)
                    });
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        error_response.to_string(),
                    )
                        .into_response()
                }
            }
        }
        Err(e) => {
            // <!-- Log the error -->
            tracing::error!("Hotel info retrieval failed: {:?}", e);

            // <!-- Return error response -->
            let error_response = json!({
                "error": format!("Failed to get hotel info: {}", e)
            });

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                error_response.to_string(),
            )
                .into_response()
        }
    }
}

#[axum::debug_handler]
pub async fn get_hotel_rates_api_server_fn_route(
    State(state): State<AppState>,
    body: String,
) -> Response {
    // <!-- Parse input string to struct -->
    let request: DomainHotelInfoCriteria = match serde_json::from_str(&body) {
        Ok(req) => req,
        Err(e) => {
            tracing::error!("Failed to parse hotel rates request: {:?}", e);
            let error_response = json!({
                "error": format!("Invalid input format: {}", e)
            });
            return (StatusCode::BAD_REQUEST, error_response.to_string()).into_response();
        }
    };

    // <!-- Create the hotel service with LiteApiAdapter -->
    let liteapi_adapter = LiteApiAdapter::new(state.liteapi_client.clone());
    let hotel_service = HotelService::new(liteapi_adapter);

    // <!-- Get hotel rates -->
    match hotel_service.get_hotel_rates(request).await {
        Ok(result) => {
            // <!-- Serialize response to string -->
            match serde_json::to_string(&result) {
                Ok(json_string) => (StatusCode::OK, json_string).into_response(),
                Err(e) => {
                    tracing::error!("Failed to serialize response: {:?}", e);
                    let error_response = json!({
                        "error": format!("Failed to serialize response: {}", e)
                    });
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        error_response.to_string(),
                    )
                        .into_response()
                }
            }
        }
        Err(e) => {
            // <!-- Log the error -->
            tracing::error!("Hotel rates retrieval failed: {:?}", e);

            // <!-- Return error response -->
            let error_response = json!({
                "error": format!("Failed to get hotel rates: {}", e)
            });

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                error_response.to_string(),
            )
                .into_response()
        }
    }
}
#[axum::debug_handler]
pub async fn block_room_api_server_fn_route(
    State(state): State<AppState>,
    body: String,
) -> Response {
    // <!-- Parse input string to struct -->
    let request: DomainBlockRoomRequest = match serde_json::from_str(&body) {
        Ok(req) => req,
        Err(e) => {
            tracing::error!("Failed to parse block room request: {:?}", e);
            let error_response = json!({
                "error": format!("Invalid input format: {}", e)
            });
            return (StatusCode::BAD_REQUEST, error_response.to_string()).into_response();
        }
    };

    // <!-- Determine which provider to use based on configuration or request -->
    // <!-- For now, we'll use LiteAPI as default, but this could be configurable -->
    let use_liteapi = true; // Could be from config or request

    let result: Result<DomainBlockRoomResponse, _> = if use_liteapi {
        // <!-- Use LiteAPI adapter -->
        let liteapi_adapter = LiteApiAdapter::new(state.liteapi_client.clone());
        liteapi_adapter.block_room(request).await
    } else {
        // <!-- Use Provab adapter -->
        let provab_adapter = ProvabAdapter::new(state.provab_client.clone());
        provab_adapter.block_room(request).await
    };

    match result {
        Ok(block_response) => {
            // <!-- Serialize response to string -->
            match serde_json::to_string(&block_response) {
                Ok(json_string) => (StatusCode::OK, json_string).into_response(),
                Err(e) => {
                    tracing::error!("Failed to serialize block room response: {:?}", e);
                    let error_response = json!({
                        "error": format!("Failed to serialize response: {}", e)
                    });
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        error_response.to_string(),
                    )
                        .into_response()
                }
            }
        }
        Err(e) => {
            // <!-- Log the error -->
            tracing::error!("Block room failed: {:?}", e);

            // <!-- Return error response -->
            let error_response = json!({
                "error": format!("Block room failed: {}", e)
            });

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                error_response.to_string(),
            )
                .into_response()
        }
    }
}

#[axum::debug_handler]
pub async fn book_room_api_server_fn_route(
    State(_state): State<AppState>,
    body: String,
) -> Result<Response, StatusCode> {
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

    // Create hotel service with provider
    // For now, use LiteAPI. In the future, this could be configurable
    let liteapi_client = LiteApiHTTPClient::default();
    let provider = LiteApiAdapter::new(liteapi_client);
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
