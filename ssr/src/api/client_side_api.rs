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
}

#[derive(Clone)]
pub struct ClientSideApiClient;

impl ClientSideApiClient {
    pub fn new() -> Self {
        Self
    }

    pub async fn search_hotel(
        &self,
        request: DomainHotelSearchCriteria,
    ) -> Option<DomainHotelListAfterSearch> {
        // <!-- Serialize request to JSON string -->
        let body = match serde_json::to_string(&request) {
            Ok(json) => json,
            Err(e) => {
                log!("Failed to serialize request: {}", e);
                return None;
            }
        };

        let client = reqwest::Client::new();
        let response = client
            .post(
                join_base_and_path_url(APP_URL.as_str(), "server_fn_api/search_hotel_api")
                    .unwrap_or_else(|e| {
                        log!("Failed to build URL: {}", e);
                        format!("{}server_fn_api/search_hotel_api", APP_URL.as_str())
                    }),
            )
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
                            log!("Failed to get response text: {}", e);
                            None
                        }
                    }
                } else {
                    log!("API call failed with status: {}", res.status());
                    None
                }
            }
            Err(e) => {
                log!("API call error: {}", e);
                None
            }
        }
    }

    pub async fn get_hotel_info(
        &self,
        request: DomainHotelInfoCriteria,
    ) -> Option<DomainHotelDetails> {
        // <!-- Serialize request to JSON string -->
        let body = match serde_json::to_string(&request) {
            Ok(json) => json,
            Err(e) => {
                log!("Failed to serialize request: {}", e);
                return None;
            }
        };

        let client = reqwest::Client::new();
        let response = client
            .post(
                join_base_and_path_url(APP_URL.as_str(), "server_fn_api/get_hotel_info_api")
                    .unwrap_or_else(|e| {
                        log!("Failed to build URL: {}", e);
                        format!("{}/server_fn_api/get_hotel_info_api", APP_URL.as_str())
                    }),
            )
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
                            log!("Failed to get response text: {}", e);
                            None
                        }
                    }
                } else {
                    log!("API call failed with status: {}", res.status());
                    None
                }
            }
            Err(e) => {
                log!("API call error: {}", e);
                None
            }
        }
    }

    pub async fn get_hotel_rates(
        &self,
        request: DomainHotelInfoCriteria,
    ) -> Option<DomainHotelDetails> {
        // <!-- Serialize request to JSON string -->
        let body = match serde_json::to_string(&request) {
            Ok(json) => json,
            Err(e) => {
                log!("Failed to serialize request: {}", e);
                return None;
            }
        };

        let client = reqwest::Client::new();
        let response = client
            .post(
                join_base_and_path_url(APP_URL.as_str(), "server_fn_api/get_hotel_rates_api")
                    .unwrap_or_else(|e| {
                        log!("Failed to build URL: {}", e);
                        format!("{}/server_fn_api/get_hotel_rates_api", APP_URL.as_str())
                    }),
            )
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
                            log!("Failed to get response text: {}", e);
                            None
                        }
                    }
                } else {
                    log!("API call failed with status: {}", res.status());
                    None
                }
            }
            Err(e) => {
                log!("API call error: {}", e);
                None
            }
        }
    }

    pub async fn block_room(
        &self,
        request: DomainBlockRoomRequest,
    ) -> Option<DomainBlockRoomResponse> {
        // <!-- Serialize request to JSON string -->
        let body = match serde_json::to_string(&request) {
            Ok(json) => json,
            Err(e) => {
                log!("Failed to serialize block room request: {}", e);
                return None;
            }
        };

        let client = reqwest::Client::new();
        let response = client
            .post(
                join_base_and_path_url(APP_URL.as_str(), "server_fn_api/block_room_api")
                    .unwrap_or_else(|e| {
                        log!("Failed to build URL: {}", e);
                        format!("{}/server_fn_api/block_room_api", APP_URL.as_str())
                    }),
            )
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
                            log!("Failed to get block room response text: {}", e);
                            None
                        }
                    }
                } else {
                    log!("Block room API call failed with status: {}", res.status());
                    None
                }
            }
            Err(e) => {
                log!("Block room API call error: {}", e);
                None
            }
        }
    }

    pub async fn process_confirmation(
        &self,
        request: ConfirmationProcessRequest,
    ) -> Option<ConfirmationProcessResponse> {
        // Serialize request to JSON string
        let body = match serde_json::to_string(&request) {
            Ok(json) => json,
            Err(e) => {
                log!("Failed to serialize confirmation request: {}", e);
                return None;
            }
        };

        let client = reqwest::Client::new();
        let response = client
            .post(
                join_base_and_path_url(APP_URL.as_str(), "server_fn_api/process_confirmation_api")
                    .unwrap_or_else(|e| {
                        log!("Failed to build URL: {}", e);
                        format!(
                            "{}/server_fn_api/process_confirmation_api",
                            APP_URL.as_str()
                        )
                    }),
            )
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
                            log!("Failed to get confirmation response text: {}", e);
                            None
                        }
                    }
                } else {
                    log!("Confirmation API call failed with status: {}", res.status());
                    None
                }
            }
            Err(e) => {
                log!("Confirmation API call error: {}", e);
                None
            }
        }
    }

    pub async fn integrated_block_room(
        &self,
        request: IntegratedBlockRoomRequest,
    ) -> Option<IntegratedBlockRoomResponse> {
        // Serialize request to JSON string
        let body = match serde_json::to_string(&request) {
            Ok(json) => json,
            Err(e) => {
                log!("Failed to serialize integrated block room request: {}", e);
                return None;
            }
        };

        let client = reqwest::Client::new();
        let response = client
            .post(
                join_base_and_path_url(APP_URL.as_str(), "server_fn_api/integrated_block_room_api")
                    .unwrap_or_else(|e| {
                        log!("Failed to build URL: {}", e);
                        format!(
                            "{}/server_fn_api/integrated_block_room_api",
                            APP_URL.as_str()
                        )
                    }),
            )
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
                            log!("Failed to get integrated block room response text: {}", e);
                            None
                        }
                    }
                } else {
                    log!(
                        "Integrated block room API call failed with status: {}",
                        res.status()
                    );
                    None
                }
            }
            Err(e) => {
                log!("Integrated block room API call error: {}", e);
                None
            }
        }
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
        // Serialize request to JSON string
        let body = match serde_json::to_string(&request) {
            Ok(json) => json,
            Err(e) => {
                log!("Failed to serialize payment invoice request: {}", e);
                return None;
            }
        };

        // Make HTTP request to the payment API endpoint
        let client = reqwest::Client::new();
        let response = client
            .post(
                join_base_and_path_url(
                    APP_URL.as_str(),
                    "server_fn_api/create_payment_invoice_api",
                )
                .unwrap_or_else(|e| {
                    log!("Failed to build URL: {}", e);
                    format!(
                        "{}/server_fn_api/create_payment_invoice_api",
                        APP_URL.as_str()
                    )
                }),
            )
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await;

        match response {
            Ok(resp) => {
                let status = resp.status();
                let text = resp.text().await.unwrap_or_default();

                if status == 200 {
                    match Self::parse_server_response::<DomainCreateInvoiceResponse>(&text) {
                        Ok(payment_response) => {
                            log!("Payment invoice created successfully");
                            Some(payment_response)
                        }
                        Err(e) => {
                            log!("Failed to parse payment response: {}", e);
                            None
                        }
                    }
                } else {
                    log!("Payment API error - Status: {}, Response: {}", status, text);
                    None
                }
            }
            Err(e) => {
                log!("Payment API request failed: {:?}", e);
                None
            }
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
