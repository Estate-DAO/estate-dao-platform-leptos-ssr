use leptos::*;
use reqwest::Method;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::api::api_client::{ApiClient, ApiRequest, ApiRequestMeta};
use crate::api::liteapi::client::LiteApiHTTPClient;
use crate::api::liteapi::traits::LiteApiReq;
use crate::{api::consts::EnvVarConfig, log};
use reqwest::header::HeaderMap;

cfg_if::cfg_if! {
    if #[cfg(feature = "mock-provab")] {
        use fake::{Dummy, Fake, Faker};
        use rand::rngs::StdRng;
        use rand::SeedableRng;
    }
}

// Request structures
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiOccupancy {
    pub adults: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<u32>>, // Array of children ages
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiHotelRatesRequest {
    #[serde(rename = "hotelIds")]
    pub hotel_ids: Vec<String>,
    pub occupancies: Vec<LiteApiOccupancy>,
    pub currency: String,
    #[serde(rename = "guestNationality")]
    pub guest_nationality: String,
    pub checkin: String,  // Format: "YYYY-MM-DD"
    pub checkout: String, // Format: "YYYY-MM-DD"
    #[serde(rename = "roomMapping")]
    pub room_mapping: bool,
}

// Response structures - only parsing necessary fields
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiAmount {
    pub amount: f64,
    pub currency: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiTaxAndFee {
    #[serde(default)]
    pub included: bool,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub amount: f64,
    #[serde(default)]
    pub currency: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiRetailRate {
    pub total: Vec<LiteApiAmount>,
    #[serde(rename = "suggestedSellingPrice")]
    pub suggested_selling_price: Vec<LiteApiAmount>,
    #[serde(rename = "taxesAndFees")]
    pub taxes_and_fees: Option<Vec<LiteApiTaxAndFee>>,
    // #[serde(rename = "initialPrice")]
    // pub initial_price: Vec<LiteApiAmount>,
    // #[serde(rename = "taxesAndFees")]
    // pub taxes_and_fees: Vec<LiteApiTaxAndFee>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiRate {
    #[serde(rename = "rateId")]
    pub rate_id: String,
    #[serde(rename = "occupancyNumber")]
    pub occupancy_number: Option<u32>,
    pub name: String,
    #[serde(rename = "maxOccupancy")]
    pub max_occupancy: u32,
    #[serde(rename = "mappedRoomId")]
    pub mapped_room_id: u32,
    #[serde(rename = "adultCount")]
    pub adult_count: u32,
    #[serde(rename = "childCount")]
    pub child_count: u32,
    #[serde(rename = "boardType")]
    pub board_type: String,
    #[serde(rename = "boardName")]
    pub board_name: String,
    #[serde(default)]
    pub remarks: Option<String>,
    #[serde(rename = "retailRate")]
    pub retail_rate: LiteApiRetailRate,
    #[serde(rename = "cancellationPolicies")]
    pub cancellation_policies: super::l03_book::LiteApiCancellationPolicies,
    // #[serde(default)]
    // pub perks: Vec<String>,
    #[serde(default)]
    pub promotions: Option<String>,
    // #[serde(rename = "priceType")]
    // pub price_type: String,
    // pub commission: Vec<LiteApiAmount>,
    // #[serde(rename = "paymentTypes")]
    // pub payment_types: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiRoomType {
    #[serde(rename = "roomTypeId")]
    pub room_type_id: String,
    #[serde(rename = "offerId")]
    pub offer_id: String,
    // pub supplier: String,
    // #[serde(rename = "supplierId")]
    // pub supplier_id: i32,
    pub rates: Vec<LiteApiRate>,
    // #[serde(rename = "offerRetailRate")]
    // pub offer_retail_rate: LiteApiAmount,
    #[serde(rename = "suggestedSellingPrice")]
    pub suggested_selling_price: LiteApiAmount,

    #[serde(rename = "offerRetailRate")]
    pub offer_retail_rate: LiteApiAmount,
    // #[serde(rename = "offerInitialPrice")]
    // pub offer_initial_price: LiteApiAmount,
    // #[serde(rename = "priceType")]
    // pub price_type: String,
    // #[serde(rename = "rateType")]
    // pub rate_type: String,
    // #[serde(rename = "paymentTypes")]
    // pub payment_types: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiHotelData {
    #[serde(rename = "hotelId")]
    pub hotel_id: String,
    #[serde(rename = "roomTypes")]
    pub room_types: Vec<LiteApiRoomType>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiError {
    pub code: i32,
    pub message: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiHotelRatesResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Vec<LiteApiHotelData>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<LiteApiError>,
}

impl LiteApiHotelRatesResponse {
    // if we only have request with one hotelId, we will get only that hotelId in response.
    // hence we can safely assume .first() to be that hotel - since this will be used on hotel_details page
    pub fn get_first_hotel_data(&self) -> Option<&LiteApiHotelData> {
        self.data.as_ref()?.first()
    }

    pub fn get_first_room_type(&self) -> Option<&LiteApiRoomType> {
        self.data.as_ref()?.first()?.room_types.first()
    }

    pub fn get_first_rate(&self) -> Option<&LiteApiRate> {
        self.get_first_room_type()?.rates.first()
    }

    pub fn is_error_response(&self) -> bool {
        self.error.is_some()
    }

    pub fn is_no_availability(&self) -> bool {
        if let Some(error) = &self.error {
            error.code == 2001
            // && error.message.contains("no availability found")
        } else {
            false
        }
    }
}

// Implement traits for API integration

impl LiteApiReq for LiteApiHotelRatesRequest {
    fn path_suffix() -> &'static str {
        "hotels/rates"
    }
}

impl ApiRequestMeta for LiteApiHotelRatesRequest {
    const METHOD: Method = Method::POST;
    const GZIP: bool = false;
    type Response = LiteApiHotelRatesResponse;
}

impl ApiRequest for LiteApiHotelRatesRequest {
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
pub async fn liteapi_hotel_rates(
    request: LiteApiHotelRatesRequest,
) -> Result<LiteApiHotelRatesResponse, crate::api::ApiError> {
    // Validate request
    if request.hotel_ids.is_empty() {
        return Err(crate::api::ApiError::Other(
            "Hotel IDs cannot be empty".to_string(),
        ));
    }

    let client = LiteApiHTTPClient::default();
    client.send(request).await
}
