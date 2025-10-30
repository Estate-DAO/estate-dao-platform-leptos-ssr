use serde::{Deserialize, Serialize};

// <!-- LiteAPI Book Room Request/Response Types -->
// Based on LiteAPI /book endpoint specification

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiBookRequest {
    pub holder: LiteApiBookHolder,
    pub metadata: Option<LiteApiBookMetadata>,
    #[serde(rename = "guestPayment")]
    pub guest_payment: Option<LiteApiGuestPayment>,
    pub payment: LiteApiPayment,
    #[serde(rename = "prebookId")]
    pub prebook_id: String,
    pub guests: Vec<LiteApiBookGuest>,
    #[serde(rename = "clientReference", skip_serializing_if = "Option::is_none")]
    pub client_reference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiBookHolder {
    #[serde(rename = "firstName")]
    pub first_name: String,
    #[serde(rename = "lastName")]
    pub last_name: String,
    pub email: String,
    pub phone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiBookMetadata {
    pub ip: String,
    pub country: String,
    pub language: String,
    pub platform: String,
    pub device_id: String,
    pub user_agent: String,
    pub utm_medium: Option<String>,
    pub utm_source: Option<String>,
    pub utm_campaign: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiGuestPayment {
    pub address: LiteApiPaymentAddress,
    pub method: String,
    pub phone: String,
    pub payee_last_name: String,
    pub payee_first_name: String,
    pub last_4_digits: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiPaymentAddress {
    pub city: String,
    pub address: String,
    pub country: String,
    pub postal_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiPayment {
    pub method: String, // ACC_CREDIT_CARD or WALLET
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiBookGuest {
    #[serde(rename = "occupancyNumber")]
    pub occupancy_number: u32,
    pub remarks: Option<String>,
    #[serde(rename = "firstName")]
    pub first_name: String,
    #[serde(rename = "lastName")]
    pub last_name: String,
    pub email: String,
    pub phone: String,
}

// <!-- LiteAPI Book Response Types -->

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiBookResponse {
    pub data: LiteApiBookData,
    #[serde(rename = "guestLevel")]
    pub guest_level: u32,
    pub sandbox: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiBookData {
    #[serde(rename = "bookingId")]
    pub booking_id: String,
    #[serde(rename = "clientReference")]
    pub client_reference: String,
    #[serde(rename = "supplierBookingId")]
    pub supplier_booking_id: String,
    #[serde(rename = "supplierBookingName")]
    pub supplier_booking_name: String,
    pub supplier: String,
    #[serde(rename = "supplierId")]
    pub supplier_id: u32,
    pub status: String, // CONFIRMED, PENDING, etc.
    #[serde(rename = "hotelConfirmationCode")]
    pub hotel_confirmation_code: String,
    pub checkin: String,
    pub checkout: String,
    pub hotel: LiteApiBookedHotel,
    #[serde(rename = "bookedRooms")]
    pub booked_rooms: Vec<LiteApiBookedRoom>,
    pub holder: LiteApiBookHolder,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "cancellationPolicies")]
    pub cancellation_policies: LiteApiCancellationPolicies,
    #[serde(rename = "specialRemarks")]
    pub special_remarks: Option<String>,
    #[serde(rename = "optionalFees")]
    pub optional_fees: Option<String>,
    #[serde(rename = "mandatoryFees")]
    pub mandatory_fees: Option<String>,
    #[serde(rename = "knowBeforeYouGo")]
    pub know_before_you_go: Option<String>,
    pub price: f64,
    pub commission: f64,
    pub currency: String,
    pub remarks: Option<String>,
    #[serde(rename = "guestId")]
    pub guest_id: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiBookedHotel {
    #[serde(rename = "hotelId")]
    pub hotel_id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiBookedRoom {
    #[serde(rename = "roomType")]
    pub room_type: LiteApiBookedRoomType,
    #[serde(rename = "boardType")]
    pub board_type: String,
    #[serde(rename = "boardName")]
    pub board_name: String,
    pub adults: u32,
    pub children: u32,
    #[serde(flatten)]
    pub flattened_rate: LiteApiBookedPrice,
    // pub rate: LiteApiBookedRoomRate, --- Maybe Rate key NA ---
    #[serde(rename = "firstName")]
    pub first_name: String,
    #[serde(rename = "lastName")]
    pub last_name: String,
    #[serde(rename = "mappedRoomId")]
    pub mapped_room_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    #[test]
    fn test_serialize_booked_room() {
        let room = LiteApiBookedRoom {
            room_type: LiteApiBookedRoomType {
                name: "Deluxe Room".to_string(),
            },
            board_type: "HB".to_string(),
            board_name: "Half Board".to_string(),
            adults: 2,
            children: 1,
            flattened_rate: LiteApiBookedPrice {
                amount: 200.50,
                currency: "USD".to_string(),
            },
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            mapped_room_id: Some("R123".to_string()),
        };

        let json_str = serde_json::to_string_pretty(&room).unwrap();
        let v: Value = serde_json::from_str(&json_str).unwrap();

        assert_eq!(v["roomType"]["name"], "Deluxe Room");
        assert_eq!(v["boardType"], "HB");
        assert_eq!(v["boardName"], "Half Board");
        assert_eq!(v["adults"], 2);
        assert_eq!(v["children"], 1);
        assert_eq!(v["amount"], 200.50);
        assert_eq!(v["currency"], "USD");
        assert_eq!(v["firstName"], "John");
        assert_eq!(v["lastName"], "Doe");
        assert_eq!(v["mappedRoomId"], "R123");
    }

    #[test]
    fn test_deserialize_booked_room() {
        let data = json!({
            "roomType": { "name": "Deluxe Room" },
            "boardType": "HB",
            "boardName": "Half Board",
            "adults": 2,
            "children": 1,
            "amount": 200.50,
            "currency": "USD",
            "firstName": "John",
            "lastName": "Doe",
            "mappedRoomId": "R123"
        });

        let room: LiteApiBookedRoom = serde_json::from_value(data).unwrap();

        assert_eq!(room.room_type.name, "Deluxe Room");
        assert_eq!(room.board_type, "HB");
        assert_eq!(room.board_name, "Half Board");
        assert_eq!(room.adults, 2);
        assert_eq!(room.children, 1);
        assert_eq!(room.flattened_rate.amount, 200.50);
        assert_eq!(room.flattened_rate.currency, "USD");
        assert_eq!(room.first_name, "John");
        assert_eq!(room.last_name, "Doe");
        assert_eq!(room.mapped_room_id, Some("R123".to_string()));
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiBookedRoomType {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiBookedRoomRate {
    #[serde(rename = "retailRate")]
    pub retail_rate: LiteApiBookedRetailRate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiBookedRetailRate {
    pub total: LiteApiBookedPrice,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiBookedPrice {
    pub amount: f64,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiCancellationPolicies {
    #[serde(rename = "cancelPolicyInfos")]
    pub cancel_policy_infos: Option<Vec<LiteApiCancelPolicyInfo>>,
    #[serde(rename = "hotelRemarks")]
    pub hotel_remarks: Option<Vec<String>>,
    #[serde(rename = "refundableTag")]
    pub refundable_tag: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiCancelPolicyInfo {
    #[serde(rename = "cancelTime")]
    pub cancel_time: String,
    pub amount: f64,
    #[serde(rename = "type")]
    pub policy_type: String,
    pub timezone: String,
    pub currency: String,
}

// <!-- API Implementation following LiteAPI pattern -->

use leptos::prelude::*;
use reqwest::header::HeaderMap;
use reqwest::Method;

use crate::api::api_client::{ApiClient, ApiRequest, ApiRequestMeta};
use crate::api::consts::EnvVarConfig;
use crate::api::liteapi::client::LiteApiHTTPClient;
use crate::api::liteapi::traits::LiteApiReq;

cfg_if::cfg_if! {
    if #[cfg(feature = "mock-provab")] {
        use fake::{Dummy, Fake, Faker};
        use rand::rngs::StdRng;
        use rand::SeedableRng;
    }
}

// Custom trait implementation for booking - uses book base URL
impl LiteApiReq for LiteApiBookRequest {
    fn path_suffix() -> &'static str {
        "rates/book"
    }

    // Override the base path to use book base URL (same as prebook for now)
    fn base_path() -> String {
        let env_var_config = EnvVarConfig::expect_context_or_try_from_env();
        env_var_config.liteapi_prebook_base_url
    }
}

impl ApiRequestMeta for LiteApiBookRequest {
    const METHOD: Method = Method::POST;
    const GZIP: bool = false;
    type Response = LiteApiBookResponse;
}

impl ApiRequest for LiteApiBookRequest {
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
pub async fn liteapi_book_room(
    request: LiteApiBookRequest,
) -> Result<LiteApiBookResponse, crate::api::ApiError> {
    // Validate request
    if request.prebook_id.is_empty() {
        return Err(crate::api::ApiError::Other(
            "Prebook ID cannot be empty".to_string(),
        ));
    }

    if request.holder.first_name.is_empty() || request.holder.last_name.is_empty() {
        return Err(crate::api::ApiError::Other(
            "Holder first name and last name are required".to_string(),
        ));
    }

    if request.guests.is_empty() {
        return Err(crate::api::ApiError::Other(
            "At least one guest is required".to_string(),
        ));
    }

    let client = LiteApiHTTPClient::default();
    client.send(request).await
}
