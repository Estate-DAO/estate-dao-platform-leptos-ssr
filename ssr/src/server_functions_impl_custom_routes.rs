use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use estate_fe::view_state_layer::AppState;
use estate_fe::{
    adapters::LiteApiAdapter,
    adapters::ProvabAdapter,
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

// Helper function to parse JSON requests with consistent error handling
fn parse_json_request<T: DeserializeOwned>(body: &str) -> Result<T, Response> {
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
async fn call_block_room_api(
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

// Helper function to filter hotels with valid pricing
fn filter_hotels_with_valid_pricing(
    mut search_result: DomainHotelListAfterSearch,
) -> DomainHotelListAfterSearch {
    let original_count = search_result.hotel_results.len();
    let hotels_without_pricing = search_result
        .hotel_results
        .iter()
        .filter(|hotel| hotel.price.room_price <= 0.0)
        .count();

    search_result
        .hotel_results
        .retain(|hotel| hotel.price.room_price > 0.0);

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

pub fn api_routes() -> Router<AppState> {
    Router::new()
        .route("/search_hotel_api", post(search_hotel_api_server_fn_route))
        .route("/block_room_api", post(block_room_api_server_fn_route))
        .route("/book_room_api", post(book_room_api_server_fn_route))
        .route(
            "/get_hotel_info_api",
            post(get_hotel_info_api_server_fn_route),
        )
        .route(
            "/get_hotel_rates_api",
            post(get_hotel_rates_api_server_fn_route),
        )
        .route(
            "/process_confirmation_api",
            post(process_confirmation_api_server_fn_route),
        )
        .route(
            "/integrated_block_room_api",
            post(integrated_block_room_api_server_fn_route),
        )
        .route(
            "/create_payment_invoice_api",
            post(create_payment_invoice_api_server_fn_route),
        )
}

#[axum::debug_handler]
pub async fn search_hotel_api_server_fn_route(
    State(state): State<AppState>,
    body: String,
) -> Result<Response, Response> {
    // <!-- Parse input string to struct -->
    let request: DomainHotelSearchCriteria = parse_json_request(&body)?;

    // <!-- Create the hotel service with LiteApiAdapter -->
    let liteapi_adapter = LiteApiAdapter::new(state.liteapi_client.clone());
    let hotel_service = HotelService::new(liteapi_adapter);

    // <!-- Perform the hotel search -->
    let result = hotel_service.search_hotels(request).await.map_err(|e| {
        tracing::error!("Hotel search failed: {:?}", e);
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

#[axum::debug_handler]
pub async fn get_hotel_info_api_server_fn_route(
    State(state): State<AppState>,
    body: String,
) -> Result<Response, Response> {
    // <!-- Parse input string to struct -->
    let request: DomainHotelInfoCriteria = parse_json_request(&body)?;

    // <!-- Create the hotel service with LiteApiAdapter -->
    let liteapi_adapter = LiteApiAdapter::new(state.liteapi_client.clone());
    let hotel_service = HotelService::new(liteapi_adapter);

    // <!-- Get hotel information -->
    let result = hotel_service
        .get_hotel_details(request)
        .await
        .map_err(|e| {
            tracing::error!("Hotel info retrieval failed: {:?}", e);
            let error_response = json!({
                "error": format!("Failed to get hotel info: {}", e)
            });
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                error_response.to_string(),
            )
                .into_response()
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

#[axum::debug_handler]
pub async fn get_hotel_rates_api_server_fn_route(
    State(state): State<AppState>,
    body: String,
) -> Result<Response, Response> {
    // <!-- Parse input string to struct -->
    let request: DomainHotelInfoCriteria = parse_json_request(&body)?;

    // <!-- Create the hotel service with LiteApiAdapter -->
    let liteapi_adapter = LiteApiAdapter::new(state.liteapi_client.clone());
    let hotel_service = HotelService::new(liteapi_adapter);

    // <!-- Get hotel rates -->
    let result = hotel_service.get_hotel_rates(request).await.map_err(|e| {
        tracing::error!("Hotel rates retrieval failed: {:?}", e);
        let error_response = json!({
            "error": format!("Failed to get hotel rates: {}", e)
        });
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            error_response.to_string(),
        )
            .into_response()
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
#[axum::debug_handler]
pub async fn block_room_api_server_fn_route(
    State(state): State<AppState>,
    body: String,
) -> Result<Response, Response> {
    // <!-- Parse input string to struct -->
    let request: DomainBlockRoomRequest = parse_json_request(&body)?;

    // Use the shared helper function for block room API call
    let block_response = call_block_room_api(&state, request).await.map_err(|e| {
        tracing::error!("Block room failed: {:?}", e);
        let error_response = json!({
            "error": format!("Block room failed: {}", e)
        });
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            error_response.to_string(),
        )
            .into_response()
    })?;

    // <!-- Serialize response to string -->
    let json_string = serde_json::to_string(&block_response).map_err(|e| {
        tracing::error!("Failed to serialize block room response: {:?}", e);
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
pub async fn book_room_api_server_fn_route(
    State(_state): State<AppState>,
    body: String,
) -> Result<Response, StatusCode> {
    estate_fe::log!(
        "Starting book_room_api_server_fn_route with body: {}",
        &body[0..100.min(body.len())]
    );

    // Parse the JSON request
    let book_request: DomainBookRoomRequest = match serde_json::from_str(&body) {
        Ok(req) => {
            estate_fe::log!("Successfully parsed book room request");
            req
        }
        Err(e) => {
            estate_fe::log!("Failed to parse JSON request: {:?}", e);
            return Ok(axum::Json(json!({
                "error": format!("Invalid JSON request: {}", e)
            }))
            .into_response());
        }
    };

    // Create hotel service with provider
    // For now, use LiteAPI. In the future, this could be configurable
    let liteapi_client = LiteApiHTTPClient::default();
    let provider = LiteApiAdapter::new(liteapi_client);
    let hotel_service = HotelService::new(provider);

    estate_fe::log!(
        "Calling hotel service book_room with block_id: {}",
        book_request.block_id
    );

    // Call the hotel service
    match hotel_service.book_room(book_request).await {
        Ok(book_response) => {
            estate_fe::log!(
                "Successfully booked room. Booking ID: {}",
                book_response.booking_id
            );

            // Return the domain response as JSON
            match serde_json::to_value(&book_response) {
                Ok(json_response) => Ok(axum::Json(json_response).into_response()),
                Err(e) => {
                    estate_fe::log!("Failed to serialize book response: {:?}", e);
                    Ok(axum::Json(json!({
                        "error": format!("Failed to serialize response: {}", e)
                    }))
                    .into_response())
                }
            }
        }
        Err(e) => {
            estate_fe::log!("Book room failed: {:?}", e);

            // Return error response
            Ok(axum::Json(json!({
                "error": format!("Book room failed: {}", e)
            }))
            .into_response())
        }
    }
}

#[axum::debug_handler]
pub async fn process_confirmation_api_server_fn_route(
    State(_state): State<AppState>,
    body: String,
) -> Response {
    tracing::info!(
        "Starting confirmation processing with body: {}",
        &body[0..100.min(body.len())]
    );

    // Parse the JSON request
    let request: ConfirmationProcessRequest = match parse_json_request(&body) {
        Ok(req) => {
            tracing::info!("Successfully parsed confirmation process request");
            req
        }
        Err(_) => {
            // Custom error for this endpoint with specific response format
            let error_response = json!({
                "success": false,
                "message": "Invalid JSON request format",
                "order_id": null,
                "user_email": null
            });
            return (StatusCode::BAD_REQUEST, error_response.to_string()).into_response();
        }
    };

    // Validate required parameters
    let (payment_id, app_reference) = match (request.payment_id, request.app_reference) {
        (Some(pay_id), Some(app_ref)) => (pay_id, app_ref),
        _ => {
            let error_msg =
                "Missing required parameters: payment_id and app_reference are required";
            tracing::error!("{}", error_msg);
            let error_response = json!({
                "success": false,
                "message": error_msg,
                "order_id": null,
                "user_email": null
            });
            return (StatusCode::BAD_REQUEST, error_response.to_string()).into_response();
        }
    };

    // Extract app_reference and order_id using existing utilities
    // The order_id from payment provider needs to be converted to get the actual booking order_id
    // payment_id is what we got from payment provider, but we need the order_id from app_reference

    // First, try to extract email from order_id if it's the proper format
    let (order_id, user_email) = if let Some(booking_id) = BookingId::from_order_id(&app_reference)
    {
        // app_reference is actually the order_id in proper format
        (app_reference.clone(), booking_id.email)
    } else {
        // app_reference might be the simple app reference, need to build order_id
        // For now, assume app_reference contains the order_id format
        return {
            let error_msg = format!(
                "Failed to parse BookingId from app_reference: {}",
                app_reference
            );
            tracing::error!("{}", error_msg);
            let error_response = json!({
                "success": false,
                "message": error_msg,
                "order_id": null,
                "user_email": null
            });
            (StatusCode::BAD_REQUEST, error_response.to_string()).into_response()
        };
    };

    if user_email.is_empty() {
        let error_msg = "Failed to extract user email from app_reference/order_id";
        tracing::error!("{}", error_msg);
        let error_response = json!({
            "success": false,
            "message": error_msg,
            "order_id": Some(order_id),
            "user_email": null
        });
        return (StatusCode::BAD_REQUEST, error_response.to_string()).into_response();
    }

    // Validate that we can create BookingId from order_id (double-check)
    let booking_id = match BookingId::from_order_id(&order_id) {
        Some(bid) => bid,
        None => {
            let error_msg = format!("Failed to create BookingId from order_id: {}", order_id);
            tracing::error!("{}", error_msg);
            let error_response = json!({
                "success": false,
                "message": error_msg,
                "order_id": Some(order_id),
                "user_email": Some(user_email)
            });
            return (StatusCode::BAD_REQUEST, error_response.to_string()).into_response();
        }
    };

    tracing::info!(
        "Extracted booking_id: app_reference = {}, email = {}",
        booking_id.app_reference,
        booking_id.email
    );

    // Create ServerSideBookingEvent
    let event = ServerSideBookingEvent {
        payment_id: Some(payment_id.clone()),
        order_id: order_id.clone(),
        provider: "nowpayments".to_string(),
        user_email: user_email.clone(),
        payment_status: None,
        backend_payment_status: Some("confirmation_page_initiated".to_string()),
        backend_booking_status: None,
        backend_booking_struct: None,
    };

    // Create pipeline steps
    let payment_status_step =
        SSRBookingPipelineStep::PaymentStatus(GetPaymentStatusFromPaymentProvider);
    let book_room_step = SSRBookingPipelineStep::BookRoom(MakeBookingFromBookingProvider);
    let send_email_step = SSRBookingPipelineStep::SendEmail(SendEmailAfterSuccessfullBooking);

    let steps = vec![payment_status_step, book_room_step, send_email_step];

    tracing::info!(
        "Executing booking pipeline for order_id: {}, payment_id: {}",
        order_id,
        payment_id
    );

    // Execute the pipeline - this will publish events to the eventbus
    match process_pipeline(event, &steps, None).await {
        Ok(final_event) => {
            tracing::info!(
                "Booking pipeline completed successfully for payment_id: {}",
                payment_id
            );

            let success_response = json!({
                "success": true,
                "message": "Booking process initiated successfully. Check eventbus for updates.",
                "order_id": order_id,
                "user_email": user_email
            });

            (StatusCode::OK, success_response.to_string()).into_response()
        }
        Err(e) => {
            tracing::error!("Booking pipeline failed: {}", e);

            let error_response = json!({
                "success": false,
                "message": format!("Booking processing failed: {}", e),
                "order_id": Some(order_id),
                "user_email": Some(user_email)
            });

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                error_response.to_string(),
            )
                .into_response()
        }
    }
}

#[axum::debug_handler]
#[tracing::instrument(skip(state))]
pub async fn integrated_block_room_api_server_fn_route(
    State(state): State<AppState>,
    body: String,
) -> Response {
    tracing::info!(
        "Starting integrated block room API with body: {}",
        &body[0..100.min(body.len())]
    );

    // Parse the JSON request
    let request: IntegratedBlockRoomRequest = match parse_json_request(&body) {
        Ok(req) => {
            tracing::info!("Successfully parsed integrated block room request");
            req
        }
        Err(_) => {
            // Custom error for this endpoint with specific response format
            let error_response = json!({
                "success": false,
                "message": "Invalid JSON request format",
                "block_room_response": null,
                "booking_id": ""
            });
            return (StatusCode::BAD_REQUEST, error_response.to_string()).into_response();
        }
    };

    // Extract data from request
    let block_room_request = request.block_room_request;
    let booking_id = request.booking_id;
    let email = request.email;
    let hotel_token = request.hotel_token;

    tracing::info!(
        "Processing integrated block room for booking_id: {}, email: {}",
        booking_id,
        email
    );

    // Step 1: Call block room API using shared helper
    let block_result = match call_block_room_api(&state, block_room_request.clone()).await {
        Ok(response) => {
            tracing::info!(
                "Block room API call successful: block_id = {}",
                response.block_id
            );
            response
        }
        Err(e) => {
            tracing::error!("Block room API call failed: {:?}", e);
            let error_response = json!({
                "success": false,
                "message": format!("Block room failed: {}", e),
                "block_room_response": null,
                "booking_id": booking_id
            });
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                error_response.to_string(),
            )
                .into_response();
        }
    };

    // Step 2: Create backend booking using BookingBackendConversions
    let backend_booking_result = async {
        // Parse booking_id to get BookingId
        let booking_id_struct = BookingId::from_order_id(&booking_id).ok_or_else(|| {
            BookingError::ValidationError("Invalid booking_id format".to_string())
        })?;

        // Create backend booking from block room data
        let destination = block_room_request
            .hotel_info_criteria
            .search_criteria
            .destination_city_name;
        let destination_domain = estate_fe::domain::DomainDestination {
            city_id: block_room_request
                .hotel_info_criteria
                .search_criteria
                .destination_city_id,
            city_name: destination.clone(),
            country_code: block_room_request
                .hotel_info_criteria
                .search_criteria
                .destination_country_code,
            country_name: block_room_request
                .hotel_info_criteria
                .search_criteria
                .destination_country_name,
        };

        let date_range = estate_fe::domain::DomainSelectedDateRange {
            start: block_room_request
                .hotel_info_criteria
                .search_criteria
                .check_in_date,
            end: block_room_request
                .hotel_info_criteria
                .search_criteria
                .check_out_date,
        };

        // Extract room details from selected rooms
        let room_details: Vec<estate_fe::domain::DomainRoomData> = block_room_request
            .selected_rooms
            .into_iter()
            .map(|room| room.room_data)
            .collect();

        // Create temporary SelectedDateRange for date formatting
        let formatted_dates = estate_fe::component::SelectedDateRange {
            start: date_range.start,
            end: date_range.end,
        };

        // Extract user details from block room request
        let user_details = block_room_request.user_details;

        // Build hotel details
        let hotel_details = estate_fe::domain::DomainHotelDetails {
            checkin: formatted_dates.dd_month_yyyy_start(),
            checkout: formatted_dates.dd_month_yyyy_end(),
            hotel_name: "Hotel".to_string(), // Could be extracted from request if available
            hotel_code: block_room_request.hotel_info_criteria.token.clone(),
            star_rating: 4,
            description: "Hotel description".to_string(),
            hotel_facilities: vec![],
            address: "Hotel address".to_string(),
            images: vec![],
            all_rooms: room_details
                .iter()
                .map(|room_data| estate_fe::domain::DomainRoomOption {
                    price: block_result.total_price.clone(),
                    room_data: room_data.clone(),
                    meal_plan: None,
                    occupancy_info: None,
                })
                .collect(),
            amenities: vec![],
        };

        let payment_amount = block_result.total_price.room_price;
        let payment_currency = "USD".to_string();
        let block_room_id = Some(block_result.block_id.clone());

        // Create backend booking
        let backend_booking = BookingBackendConversions::create_backend_booking(
            Some(destination_domain),
            date_range,
            room_details,
            hotel_details,
            user_details,
            booking_id_struct,
            payment_amount,
            payment_currency,
            block_room_id,
            hotel_token,
        )?;

        // Validate the booking
        BookingBackendConversions::validate_backend_booking(&backend_booking)?;

        Ok::<_, BookingError>(backend_booking)
    }
    .await;

    let backend_booking = match backend_booking_result {
        Ok(booking) => booking,
        Err(e) => {
            tracing::error!("Failed to create backend booking: {:?}", e);
            let error_response = json!({
                "success": false,
                "message": format!("Failed to create backend booking: {}", e),
                "block_room_response": serde_json::to_value(&block_result).ok(),
                "booking_id": booking_id
            });
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                error_response.to_string(),
            )
                .into_response();
        }
    };

    // Step 3: Save to backend canister
    let canister_result = call_add_booking_backend(email.clone(), backend_booking).await;

    match canister_result {
        Ok(_) => {
            tracing::info!(
                "Successfully saved booking to backend canister for booking_id: {}",
                booking_id
            );

            let success_response = json!({
                "success": true,
                "message": "Room blocked and booking saved successfully",
                "block_room_response": serde_json::to_value(&block_result).ok(),
                "booking_id": booking_id
            });

            (StatusCode::OK, success_response.to_string()).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to save booking to canister: {}", e);

            // Return partial success - block room succeeded but canister save failed
            let error_response = json!({
                "success": false,
                "message": format!("Room blocked successfully, but failed to save to backend: {}", e),
                "block_room_response": serde_json::to_value(&block_result).ok(),
                "booking_id": booking_id
            });

            (StatusCode::PARTIAL_CONTENT, error_response.to_string()).into_response()
        }
    }
}

#[axum::debug_handler]
#[tracing::instrument(skip(_state))]
pub async fn create_payment_invoice_api_server_fn_route(
    State(_state): State<AppState>,
    body: String,
) -> Result<Response, Response> {
    tracing::info!(
        "Starting create payment invoice API with body: {}",
        &body[0..100.min(body.len())]
    );

    // Parse the JSON request
    let request: DomainCreateInvoiceRequest = {
        let req = parse_json_request(&body)?;
        tracing::info!("Successfully parsed payment invoice request");
        req
    };

    tracing::info!(
        "Processing payment request - Provider: {:?}, Order ID: {}, Amount: {}",
        request.provider,
        request.order_id,
        request.price_amount
    );

    // Initialize the payment service
    let payment_service = PaymentServiceImpl::new();

    // Create the invoice using the abstracted service
    let response = payment_service
        .create_invoice(request)
        .await
        .map_err(|payment_error| {
            tracing::error!("Payment invoice creation failed: {}", payment_error);
            let error_response = json!({
                "error": format!("Payment creation failed: {}", payment_error),
            });
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                error_response.to_string(),
            )
                .into_response()
        })?;

    tracing::info!(
        "Payment invoice created successfully - Invoice ID: {}, Payment URL: {}",
        response.invoice_id,
        response.payment_url
    );

    // Return the successful response
    let success_response = json!(response);
    Ok((StatusCode::OK, success_response.to_string()).into_response())
}
