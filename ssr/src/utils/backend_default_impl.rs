use crate::{
    canister::backend::{
        self, BackendPaymentStatus, BePaymentApiResponse, Booking, BookingId, BookingSummary,
        Destination, HotelDetails, HotelRoomDetails, PaymentDetails, RoomDetails, UserDetails,
    },
    component::{Destination as FrontendDestination, SelectedDateRange as FrontendDateRange},
    domain::{
        DomainAdultDetail, DomainBookRoomRequest, DomainBookRoomResponse, DomainBookedHotel,
        DomainBookedRoom, DomainBookingGuest, DomainBookingHolder, DomainBookingStatus,
        DomainCancellationPolicies, DomainChildDetail, DomainDestination, DomainDetailedPrice,
        DomainFirstRoomDetails, DomainGuestPaymentInfo, DomainHotelDetails, DomainPaymentAddress,
        DomainPaymentInfo, DomainPrice, DomainRoomData, DomainSelectedDateRange, DomainUserDetails,
    },
    view_state_layer::view_state::{
        AdultDetail as FrontendAdultDetail, ChildDetail as FrontendChildDetail,
    },
};

use crate::utils::app_reference::BookingId as AppReferenceBookingId;

const ROOM_ID_OCC_SUFFIX: &str = "__occ__";

/// Encode occupancy number alongside the room identifier so we can persist it without schema changes.
pub fn encode_room_id_with_occupancy(room_id: &str, occupancy_number: Option<u32>) -> String {
    occupancy_number
        .map(|num| format!("{room_id}{ROOM_ID_OCC_SUFFIX}{num}"))
        .unwrap_or_else(|| room_id.to_string())
}

/// Decode the stored room identifier into (original_id, occupancy_number).
pub fn decode_room_id_with_occupancy(encoded_id: &str) -> (String, Option<u32>) {
    if let Some(pos) = encoded_id.rfind(ROOM_ID_OCC_SUFFIX) {
        if let Ok(num) = encoded_id[pos + ROOM_ID_OCC_SUFFIX.len()..].parse::<u32>() {
            return (encoded_id[..pos].to_string(), Some(num));
        }
    }
    (encoded_id.to_string(), None)
}

impl PartialEq for BookingSummary {
    fn eq(&self, other: &Self) -> bool {
        self.booking_id.app_reference == other.booking_id.app_reference
    }
}

impl Default for BePaymentApiResponse {
    fn default() -> Self {
        Self {
            updated_at: String::default(),
            actually_paid: f64::default(),
            provider: String::default(),
            invoice_id: u64::default(),
            order_description: String::default(),
            pay_amount: f64::default(),
            pay_currency: String::default(),
            created_at: String::default(),
            payment_status: String::default(),
            price_amount: u64::default(),
            purchase_id: u64::default(),
            order_id: String::default(),
            price_currency: String::default(),
            payment_id: u64::default(),
            payment_id_v2: String::default(),
        }
    }
}

impl From<backend::SelectedDateRange> for FrontendDateRange {
    fn from(backend: backend::SelectedDateRange) -> Self {
        Self {
            start: (backend.start.0, backend.start.1, backend.start.2),
            end: (backend.end.0, backend.end.1, backend.end.2),
        }
    }
}

impl From<FrontendDateRange> for backend::SelectedDateRange {
    fn from(frontend: FrontendDateRange) -> Self {
        Self {
            start: frontend.start,
            end: frontend.end,
        }
    }
}

impl Default for BookingId {
    fn default() -> Self {
        Self {
            app_reference: "".to_string(),
            email: "".to_string(),
        }
    }
}

impl From<backend::Destination> for FrontendDestination {
    fn from(backend_destination: backend::Destination) -> Self {
        Self {
            city: backend_destination.city,
            country_name: backend_destination.country_name,
            country_code: backend_destination.country_code,
            city_id: backend_destination.city_id,
            latitude: None,
            longitude: None,
        }
    }
}

impl From<FrontendDestination> for backend::Destination {
    fn from(frontend_destination: FrontendDestination) -> Self {
        Self {
            city_id: frontend_destination.city_id,
            city: frontend_destination.city,
            country_code: frontend_destination.country_code,
            country_name: frontend_destination.country_name,
        }
    }
}

impl From<FrontendAdultDetail> for crate::canister::backend::AdultDetail {
    fn from(value: FrontendAdultDetail) -> Self {
        Self {
            email: value.email,
            first_name: value.first_name,
            last_name: value.last_name,
            phone: value.phone,
        }
    }
}

impl From<backend::AdultDetail> for FrontendAdultDetail {
    fn from(value: backend::AdultDetail) -> Self {
        Self {
            email: value.email,
            first_name: value.first_name,
            last_name: value.last_name,
            phone: value.phone,
        }
    }
}

