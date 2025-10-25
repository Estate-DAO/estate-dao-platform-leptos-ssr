use leptos::prelude::*;
use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::api::api_client::{ApiClient, ApiRequest, ApiRequestMeta};
use crate::api::consts::EnvVarConfig;
use crate::api::liteapi::client::LiteApiHTTPClient;
use crate::api::liteapi::traits::LiteApiReq;
use reqwest::header::HeaderMap;

cfg_if::cfg_if! {
    if #[cfg(feature = "mock-provab")] {
        use fake::{Dummy, Fake, Faker};
        use rand::rngs::StdRng;
        use rand::SeedableRng;
    }
}

// <!-- Request structure for getting booking details -->
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiGetBookingRequest {
    #[serde(rename = "clientReference", skip_serializing_if = "Option::is_none")]
    pub client_reference: Option<String>,
    #[serde(rename = "guestId", skip_serializing_if = "Option::is_none")]
    pub guest_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<f64>,
}

// <!-- Response structures based on API documentation -->
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiBookingHotel {
    #[serde(rename = "hotelId")]
    pub hotel_id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiBookingGuest {
    #[serde(rename = "firstName")]
    pub first_name: String,
    #[serde(rename = "lastName")]
    pub last_name: String,
    pub email: String,
    pub phone: String,
    pub remarks: Option<String>,
    #[serde(rename = "occupancyNumber")]
    pub occupancy_number: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiBookingRoom {
    pub adults: u32,
    pub children: String,
    #[serde(rename = "firstName")]
    pub first_name: String,
    #[serde(rename = "lastName")]
    pub last_name: String,
    #[serde(rename = "childrenAges")]
    pub children_ages: Option<String>,
    #[serde(rename = "room_id")]
    pub room_id: String,
    #[serde(rename = "occupancy_number")]
    pub occupancy_number: u32,
    pub amount: f64,
    pub currency: String,
    #[serde(rename = "children_count")]
    pub children_count: u32,
    pub remarks: Option<String>,
    pub guests: Vec<LiteApiBookingGuest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiBookingHolder {
    #[serde(rename = "firstName")]
    pub first_name: String,
    #[serde(rename = "lastName")]
    pub last_name: String,
    pub email: String,
    pub phone: String,
    #[serde(rename = "holderTitle", skip_serializing_if = "Option::is_none")]
    pub holder_title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiBookingCancelPolicyInfo {
    #[serde(rename = "cancelTime")]
    pub cancel_time: String,
    pub amount: f64,
    #[serde(rename = "type")]
    pub policy_type: String,
    pub timezone: String,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiBookingCancellationPolicies {
    #[serde(rename = "cancelPolicyInfos")]
    pub cancel_policy_infos: Option<Vec<LiteApiBookingCancelPolicyInfo>>,
    #[serde(rename = "hotelRemarks")]
    pub hotel_remarks: Option<String>,
    #[serde(rename = "refundableTag")]
    pub refundable_tag: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiBookingDetails {
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
    pub status: String,
    #[serde(rename = "hotelConfirmationCode")]
    pub hotel_confirmation_code: String,
    pub checkin: String,
    pub checkout: String,
    pub hotel: LiteApiBookingHotel,
    pub rooms: Vec<LiteApiBookingRoom>,
    pub holder: LiteApiBookingHolder,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    #[serde(rename = "cancellationPolicies")]
    pub cancellation_policies: LiteApiBookingCancellationPolicies,
    #[serde(rename = "specialRemarks")]
    pub special_remarks: Option<String>,
    pub price: f64,
    pub commission: f64,
    #[serde(rename = "clientCommission")]
    pub client_commission: f64,
    pub currency: String,
    #[serde(rename = "guestId")]
    pub guest_id: Option<String>,
    #[serde(rename = "trackingId")]
    pub tracking_id: String,
    #[serde(rename = "prebookId")]
    pub prebook_id: String,
    #[serde(rename = "paymentStatus")]
    pub payment_status: String,
    #[serde(rename = "paymentTransactionId")]
    pub payment_transaction_id: String,
    #[serde(rename = "sellingPrice")]
    pub selling_price: String,
    #[serde(rename = "exchangeRate")]
    pub exchange_rate: f64,
    #[serde(rename = "exchangeRateUsd")]
    pub exchange_rate_usd: f64,
    pub email: String,
    pub tag: String,
    #[serde(rename = "lastFreeCancellationDate")]
    pub last_free_cancellation_date: String,
    #[serde(rename = "apiCommission")]
    pub api_commission: f64,
    #[serde(rename = "userId")]
    pub user_id: u32,
    pub nationality: String,
    #[serde(rename = "loyaltyGuestId")]
    pub loyalty_guest_id: u32,
    #[serde(rename = "cancelledAt")]
    pub cancelled_at: Option<String>,
    #[serde(rename = "refundedAt")]
    pub refunded_at: Option<String>,
    #[serde(rename = "cancelledBy")]
    pub cancelled_by: Option<u32>,
    pub sandbox: u32,
    #[serde(rename = "voucherId")]
    pub voucher_id: Option<String>,
    #[serde(rename = "voucherTotalAmount")]
    pub voucher_total_amount: f64,
    #[serde(rename = "voucherTransationId")]
    pub voucher_transaction_id: Option<String>,
    #[serde(rename = "processingFee")]
    pub processing_fee: f64,
    #[serde(rename = "amountRefunded")]
    pub amount_refunded: f64,
    #[serde(rename = "refundType")]
    pub refund_type: String,
    #[serde(rename = "paymentScheduledAt")]
    pub payment_scheduled_at: Option<String>,
    #[serde(rename = "addonsTotalAmount")]
    pub addons_total_amount: Option<f64>,
    #[serde(rename = "addonsRedemptions")]
    pub addons_redemptions: Option<String>,
    #[serde(rename = "rebookFrom")]
    pub rebook_from: String,
    #[serde(rename = "agentId")]
    pub agent_id: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiGetBookingResponse {
    pub data: Vec<LiteApiBookingDetails>,
}

impl LiteApiGetBookingResponse {
    pub fn get_bookings(&self) -> &Vec<LiteApiBookingDetails> {
        &self.data
    }

    pub fn find_booking_by_client_reference(
        &self,
        client_ref: &str,
    ) -> Option<&LiteApiBookingDetails> {
        self.data
            .iter()
            .find(|booking| booking.client_reference == client_ref)
    }

    pub fn find_booking_by_booking_id(&self, booking_id: &str) -> Option<&LiteApiBookingDetails> {
        self.data
            .iter()
            .find(|booking| booking.booking_id == booking_id)
    }
}

// Custom trait implementation for booking details - uses book base URL
impl LiteApiReq for LiteApiGetBookingRequest {
    fn path_suffix() -> &'static str {
        "bookings"
    }

    // Override the base path to use book base URL
    fn base_path() -> String {
        let env_var_config = EnvVarConfig::expect_context_or_try_from_env();
        env_var_config.liteapi_prebook_base_url
    }
}

impl ApiRequestMeta for LiteApiGetBookingRequest {
    const METHOD: Method = Method::GET;
    const GZIP: bool = false;
    type Response = LiteApiGetBookingResponse;
}

impl ApiRequest for LiteApiGetBookingRequest {
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
pub async fn liteapi_get_booking_details(
    request: LiteApiGetBookingRequest,
) -> Result<LiteApiGetBookingResponse, crate::api::ApiError> {
    // Validate that at least one identifier is provided
    if request.client_reference.is_none() && request.guest_id.is_none() {
        return Err(crate::api::ApiError::Other(
            "Either clientReference or guestId must be provided".to_string(),
        ));
    }

    let client = LiteApiHTTPClient::default();
    client.send(request).await
}
