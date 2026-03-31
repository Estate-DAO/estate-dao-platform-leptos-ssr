use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use estate_fe::view_state_layer::AppState;
use estate_fe::application_services::HotelService;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetHotelDetailsQuery {
    pub hotel_id: String,
}

use axum::http::HeaderMap;

#[cfg_attr(feature = "debug_log", axum::debug_handler)]
#[cfg_attr(feature = "debug_log", tracing::instrument(skip(state)))]
pub async fn get_hotel_details_api_server_fn_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<GetHotelDetailsQuery>,
) -> Result<Response, Response> {
    // Validate hotel_id is provided
    if request.hotel_id.trim().is_empty() {
        let error_response = json!({
            "error": "Hotel ID cannot be empty"
        });
        return Err((StatusCode::BAD_REQUEST, error_response.to_string()).into_response());
    }

    // Create the hotel service with provider registry (currency enabled)
    let provider = crate::server_functions_impl_custom_routes::get_currency_aware_provider_registry(&headers).hotel_provider();
    let hotel_service = HotelService::new(provider);

    // Get hotel details without rates
    let result = hotel_service
        .get_hotel_static_details(&request.hotel_id)
        .await
        .map_err(|e| {
            tracing::error!("Hotel details retrieval failed: {:?}", e);
            let error_response = json!({
                "error": format!("Failed to get hotel details: {}", e)
            });
            (StatusCode::UNPROCESSABLE_ENTITY, error_response.to_string()).into_response()
        })?;

    // Serialize response to string
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
