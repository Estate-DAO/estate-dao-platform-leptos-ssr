use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use estate_fe::view_state_layer::AppState;
use estate_fe::{
    adapters::ProvabAdapter,
    application_services::HotelService,
    domain::{DomainHotelListAfterSearch, DomainHotelSearchCriteria},
    ports::hotel_provider_port::HotelProviderPort,
};
use serde_json::json;

pub fn api_routes() -> Router<AppState> {
    Router::new().route("/search_hotel_api", post(search_hotel_api_server_fn_route))
    // You can add more routes here
    // .route("/other-endpoint", get(other_handler))
    // .nest("/sub-api", sub_api_routes())
}

#[axum::debug_handler]
pub async fn search_hotel_api_server_fn_route(
    State(state): State<AppState>,
    Json(request): Json<DomainHotelSearchCriteria>,
) -> Response {
    // Create the hotel service with ProvabAdapter
    let provab_adapter = ProvabAdapter::new(state.provab_client.clone());
    let hotel_service = HotelService::new(provab_adapter);

    // Perform the hotel search
    match hotel_service.search_hotels(request).await {
        Ok(result) => {
            // Return successful response
            (StatusCode::OK, Json(result)).into_response()
        }
        Err(e) => {
            // Log the error
            tracing::error!("Hotel search failed: {:?}", e);

            // Return error response
            let error_response = json!({
                "error": format!("Hotel search failed: {}", e)
            });

            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response()
        }
    }
}
