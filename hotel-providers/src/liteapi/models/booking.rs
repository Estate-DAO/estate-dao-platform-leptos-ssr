use super::search::{LiteApiErrorDetail, LiteApiRate};
use serde::{Deserialize, Serialize};

// Re-using LiteApiOccupancy and others from search mod if they match structure,
// otherwise define new ones. LiteApiOccupancy matches standard {adults, children[]}.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteApiPrebookRequest {
    #[serde(rename = "offerId")]
    pub offer_id: String,
    #[serde(rename = "usePaymentSdk")]
    pub use_payment_sdk: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub addons: Option<Vec<LiteApiAddonRequest>>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "includeCreditBalance"
    )]
    pub include_credit_balance: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteApiPrebookResponse {
    pub data: Option<LiteApiPrebookData>,
    pub error: Option<LiteApiErrorDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteApiPrebookData {
    #[serde(rename = "prebookId")]
    pub prebook_id: String,
    #[serde(rename = "offerId")]
    pub offer_id: String,
    pub price: Option<f64>,
    pub currency: Option<String>,
    #[serde(rename = "roomTypes")]
    // Note: Structure here differs slightly from search result structure.
    // Booking API returns `roomTypes` array BUT items don't have `roomTypeId`.
    // Instead they contain `rates` array directly?
    // Wait, viewing `booking_openapi.json`:
    // "roomTypes": { "items": { "properties": { "rates": { "type": "array", ... } } } }
    pub room_types: Vec<LiteApiPrebookRoomType>,

    // other fields like hotelId, checkin, checkout
    #[serde(rename = "hotelId")]
    pub hotel_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteApiPrebookRoomType {
    pub rates: Vec<LiteApiRate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteApiBookRequest {
    #[serde(rename = "prebookId")]
    pub prebook_id: String,
    #[serde(rename = "clientReference", skip_serializing_if = "Option::is_none")]
    pub client_reference: Option<String>,
    pub holder: LiteApiHolder,
    pub guests: Vec<LiteApiGuest>,
    pub payment: LiteApiPayment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteApiBookResponse {
    pub data: Option<LiteApiBookingData>,
    pub error: Option<LiteApiErrorDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteApiBookingData {
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
    pub hotel: LiteApiBookedHotel,
    #[serde(rename = "bookedRooms")]
    pub booked_rooms: Vec<LiteApiBookedRoom>,
    pub holder: LiteApiHolder,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "cancellationPolicies")]
    pub cancellation_policies: LiteApiBookingCancellationPolicies,
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
pub struct LiteApiBookedHotel {
    #[serde(rename = "hotelId")]
    pub hotel_id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    #[serde(rename = "firstName")]
    pub first_name: String,
    #[serde(rename = "lastName")]
    pub last_name: String,
    #[serde(rename = "mappedRoomId")]
    pub mapped_room_id: Option<String>, // Keep as String to handle potentially number or string from json
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteApiBookedRoomType {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteApiBookedPrice {
    pub amount: f64,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteApiBookingCancellationPolicies {
    #[serde(rename = "cancelPolicyInfos")]
    pub cancel_policy_infos: Option<Vec<LiteApiBookingCancelPolicyInfo>>,
    #[serde(rename = "hotelRemarks")]
    pub hotel_remarks: Option<Vec<String>>,
    #[serde(rename = "refundableTag")]
    pub refundable_tag: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
pub struct LiteApiGetBookingResponse {
    pub data: Vec<LiteApiBookingDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteApiHolder {
    #[serde(rename = "firstName")]
    pub first_name: String,
    #[serde(rename = "lastName")]
    pub last_name: String,
    pub email: String,
    pub phone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteApiGuest {
    #[serde(rename = "occupancyNumber")]
    pub occupancy_number: i32,
    #[serde(rename = "firstName")]
    pub first_name: String,
    #[serde(rename = "lastName")]
    pub last_name: String,
    pub email: String,
    pub phone: String,
    pub remarks: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteApiPayment {
    pub method: String,
    #[serde(rename = "transactionId", skip_serializing_if = "Option::is_none")]
    pub transaction_id: Option<String>,
    // For credit card payment, would need card details struct
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteApiAddonRequest {
    pub addon: String,
    pub value: f64,
    pub currency: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteApiGetBookingRequest {
    #[serde(rename = "clientReference", skip_serializing_if = "Option::is_none")]
    pub client_reference: Option<String>,
    #[serde(rename = "guestId", skip_serializing_if = "Option::is_none")]
    pub guest_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteApiBookingHotel {
    #[serde(rename = "hotelId")]
    pub hotel_id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteApiBookingRoom {
    pub adults: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<String>, // Legacy model had String for children? Or Vec? Legacy definition says String.
    #[serde(rename = "firstName")]
    pub first_name: String,
    #[serde(rename = "lastName")]
    pub last_name: String,
    // other fields omitted for brevity if not critical, but legacy has them.
    #[serde(rename = "room_id")]
    pub room_id: Option<String>, // Legacy had room_id but sometimes it's missing or named differently?
    pub amount: f64,
    pub currency: String,
    #[serde(rename = "children_count")]
    pub children_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteApiBookingDetails {
    #[serde(rename = "bookingId")]
    pub booking_id: String,
    #[serde(rename = "clientReference")]
    pub client_reference: String,
    pub status: String,
    pub hotel: LiteApiBookingHotel,
    pub rooms: Vec<LiteApiBookingRoom>,
    pub holder: Option<LiteApiHolder>, // Legacy had separate Holder struct
    pub price: f64,
    pub currency: String,
    #[serde(rename = "guestId")]
    pub guest_id: Option<String>,
    // Add other fields if necessary
}
