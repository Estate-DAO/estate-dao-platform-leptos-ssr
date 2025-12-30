//! Booking domain types - provider-agnostic
//!
//! These types are used for hotel booking operations.

use serde::{Deserialize, Serialize};

use super::DomainCancellationPolicies;

// BOOKING DOMAIN TYPES
// Domain types for hotel booking operations - provider-agnostic

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainBookRoomRequest {
    pub block_id: String,
    pub holder: DomainBookingHolder,
    pub guests: Vec<DomainBookingGuest>,
    pub payment: DomainPaymentInfo,
    pub guest_payment: Option<DomainGuestPaymentInfo>,
    pub special_requests: Option<String>,
    pub booking_context: DomainBookingContext,
    pub client_reference: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainBookingHolder {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainBookingGuest {
    pub occupancy_number: u32,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: String,
    pub remarks: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainPaymentInfo {
    pub method: DomainPaymentMethod,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DomainPlace {
    pub place_id: String,
    pub display_name: String,
    pub formatted_address: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DomainPlacesResponse {
    pub data: Vec<DomainPlace>,
}

#[derive(Clone, Debug, Serialize, Default, Deserialize)]
pub enum DomainPaymentMethod {
    #[default]
    AccCreditCard,
    Wallet,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainGuestPaymentInfo {
    pub address: DomainPaymentAddress,
    pub method: String,
    pub phone: String,
    pub payee_first_name: String,
    pub payee_last_name: String,
    pub last_4_digits: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainPaymentAddress {
    pub city: String,
    pub address: String,
    pub country: String,
    pub postal_code: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainBookingMetadata {
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainBookRoomResponse {
    pub booking_id: String,
    pub client_reference: String,
    pub supplier_booking_id: String,
    pub supplier_booking_name: String,
    pub supplier: String,
    pub supplier_id: u32,
    pub status: DomainBookingStatus,
    pub hotel_confirmation_code: String,
    pub checkin: String,
    pub checkout: String,
    pub hotel: DomainBookedHotel,
    pub booked_rooms: Vec<DomainBookedRoom>,
    pub holder: DomainBookingHolder,
    pub created_at: String,
    pub cancellation_policies: DomainCancellationPolicies,
    pub price: f64,
    pub commission: f64,
    pub currency: String,
    pub special_remarks: Option<String>,
    pub optional_fees: Option<String>,
    pub mandatory_fees: Option<String>,
    pub know_before_you_go: Option<String>,
    pub remarks: Option<String>,
    pub guest_id: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DomainBookingStatus {
    Confirmed,
    Pending,
    Failed,
    Cancelled,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainBookedHotel {
    pub hotel_id: String,
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainBookedRoom {
    pub room_type: DomainRoomTypeInfo,
    pub board_type: String,
    pub board_name: String,
    pub adults: u32,
    pub children: u32,
    pub rate: DomainBookedRoomRate,
    pub first_name: String,
    pub last_name: String,
    pub mapped_room_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainRoomTypeInfo {
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainBookedRoomRate {
    pub retail_rate: DomainBookedRetailRate,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainBookedRetailRate {
    pub total: DomainBookedPrice,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainBookedPrice {
    pub amount: f64,
    pub currency: String,
}

// BOOKING CONTEXT TYPES

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainBookingContext {
    pub number_of_rooms: u32,
    pub room_occupancies: Vec<DomainRoomOccupancyForBooking>,
    pub total_guests: u32,
    pub original_search_criteria: Option<DomainOriginalSearchInfo>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainRoomOccupancyForBooking {
    pub room_number: u32,
    pub adults: u32,
    pub children: u32,
    pub children_ages: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainOriginalSearchInfo {
    pub hotel_id: String,
    pub checkin_date: String,
    pub checkout_date: String,
    pub guest_nationality: Option<String>,
}

// GET BOOKING DETAILS DOMAIN TYPES

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainGetBookingRequest {
    pub client_reference: Option<String>,
    pub guest_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainGetBookingResponse {
    pub bookings: Vec<DomainBookingDetails>,
}

impl DomainGetBookingResponse {
    pub fn find_booking_by_client_reference(
        &self,
        client_ref: &str,
    ) -> Option<&DomainBookingDetails> {
        self.bookings.iter().find(|booking| {
            booking
                .client_reference
                .as_ref()
                .map(|ref_val| ref_val == client_ref)
                .unwrap_or(false)
        })
    }

    pub fn find_booking_by_booking_id(&self, booking_id: &str) -> Option<&DomainBookingDetails> {
        self.bookings
            .iter()
            .find(|booking| booking.booking_id == booking_id)
    }

    pub fn get_first_booking(&self) -> Option<&DomainBookingDetails> {
        self.bookings.first()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainBookingDetails {
    pub booking_id: String,
    pub client_reference: Option<String>,
    pub status: String,
    pub hotel: DomainBookingHotelInfo,
    pub holder: DomainBookingHolder,
    pub price: f64,
    pub currency: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainBookingHotelInfo {
    pub hotel_id: String,
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainBookingRoomInfo {
    pub adults: u32,
    pub children: String,
    pub first_name: String,
    pub last_name: String,
    pub children_ages: Option<String>,
    pub room_id: String,
    pub occupancy_number: u32,
    pub amount: f64,
    pub currency: String,
    pub children_count: u32,
    pub remarks: Option<String>,
    pub guests: Vec<DomainBookingGuestInfo>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainBookingGuestInfo {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: String,
    pub remarks: Option<String>,
    pub occupancy_number: u32,
}
