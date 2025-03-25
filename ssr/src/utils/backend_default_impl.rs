use crate::canister::backend::BePaymentApiResponse;

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
impl Default for BookingId {
    fn default() -> Self {
        Self {
            app_reference: "".to_string(),
            email: "".to_string(),
        }
    }
}

use crate::canister::backend::BookingId;
use crate::canister::backend::PaymentDetails;
impl Default for PaymentDetails {
    fn default() -> Self {
        Self {
            payment_status: BackendPaymentStatus::Unpaid(None),
            booking_id: BookingId::default(),
            payment_api_response: BePaymentApiResponse::default(),
        }
    }
}

use crate::canister::backend::BackendPaymentStatus;
impl Default for BackendPaymentStatus {
    fn default() -> Self {
        BackendPaymentStatus::Unpaid(None)
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
//  pub enum BackendPaymentStatus { Paid(String), Unpaid(Option<String>) }
// #[derive(CandidType, serde::Deserialize, serde::Serialize, Debug, Clone)]
// pub struct Booking {
//   pub user_selected_hotel_room_details: HotelRoomDetails,
//   pub guests: UserDetails,
//   pub booking_id: BookingId,
//   pub book_room_status: Option<BeBookRoomResponse>,
//   pub payment_details: PaymentDetails,
// }
