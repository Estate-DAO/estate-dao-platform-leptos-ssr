use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use estate_fe::{
    init::{get_primary_hotel_provider, update_primary_hotel_provider},
    view_state_layer::AppState,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{error, info, instrument};

use crate::basic_auth::validate_basic_auth_from_headers;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotelProviderConfigResponse {
    pub primary_hotel_provider: String,
    pub available_providers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateHotelProviderConfigRequest {
    pub primary_hotel_provider: String,
}

#[instrument(name = "get_hotel_provider_config_handler", skip(state, headers))]
pub async fn get_hotel_provider_config(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Response {
    if let Err(status_code) = validate_basic_auth_from_headers(&headers, &state) {
        return (
            status_code,
            Json(json!({
                "error": "Authentication required",
                "details": "Invalid or missing basic authentication credentials"
            })),
        )
            .into_response();
    }

    let response = HotelProviderConfigResponse {
        primary_hotel_provider: get_primary_hotel_provider(),
        available_providers: vec!["liteapi".to_string(), "booking".to_string()],
    };

    Json(response).into_response()
}

#[instrument(
    name = "update_hotel_provider_config_handler",
    skip(state, headers, request)
)]
pub async fn update_hotel_provider_config(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<UpdateHotelProviderConfigRequest>,
) -> Response {
    if let Err(status_code) = validate_basic_auth_from_headers(&headers, &state) {
        return (
            status_code,
            Json(json!({
                "error": "Authentication required",
                "details": "Invalid or missing basic authentication credentials"
            })),
        )
            .into_response();
    }

    match update_primary_hotel_provider(&request.primary_hotel_provider) {
        Ok(updated_provider) => {
            info!(
                "Admin updated primary hotel provider at runtime: {}",
                updated_provider
            );
            Json(HotelProviderConfigResponse {
                primary_hotel_provider: updated_provider,
                available_providers: vec!["liteapi".to_string(), "booking".to_string()],
            })
            .into_response()
        }
        Err(e) => {
            error!("Failed to update primary hotel provider: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "Failed to update primary hotel provider",
                    "details": e
                })),
            )
                .into_response()
        }
    }
}
