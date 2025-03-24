// fake imports
use fake::{Dummy, Fake, Faker};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::api::SuccessGetPaymentStatusResponse;

impl Dummy<Faker> for SuccessGetPaymentStatusResponse {
    fn dummy_with_rng<R: Rng + ?Sized>(_: &Faker, _rng: &mut R) -> Self {
        Self {
            payment_id: 1234567890,
            invoice_id: 1234567890,
            payment_status: "finished".to_string(),
            price_amount: 10,
            price_currency: "USD".to_string(),
            pay_amount: 10.0,
            pay_currency: "USD".to_string(),
            order_id: "1234567890".to_string(),
            order_description: "Test Order".to_string(),
            purchase_id: 1234567890,
            created_at: "2023-01-01T00:00:00Z".to_string(),
            updated_at: "2023-01-01T00:00:00Z".to_string(),
            actually_paid: 10.0,
        }
    }
}
