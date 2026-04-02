use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type TranslatedString = HashMap<String, Option<String>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookerInput {
    pub country: String,
    pub platform: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub travel_purpose: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_groups: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuestAllocation {
    pub number_of_adults: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuestsInput {
    pub number_of_adults: u32,
    pub number_of_rooms: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<u8>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allocation: Option<Vec<GuestAllocation>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinatesInput {
    pub latitude: f64,
    pub longitude: f64,
    pub radius: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccommodationsSearchInput {
    pub booker: BookerInput,
    pub checkin: String,
    pub checkout: String,
    pub guests: GuestsInput,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coordinates: Option<CoordinatesInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extras: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtraChargesSimple {
    pub excluded: Option<f64>,
    pub included: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchPrice {
    pub base: Option<f64>,
    pub book: Option<f64>,
    pub total: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_charges: Option<ExtraChargesSimple>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchProduct {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccommodationsSearchData {
    pub id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<SearchPrice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub products: Option<Vec<SearchProduct>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deep_link_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccommodationsSearchOutput {
    pub request_id: Option<String>,
    pub data: Vec<AccommodationsSearchData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_page: Option<String>,
}

// ---- Details ----

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccommodationsDetailsInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accommodations: Option<Vec<i64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extras: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub languages: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationCoordinates {
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<TranslatedString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coordinates: Option<LocationCoordinates>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_score: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_of_reviews: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stars: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhotoUrls {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub large: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub standard: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhotoInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub main_photo: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<PhotoUrls>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacilityInfo {
    pub id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomInfo {
    pub id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<TranslatedString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub photos: Option<Vec<PhotoInfo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub facilities: Option<Vec<FacilityInfo>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccommodationDetails {
    pub id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<LocationInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review: Option<ReviewInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub photos: Option<Vec<PhotoInfo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub facilities: Option<Vec<FacilityInfo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<TranslatedString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rooms: Option<Vec<RoomInfo>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccommodationsDetailsOutput {
    pub request_id: Option<String>,
    pub data: Vec<AccommodationDetails>,
}

// ---- Availability ----

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccommodationsAvailabilityInput {
    pub accommodation: i64,
    pub booker: BookerInput,
    pub checkin: String,
    pub checkout: String,
    pub guests: GuestsInput,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extras: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub products: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancellationSchedule {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    pub price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancellationPolicy {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule: Option<Vec<CancellationSchedule>>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MealPlanPolicy {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meals: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentPolicy {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timings: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prepayment_required: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductPolicies {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancellation: Option<CancellationPolicy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meal_plan: Option<MealPlanPolicy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment: Option<PaymentPolicy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaximumOccupancy {
    pub adults: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<ChildrenOccupancy>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildrenOccupancy {
    pub total: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_age: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_age: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailabilityProduct {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_available_at_this_price: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum_occupancy: Option<MaximumOccupancy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<SearchPrice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policies: Option<ProductPolicies>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailabilityData {
    pub id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub products: Option<Vec<AvailabilityProduct>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccommodationsAvailabilityOutput {
    pub request_id: Option<String>,
    pub data: AvailabilityData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccommodationsBulkAvailabilityInput {
    pub accommodations: Vec<i64>,
    pub booker: BookerInput,
    pub checkin: String,
    pub checkout: String,
    pub guests: GuestsInput,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extras: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccommodationsBulkAvailabilityOutput {
    pub request_id: Option<String>,
    pub data: Vec<AvailabilityData>,
}

// ---- Orders preview/create/details ----

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrdersPreviewProductInput {
    pub id: String,
    pub allocation: GuestAllocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrdersPreviewAccommodationInput {
    pub id: i64,
    pub checkin: String,
    pub checkout: String,
    pub products: Vec<OrdersPreviewProductInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrdersPreviewInput {
    pub booker: BookerInput,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    pub accommodation: OrdersPreviewAccommodationInput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceCurrency {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accommodation_currency: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub booker_currency: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewProductPrice {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base: Option<PriceCurrency>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<PriceCurrency>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrdersPreviewProductOutput {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<PreviewProductPrice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policies: Option<ProductPolicies>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrdersPreviewAccommodationOutput {
    pub id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub products: Option<Vec<OrdersPreviewProductOutput>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrdersPreviewData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accommodation: Option<OrdersPreviewAccommodationOutput>,
    pub order_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrdersPreviewOutput {
    pub request_id: Option<String>,
    pub data: OrdersPreviewData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderCreateGuestInput {
    pub email: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderCreateProductInput {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guests: Option<Vec<OrderCreateGuestInput>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderCreateAccommodationInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub products: Vec<OrderCreateProductInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remarks: Option<OrderCreateRemarksInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderCreateRemarksInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub special_requests: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderCreateAddressInput {
    pub address_line: String,
    pub city: String,
    pub country: String,
    pub post_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderCreateBookerNameInput {
    pub first_name: String,
    pub last_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderCreateBookerInput {
    pub address: OrderCreateAddressInput,
    pub email: String,
    pub name: OrderCreateBookerNameInput,
    pub telephone: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderCreateInput {
    pub order_token: String,
    pub booker: OrderCreateBookerInput,
    pub accommodation: OrderCreateAccommodationInput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderCreateAccommodationOutput {
    pub order: Option<String>,
    pub reservation: Option<i64>,
    pub pincode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub third_party_inventory: Option<ThirdPartyInventoryOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThirdPartyInventoryOutput {
    pub checkin_number: Option<String>,
    pub confirmation_number: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderCreateDataOutput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accommodation: Option<OrderCreateAccommodationOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderCreateOutput {
    pub request_id: Option<String>,
    pub data: Option<OrderCreateDataOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrdersDetailsAccommodationsInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orders: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reservations: Option<Vec<i64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extras: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrdersDetailsAccommodationOutput {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accommodation: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accommodation_details: Option<OrderAccommodationDetailsOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderAccommodationDetailsOutput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrdersDetailsAccommodationsOutput {
    pub request_id: Option<String>,
    pub data: Vec<OrdersDetailsAccommodationOutput>,
}
