use serde::{Deserialize, Serialize};

/// Helper to deserialize a field that can be either a string or an integer into Option<String>
mod string_or_i64 {
    use serde::{self, Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum StringOrInt {
            String(String),
            Int(i64),
        }

        let opt: Option<StringOrInt> = Option::deserialize(deserializer)?;
        Ok(opt.map(|v| match v {
            StringOrInt::String(s) => s,
            StringOrInt::Int(i) => i.to_string(),
        }))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteApiHotelRatesRequest {
    #[serde(rename = "hotelIds")]
    pub hotel_ids: Vec<String>,
    pub occupancies: Vec<LiteApiOccupancy>,
    pub currency: String,
    #[serde(rename = "guestNationality")]
    pub guest_nationality: String,
    pub checkin: String,
    pub checkout: String,
    // Optional fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "maxRatesPerHotel")]
    pub max_rates_per_hotel: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "boardType")]
    pub board_type: Option<String>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "refundableRatesOnly"
    )]
    pub refundable_rates_only: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<Vec<LiteApiSort>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "roomMapping")]
    pub room_mapping: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "includeHotelData")]
    pub include_hotel_data: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteApiOccupancy {
    pub adults: i32,
    pub children: Vec<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteApiSort {
    pub field: String,
    pub direction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteApiHotelRatesResponse {
    pub data: Option<Vec<LiteApiHotelRateData>>,
    pub error: Option<LiteApiErrorDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteApiHotelRateData {
    #[serde(rename = "hotelId")]
    pub hotel_id: String,
    #[serde(rename = "roomTypes")]
    pub room_types: Vec<LiteApiSearchRoomType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteApiSearchRoomType {
    #[serde(rename = "roomTypeId")]
    pub room_type_id: String,
    #[serde(rename = "offerId")]
    pub offer_id: String,
    pub rates: Vec<LiteApiRate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteApiRate {
    #[serde(rename = "rateId")]
    pub rate_id: String,
    #[serde(rename = "occupancyNumber")]
    pub occupancy_number: i32,
    pub name: String,
    #[serde(rename = "maxOccupancy")]
    pub max_occupancy: i32,
    #[serde(rename = "adultCount")]
    pub adult_count: i32,
    #[serde(rename = "childCount", default)]
    pub child_count: i32,
    #[serde(
        rename = "mappedRoomId",
        default,
        deserialize_with = "string_or_i64::deserialize"
    )]
    pub mapped_room_id: Option<String>,
    #[serde(rename = "boardType")]
    pub board_type: Option<String>,
    #[serde(rename = "boardName")]
    pub board_name: Option<String>,
    #[serde(rename = "retailRate")]
    pub retail_rate: LiteApiRetailRate,
    #[serde(rename = "cancellationPolicies")]
    pub cancellation_policies: Option<LiteApiCancellationPolicies>,
    #[serde(rename = "paymentTypes")]
    pub payment_types: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteApiRetailRate {
    pub total: Vec<LiteApiAmount>,
    #[serde(rename = "suggestedSellingPrice")]
    pub suggested_selling_price: Option<Vec<LiteApiAmount>>,
    #[serde(rename = "taxesAndFees")]
    pub taxes_and_fees: Option<Vec<LiteApiTaxFee>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteApiAmount {
    pub amount: f64,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteApiTaxFee {
    pub included: bool,
    pub description: Option<String>,
    pub amount: f64,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteApiCancellationPolicies {
    #[serde(rename = "cancelPolicyInfos")]
    pub cancel_policy_infos: Option<Vec<LiteApiCancelPolicyInfo>>,
    #[serde(rename = "refundableTag")]
    pub refundable_tag: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteApiCancelPolicyInfo {
    #[serde(rename = "cancelTime")]
    pub cancel_time: Option<String>,
    pub amount: f64,
    pub currency: String,
    pub r#type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteApiErrorDetail {
    pub code: i32,
    pub message: String,
    pub description: Option<String>,
}
// Min Rates Models

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LiteApiMinRatesRequest {
    #[serde(rename = "hotelIds")]
    pub hotel_ids: Vec<String>,
    pub occupancies: Vec<LiteApiOccupancy>,
    pub currency: String,
    #[serde(rename = "guestNationality")]
    pub guest_nationality: String,
    pub checkin: String,
    pub checkout: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LiteApiMinRateHotel {
    #[serde(rename = "hotelId")]
    pub hotel_id: String,
    pub price: f64,
    #[serde(rename = "suggestedSellingPrice")]
    pub suggested_selling_price: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LiteApiMinRatesResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Vec<LiteApiMinRateHotel>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<LiteApiErrorDetail>,
}
