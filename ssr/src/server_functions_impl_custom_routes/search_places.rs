use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use estate_fe::{
    adapters::LiteApiAdapter, application_services::HotelService, domain::{DomainHotelSearchCriteria, DomainPlaceDetailsPayload},
};
use estate_fe::{
    application_services::PlaceService, domain::DomainPlacesSearchPayload,
    view_state_layer::AppState,
};
use serde_json::json;

use super::parse_json_request;

#[axum::debug_handler]
pub async fn search_places_api_server_fn_route(
    State(state): State<AppState>,
    body: String,
) -> Result<Response, Response> {
    // <!-- Parse input string to struct -->
    let request: DomainPlacesSearchPayload = parse_json_request(&body)?;

    // <!-- Create the places service with LiteApiAdapter -->
    let liteapi_adapter = LiteApiAdapter::new(state.liteapi_client.clone());
    let places_service = PlaceService::new(liteapi_adapter);

    // <!-- Perform the places search -->
    let result = places_service.search_places_with_filters(request).await.map_err(|e| {
        tracing::error!("Places search failed: {:?}", e);
        let error_response = json!({
            "error": format!("Places search failed: {}", e)
        });
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            error_response.to_string(),
        )
            .into_response()
    })?;


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


#[axum::debug_handler]
pub async fn search_places_details_api_server_fn_route(
    State(state): State<AppState>,
    body: String,
) -> Result<Response, Response> {
    // <!-- Parse input string to struct -->
    let request: DomainPlaceDetailsPayload = parse_json_request(&body)?;

    // <!-- Create the places service with LiteApiAdapter -->
    let liteapi_adapter = LiteApiAdapter::new(state.liteapi_client.clone());
    let places_service = PlaceService::new(liteapi_adapter);

    // <!-- Perform the places search -->
    let result = places_service.get_single_place_details(request).await.map_err(|e| {
        tracing::error!("Places details failed: {:?}", e);
        let error_response = json!({
            "error": format!("Places details failed: {}", e)
        });
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            error_response.to_string(),
        )
            .into_response()
    })?;


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
