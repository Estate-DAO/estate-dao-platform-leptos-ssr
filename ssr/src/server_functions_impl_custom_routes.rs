use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use estate_fe::view_state_layer::AppState;
use estate_fe::{
    adapters::ProvabAdapter,
    application_services::HotelService,
    domain::{
        DomainHotelDetails, DomainHotelInfoCriteria, DomainHotelListAfterSearch,
        DomainHotelSearchCriteria,
    },
    ports::traits::HotelProviderPort,
};
use serde_json::json;

pub fn api_routes() -> Router<AppState> {
    Router::new().route("/search_hotel_api", post(search_hotel_api_server_fn_route))
    // .route("/get_hotel_info_api", post(get_hotel_info_api_server_fn_route))
    // .route("/get_hotel_rates_api", post(get_hotel_rates_api_server_fn_route))
}

#[axum::debug_handler]
pub async fn search_hotel_api_server_fn_route(
    State(state): State<AppState>,
    body: String,
) -> Response {
    // <!-- Parse input string to struct -->
    let request: DomainHotelSearchCriteria = match serde_json::from_str(&body) {
        Ok(req) => req,
        Err(e) => {
            tracing::error!("Failed to parse search request: {:?}", e);
            let error_response = json!({
                "error": format!("Invalid input format: {}", e)
            });
            return (StatusCode::BAD_REQUEST, error_response.to_string()).into_response();
        }
    };

    // <!-- Create the hotel service with ProvabAdapter -->
    let provab_adapter = ProvabAdapter::new(state.provab_client.clone());
    let hotel_service = HotelService::new(provab_adapter);

    // <!-- Perform the hotel search -->
    match hotel_service.search_hotels(request).await {
        Ok(result) => {
            // <!-- Serialize response to string -->
            match serde_json::to_string(&result) {
                Ok(json_string) => (StatusCode::OK, json_string).into_response(),
                Err(e) => {
                    tracing::error!("Failed to serialize response: {:?}", e);
                    let error_response = json!({
                        "error": format!("Failed to serialize response: {}", e)
                    });
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        error_response.to_string(),
                    )
                        .into_response()
                }
            }
        }
        Err(e) => {
            // <!-- Log the error -->
            tracing::error!("Hotel search failed: {:?}", e);

            // <!-- Return error response -->
            let error_response = json!({
                "error": format!("Hotel search failed: {}", e)
            });

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                error_response.to_string(),
            )
                .into_response()
        }
    }
}

// #[axum::debug_handler]
// pub async fn get_hotel_info_api_server_fn_route(
//     State(state): State<AppState>,
//     body: String,
// ) -> Response {
//     // <!-- Parse input string to struct -->
//     let request: DomainHotelInfoCriteria = match serde_json::from_str(&body) {
//         Ok(req) => req,
//         Err(e) => {
//             tracing::error!("Failed to parse hotel info request: {:?}", e);
//             let error_response = json!({
//                 "error": format!("Invalid input format: {}", e)
//             });
//             return (StatusCode::BAD_REQUEST, error_response.to_string()).into_response();
//         }
//     };

//     // <!-- Create the hotel service with ProvabAdapter -->
//     let provab_adapter = ProvabAdapter::new(state.provab_client.clone());
//     let hotel_service = HotelService::new(provab_adapter);

//     // <!-- Get hotel information -->
//     match hotel_service.get_hotel_details(request).await {
//         Ok(result) => {
//             // <!-- Serialize response to string -->
//             match serde_json::to_string(&result) {
//                 Ok(json_string) => (StatusCode::OK, json_string).into_response(),
//                 Err(e) => {
//                     tracing::error!("Failed to serialize response: {:?}", e);
//                     let error_response = json!({
//                         "error": format!("Failed to serialize response: {}", e)
//                     });
//                     (StatusCode::INTERNAL_SERVER_ERROR, error_response.to_string()).into_response()
//                 }
//             }
//         }
//         Err(e) => {
//             // <!-- Log the error -->
//             tracing::error!("Hotel info retrieval failed: {:?}", e);

//             // <!-- Return error response -->
//             let error_response = json!({
//                 "error": format!("Failed to get hotel info: {}", e)
//             });

//             (StatusCode::INTERNAL_SERVER_ERROR, error_response.to_string()).into_response()
//         }
//     }
// }

// #[axum::debug_handler]
// pub async fn get_hotel_rates_api_server_fn_route(
//     State(state): State<AppState>,
//     body: String,
// ) -> Response {
//     // <!-- Parse input string to struct -->
//     let request: DomainHotelInfoCriteria = match serde_json::from_str(&body) {
//         Ok(req) => req,
//         Err(e) => {
//             tracing::error!("Failed to parse hotel rates request: {:?}", e);
//             let error_response = json!({
//                 "error": format!("Invalid input format: {}", e)
//             });
//             return (StatusCode::BAD_REQUEST, error_response.to_string()).into_response();
//         }
//     };

//     // <!-- Create the hotel service with ProvabAdapter -->
//     let provab_adapter = ProvabAdapter::new(state.provab_client.clone());
//     let hotel_service = HotelService::new(provab_adapter);

//     // <!-- Get hotel rates -->
//     match hotel_service.get_hotel_rates(request).await {
//         Ok(result) => {
//             // <!-- Serialize response to string -->
//             match serde_json::to_string(&result) {
//                 Ok(json_string) => (StatusCode::OK, json_string).into_response(),
//                 Err(e) => {
//                     tracing::error!("Failed to serialize response: {:?}", e);
//                     let error_response = json!({
//                         "error": format!("Failed to serialize response: {}", e)
//                     });
//                     (StatusCode::INTERNAL_SERVER_ERROR, error_response.to_string()).into_response()
//                 }
//             }
//         }
//         Err(e) => {
//             // <!-- Log the error -->
//             tracing::error!("Hotel rates retrieval failed: {:?}", e);

//             // <!-- Return error response -->
//             let error_response = json!({
//                 "error": format!("Failed to get hotel rates: {}", e)
//             });

//             (StatusCode::INTERNAL_SERVER_ERROR, error_response.to_string()).into_response()
//         }
//     }
// }
