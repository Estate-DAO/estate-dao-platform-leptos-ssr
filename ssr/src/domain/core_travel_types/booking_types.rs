use serde::{Deserialize, Serialize};

// <!-- BOOKING DOMAIN TYPES -->
// Domain types for hotel booking operations - provider-agnostic

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainBookRoomRequest {
    // Block ID from previous block_room call
    pub block_id: String,

    // Booking holder information (credit card holder/main contact)
    pub holder: DomainBookingHolder,

    // Guest details for each room (one primary contact per room)
    pub guests: Vec<DomainBookingGuest>,

    // Payment information
    pub payment: DomainPaymentInfo,

    // Optional guest payment info (for external payment processing)
    pub guest_payment: Option<DomainGuestPaymentInfo>,

    // Metadata for tracking and analytics
    // pub metadata: Option<DomainBookingMetadata>,

    // Special requests or remarks
    pub special_requests: Option<String>,

    // Booking context - provides validation data for different providers
    pub booking_context: DomainBookingContext,

    // Client-defined reference for idempotency (prevents duplicate bookings)
    // Same concept as Provab's app_reference
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
    // Which room this guest is the primary contact for (starts from 1)
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DomainPaymentMethod {
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
    // Unique booking identifier
    pub booking_id: String,

    // App-specific reference for internal tracking
    pub client_reference: String,

    // Supplier-specific booking information
    pub supplier_booking_id: String,
    pub supplier_booking_name: String,
    pub supplier: String,
    pub supplier_id: u32,

    // Booking status (normalized across providers)
    pub status: DomainBookingStatus,

    // Hotel confirmation code
    pub hotel_confirmation_code: String,

    // Check-in/out dates
    pub checkin: String,
    pub checkout: String,

    // Hotel information
    pub hotel: DomainBookedHotel,

    // Booked rooms details
    pub booked_rooms: Vec<DomainBookedRoom>,

    // Booking holder
    pub holder: DomainBookingHolder,

    // Booking creation timestamp
    pub created_at: String,

    // Cancellation policies
    pub cancellation_policies: DomainCancellationPolicies,

    // Price information
    pub price: f64,
    pub commission: f64,
    pub currency: String,

    // Additional information
    pub special_remarks: Option<String>,
    pub optional_fees: Option<String>,
    pub mandatory_fees: Option<String>,
    pub know_before_you_go: Option<String>,
    pub remarks: Option<String>,

    // Provider-specific data
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainCancellationPolicies {
    pub cancel_policy_infos: Vec<DomainCancelPolicyInfo>,
    pub hotel_remarks: Option<String>,
    pub refundable_tag: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainCancelPolicyInfo {
    pub cancel_time: String,
    pub amount: f64,
    pub policy_type: String,
    pub timezone: String,
    pub currency: String,
}

// <!-- BOOKING CONTEXT TYPES -->
// Additional context needed for proper validation across different providers

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainBookingContext {
    // Number of rooms being booked (required for guest validation)
    pub number_of_rooms: u32,

    // Original occupancy information from the search/rates request
    // This helps validate guest counts and room assignments
    pub room_occupancies: Vec<DomainRoomOccupancy>,

    // Total number of guests across all rooms
    pub total_guests: u32,

    // Original search criteria that led to this booking
    pub original_search_criteria: Option<DomainOriginalSearchInfo>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainRoomOccupancy {
    // Room number (1-indexed)
    pub room_number: u32,

    // Number of adults in this room
    pub adults: u32,

    // Number of children in this room
    pub children: u32,

    // Ages of children in this room (for validation)
    pub children_ages: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainOriginalSearchInfo {
    // Hotel ID being booked
    pub hotel_id: String,

    // Check-in and check-out dates
    pub checkin_date: String,
    pub checkout_date: String,

    // Guest nationality (some providers require this)
    pub guest_nationality: Option<String>,
}