impl From<backend::ChildDetail> for FrontendChildDetail {
    fn from(value: backend::ChildDetail) -> Self {
        Self {
            age: Some(value.age),
            first_name: value.first_name,
            last_name: value.last_name,
        }
    }
}

impl From<FrontendChildDetail> for backend::ChildDetail {
    fn from(value: FrontendChildDetail) -> Self {
        Self {
            age: value.age.unwrap_or_default(),
            first_name: value.first_name,
            last_name: value.last_name,
        }
    }
}

impl Default for PaymentDetails {
    fn default() -> Self {
        Self {
            payment_status: BackendPaymentStatus::Unpaid(None),
            booking_id: BookingId::default(),
            payment_api_response: BePaymentApiResponse::default(),
        }
    }
}

impl Default for BackendPaymentStatus {
    fn default() -> Self {
        BackendPaymentStatus::Unpaid(None)
    }
}

impl Default for backend::AdultDetail {
    fn default() -> Self {
        Self {
            email: None,
            first_name: String::default(),
            last_name: None,
            phone: None,
        }
    }
}

impl BackendPaymentStatus {
    // todo: log actual status / reason
    pub fn to_string(&self) -> String {
        match self {
            Self::Paid(status) => String::from("finished"),
            Self::Unpaid(reason) => String::from("unpaid"),
        }
    }
}

impl Booking {
    pub fn get_destination(&self) -> Option<Destination> {
        self.user_selected_hotel_room_details.get_destination()
    }
    pub fn get_user_email(&self) -> String {
        self.booking_id.email.clone()
    }
    pub fn get_user_name(&self) -> String {
        self.guests.get_first_name()
    }
    pub fn get_user_phone(&self) -> String {
        self.guests.get_first_phone()
    }
    /// Hotel name for emails
    pub fn get_hotel_name(&self) -> String {
        self.user_selected_hotel_room_details
            .hotel_details
            .hotel_name
            .clone()
    }
    /// Hotel location for emails
    pub fn get_hotel_location(&self) -> String {
        self.user_selected_hotel_room_details
            .hotel_details
            .hotel_location
            .clone()
    }
    /// Hotel code for deep links
    pub fn get_hotel_code(&self) -> String {
        self.user_selected_hotel_room_details
            .hotel_details
            .hotel_code
            .clone()
    }
    /// Primary hotel image for emails (may be empty)
    pub fn get_hotel_image(&self) -> String {
        self.user_selected_hotel_room_details
            .hotel_details
            .hotel_image
            .clone()
    }
    /// Booking reference/app id
    pub fn get_booking_ref(&self) -> String {
        self.booking_id.app_reference.clone()
    }
    /// Booking reference number from provider (if booking completed)
    pub fn get_booking_ref_no(&self) -> String {
        self.book_room_status
            .as_ref()
            .map(|status| status.commit_booking.booking_ref_no.clone())
            .unwrap_or_default()
    }
    /// Confirmation number from provider (if booking completed)
    pub fn get_confirmation_no(&self) -> String {
        self.book_room_status
            .as_ref()
            .map(|status| status.commit_booking.confirmation_no.clone())
            .unwrap_or_default()
    }
    /// Check if booking is actually confirmed based on booking status
    pub fn is_booking_confirmed(&self) -> bool {
        self.book_room_status
            .as_ref()
            .map(|status| {
                matches!(
                    status.commit_booking.api_status,
                    crate::canister::backend::BookingStatus::Confirmed
                )
            })
            .unwrap_or(false)
    }
    /// Get booking status message
    pub fn get_booking_status_message(&self) -> String {
        self.book_room_status
            .as_ref()
            .map(|status| status.commit_booking.booking_status.clone())
            .unwrap_or_else(|| "Pending confirmation".to_string())
    }
    /// Check-in date as YYYY-MM-DD
    pub fn get_check_in_date(&self) -> String {
        let (y, m, d) = self.user_selected_hotel_room_details.date_range.start;
        format!("{:04}-{:02}-{:02}", y, m, d)
    }
    /// Check-out date as YYYY-MM-DD
    pub fn get_check_out_date(&self) -> String {
        let (y, m, d) = self.user_selected_hotel_room_details.date_range.end;
        format!("{:04}-{:02}-{:02}", y, m, d)
    }
    /// Check-in date formatted as "04th April 2025"
    pub fn get_check_in_date_formatted(&self) -> String {
        let (y, m, d) = self.user_selected_hotel_room_details.date_range.start;
        Self::format_date_display(y, m, d)
    }
    /// Check-out date formatted as "06th April 2025"
    pub fn get_check_out_date_formatted(&self) -> String {
        let (y, m, d) = self.user_selected_hotel_room_details.date_range.end;
        Self::format_date_display(y, m, d)
    }
    /// Format date as "04th April 2025"
    fn format_date_display(year: u32, month: u32, day: u32) -> String {
        let month_names = [
            "",
            "January",
            "February",
            "March",
            "April",
            "May",
            "June",
            "July",
            "August",
            "September",
            "October",
            "November",
            "December",
        ];

        let month_name = month_names.get(month as usize).unwrap_or(&"");
        let day_suffix = match day % 10 {
            1 if day != 11 => "st",
            2 if day != 12 => "nd",
            3 if day != 13 => "rd",
            _ => "th",
        };

        format!("{:02}{} {} {}", day, day_suffix, month_name, year)
    }
    /// Number of adults
    pub fn get_number_of_adults(&self) -> usize {
        self.guests.adults.len()
    }
    /// Number of children
    pub fn get_number_of_children(&self) -> usize {
        self.guests.children.len()
    }
    /// Number of rooms requested/booked
    pub fn get_number_of_rooms(&self) -> usize {
        self.user_selected_hotel_room_details
            .room_details
            .len()
            .max(1)
    }
    /// Amount paid by user
    pub fn get_amount_paid(&self) -> f64 {
        self.payment_details.payment_api_response.actually_paid
    }
    /// Number of nights between check-in and check-out
    pub fn get_number_of_nights(&self) -> u32 {
        let (start_y, start_m, start_d) = self.user_selected_hotel_room_details.date_range.start;
        let (end_y, end_m, end_d) = self.user_selected_hotel_room_details.date_range.end;

        // Simple calculation - for production use proper date library
        let start_days = start_y * 365 + start_m * 30 + start_d;
        let end_days = end_y * 365 + end_m * 30 + end_d;

        if end_days > start_days {
            end_days - start_days
        } else {
            1 // Default to 1 night
        }
    }
    /// Email of the first adult guest
    pub fn get_first_adult_email(&self) -> Option<String> {
        self.guests
            .adults
            .first()
            .unwrap_or(&backend::AdultDetail::default())
            .email
            .clone()
    }
    /// Email of the last adult guest
    pub fn get_last_adult_email(&self) -> Option<String> {
        self.guests
            .adults
            .last()
            .unwrap_or(&backend::AdultDetail::default())
            .email
            .clone()
    }
}

