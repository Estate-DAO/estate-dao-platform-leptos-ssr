use crate::log;
use crate::utils::app_reference::{generate_app_reference, BookingId};
use crate::view_state_layer::GlobalStateForLeptos;
use leptos::prelude::*;

#[derive(Debug, Clone, Default)]
pub struct BookingIdState {
    pub current_booking_id: RwSignal<Option<BookingId>>,
}

impl GlobalStateForLeptos for BookingIdState {}

impl BookingIdState {
    pub fn from_leptos_context() -> Self {
        Self::get()
    }

    /// Central function to get booking ID - implements the required logic:
    /// 1. Read from local storage first
    /// 2. If present, return that
    /// 3. If not present, log warning and generate new one
    pub fn get_or_create_booking_id(email: String) -> Option<BookingId> {
        let this = Self::from_leptos_context();

        // Check if we already have it in state
        if let Some(existing) = this.current_booking_id.get_untracked() {
            log!(
                "Using existing booking ID from state: {}",
                existing.to_order_id()
            );
            return Some(existing);
        }

        // Try to read from local storage
        if let Some(stored_booking_id) = BookingId::read_from_local_storage() {
            log!(
                "Found booking ID in local storage: {}",
                stored_booking_id.to_order_id()
            );
            // Store it in state for future use
            this.current_booking_id.set(Some(stored_booking_id.clone()));
            return Some(stored_booking_id);
        }

        // Not found in storage - log warning and generate new one
        log::warn!(
            "No booking ID found in local storage, generating new one for email: {}",
            email
        );

        // Generate new booking ID (this will also store it in local storage)
        let app_reference_signal = generate_app_reference(email.clone());
        if let Some(new_booking_id) = app_reference_signal.get_untracked() {
            log!("Generated new booking ID: {}", new_booking_id.to_order_id());
            // Store it in state for future use
            this.current_booking_id.set(Some(new_booking_id.clone()));
            Some(new_booking_id)
        } else {
            log::error!("Failed to generate new booking ID for email: {}", email);
            None
        }
    }

    /// Get current booking ID from state (if any)
    pub fn get_current_booking_id() -> Option<BookingId> {
        let this = Self::from_leptos_context();
        this.current_booking_id.get()
    }

    /// Get current booking ID as order ID string
    pub fn get_current_order_id() -> Option<String> {
        Self::get_current_booking_id().map(|booking_id| booking_id.to_order_id())
    }

    /// Clear the current booking ID (useful for testing or reset)
    pub fn clear_booking_id() {
        let this = Self::from_leptos_context();
        this.current_booking_id.set(None);
        log!("Cleared booking ID from state");
    }

    /// Force set a booking ID (useful for testing)
    pub fn set_booking_id(booking_id: BookingId) {
        let this = Self::from_leptos_context();
        this.current_booking_id.set(Some(booking_id.clone()));
        log!("Set booking ID in state: {}", booking_id.to_order_id());
    }

    /// Create a new booking ID (always generates fresh, ignores existing)
    pub fn create_booking_id(email: String) -> Option<BookingId> {
        let this = Self::from_leptos_context();

        log!("Creating new booking ID for email: {}", email);

        // Generate new booking ID (this will also store it in local storage)
        let app_reference_signal = generate_app_reference(email.clone());
        if let Some(new_booking_id) = app_reference_signal.get_untracked() {
            log!("Created new booking ID: {}", new_booking_id.to_order_id());
            // Store it in state for future use
            this.current_booking_id.set(Some(new_booking_id.clone()));
            Some(new_booking_id)
        } else {
            log::error!("Failed to create new booking ID for email: {}", email);
            None
        }
    }

    /// Debug helper to log current state
    #[cfg(feature = "debug_log")]
    pub fn log_state() {
        let this = Self::from_leptos_context();
        if let Some(booking_id) = this.current_booking_id.get() {
            log!(
                "BookingIdState - Current booking ID: {}",
                booking_id.to_order_id()
            );
        } else {
            log!("BookingIdState - No booking ID in state");
        }
    }
}
