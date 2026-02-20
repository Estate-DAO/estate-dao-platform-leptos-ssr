use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use estate_fe::view_state_layer::AppState;
use estate_fe::{application_services::HotelService, domain::DomainHotelInfoCriteria};
use serde_json::json;

use super::{get_currency_aware_liteapi_driver, parse_json_request};

#[axum::debug_handler]
pub async fn get_hotel_rates_api_server_fn_route(
    State(_state): State<AppState>,
    headers: HeaderMap,
    body: String,
) -> Result<Response, Response> {
    // <!-- Parse input string to struct -->
    let request: DomainHotelInfoCriteria = parse_json_request(&body)?;

    // <!-- Create the hotel service with LiteApiDriver from global client -->
    let liteapi_driver = get_currency_aware_liteapi_driver(&headers);
    let hotel_service = HotelService::new(liteapi_driver);

    // <!-- Get hotel rates -->
    let result = hotel_service.get_hotel_rates(request).await.map_err(|e| {
        tracing::error!("Hotel rates retrieval failed: {:?}", e);
        let error_response = json!({
            "error": format!("{}", e)
        });
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            error_response.to_string(),
        )
            .into_response()
    })?;

    // <!-- Serialize response to string -->
    let json_string = serde_json::to_string(&result).map_err(|e| {
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
