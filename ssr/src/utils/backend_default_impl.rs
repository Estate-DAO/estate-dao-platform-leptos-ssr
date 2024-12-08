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

use crate::canister::backend::PaymentDetails;
impl Default for PaymentDetails {
    fn default() -> Self {
        Self {
            payment_status: BackendPaymentStatus::Unpaid(None),
            booking_id: ("".to_string(), "".to_string()),
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
