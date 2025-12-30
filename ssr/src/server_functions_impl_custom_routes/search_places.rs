use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use estate_fe::{
    application_services::HotelService,
    domain::{DomainHotelSearchCriteria, DomainPlaceDetailsPayload},
    init::get_liteapi_adapter,
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

    // Normalize cache key (lowercase, trimmed)
    let cache_key = request.text_query.trim().to_lowercase();

    // <!-- Create the places service with LiteApiAdapter from global client -->
    let liteapi_adapter = get_liteapi_adapter();
    let places_service = PlaceService::new(liteapi_adapter);

    // <!-- Try API first, fall back to cache on failure -->
    let api_result = places_service.search_places_with_filters(request).await;

    match api_result {
        Ok(result) => {
            // Success - cache the result and return
            state
                .place_search_cache
                .insert(cache_key.clone(), result.clone())
                .await;
            tracing::info!("PlaceSearchCache STORED for query: '{}'", cache_key);

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
        Err(e) => {
            tracing::warn!("Places API failed: {:?}, trying cache fallback...", e);

            // API failed - try cache with fuzzy matching
            let cached_result = find_cached_result(&state.place_search_cache, &cache_key).await;

            if let Some(cached) = cached_result {
                tracing::info!(
                    "PlaceSearchCache FALLBACK for query: '{}' (API failed)",
                    cache_key
                );
                let json_string = serde_json::to_string(&cached).map_err(|e| {
                    tracing::error!("Failed to serialize cached response: {:?}", e);
                    let error_response = json!({
                        "error": format!("Failed to serialize response: {}", e)
                    });
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        error_response.to_string(),
                    )
                        .into_response()
                })?;
                return Ok((StatusCode::OK, json_string).into_response());
            }

            // No cache available - return error
            tracing::error!("Places search failed (no cache available): {:?}", e);
            let error_response = json!({
                "error": format!("Places search failed: {}", e)
            });
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                error_response.to_string(),
            )
                .into_response())
        }
    }
}

/// Helper function to find cached result with fuzzy matching
async fn find_cached_result(
    cache: &estate_fe::view_state_layer::PlaceSearchCache,
    cache_key: &str,
) -> Option<estate_fe::domain::DomainPlacesResponse> {
    // Try exact match first
    if let Some(result) = cache.get(cache_key).await {
        tracing::info!("PlaceSearchCache HIT (exact) for query: '{}'", cache_key);
        return Some(result);
    }

    // Try fuzzy matching - find the longest cached prefix
    let base_query = cache_key.trim_end_matches(|c: char| !c.is_alphanumeric());
    let mut prefix = base_query.to_string();

    // Try progressively shorter prefixes (minimum 3 chars)
    while prefix.len() >= 3 {
        if let Some(result) = cache.get(&prefix).await {
            tracing::info!(
                "PlaceSearchCache HIT (fuzzy) for query: '{}' using cached prefix: '{}'",
                cache_key,
                prefix
            );
            return Some(result);
        }
        // Remove last character and try again
        prefix.pop();
        // Trim any trailing spaces/punctuation
        prefix = prefix
            .trim_end_matches(|c: char| !c.is_alphanumeric())
            .to_string();
    }

    None
}

#[axum::debug_handler]
pub async fn search_places_details_api_server_fn_route(
    State(state): State<AppState>,
    body: String,
) -> Result<Response, Response> {
    // <!-- Parse input string to struct -->
    let request: DomainPlaceDetailsPayload = parse_json_request(&body)?;

    // <!-- Create the places service with LiteApiAdapter from global client -->
    let liteapi_adapter = get_liteapi_adapter();
    let places_service = PlaceService::new(liteapi_adapter);

    // <!-- Perform the places search -->
    let result = places_service
        .get_single_place_details(request)
        .await
        .map_err(|e| {
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
