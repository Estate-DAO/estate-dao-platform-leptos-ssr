use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::api::api_client::{ApiRequest, ApiRequestMeta};
use crate::api::liteapi::client::LiteApiHTTPClient;
use crate::api::liteapi::l01_get_hotel_info_rates::LiteApiOccupancy;
use crate::api::liteapi::traits::LiteApiReq;
use reqwest::header::HeaderMap;

cfg_if::cfg_if! {
    if #[cfg(feature = "mock-provab")] {
        use fake::{Dummy, Fake, Faker};
        use rand::rngs::StdRng;
        use rand::SeedableRng;
    }
}

// Request structure - similar to LiteApiHotelRatesRequest but with timeout
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiMinRatesRequest {
    #[serde(rename = "hotelIds")]
    pub hotel_ids: Vec<String>,
    pub occupancies: Vec<LiteApiOccupancy>,
    pub currency: String,
    #[serde(rename = "guestNationality")]
    pub guest_nationality: String,
    pub checkin: String,  // Format: "YYYY-MM-DD"
    pub checkout: String, // Format: "YYYY-MM-DD"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u32>, // Optional timeout in seconds
}

// Response structures - lightweight min-rates response
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiMinRateHotel {
    #[serde(rename = "hotelId")]
    pub hotel_id: String,
    pub price: f64,
    #[serde(rename = "suggestedSellingPrice")]
    pub suggested_selling_price: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiMinRatesError {
    pub code: i32,
    pub message: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiMinRatesResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Vec<LiteApiMinRateHotel>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<LiteApiMinRatesError>,
    #[serde(default)]
    pub sandbox: bool,
}

impl LiteApiMinRatesResponse {
    pub fn is_error_response(&self) -> bool {
        self.error.is_some()
    }

    pub fn is_no_availability(&self) -> bool {
        if let Some(error) = &self.error {
            error.code == 2001
        } else {
            false
        }
    }

    /// Get a map of hotel_id -> (price, suggested_selling_price)
    pub fn to_price_map(&self) -> std::collections::HashMap<String, (f64, f64)> {
        let mut map = std::collections::HashMap::new();
        if let Some(data) = &self.data {
            for hotel in data {
                map.insert(
                    hotel.hotel_id.clone(),
                    (hotel.price, hotel.suggested_selling_price),
                );
            }
        }
        map
    }
}

// Implement traits for API integration

impl LiteApiReq for LiteApiMinRatesRequest {
    fn path_suffix() -> &'static str {
        "hotels/min-rates"
    }
}

impl ApiRequestMeta for LiteApiMinRatesRequest {
    const METHOD: Method = Method::POST;
    const GZIP: bool = false;
    type Response = LiteApiMinRatesResponse;
}

impl ApiRequest for LiteApiMinRatesRequest {
    fn base_path() -> String {
        <Self as LiteApiReq>::base_path()
    }

    fn path_suffix() -> &'static str {
        <Self as LiteApiReq>::path_suffix()
    }

    fn custom_headers() -> HeaderMap {
        <Self as LiteApiReq>::custom_headers()
    }
}

#[cfg(feature = "ssr")]
pub async fn liteapi_hotel_min_rates(
    request: LiteApiMinRatesRequest,
) -> Result<LiteApiMinRatesResponse, crate::api::ApiError> {
    // Validate request

    use crate::api::api_client::ApiClient;
    if request.hotel_ids.is_empty() {
        return Err(crate::api::ApiError::Other(
            "Hotel IDs cannot be empty".to_string(),
        ));
    }

    let client = LiteApiHTTPClient::default();
    client.send(request).await
}
