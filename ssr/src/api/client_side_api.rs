use crate::api::consts::APP_URL;
use crate::domain::{DomainHotelListAfterSearch, DomainHotelSearchCriteria};
use crate::log;
use leptos::*;

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
        let client = reqwest::Client::new();
        let response = client
            .post(format!(
                "{}/server_fn_api/search_hotel_api",
                APP_URL.as_str()
            ))
            .json(&request)
            .send()
            .await;

        match response {
            Ok(res) => {
                if res.status().is_success() {
                    res.json::<DomainHotelListAfterSearch>().await.ok()
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
}

impl Default for ClientSideApiClient {
    fn default() -> Self {
        Self::new()
    }
}
