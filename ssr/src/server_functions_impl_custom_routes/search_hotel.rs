use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use estate_fe::view_state_layer::AppState;
use estate_fe::{
    application_services::HotelService,
    domain::DomainHotelSearchCriteria,
    init::get_liteapi_driver,
    ports::hotel_provider_port::ProviderError,
    utils::error_alerts::{CriticalError, ErrorType},
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
    // <!-- Create the hotel service with LiteApiDriver from global client -->
    let liteapi_driver = get_liteapi_driver();
    let hotel_service = HotelService::new(liteapi_driver);

    // <!-- Perform the hotel search -->
    let result = hotel_service
        .search_hotels(request.clone())
        .await
        .map_err(|e| {
            tracing::error!("Hotel search failed: {:?}", e);

            // Report critical error to alert service
            let error_alert_service = state.error_alert_service;
            let error_message = format!("{}", e);
            let error_type = match &e {
                ProviderError(details)
                    if matches!(
                        details.error_kind,
                        hotel_types::ports::ProviderErrorKind::ParseError
                    ) =>
                {
                    // For parse errors, try to extract path from error message
                    let json_path = if details.message.contains("path:") {
                        details
                            .message
                            .split(" - inner:")
                            .next()
                            .map(|s| s.replace("path: ", ""))
                            .map(String::from)
                    } else {
                        None
                    };
                    ErrorType::JsonParseFailed {
                        json_path,
                        expected_type: None,
                        actual_type: None,
                    }
                }
                _ => ErrorType::BookingProviderFailure {
                    provider: "liteapi".to_string(),
                    hotel_id: None,
                    operation: "search".to_string(),
                },
            };

            let critical_error = CriticalError::new(error_type, error_message.clone())
                .with_request("POST", "/server_fn_api/search_hotel_api")
                .with_source(
                    "ssr/src/server_functions_impl_custom_routes/search_hotel.rs",
                    28,
                    "search_hotel_api_server_fn_route",
                );

            // Spawn a task to report the error (non-blocking)
            tokio::spawn(async move {
                error_alert_service.report(critical_error).await;
            });

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
