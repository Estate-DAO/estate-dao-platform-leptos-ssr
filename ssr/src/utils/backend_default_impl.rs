use crate::{
    canister::backend::{
        self, BackendPaymentStatus, BePaymentApiResponse, Booking, BookingId, Destination,
        HotelRoomDetails, PaymentDetails,
    },
    component::{Destination as FrontendDestination, SelectedDateRange as FrontendDateRange},
    state::view_state::{AdultDetail as FrontendAdultDetail, ChildDetail as FrontendChildDetail},
};

use crate::utils::app_reference::BookingId as AppReferenceBookingId;

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
    /// Booking reference/app id
    pub fn get_booking_ref(&self) -> String {
        self.booking_id.app_reference.clone()
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
    /// Number of adults
    pub fn get_number_of_adults(&self) -> usize {
        self.guests.adults.len()
    }
    /// Number of children
    pub fn get_number_of_children(&self) -> usize {
        self.guests.children.len()
    }
    /// Amount paid by user
    pub fn get_amount_paid(&self) -> f64 {
        self.payment_details.payment_api_response.actually_paid
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
