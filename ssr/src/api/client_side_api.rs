use crate::api::consts::APP_URL;
use crate::api::payments::domain::{DomainCreateInvoiceRequest, DomainCreateInvoiceResponse};
use crate::domain::{
    DomainBlockRoomRequest, DomainBlockRoomResponse, DomainHotelDetails, DomainHotelInfoCriteria,
    DomainHotelListAfterSearch, DomainHotelSearchCriteria,
};
use crate::log;
use crate::utils::route::join_base_and_path_url;
use leptos::*;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;

// Import the integrated request/response types
use crate::application_services::booking_service::{
    IntegratedBlockRoomRequest, IntegratedBlockRoomResponse,
};

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

    async fn make_post_request<T: DeserializeOwned>(
        endpoint: &str,
        body: String,
        context: &str,
    ) -> Option<T> {
        let client = reqwest::Client::new();
        let response = client
            .post(Self::build_api_url(endpoint))
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await;

        match response {
            Ok(res) => {
                if res.status().is_success() {
                    match res.text().await {
                        Ok(text) => Self::parse_server_response(&text).ok(),
                        Err(e) => {
                            log!("Failed to get {} response text: {}", context, e);
                            None
                        }
                    }
                } else {
                    log!("{} API call failed with status: {}", context, res.status());
                    None
                }
            }
            Err(e) => {
                log!("{} API call error: {}", context, e);
                None
            }
        }
    }

    async fn api_call<Req: Serialize, Res: DeserializeOwned>(
        request: Req,
        endpoint: &str,
        context: &str,
    ) -> Option<Res> {
        let body = Self::serialize_request(&request, context)?;
        Self::make_post_request(endpoint, body, context).await
    }

    pub async fn search_hotel(
        &self,
        request: DomainHotelSearchCriteria,
    ) -> Option<DomainHotelListAfterSearch> {
        Self::api_call(request, "server_fn_api/search_hotel_api", "search hotel").await
    }

    pub async fn get_hotel_info(
        &self,
        request: DomainHotelInfoCriteria,
    ) -> Option<DomainHotelDetails> {
        Self::api_call(
            request,
            "server_fn_api/get_hotel_info_api",
            "get hotel info",
        )
        .await
    }

    pub async fn get_hotel_rates(
        &self,
        request: DomainHotelInfoCriteria,
    ) -> Option<DomainHotelDetails> {
        Self::api_call(
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
