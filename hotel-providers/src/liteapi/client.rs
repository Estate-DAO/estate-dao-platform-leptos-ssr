use crate::liteapi::models::booking::{
    LiteApiBookRequest, LiteApiBookResponse, LiteApiGetBookingRequest, LiteApiGetBookingResponse,
    LiteApiPrebookRequest, LiteApiPrebookResponse,
};
use crate::liteapi::models::hotel_details::{
    LiteApiHotelDataResponse, LiteApiHotelSearchRequest, LiteApiHotelSearchResponse,
};
use crate::liteapi::models::places::{
    LiteApiGetPlaceRequest, LiteApiGetPlaceResponse, LiteApiGetPlacesRequest,
    LiteApiGetPlacesResponse,
};
use crate::liteapi::models::search::{
    LiteApiHotelRatesRequest, LiteApiHotelRatesResponse, LiteApiMinRatesRequest,
    LiteApiMinRatesResponse,
};
use crate::ports::{ProviderError, ProviderErrorKind, ProviderSteps};
use reqwest::{Client, StatusCode};
use serde::de::DeserializeOwned;
use serde::Serialize;

#[derive(Clone, Debug)]
pub struct LiteApiClient {
    client: Client,
    api_key: String,
    base_url: String,
    /// Currency code for API requests (e.g., "USD", "SGD", "EUR")
    currency: String,
}

impl LiteApiClient {
    pub fn new(api_key: String, base_url: Option<String>) -> Self {
        Self::with_currency(api_key, base_url, "USD".to_string())
    }

