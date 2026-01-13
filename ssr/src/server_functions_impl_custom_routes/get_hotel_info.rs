use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use estate_fe::view_state_layer::AppState;
use estate_fe::{
    application_services::HotelService, domain::DomainHotelInfoCriteria, init::get_liteapi_driver,
};
use serde_json::json;

use super::parse_json_request;

#[cfg_attr(feature = "debug_log", axum::debug_handler)]
#[cfg_attr(feature = "debug_log", tracing::instrument(skip(state)))]
pub async fn get_hotel_info_api_server_fn_route(
    State(state): State<AppState>,
    body: String,
) -> Result<Response, Response> {
    // <!-- Parse input string to struct -->
    let request: DomainHotelInfoCriteria = parse_json_request(&body)?;

    // <!-- Create the hotel service with LiteApiDriver from global client -->
    let liteapi_driver = get_liteapi_driver();
    let hotel_service = HotelService::new(liteapi_driver);

    // <!-- Get hotel information -->
    let result = hotel_service
        .get_hotel_details(request)
        .await
        .map_err(|e| {
            tracing::error!("Hotel info retrieval failed: {:?}", e);
            let error_response = json!({
                "error": format!("Failed to get hotel info: {}", e)
            });
            (StatusCode::UNPROCESSABLE_ENTITY, error_response.to_string()).into_response()
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
