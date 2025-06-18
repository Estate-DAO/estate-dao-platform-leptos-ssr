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

// Request structure
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiPrebookRequest {
    #[serde(rename = "offerId")]
    pub offer_id: String,
    #[serde(rename = "usePaymentSdk")]
    pub use_payment_sdk: bool,
}

// Response structures based on actual API response
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiPrebookAmount {
    pub amount: f64,
    pub currency: String,
    #[serde(default)]
    pub source: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiPrebookTaxAndFee {
    pub included: bool,
    pub description: String,
    pub amount: f64,
    pub currency: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiPrebookRetailRate {
    pub total: Vec<LiteApiPrebookAmount>,
    #[serde(rename = "suggestedSellingPrice")]
    pub suggested_selling_price: Vec<LiteApiPrebookAmount>,
    #[serde(rename = "initialPrice", skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "mock-provab", dummy(default))]
    pub initial_price: Option<String>, // Can be null, use String for mock compatibility
    #[serde(rename = "taxesAndFees")]
    pub taxes_and_fees: Option<Vec<LiteApiPrebookTaxAndFee>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiPrebookCancelPolicyInfo {
    #[serde(rename = "cancelTime")]
    pub cancel_time: String,
    pub amount: f64,
    #[serde(default = "default_currency")]
    pub currency: String,
    #[serde(rename = "type")]
    pub policy_type: String,
    pub timezone: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiPrebookCancellationPolicies {
    #[serde(rename = "cancelPolicyInfos")]
    pub cancel_policy_infos: Vec<LiteApiPrebookCancelPolicyInfo>,
    #[serde(rename = "hotelRemarks")]
    pub hotel_remarks: Vec<String>,
    #[serde(rename = "refundableTag")]
    pub refundable_tag: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiPrebookRate {
    #[serde(rename = "rateId")]
    pub rate_id: String,
    #[serde(rename = "occupancyNumber")]
    pub occupancy_number: i32,
    pub name: String,
    #[serde(rename = "maxOccupancy")]
    pub max_occupancy: i32,
    #[serde(rename = "adultCount")]
    pub adult_count: i32,
    #[serde(rename = "childCount")]
    pub child_count: i32,
    #[serde(rename = "boardType")]
    pub board_type: String,
    #[serde(rename = "boardName")]
    pub board_name: String,
    pub remarks: String,
    #[serde(rename = "priceType")]
    pub price_type: String,
    pub commission: Vec<LiteApiPrebookAmount>,
    #[serde(rename = "retailRate", skip_serializing_if = "Option::is_none")]
    pub retail_rate: Option<LiteApiPrebookRetailRate>,
    #[serde(rename = "cancellationPolicies")]
    pub cancellation_policies: LiteApiPrebookCancellationPolicies,
    #[serde(rename = "paymentTypes")]
    pub payment_types: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiPrebookRoomType {
    pub rates: Vec<LiteApiPrebookRate>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiPrebookData {
    #[serde(rename = "prebookId")]
    pub prebook_id: String,
    #[serde(rename = "offerId")]
    pub offer_id: String,
    #[serde(rename = "hotelId")]
    pub hotel_id: String,
    pub currency: String,
    #[serde(rename = "termsAndConditions")]
    pub terms_and_conditions: String,
    #[serde(rename = "roomTypes")]
    pub room_types: Vec<LiteApiPrebookRoomType>,
    #[serde(rename = "suggestedSellingPrice")]
    pub suggested_selling_price: f64,
    #[serde(rename = "isPackageRate")]
    pub is_package_rate: bool,
    pub commission: f64,
    pub price: f64,
    #[serde(rename = "priceType")]
    pub price_type: String,
    #[serde(rename = "priceDifferencePercent")]
    pub price_difference_percent: f64,
    #[serde(rename = "cancellationChanged")]
    pub cancellation_changed: bool,
    #[serde(rename = "boardChanged")]
    pub board_changed: bool,
    pub supplier: String,
    #[serde(rename = "supplierId")]
    pub supplier_id: i32,
    #[serde(rename = "paymentTypes")]
    pub payment_types: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiPrebookResponse {
    pub data: LiteApiPrebookData,
    #[serde(rename = "guestLevel")]
    pub guest_level: i32,
    pub sandbox: bool,
}

impl LiteApiPrebookResponse {
    pub fn get_prebook_id(&self) -> &str {
        &self.data.prebook_id
    }

    pub fn get_hotel_id(&self) -> &str {
        &self.data.hotel_id
    }
}

// Custom trait implementation for prebook - uses different base URL
impl LiteApiReq for LiteApiPrebookRequest {
    fn path_suffix() -> &'static str {
        "rates/prebook"
    }

    // Override the base path to use prebook base URL
    fn base_path() -> String {
        let env_var_config = EnvVarConfig::expect_context_or_try_from_env();
        env_var_config.liteapi_prebook_base_url
    }
}

impl ApiRequestMeta for LiteApiPrebookRequest {
    const METHOD: Method = Method::POST;
    const GZIP: bool = false;
    type Response = LiteApiPrebookResponse;
}

impl ApiRequest for LiteApiPrebookRequest {
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

fn default_currency() -> String {
    "USD".to_string()
}

#[cfg(feature = "ssr")]
pub async fn liteapi_prebook(
    request: LiteApiPrebookRequest,
) -> Result<LiteApiPrebookResponse, crate::api::ApiError> {
    // Validate request
    if request.offer_id.is_empty() {
        return Err(crate::api::ApiError::Other(
            "Offer ID cannot be empty".to_string(),
        ));
    }

    let client = LiteApiHTTPClient::default();
    client.send(request).await
}
