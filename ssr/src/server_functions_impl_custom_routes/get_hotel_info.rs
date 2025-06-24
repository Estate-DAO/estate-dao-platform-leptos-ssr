use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use estate_fe::view_state_layer::AppState;
use estate_fe::{
    adapters::LiteApiAdapter, application_services::HotelService, domain::DomainHotelInfoCriteria,
};
use serde_json::json;

use super::parse_json_request;

#[axum::debug_handler]
pub async fn get_hotel_info_api_server_fn_route(
    State(state): State<AppState>,
    body: String,
) -> Result<Response, Response> {
    // <!-- Parse input string to struct -->
    let request: DomainHotelInfoCriteria = parse_json_request(&body)?;

    // <!-- Create the hotel service with LiteApiAdapter -->
    let liteapi_adapter = LiteApiAdapter::new(state.liteapi_client.clone());
    let hotel_service = HotelService::new(liteapi_adapter);

    // <!-- Get hotel information -->
    let result = hotel_service
        .get_hotel_details(request)
        .await
        .map_err(|e| {
            tracing::error!("Hotel info retrieval failed: {:?}", e);
            let error_response = json!({
                "error": format!("Failed to get hotel info: {}", e)
            });
            (StatusCode::BAD_REQUEST, error_response.to_string()).into_response()
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
