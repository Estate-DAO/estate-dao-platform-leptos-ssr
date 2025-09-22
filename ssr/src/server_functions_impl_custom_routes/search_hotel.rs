use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use estate_fe::view_state_layer::AppState;
use estate_fe::{
    adapters::LiteApiAdapter, application_services::HotelService, domain::DomainHotelSearchCriteria,
};
use serde_json::json;

use super::{filter_hotels_with_valid_pricing, parse_json_request};

#[axum::debug_handler]
pub async fn search_hotel_api_server_fn_route(
    State(state): State<AppState>,
    body: String,
) -> Result<Response, Response> {
    // <!-- Parse input string to struct -->
    let request: DomainHotelSearchCriteria = parse_json_request(&body)?;
    tracing::error!("Hotel search request: {:?}", request);
    // <!-- Create the hotel service with LiteApiAdapter -->
    let liteapi_adapter = LiteApiAdapter::new(state.liteapi_client.clone());
    let hotel_service = HotelService::new(liteapi_adapter);

    // <!-- Perform the hotel search -->
    let result = hotel_service.search_hotels(request).await.map_err(|e| {
        tracing::error!("Hotel search failed: {:?}", e);
        let error_response = json!({
            "error": format!("Hotel search failed: {}", e)
        });
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            error_response.to_string(),
        )
            .into_response()
    })?;

    // <!-- Filter out hotels with zero pricing -->
    let filtered_result = filter_hotels_with_valid_pricing(result);

    // <!-- Serialize response to string -->
    let json_string = serde_json::to_string(&filtered_result).map_err(|e| {
        tracing::error!("Failed to serialize response: {:?}", e);
        let error_response = json!({
            "error": format!("Failed to serialize response: {}", e)
        });
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            error_response.to_string(),
        )
            .into_response()
    })?;

    Ok((StatusCode::OK, json_string).into_response())
}