    /// Create a new client with a specific currency
    pub fn with_currency(api_key: String, base_url: Option<String>, currency: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: base_url.unwrap_or_else(|| "https://api.liteapi.travel/v3.0".to_string()),
            currency,
        }
    }

    /// Get the configured currency
    pub fn currency(&self) -> &str {
        &self.currency
    }

    async fn post<T, R>(
        &self,
        path: &str,
        body: &T,
        step: ProviderSteps,
    ) -> Result<R, ProviderError>
    where
        T: Serialize,
        R: DeserializeOwned,
    {
        let url = self.build_url(path);
        let resp = self
            .client
            .post(&url)
            .header("X-API-Key", &self.api_key)
            .json(body)
            .send()
            .await
            .map_err(|e| ProviderError::network("LiteAPI", step.clone(), e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(self.map_api_error(status, text, step));
        }

        // Get response text first for debugging
        let response_text = resp.text().await.map_err(|e| {
            ProviderError::parse_error(
                "LiteAPI",
                step.clone(),
                format!("Failed to read response: {}", e),
            )
        })?;

        // Try to parse the response
        serde_json::from_str(&response_text).map_err(|e| {
            // Log first 500 chars of response for debugging
            let preview: String = response_text.chars().take(500).collect();
            tracing::error!(
                target: "hotel_providers::liteapi",
                path = %path,
                error = %e,
                response_preview = %preview,
                "Failed to parse LiteAPI response"
            );
            ProviderError::parse_error("LiteAPI", step, e.to_string())
        })
    }

    async fn get<R>(&self, path: &str, step: ProviderSteps) -> Result<R, ProviderError>
    where
        R: DeserializeOwned,
    {
        let url = self.build_url(path);
        let api_key_prefix = if self.api_key.len() > 8 {
            &self.api_key[..8]
        } else {
            &self.api_key
        };
        tracing::debug!(
            target: "hotel_providers::liteapi",
            url = %url,
            api_key_prefix = %api_key_prefix,
            step = ?step,
            "LiteAPI GET request"
        );

        let resp = self
            .client
            .get(&url)
            .header("X-API-Key", &self.api_key)
            .send()
            .await
            .map_err(|e| {
                tracing::error!(target: "hotel_providers::liteapi", error = %e, "Network error");
                ProviderError::network("LiteAPI", step.clone(), e.to_string())
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            tracing::error!(
                target: "hotel_providers::liteapi",
                status = %status,
                response_body = %text,
                "LiteAPI error response"
            );
            return Err(self.map_api_error(status, text, step));
        }

        resp.json::<R>().await.map_err(|e| {
            tracing::error!(target: "hotel_providers::liteapi", error = %e, "Parse error");
            ProviderError::parse_error("LiteAPI", step, e.to_string())
        })
    }

    fn build_url(&self, path: &str) -> String {
        if (path.starts_with("/rates/prebook") || path.starts_with("/rates/book"))
            && self.base_url.contains("api.liteapi.travel")
        {
            // Switch to book.liteapi.travel if default api url is used
            self.base_url
                .replace("api.liteapi.travel", "book.liteapi.travel")
                + path
        } else {
            format!("{}{}", self.base_url, path)
        }
    }

    fn map_api_error(
        &self,
        status: StatusCode,
        text: String,
        step: ProviderSteps,
    ) -> ProviderError {
        let msg = format!("LiteAPI Error {}: {}", status, text);
        match status {
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                ProviderError::new("LiteAPI", ProviderErrorKind::Auth, step, msg)
            }
            StatusCode::NOT_FOUND => ProviderError::not_found("LiteAPI", step, msg),
            StatusCode::TOO_MANY_REQUESTS => {
                ProviderError::new("LiteAPI", ProviderErrorKind::RateLimited, step, msg)
            }
            StatusCode::INTERNAL_SERVER_ERROR
            | StatusCode::BAD_GATEWAY
            | StatusCode::SERVICE_UNAVAILABLE => {
                ProviderError::service_unavailable("LiteAPI", step, msg)
            }
            _ => ProviderError::new("LiteAPI", ProviderErrorKind::Other, step, msg),
        }
    }

    pub async fn get_min_rates(
        &self,
        req: &LiteApiMinRatesRequest,
    ) -> Result<LiteApiMinRatesResponse, ProviderError> {
        self.post("/hotels/min-rates", req, ProviderSteps::HotelSearch)
            .await
    }

    pub async fn get_hotel_rates(
        &self,
        req: &LiteApiHotelRatesRequest,
    ) -> Result<LiteApiHotelRatesResponse, ProviderError> {
        self.post("/hotels/rates", req, ProviderSteps::HotelSearch)
            .await
    }

    pub async fn get_hotels(
        &self,
        req: &LiteApiHotelSearchRequest,
    ) -> Result<LiteApiHotelSearchResponse, ProviderError> {
        let url = format!("{}{}", self.base_url, "/data/hotels");
        let resp = self
            .client
            .get(&url)
            .header("X-API-Key", &self.api_key)
            .query(req)
            .send()
            .await
            .map_err(|e| {
                ProviderError::network("LiteAPI", ProviderSteps::HotelSearch, e.to_string())
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(self.map_api_error(status, text, ProviderSteps::HotelSearch));
        }

        resp.json::<LiteApiHotelSearchResponse>()
            .await
            .map_err(|e| {
                ProviderError::parse_error("LiteAPI", ProviderSteps::HotelSearch, e.to_string())
            })
    }

    pub async fn prebook(
        &self,
        req: &LiteApiPrebookRequest,
    ) -> Result<LiteApiPrebookResponse, ProviderError> {
        self.post("/rates/prebook", req, ProviderSteps::HotelBlockRoom)
            .await
    }

    pub async fn book(
        &self,
        req: &LiteApiBookRequest,
    ) -> Result<LiteApiBookResponse, ProviderError> {
        self.post("/rates/book", req, ProviderSteps::HotelBookRoom)
            .await
    }

    pub async fn get_hotel_details(
        &self,
        hotel_ids: Vec<String>,
    ) -> Result<LiteApiHotelDataResponse, ProviderError> {
        let ids = hotel_ids.join(",");
        let path = format!("/data/hotels?hotelIds={}", ids);
        self.get(&path, ProviderSteps::HotelDetails).await
    }

    pub async fn get_hotel_static_details(
        &self,
        hotel_id: &str,
    ) -> Result<
        crate::liteapi::models::hotel_details::LiteApiSingleHotelDetailResponse,
        ProviderError,
    > {
        // LiteAPI expects hotelId as a query parameter, not path parameter
        let url = self.build_url("/data/hotel");

        let api_key_prefix = if self.api_key.len() > 8 {
            &self.api_key[..8]
        } else {
            &self.api_key
        };

        tracing::debug!(
            target: "hotel_providers::liteapi",
            url = %url,
            hotel_id = %hotel_id,
            api_key_prefix = %api_key_prefix,
            "GET request for hotel static details"
        );

        let resp = self
            .client
            .get(&url)
            .header("X-API-Key", &self.api_key)
            .query(&[("hotelId", hotel_id)])
            .send()
            .await
            .map_err(|e| {
                ProviderError::network("LiteAPI", ProviderSteps::HotelDetails, e.to_string())
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            tracing::error!(
                target: "hotel_providers::liteapi",
                status = %status,
                response_body = %text,
                "LiteAPI error response"
            );
            return Err(self.map_api_error(status, text, ProviderSteps::HotelDetails));
        }

        // Get response text first for debugging
        let response_text = resp.text().await.map_err(|e| {
            ProviderError::parse_error(
                "LiteAPI",
                ProviderSteps::HotelDetails,
                format!("Failed to read response: {}", e),
            )
        })?;

        // Try to parse the response
        serde_json::from_str(&response_text).map_err(|e| {
            let preview: String = response_text.chars().take(500).collect();
            tracing::error!(
                target: "hotel_providers::liteapi",
                path = "/data/hotel",
                error = %e,
                response_preview = %preview,
                "Failed to parse LiteAPI response"
            );
            ProviderError::parse_error("LiteAPI", ProviderSteps::HotelDetails, e.to_string())
        })
    }

    pub async fn get_booking_details(
        &self,
        req: &LiteApiGetBookingRequest,
    ) -> Result<LiteApiGetBookingResponse, ProviderError> {
        let url = format!("{}{}", self.build_url("/bookings"), "");
        let resp = self
            .client
            .get(&url)
            .header("X-API-Key", &self.api_key)
            .query(req)
            .send()
            .await
            .map_err(|e| {
                ProviderError::network("LiteAPI", ProviderSteps::GetBookingDetails, e.to_string())
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(self.map_api_error(status, text, ProviderSteps::GetBookingDetails));
        }

        resp.json::<LiteApiGetBookingResponse>().await.map_err(|e| {
            ProviderError::parse_error("LiteAPI", ProviderSteps::GetBookingDetails, e.to_string())
        })
    }

    pub async fn get_places(
        &self,
        req: &LiteApiGetPlacesRequest,
    ) -> Result<LiteApiGetPlacesResponse, ProviderError> {
        let url = format!("{}{}", self.base_url, "/data/places");
        let resp = self
            .client
            .get(&url)
            .header("X-API-Key", &self.api_key)
            .query(req)
            .send()
            .await
            .map_err(|e| {
                ProviderError::network("LiteAPI", ProviderSteps::PlaceSearch, e.to_string())
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(self.map_api_error(status, text, ProviderSteps::PlaceSearch));
        }

        resp.json::<LiteApiGetPlacesResponse>().await.map_err(|e| {
            ProviderError::parse_error("LiteAPI", ProviderSteps::PlaceSearch, e.to_string())
        })
    }

    pub async fn get_place(
        &self,
        req: &LiteApiGetPlaceRequest,
    ) -> Result<LiteApiGetPlaceResponse, ProviderError> {
        let path = format!("/data/places/{}", req.place_id);
        self.get(&path, ProviderSteps::PlaceDetails).await
    }
}
