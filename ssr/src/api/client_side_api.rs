use crate::api::consts::APP_URL;
use crate::api::payments::domain::{DomainCreateInvoiceRequest, DomainCreateInvoiceResponse};
use crate::api::payments::ports::GetPaymentStatusResponse;
use crate::canister::backend::{Booking, PaymentDetails};

use crate::domain::{
    DomainBlockRoomRequest, DomainBlockRoomResponse, DomainHotelCodeId, DomainHotelDetails,
    DomainHotelInfoCriteria, DomainHotelListAfterSearch, DomainHotelSearchCriteria,
};
use crate::log;
use crate::utils::route::join_base_and_path_url;
use candid::Principal;
use leptos::*;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;
// use yral_types::delegated_identity::DelegatedIdentityWire;

#[cfg(not(feature = "ssr"))]
use web_sys;

#[cfg(not(feature = "ssr"))]
use wasm_bindgen::JsCast;

// Import the integrated request/response types
use crate::application_services::booking_service::{
    IntegratedBlockRoomRequest, IntegratedBlockRoomResponse,
};

// // Import city search types
// use crate::server_functions_impl_custom_routes::search_cities::{
//     CitySearchResult, SearchCitiesRequest, SearchCitiesResponse,
// };

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmationProcessRequest {
    pub payment_id: Option<String>,
    pub app_reference: Option<String>,
    pub email: Option<String>,
    pub query_params: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmationProcessResponse {
    pub success: bool,
    pub message: String,
    pub order_id: Option<String>,
    pub user_email: Option<String>,
    pub booking_data: Option<serde_json::Value>,
}

// Admin Payment API Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckPaymentStatusRequest {
    pub payment_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetBackendBookingRequest {
    pub email: String,
    pub app_reference: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePaymentRequest {
    pub email: String,
    pub app_reference: String,
    pub payment_details: PaymentDetails,
}

// Email Verification API Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendOtpRequest {
    pub email: String,
    pub booking_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendOtpResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyOtpRequest {
    pub booking_id: String,
    pub otp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserPrincipalEmailRequest {
    pub user_email: String,
    pub principal: Principal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyOtpResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchCitiesRequest {
    pub prefix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchPlacesRequest {
    pub text_query: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SearchPlacesResponse {
    pub data: Vec<Place>,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Place {
    pub place_id: String,
    pub display_name: String,
    pub formatted_address: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlaceDetailsRequest {
    pub place_id: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceDetailsResponse {
    pub data: PlaceData,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceData {
    pub address_components: Vec<AddressComponent>,
    pub location: Location,
    pub viewport: Viewport,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddressComponent {
    pub language_code: String,
    pub long_text: String,
    pub short_text: String,
    pub types: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Viewport {
    pub high: High,
    pub low: Low,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct High {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Low {
    pub latitude: f64,
    pub longitude: f64,
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

#[derive(Clone)]
pub struct ClientSideApiClient;

impl ClientSideApiClient {
    pub fn new() -> Self {
        Self
    }

    fn serialize_request<T: Serialize>(request: &T, context: &str) -> Option<String> {
        match serde_json::to_string(request) {
            Ok(json) => Some(json),
            Err(e) => {
                log!("Failed to serialize {} request: {}", context, e);
                None
            }
        }
    }

    fn build_api_url(endpoint: &str) -> String {
        join_base_and_path_url(APP_URL.as_str(), endpoint).unwrap_or_else(|e| {
            log!("Failed to build URL: {}", e);
            format!("{}/{}", APP_URL.as_str(), endpoint)
        })
    }

    fn get_basic_auth_header() -> Option<String> {
        #[cfg(not(feature = "ssr"))]
        {
            use base64::{engine::general_purpose, Engine as _};

            // In a real application, you would get these from secure storage
            // For now, we'll use browser prompt or stored credentials
            if let Some(window) = web_sys::window() {
                // Try to get stored credentials from sessionStorage
                if let Ok(Some(storage)) = window.session_storage() {
                    if let Ok(Some(username)) = storage.get_item("admin_username") {
                        if let Ok(Some(password)) = storage.get_item("admin_password") {
                            let credentials = format!("{}:{}", username, password);
                            let encoded = general_purpose::STANDARD.encode(credentials.as_bytes());
                            return Some(format!("Basic {}", encoded));
                        }
                    }
                }

                // Fallback: prompt user for credentials
                if let Ok(Some(username)) = window.prompt_with_message("Enter admin username:") {
                    if let Ok(Some(password)) = window.prompt_with_message("Enter admin password:")
                    {
                        let credentials = format!("{}:{}", username, password);
                        let encoded = general_purpose::STANDARD.encode(credentials.as_bytes());

                        // Store credentials in sessionStorage for this session
                        if let Ok(Some(storage)) = window.session_storage() {
                            let _ = storage.set_item("admin_username", &username);
                            let _ = storage.set_item("admin_password", &password);
                        }

                        return Some(format!("Basic {}", encoded));
                    }
                }
            }
            None
        }
        #[cfg(feature = "ssr")]
        {
            None
        }
    }

    fn log_browser_environment() {
        #[cfg(not(feature = "ssr"))]
        {
            if let Some(window) = web_sys::window() {
                if let Some(_document) = window.document() {
                    log!("[CLIENT_API_DEBUG] Browser environment detected: cookies will be handled automatically by reqwest/browser");
                } else {
                    log!("[CLIENT_API_DEBUG] No document available in browser environment");
                }
            } else {
                log!("[CLIENT_API_DEBUG] No window available - not in browser environment");
            }
        }

        #[cfg(feature = "ssr")]
        {
            log!("[CLIENT_API_DEBUG] Server-side environment - no browser cookies available");
        }
    }

    async fn make_post_request<T: DeserializeOwned>(
        endpoint: &str,
        body: String,
        context: &str,
    ) -> Result<T, String> {
        log!("[CLIENT_API_DEBUG] Making POST request to: {}", endpoint);
        log!("[CLIENT_API_DEBUG] Request context: {}", context);
        log!("[CLIENT_API_DEBUG] Request body: {}", body);

        #[cfg(not(feature = "ssr"))]
        log!("[CLIENT_COOKIE_AUTO] Running in client-side environment - cookies will be handled");

        #[cfg(feature = "ssr")]
        log!("[CLIENT_COOKIE_AUTO] Running in server-side environment - no cookie handling");

        // Log browser environment and cookie handling approach
        Self::log_browser_environment();

        // Create client - in WASM, cookies are handled automatically by the browser
        let client = reqwest::Client::new();

        let mut request_builder = client
            .post(Self::build_api_url(endpoint))
            .header("Content-Type", "application/json")
            .body(body);

        // Add basic auth headers for admin endpoints
        if endpoint.contains("/admin/") {
            if let Some(auth_header) = Self::get_basic_auth_header() {
                // tracing::debug!("[CLIENT_API_DEBUG] Adding basic auth header for admin endpoint");
                request_builder = request_builder.header("Authorization", auth_header);
            }
        }

        // Note: In WASM/browser environments, reqwest automatically handles cookies
        // via the browser's cookie store for same-origin requests

        // tracing::debug!("[CLIENT_API_DEBUG] Sending HTTP request...");
        let response = request_builder.send().await;

        match response {
            Ok(res) => {
                let status = res.status();
                // tracing::debug!("[CLIENT_API_DEBUG] Response status: {}", status);
                // tracing::debug!("[CLIENT_API_DEBUG] Response headers: {:#?}", res.headers());

                // Note: In browser environments, Set-Cookie headers are handled automatically by the browser
                // and are not accessible via reqwest/fetch API for security reasons
                #[cfg(not(feature = "ssr"))]
                {
                    log!("[CLIENT_COOKIE_AUTO] Cookies from Set-Cookie headers are handled automatically by browser");
                    log!("[CLIENT_COOKIE_AUTO] Checking if cookies were set by reading document.cookie...");

                    if let Some(window) = web_sys::window() {
                        if let Some(document) = window.document() {
                            if let Ok(html_document) = document.dyn_into::<web_sys::HtmlDocument>()
                            {
                                match html_document.cookie() {
                                    Ok(cookie_string) => {
                                        log!(
                                            "[CLIENT_COOKIE_AUTO] Current browser cookies: {}",
                                            cookie_string
                                        );

                                        // Check for specific auth cookies
                                        if cookie_string.contains("google-csrf-token") {
                                            log!("[CLIENT_COOKIE_AUTO] ✅ google-csrf-token cookie detected");
                                        }
                                        if cookie_string.contains("google-pkce-verifier") {
                                            log!("[CLIENT_COOKIE_AUTO] ✅ google-pkce-verifier cookie detected");
                                        }
                                    }
                                    Err(e) => {
                                        log!("[CLIENT_COOKIE_AUTO] ❌ Failed to read document.cookie: {:?}", e);
                                    }
                                }
                            }
                        }
                    }
                    log!("[CLIENT_COOKIE_AUTO] Cookie check completed");
                }

                let text_result = res.text().await;
                match text_result {
                    Ok(text) => {
                        log!("[CLIENT_API_DEBUG] Response body: {}", text);
                        if status.is_success() {
                            Self::parse_server_response(&text)
                        } else {
                            // Handle error responses (400, 422, etc.) by extracting the error message
                            let error_msg = if let Ok(error_json) =
                                serde_json::from_str::<serde_json::Value>(&text)
                            {
                                if let Some(error_msg) =
                                    error_json.get("error").and_then(|v| v.as_str())
                                {
                                    log!(
                                        "{} API call failed with status {}: {}",
                                        context,
                                        status,
                                        error_msg
                                    );
                                    error_msg.to_string()
                                } else {
                                    log!(
                                        "{} API call failed with status {}: {}",
                                        context,
                                        status,
                                        text
                                    );
                                    format!("API call failed with status {}", status)
                                }
                            } else {
                                log!(
                                    "{} API call failed with status {}: {}",
                                    context,
                                    status,
                                    text
                                );
                                format!("API call failed with status {}", status)
                            };
                            Err(error_msg)
                        }
                    }
                    Err(e) => {
                        log!("Failed to get {} response text: {}", context, e);
                        Err(format!("Failed to get response text: {}", e))
                    }
                }
            }
            Err(e) => {
                log!("{} API call error: {}", context, e);
                Err(format!("Network error: {}", e))
            }
        }
    }

    async fn api_call<Req: Serialize, Res: DeserializeOwned>(
        request: Req,
        endpoint: &str,
        context: &str,
    ) -> Option<Res> {
        let body = Self::serialize_request(&request, context)?;
        Self::make_post_request(endpoint, body, context).await.ok()
    }

    async fn api_call_with_error<Req: Serialize, Res: DeserializeOwned>(
        request: Req,
        endpoint: &str,
        context: &str,
    ) -> Result<Res, String> {
        let body = Self::serialize_request(&request, context)
            .ok_or_else(|| format!("Failed to serialize {} request", context))?;
        Self::make_post_request(endpoint, body, context).await
    }

    pub async fn search_hotel(
        &self,
        request: DomainHotelSearchCriteria,
    ) -> Option<DomainHotelListAfterSearch> {
        Self::api_call(request, "server_fn_api/search_hotel_api", "search hotel").await
    }

    pub async fn search_cities(&self, prefix: String) -> Result<Vec<CitySearchResult>, String> {
        let request = SearchCitiesRequest { prefix };
        let response: SearchCitiesResponse =
            Self::api_call_with_error(request, "server_fn_api/search_cities_api", "search cities")
                .await?;
        Ok(response.cities)
    }

    pub async fn search_places(&self, query: String) -> Result<Vec<Place>, String> {
        let request = SearchPlacesRequest { text_query: query };
        let response: SearchPlacesResponse =
            Self::api_call_with_error(request, "server_fn_api/search_place_api", "search places")
                .await?;
        Ok(response.data)
    }

    pub async fn get_place_details_by_id(&self, place_id: String) -> Result<PlaceData, String> {
        let request = PlaceDetailsRequest { place_id };
        let response: PlaceDetailsResponse = Self::api_call_with_error(
            request,
            "server_fn_api/get_place_details_api",
            "get place details",
        )
        .await?;
        Ok(response.data)
    }

    pub async fn get_hotel_info(
        &self,
        request: DomainHotelInfoCriteria,
    ) -> Result<DomainHotelDetails, String> {
        Self::api_call_with_error(
            request,
            "server_fn_api/get_hotel_info_api",
            "get hotel info",
        )
        .await
    }

    pub async fn get_hotel_static_details(
        &self,
        request: DomainHotelCodeId,
    ) -> Result<crate::domain::DomainHotelStaticDetails, String> {
        Self::api_call_with_error(
            request,
            "server_fn_api/get_hotel_static_details_api",
            "get hotel static details",
        )
        .await
    }

    pub async fn get_hotel_rates(
        &self,
        request: DomainHotelInfoCriteria,
    ) -> Result<crate::domain::DomainGroupedRoomRates, String> {
        Self::api_call_with_error(
            request,
            "server_fn_api/get_hotel_rates_api",
            "get hotel rates",
        )
        .await
    }

    pub async fn block_room(
        &self,
        request: DomainBlockRoomRequest,
    ) -> Option<DomainBlockRoomResponse> {
        Self::api_call(request, "server_fn_api/block_room_api", "block room").await
    }

    pub async fn process_confirmation(
        &self,
        request: ConfirmationProcessRequest,
    ) -> Option<ConfirmationProcessResponse> {
        Self::api_call(
            request,
            "server_fn_api/process_confirmation_api",
            "process confirmation",
        )
        .await
    }

    pub async fn integrated_block_room(
        &self,
        request: IntegratedBlockRoomRequest,
    ) -> Option<IntegratedBlockRoomResponse> {
        Self::api_call(
            request,
            "server_fn_api/integrated_block_room_api",
            "integrated block room",
        )
        .await
    }

    // <!-- Helper function for parsing server responses -->
    pub fn parse_server_response<T: DeserializeOwned>(response: &str) -> Result<T, String> {
        serde_json::from_str(response)
            .map_err(|e| format!("Failed to parse server response: {}", e))
    }

    pub async fn create_payment_invoice(
        &self,
        request: DomainCreateInvoiceRequest,
    ) -> Option<DomainCreateInvoiceResponse> {
        let result = Self::api_call::<_, DomainCreateInvoiceResponse>(
            request,
            "server_fn_api/create_payment_invoice_api",
            "create payment invoice",
        )
        .await;

        if result.is_some() {
            log!("Payment invoice created successfully");
        }

        result
    }

    // Admin Payment API Methods
    pub async fn check_payment_status(
        &self,
        request: CheckPaymentStatusRequest,
    ) -> Option<GetPaymentStatusResponse> {
        Self::api_call(
            request,
            "server_fn_api/admin/check_payment_status",
            "check payment status",
        )
        .await
    }

    pub async fn get_backend_booking(&self, request: GetBackendBookingRequest) -> Option<Booking> {
        Self::api_call(
            request,
            "server_fn_api/admin/get_backend_booking",
            "get backend booking",
        )
        .await
    }

    pub async fn update_payment_details(&self, request: UpdatePaymentRequest) -> Option<String> {
        Self::api_call(
            request,
            "server_fn_api/admin/update_payment",
            "update payment details",
        )
        .await
    }

    // Email Verification API Methods
    pub async fn send_otp_email(
        &self,
        email: String,
        booking_id: String,
    ) -> Result<SendOtpResponse, String> {
        let request = SendOtpRequest { email, booking_id };
        Self::api_call_with_error(
            request,
            "server_fn_api/send_otp_email_api",
            "send OTP email",
        )
        .await
    }

    pub async fn verify_otp(
        &self,
        booking_id: String,
        otp: String,
    ) -> Result<VerifyOtpResponse, String> {
        let request = VerifyOtpRequest { booking_id, otp };
        Self::api_call_with_error(request, "server_fn_api/verify_otp_api", "verify OTP").await
    }

    pub async fn update_user_principal_email_mapping_in_canister_client_side_fn(
        &self,
        principal: Principal,
        user_email: String,
    ) -> Result<String, String> {
        let request = UpdateUserPrincipalEmailRequest {
            principal,
            user_email,
        };
        Self::api_call_with_error(
            request,
            "server_fn_api/update_user_principal_email_mapping_in_canister_server_fn",
            "update user principal email mapping",
        )
        .await
    }

    /// Get hotel details without rates using hotel ID
    pub async fn get_hotel_details_without_rates(
        &self,
        hotel_id: &str,
    ) -> Result<crate::domain::DomainHotelDetails, String> {
        let url = format!(
            "server_fn_api/get_hotel_static_details_api?hotel_id={}",
            hotel_id
        );

        #[cfg(not(feature = "ssr"))]
        {
            use gloo_net::http::Request;

            let response = Request::get(&url)
                .send()
                .await
                .map_err(|e| format!("Network error: {}", e))?;

            if !response.ok() {
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                return Err(format!("HTTP {}: {}", response.status(), error_text));
            }

            let response_text = response
                .text()
                .await
                .map_err(|e| format!("Failed to read response: {}", e))?;

            serde_json::from_str(&response_text)
                .map_err(|e| format!("Failed to parse response: {}", e))
        }

        #[cfg(feature = "ssr")]
        {
            Err("Hotel details API not available on server side".to_string())
        }
    }
}

impl Default for ClientSideApiClient {
    fn default() -> Self {
        Self::new()
    }
}

// <!-- Public helper function for use in components -->
pub fn parse_api_response<T: DeserializeOwned>(response: &str) -> Result<T, String> {
    ClientSideApiClient::parse_server_response(response)
}
