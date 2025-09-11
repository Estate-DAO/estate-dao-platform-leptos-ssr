use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, options, post},
    Router,
};
use estate_fe::view_state_layer::AppState;
use estate_fe::{
    adapters::LiteApiAdapter,
    // adapters::ProvabAdapter,
    api::{
        canister::add_booking::call_add_booking_backend,
        liteapi::LiteApiHTTPClient,
        payments::{
            domain::{DomainCreateInvoiceRequest, DomainCreateInvoiceResponse},
            service::PaymentServiceImpl,
            PaymentService,
        },
    },
    application_services::{BookingService, HotelService},
    domain::{
        BookingError, DomainBlockRoomRequest, DomainBlockRoomResponse, DomainBookRoomRequest,
        DomainBookRoomResponse, DomainHotelDetails, DomainHotelInfoCriteria,
        DomainHotelListAfterSearch, DomainHotelSearchCriteria,
    },
    ports::traits::HotelProviderPort,
    ssr_booking::{
        booking_handler::MakeBookingFromBookingProvider,
        email_handler::SendEmailAfterSuccessfullBooking,
        payment_handler::GetPaymentStatusFromPaymentProvider, pipeline::process_pipeline,
        SSRBookingPipelineStep, ServerSideBookingEvent,
    },
    utils::{
        app_reference::BookingId, booking_backend_conversions::BookingBackendConversions,
        booking_id::PaymentIdentifiers,
    },
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::json;

// Import all route modules
mod admin_payment;
mod block_room;
mod book_room;
mod create_payment_invoice;
mod email_verification;
mod get_hotel_info;
mod get_hotel_rates;
mod integrated_block_room;
mod process_confirmation;
pub mod search_cities;
mod search_hotel;
mod update_email_principal_mapping;

pub use block_room::block_room_api_server_fn_route;
pub use book_room::book_room_api_server_fn_route;
pub use create_payment_invoice::create_payment_invoice_api_server_fn_route;
pub use email_verification::{send_otp_email_api_server_fn_route, verify_otp_api_server_fn_route};
pub use get_hotel_info::get_hotel_info_api_server_fn_route;
pub use get_hotel_rates::get_hotel_rates_api_server_fn_route;
pub use integrated_block_room::integrated_block_room_api_server_fn_route;
pub use process_confirmation::process_confirmation_api_server_fn_route;
pub use search_cities::search_cities_api_server_fn_route;
pub use search_hotel::search_hotel_api_server_fn_route;
pub use update_email_principal_mapping::update_user_principal_email_mapping_in_canister_fn_route;

use crate::server_functions_impl_custom_routes::search_cities::search_city_by_name_api_server_fn_route;

// Common helper functions and types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmationProcessRequest {
    pub payment_id: Option<String>,
    pub app_reference: Option<String>,
    pub email: Option<String>,
    pub query_params: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmationProcessResponse {
    pub success: bool,
    pub message: String,
    pub order_id: Option<String>,
    pub user_email: Option<String>,
    pub booking_data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegratedBlockRoomRequest {
    pub block_room_request: DomainBlockRoomRequest,
    pub booking_id: String,
    pub email: String,
    pub hotel_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegratedBlockRoomResponse {
    pub success: bool,
    pub message: String,
    pub block_room_response: Option<DomainBlockRoomResponse>,
    pub booking_id: String,
}

// Helper function to parse JSON requests with consistent error handling
pub fn parse_json_request<T: DeserializeOwned>(body: &str) -> Result<T, Response> {
    serde_json::from_str(body).map_err(|e| {
        tracing::error!("Failed to parse JSON request: {:?}", e);
        let error_response = json!({
            "error": format!("Invalid JSON request: {}", e),
        });
        (StatusCode::BAD_REQUEST, error_response.to_string()).into_response()
    })
}

// Helper function to call block room API using HotelService
// This properly delegates provider selection to the service layer
pub async fn call_block_room_api(
    state: &AppState,
    request: DomainBlockRoomRequest,
) -> Result<DomainBlockRoomResponse, String> {
    // For now, use LiteAPI as the default provider
    let liteapi_adapter = LiteApiAdapter::new(state.liteapi_client.clone());
    let hotel_service = HotelService::new(liteapi_adapter);

    hotel_service
        .block_room(request)
        .await
        .map_err(|e| e.to_string())
}

// Helper function to filter hotels with valid pricing
pub fn filter_hotels_with_valid_pricing(
    mut search_result: DomainHotelListAfterSearch,
) -> DomainHotelListAfterSearch {
    let original_count = search_result.hotel_results.len();
    let hotels_without_pricing = search_result
        .hotel_results
        .iter()
        .filter(|hotel| {
            hotel
                .price
                .clone()
                .map(|f| f.room_price <= 0.0)
                .unwrap_or(false)
        })
        .count();

    // search_result
    //     .hotel_results
    //     .retain(|hotel| hotel.price.room_price > 0.0);

    let final_count = search_result.hotel_results.len();

    if hotels_without_pricing > 0 {
        tracing::info!(
            "API search filtering: Found {} hotels total, {} without pricing ({}%), {} with valid pricing returned to client",
            original_count,
            hotels_without_pricing,
            if original_count > 0 { (hotels_without_pricing * 100) / original_count } else { 0 },
            final_count
        );
    } else if original_count > 0 {
        tracing::info!(
            "API search filtering: All {} hotels had valid pricing",
            original_count
        );
    }

    search_result
}

// CORS preflight handler
async fn handle_options() -> Response {
    StatusCode::OK.into_response()
}

pub fn generate_error_response(error: &str) -> Response {
    let error_response = json!({
        "error": error
    });
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        error_response.to_string(),
    )
        .into_response()
}

pub fn api_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/search_hotel_api",
            post(search_hotel_api_server_fn_route).options(handle_options),
        )
        .route(
            "/search_cities_api",
            post(search_cities_api_server_fn_route).options(handle_options),
        )
        .route(
            "/search_city_api",
            post(search_city_by_name_api_server_fn_route).options(handle_options),
        )
        .route(
            "/block_room_api",
            post(block_room_api_server_fn_route).options(handle_options),
        )
        .route(
            "/book_room_api",
            post(book_room_api_server_fn_route).options(handle_options),
        )
        .route(
            "/get_hotel_info_api",
            post(get_hotel_info_api_server_fn_route).options(handle_options),
        )
        .route(
            "/get_hotel_rates_api",
            post(get_hotel_rates_api_server_fn_route).options(handle_options),
        )
        .route(
            "/process_confirmation_api",
            post(process_confirmation_api_server_fn_route).options(handle_options),
        )
        .route(
            "/integrated_block_room_api",
            post(integrated_block_room_api_server_fn_route).options(handle_options),
        )
        .route(
            "/create_payment_invoice_api",
            post(create_payment_invoice_api_server_fn_route).options(handle_options),
        )
        .route(
            "/admin/check_payment_status",
            post(admin_payment::check_payment_status).options(handle_options),
        )
        .route(
            "/admin/get_backend_booking",
            post(admin_payment::get_backend_booking).options(handle_options),
        )
        .route(
            "/admin/update_payment",
            post(admin_payment::update_payment_details).options(handle_options),
        )
        .route(
            "/send_otp_email_api",
            post(send_otp_email_api_server_fn_route).options(handle_options),
        )
        .route(
            "/verify_otp_api",
            post(verify_otp_api_server_fn_route).options(handle_options),
        )
        .route(
            "/update_user_principal_email_mapping_in_canister_server_fn",
            post(update_user_principal_email_mapping_in_canister_fn_route).options(handle_options),
        )
    // todo(2025-08-08): add my bookings api route with user_identity
    // .route(
    //     "/my_bookings_api",
    //     post(my_bookings_api_server_fn_route).options(handle_options),
    // )
}
