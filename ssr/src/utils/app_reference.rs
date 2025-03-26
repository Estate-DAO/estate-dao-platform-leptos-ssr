use chrono::Local;
use codee::string::JsonSerdeCodec;
use leptos::SignalGetUntracked;
use leptos::{Signal, SignalGet, SignalSet};
use leptos_use::storage::{use_local_storage, UseStorageOptions};
use log::info;
// Import necessary modules
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::{canister::backend::Booking, state::local_storage::use_booking_id_store};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct BookingId {
    pub email: String,
    pub app_reference: String,
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

    /// Attempts to read the booking details from local storage
    /// Returns a tuple of (email, app_reference) if found
    pub fn read_from_storage() -> Option<(String, String)> {
        let (state, _, _) = use_booking_id_store();
        state
            .get()
            .map(|booking_id| (booking_id.get_email(), booking_id.get_app_reference()))
    }

    pub fn read_from_local_storage() -> Option<Self> {
        let (state, _, _) = use_booking_id_store();
        state.get_untracked()
    }

    /// Attempts to read the booking details from local storage with fallback
    /// Returns a tuple of (email, app_reference), using empty strings as fallback
    pub fn read_from_storage_with_fallback() -> (String, String) {
        Self::read_from_storage().unwrap_or_else(|| {
            log::warn!("Failed to read booking details from local storage, using fallback");
            (String::new(), String::new())
        })
    }

    /// Store a BookingId in local storage and return the storage signal
    fn store_booking_id_in_local_storage(&self) -> Signal<Option<BookingId>> {
        let (state, set_state, _) = use_booking_id_store();

        // Log the storage for debugging
        info!(
            "Storing booking_id with app_reference: {}",
            self.get_app_reference()
        );

        set_state(Some(self.clone()));
        state
    }
}

/// Generates a new app reference and stores it in local storage
/// Format: HB<date>-<random>-<random>
/// Example: HB2203-12345-67890
/// Generate and store a new app reference
pub fn generate_app_reference(email: String) -> Signal<Option<BookingId>> {
    // Generate a unique app reference
    let today = Local::now().format("%d%m").to_string();
    let rand_num1: u32 = rand::thread_rng().gen_range(10000..99999);
    let rand_num2: u32 = rand::thread_rng().gen_range(10000..99999);
    let app_reference_string = format!("HB{}-{}-{}", today, rand_num1, rand_num2);

    // Create new BookingId
    let booking_id = BookingId::new(email, app_reference_string);

    // Store and return
    booking_id.store_booking_id_in_local_storage()
}

/// Reads the current app_reference from local storage
/// Returns None if not found or if there was an error
// pub fn read_app_reference() -> Option<String> {
//     BookingId::read_from_local_storage().map(|(_, app_ref)| app_ref)
// }

#[cfg(test)]
mod tests {
    use crate::utils::booking_id::PaymentIdentifiers;

    use super::*;

    #[test]
    fn test_booking_id_conversion() {
        let email = "test@example.com".to_string();
        let app_reference = "HB2203-12345-67890".to_string();
        let booking_id = BookingId::new(email.clone(), app_reference.clone());

        // Test getters
        assert_eq!(booking_id.get_email(), email);
        assert_eq!(booking_id.get_app_reference(), app_reference);
    }

    #[test]
    fn test_app_reference_format() {
        let email = "test@example.com".to_string();
        let app_reference = generate_app_reference(email);

        // Get the app_reference string
        let app_ref_str = app_reference
            .get()
            .expect("Should have a value")
            .get_app_reference();

        // Test format: HB<date>-<random>-<random>
        assert!(app_ref_str.starts_with("HB"));
        assert_eq!(app_ref_str.matches('-').count(), 2);

        let parts: Vec<&str> = app_ref_str.split('-').collect();
        assert_eq!(parts.len(), 3);

        // First part should be HB<date> (6 chars)
        assert_eq!(parts[0].len(), 6);
        // Middle and last parts should be 5-digit numbers
        assert_eq!(parts[1].len(), 5);
        assert_eq!(parts[2].len(), 5);

        // Middle and last parts should be numbers
        assert!(parts[1].parse::<u32>().is_ok());
        assert!(parts[2].parse::<u32>().is_ok());
    }

    #[test]
    fn test_order_id_conversion() {
        let email = "test@example.com".to_string();
        let app_reference = "HB2203-12345-67890".to_string();
        let booking_id = BookingId::new(email.clone(), app_reference.clone());

        // Convert to order_id
        let order_id = PaymentIdentifiers::order_id_from_app_reference(&app_reference, &email);

        // Extract back from order_id
        let extracted_app_ref = PaymentIdentifiers::app_reference_from_order_id(&order_id)
            .expect("Should extract app_reference");

        assert_eq!(extracted_app_ref, app_reference);
    }

    #[test]
    fn test_fallback_behavior() {
        // Test with invalid order_id
        let invalid_order_id = "invalid-order-id";
        let app_ref = PaymentIdentifiers::app_reference_from_order_id(invalid_order_id);
        assert!(app_ref.is_none());

        // Test with empty strings
        let empty_email = String::new();
        let empty_app_ref = String::new();
        let booking_id = BookingId::new(empty_email, empty_app_ref);

        assert_eq!(booking_id.get_email(), "");
        assert_eq!(booking_id.get_app_reference(), "");
    }

    #[test]
    fn test_special_characters() {
        let email = "user+test@example.com".to_string();
        let app_reference = "HB2203-12345-67890".to_string();
        let booking_id = BookingId::new(email.clone(), app_reference.clone());

        // Convert to order_id
        let order_id = PaymentIdentifiers::order_id_from_app_reference(&app_reference, &email);

        // Extract back from order_id
        let extracted_app_ref = PaymentIdentifiers::app_reference_from_order_id(&order_id)
            .expect("Should extract app_reference");

        assert_eq!(extracted_app_ref, app_reference);
    }
}
