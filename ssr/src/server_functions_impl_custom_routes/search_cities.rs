use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use estate_fe::view_state_layer::AppState;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::parse_json_request;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchCitiesRequest {
    pub prefix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchCitiesResponse {
    pub cities: Vec<CitySearchResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitySearchResult {
    pub city_code: String,
    pub city_name: String,
    pub country_name: String,
    pub country_code: String,
    pub image_url: String,
    pub latitude: f64,
    pub longitude: f64,
}

impl From<duck_searcher::CityEntry> for CitySearchResult {
    fn from(city_entry: duck_searcher::CityEntry) -> Self {
        Self {
            city_code: city_entry.city_code,
            city_name: city_entry.city_name,
            country_name: city_entry.country_name,
            country_code: city_entry.country_code,
            image_url: city_entry.image_url,
            latitude: city_entry.latitude,
            longitude: city_entry.longitude,
        }
    }
}

#[axum::debug_handler]
#[tracing::instrument(skip_all)]
pub async fn search_cities_api_server_fn_route(
    State(_state): State<AppState>,
    body: String,
) -> Result<Response, Response> {
    // <!-- Parse input string to struct -->
    let request: SearchCitiesRequest = parse_json_request(&body)?;

    // <!-- Validate input -->
    if request.prefix.trim().is_empty() {
        return Ok((StatusCode::OK, json!({"cities": []}).to_string()).into_response());
    }

    // <!-- Call duck-searcher to get cities -->
    #[cfg(feature = "ssr")]
    {
        use duck_searcher::search_cities_by_prefix_as_entries;

        let city_entries = search_cities_by_prefix_as_entries(&request.prefix).map_err(|e| {
            tracing::error!("City search failed: {:?}", e);
            let error_response = json!({
                "error": format!("City search failed: {}", e)
            });
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                error_response.to_string(),
            )
                .into_response()
        })?;

        // <!-- Convert CityEntry to CitySearchResult -->
        let cities: Vec<CitySearchResult> = city_entries
            .into_iter()
            .map(CitySearchResult::from)
            .collect();

        let response = SearchCitiesResponse { cities };

        // <!-- Serialize response to string -->
        let json_string = serde_json::to_string(&response).map_err(|e| {
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

    #[cfg(not(feature = "ssr"))]
    {
        let error_response = json!({
            "error": "City search not available in client-side build"
        });
        Ok((
            StatusCode::INTERNAL_SERVER_ERROR,
            error_response.to_string(),
        )
            .into_response())
    }
}
