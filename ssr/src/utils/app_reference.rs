use chrono::Local;
use codee::string::JsonSerdeCodec;
use leptos::Signal;
use leptos_use::storage::{use_local_storage, UseStorageOptions}; // Import necessary modules
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::canister::backend::Booking;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct BookingId {
    email: String,
    app_reference: String,
}

impl BookingId {
    pub fn new(email: String, app_reference: String) -> Self {
        BookingId {
            email,
            app_reference,
        }
    }

    pub fn get_email(&self) -> String {
        self.email.clone()
    }

    pub fn get_app_reference(&self) -> String {
        self.app_reference.clone()
    }
}

pub fn generate_app_reference(email: String) -> Signal<BookingId> {
    let today = Local::now().format("%d%m").to_string();
    let rand_num1: u32 = rand::thread_rng().gen_range(100000..999999);
    let rand_num2: u32 = rand::thread_rng().gen_range(100000..999999);
    let app_reference_string = format!("HB{}-{}-{}", today, rand_num1, rand_num2);
    let app_reference = BookingId::new(email, app_reference_string);
    let (state, set_state, _) = use_local_storage::<BookingId, JsonSerdeCodec>("booking_id");
    set_state(app_reference);
    state
}