impl HotelRoomDetails {
    /// Returns a reference to the destination if it exists
    pub fn get_destination(&self) -> Option<Destination> {
        self.destination.clone()
    }
}

impl backend::UserDetails {
    /// Returns the first name of the first adult guest or an empty string if none
    pub fn get_first_name(&self) -> String {
        self.adults
            .first()
            .unwrap_or(&backend::AdultDetail::default())
            .first_name
            .clone()
    }
    /// Returns the phone number of the first adult guest or an empty string if none
    pub fn get_first_phone(&self) -> String {
        self.adults
            .first()
            .unwrap_or(&backend::AdultDetail::default())
            .phone
            .clone()
            .unwrap_or_default()
    }
}

//  pub enum BackendPaymentStatus { Paid(String), Unpaid(Option<String>) }
// #[derive(CandidType, serde::Deserialize, serde::Serialize, Debug, Clone)]
// pub struct Booking {
//   pub user_selected_hotel_room_details: HotelRoomDetails,
//   pub guests: UserDetails,
//   pub booking_id: BookingId,
//   pub book_room_status: Option<BeBookRoomResponse>,
//   pub payment_details: PaymentDetails,
// }

// convert BookingId to backend::BookingId
impl From<AppReferenceBookingId> for backend::BookingId {
    fn from(value: AppReferenceBookingId) -> Self {
        Self {
            app_reference: value.app_reference,
            email: value.email,
        }
    }
}

// Domain to Backend conversions

// impl From<DomainDestination> for backend::Destination {
//     fn from(domain: DomainDestination) -> Self {
//         Self {
//             city_id: domain.city_id,
//             city: domain.city_name,
//             country_code: domain.country_code,
//             country_name: domain.country_name,
//         }
//     }
// }

impl From<backend::Destination> for DomainDestination {
    fn from(backend: backend::Destination) -> Self {
        Self {
            place_id: String::new(),
            // city_name: Some(backend.city),
            // country_code: Some(backend.country_code),
            // country_name: Some(backend.country_name),
        }
    }
}

impl From<DomainSelectedDateRange> for backend::SelectedDateRange {
    fn from(domain: DomainSelectedDateRange) -> Self {
        Self {
            start: domain.start,
            end: domain.end,
        }
    }
}

