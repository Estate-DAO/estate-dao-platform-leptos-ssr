use crate::api::consts::APP_URL;
use crate::domain::{
    DomainHotelDetails, DomainHotelInfoCriteria, DomainHotelListAfterSearch,
    DomainHotelSearchCriteria,
};
use crate::log;
use leptos::*;
use serde::de::DeserializeOwned;

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
            .post(format!(
                "{}server_fn_api/search_hotel_api",
                APP_URL.as_str()
            ))
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
            .post(format!(
                "{}/server_fn_api/get_hotel_info_api",
                APP_URL.as_str()
            ))
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
            .post(format!(
                "{}/server_fn_api/get_hotel_rates_api",
                APP_URL.as_str()
            ))
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

    // <!-- Helper function for parsing server responses -->
    pub fn parse_server_response<T: DeserializeOwned>(response: &str) -> Result<T, String> {
        serde_json::from_str(response)
            .map_err(|e| format!("Failed to parse server response: {}", e))
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