impl From<backend::SelectedDateRange> for DomainSelectedDateRange {
    fn from(backend: backend::SelectedDateRange) -> Self {
        Self {
            start: backend.start,
            end: backend.end,
        }
    }
}

impl From<DomainAdultDetail> for backend::AdultDetail {
    fn from(domain: DomainAdultDetail) -> Self {
        Self {
            email: domain.email,
            first_name: domain.first_name,
            last_name: domain.last_name,
            phone: domain.phone,
        }
    }
}

impl From<backend::AdultDetail> for DomainAdultDetail {
    fn from(backend: backend::AdultDetail) -> Self {
        Self {
            email: backend.email,
            first_name: backend.first_name,
            last_name: backend.last_name,
            phone: backend.phone,
        }
    }
}

impl From<DomainChildDetail> for backend::ChildDetail {
    fn from(domain: DomainChildDetail) -> Self {
        Self {
            age: domain.age,
            first_name: domain.first_name,
            last_name: domain.last_name,
        }
    }
}

impl From<backend::ChildDetail> for DomainChildDetail {
    fn from(backend: backend::ChildDetail) -> Self {
        Self {
            age: backend.age,
            first_name: backend.first_name,
            last_name: backend.last_name,
        }
    }
}

impl From<DomainUserDetails> for backend::UserDetails {
    fn from(domain: DomainUserDetails) -> Self {
        Self {
            children: domain.children.into_iter().map(|c| c.into()).collect(),
            adults: domain.adults.into_iter().map(|a| a.into()).collect(),
        }
    }
}

impl From<backend::UserDetails> for DomainUserDetails {
    fn from(backend: backend::UserDetails) -> Self {
        Self {
            children: backend.children.into_iter().map(|c| c.into()).collect(),
            adults: backend.adults.into_iter().map(|a| a.into()).collect(),
        }
    }
}

// Hotel Room Details conversions
impl From<DomainRoomData> for backend::RoomDetails {
    fn from(domain: DomainRoomData) -> Self {
        Self {
            room_price: 0.0, // Price will need to be set separately
            room_unique_id: encode_room_id_with_occupancy(
                &domain.room_unique_id,
                domain.occupancy_number,
            ),
            room_type_name: domain.room_name,
        }
    }
}

impl From<DomainDetailedPrice> for f32 {
    fn from(domain: DomainDetailedPrice) -> Self {
        domain.room_price as f32
    }
}

// HotelRoomDetails construction helper
impl HotelRoomDetails {
    pub fn from_domain_parts(
        destination: Option<DomainDestination>,
        date_range: DomainSelectedDateRange,
        room_details: Vec<DomainRoomData>,
        hotel_details: DomainHotelDetails,
        requested_payment_amount: f64,
    ) -> Self {
        Self {
            destination: None, /* destination.map(|d| d.into()) */
            date_range: date_range.into(),
            room_details: room_details.into_iter().map(|r| r.into()).collect(),
            hotel_details: HotelDetails {
                hotel_code: hotel_details.hotel_code,
                hotel_name: hotel_details.hotel_name,
                hotel_image: hotel_details.images.first().cloned().unwrap_or_default(),
                block_room_id: "".to_string(), // Will need to be set during block room
                hotel_location: hotel_details.address,
                hotel_token: "".to_string(), // Will need to be set from search token
            },
            requested_payment_amount,
        }
    }
}

// Booking conversions
impl From<DomainBookingHolder> for backend::AdultDetail {
    fn from(domain: DomainBookingHolder) -> Self {
        Self {
            email: Some(domain.email),
            first_name: domain.first_name,
            last_name: Some(domain.last_name),
            phone: Some(domain.phone),
        }
    }
}

impl From<backend::AdultDetail> for DomainBookingHolder {
    fn from(backend: backend::AdultDetail) -> Self {
        Self {
            email: backend.email.unwrap_or_default(),
            first_name: backend.first_name,
            last_name: backend.last_name.unwrap_or_default(),
            phone: backend.phone.unwrap_or_default(),
        }
    }
}

// Helper function to create backend Booking from domain parts
impl Booking {
    pub fn from_domain_parts(
        hotel_room_details: HotelRoomDetails,
        guests: DomainUserDetails,
        booking_id: BookingId,
        payment_amount: f64,
        payment_currency: String,
    ) -> Self {
        Self {
            user_selected_hotel_room_details: hotel_room_details,
            guests: guests.into(),
            booking_id: booking_id.clone(),
            book_room_status: None,
            payment_details: PaymentDetails {
                payment_status: BackendPaymentStatus::Unpaid(None),
                booking_id,
                payment_api_response: BePaymentApiResponse {
                    pay_amount: payment_amount,
                    pay_currency: payment_currency,
                    ..Default::default()
                },
            },
        }
    }
}
