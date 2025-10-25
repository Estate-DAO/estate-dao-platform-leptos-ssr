use crate::view_state_layer::GlobalStateForLeptos;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// **Booking Status Enum**
/// Represents the current state of a hotel booking
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BookingStatus {
    /// Booking is confirmed and upcoming
    Confirmed,
    /// Booking is completed (past)
    Completed,
    /// Booking was cancelled
    Cancelled,
    /// Booking is pending confirmation
    Pending,
    /// Booking failed or was rejected
    Failed,
}

impl BookingStatus {
    pub fn to_string(&self) -> &'static str {
        match self {
            BookingStatus::Confirmed => "Confirmed",
            BookingStatus::Completed => "Completed",
            BookingStatus::Cancelled => "Cancelled",
            BookingStatus::Pending => "Pending",
            BookingStatus::Failed => "Failed",
        }
    }

    pub fn get_color_class(&self) -> &'static str {
        match self {
            BookingStatus::Confirmed => "text-green-600",
            BookingStatus::Completed => "text-blue-600",
            BookingStatus::Cancelled => "text-red-600",
            BookingStatus::Pending => "text-yellow-600",
            BookingStatus::Failed => "text-red-600",
        }
    }
}

/// **Hotel Booking Data Structure**
/// Represents a single hotel booking with all necessary information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotelBooking {
    /// Unique booking identifier
    pub booking_id: String,
    /// Hotel name
    pub hotel_name: String,
    /// Hotel location (e.g., "South Goa, India")
    pub location: String,
    /// Hotel image URL
    pub image_url: Option<String>,
    /// Booking status
    pub status: BookingStatus,
    /// Check-in date (ISO format)
    pub check_in_date: String,
    /// Check-out date (ISO format)
    pub check_out_date: String,
    /// Number of adults
    pub adults: u32,
    /// Number of children
    pub children: u32,
    /// Number of rooms
    pub rooms: u32,
    /// Total price (optional, for completed bookings)
    pub total_price: Option<f64>,
    /// Currency code (e.g., "USD", "INR")
    pub currency: Option<String>,
    /// Guest email
    pub guest_email: Option<String>,
    /// Additional booking metadata
    pub metadata: HashMap<String, String>,
}

/// **Bookings by Status**
/// Organizes bookings by their current status for easy tab rendering
#[derive(Debug, Clone, Default)]
pub struct BookingsByStatus {
    pub upcoming: Vec<HotelBooking>,
    pub completed: Vec<HotelBooking>,
    pub cancelled: Vec<HotelBooking>,
}

/// **UI Bookings State**
/// Centralized state management for user bookings across all tabs
#[derive(Debug, Clone, Default)]
pub struct UIBookingsState {
    /// All bookings organized by status
    pub bookings: RwSignal<BookingsByStatus>,
    
    /// Loading states for each tab
    pub loading_upcoming: RwSignal<bool>,
    pub loading_completed: RwSignal<bool>,
    pub loading_cancelled: RwSignal<bool>,
    
    /// Error states for each tab
    pub error_upcoming: RwSignal<Option<String>>,
    pub error_completed: RwSignal<Option<String>>,
    pub error_cancelled: RwSignal<Option<String>>,
    
    /// Last fetch timestamp for caching
    pub last_fetch: RwSignal<Option<chrono::DateTime<chrono::Utc>>>,
    
    /// Selected booking for actions (like viewing details)
    pub selected_booking: RwSignal<Option<HotelBooking>>,
}

impl GlobalStateForLeptos for UIBookingsState {}

impl UIBookingsState {
    /// **Initialize the bookings state**
    pub fn initialize() {
        let this = Self::get();
        
        // Reset all states
        this.bookings.set(BookingsByStatus::default());
        this.loading_upcoming.set(false);
        this.loading_completed.set(false);
        this.loading_cancelled.set(false);
        this.error_upcoming.set(None);
        this.error_completed.set(None);
        this.error_cancelled.set(None);
        this.last_fetch.set(None);
        this.selected_booking.set(None);
    }

    // **Getters for reactive UI access**

    /// **Get upcoming bookings**
    pub fn get_upcoming_bookings() -> Signal<Vec<HotelBooking>> {
        Signal::derive(move || {
            Self::get().bookings.get().upcoming
        })
    }

    /// **Get completed bookings**
    pub fn get_completed_bookings() -> Signal<Vec<HotelBooking>> {
        Signal::derive(move || {
            Self::get().bookings.get().completed
        })
    }

    /// **Get cancelled bookings**
    pub fn get_cancelled_bookings() -> Signal<Vec<HotelBooking>> {
        Signal::derive(move || {
            Self::get().bookings.get().cancelled
        })
    }

    /// **Check if upcoming bookings are loading**
    pub fn is_loading_upcoming() -> Signal<bool> {
        Self::get().loading_upcoming.into()
    }

    /// **Check if completed bookings are loading**
    pub fn is_loading_completed() -> Signal<bool> {
        Self::get().loading_completed.into()
    }

    /// **Check if cancelled bookings are loading**
    pub fn is_loading_cancelled() -> Signal<bool> {
        Self::get().loading_cancelled.into()
    }

    /// **Get error for upcoming bookings**
    pub fn get_error_upcoming() -> Signal<Option<String>> {
        Self::get().error_upcoming.into()
    }

    /// **Get error for completed bookings**
    pub fn get_error_completed() -> Signal<Option<String>> {
        Self::get().error_completed.into()
    }

    /// **Get error for cancelled bookings**
    pub fn get_error_cancelled() -> Signal<Option<String>> {
        Self::get().error_cancelled.into()
    }

    /// **Get selected booking**
    pub fn get_selected_booking() -> Signal<Option<HotelBooking>> {
        Self::get().selected_booking.into()
    }

    // **Setters for updating state**

    /// **Set upcoming bookings**
    pub fn set_upcoming_bookings(bookings: Vec<HotelBooking>) {
        let this = Self::get();
        let mut current_bookings = this.bookings.get_untracked();
        current_bookings.upcoming = bookings;
        this.bookings.set(current_bookings);
        this.loading_upcoming.set(false);
        this.error_upcoming.set(None);
    }

    /// **Set completed bookings**
    pub fn set_completed_bookings(bookings: Vec<HotelBooking>) {
        let this = Self::get();
        let mut current_bookings = this.bookings.get_untracked();
        current_bookings.completed = bookings;
        this.bookings.set(current_bookings);
        this.loading_completed.set(false);
        this.error_completed.set(None);
    }

    /// **Set cancelled bookings**
    pub fn set_cancelled_bookings(bookings: Vec<HotelBooking>) {
        let this = Self::get();
        let mut current_bookings = this.bookings.get_untracked();
        current_bookings.cancelled = bookings;
        this.bookings.set(current_bookings);
        this.loading_cancelled.set(false);
        this.error_cancelled.set(None);
    }

    /// **Set loading state for upcoming bookings**
    pub fn set_loading_upcoming(loading: bool) {
        Self::get().loading_upcoming.set(loading);
    }

    /// **Set loading state for completed bookings**
    pub fn set_loading_completed(loading: bool) {
        Self::get().loading_completed.set(loading);
    }

    /// **Set loading state for cancelled bookings**
    pub fn set_loading_cancelled(loading: bool) {
        Self::get().loading_cancelled.set(loading);
    }

    /// **Set error for upcoming bookings**
    pub fn set_error_upcoming(error: Option<String>) {
        let this = Self::get();
        this.error_upcoming.set(error);
        this.loading_upcoming.set(false);
    }

    /// **Set error for completed bookings**
    pub fn set_error_completed(error: Option<String>) {
        let this = Self::get();
        this.error_completed.set(error);
        this.loading_completed.set(false);
    }

    /// **Set error for cancelled bookings**
    pub fn set_error_cancelled(error: Option<String>) {
        let this = Self::get();
        this.error_cancelled.set(error);
        this.loading_cancelled.set(false);
    }

    /// **Set selected booking**
    pub fn set_selected_booking(booking: Option<HotelBooking>) {
        Self::get().selected_booking.set(booking);
    }

    // **API Methods (with TODOs)**

    /// **Fetch upcoming bookings from backend**
    pub async fn fetch_upcoming_bookings() {
        Self::set_loading_upcoming(true);
        
        // TODO: Implement actual API call to fetch upcoming bookings
        // Example:
        // match backend_api::fetch_upcoming_bookings().await {
        //     Ok(bookings) => {
        //         Self::set_upcoming_bookings(bookings);
        //     }
        //     Err(err) => {
        //         Self::set_error_upcoming(Some(err.to_string()));
        //     }
        // }
        
        // For now, simulate loading and set empty bookings
        Self::set_upcoming_bookings(vec![]);
    }

    /// **Fetch completed bookings from backend**
    pub async fn fetch_completed_bookings() {
        Self::set_loading_completed(true);
        
        // TODO: Implement actual API call to fetch completed bookings
        // Similar to fetch_upcoming_bookings but for completed status
        
        Self::set_completed_bookings(vec![]);
    }

    /// **Fetch cancelled bookings from backend**
    pub async fn fetch_cancelled_bookings() {
        Self::set_loading_cancelled(true);
        
        // TODO: Implement actual API call to fetch cancelled bookings
        // Similar to fetch_upcoming_bookings but for cancelled status
        
        Self::set_cancelled_bookings(vec![]);
    }

    /// **Fetch all bookings**
    pub async fn fetch_all_bookings() {
        // TODO: Optimize this to make a single API call that returns all bookings
        // organized by status, rather than 3 separate calls
        
        Self::fetch_upcoming_bookings().await;
        Self::fetch_completed_bookings().await;
        Self::fetch_cancelled_bookings().await;
    }

    /// **Cancel a booking**
    pub async fn cancel_booking(booking_id: String) -> Result<(), String> {
        // TODO: Implement booking cancellation API call
        // match backend_api::cancel_booking(booking_id).await {
        //     Ok(_) => {
        //         // Refresh bookings after cancellation
        //         Self::fetch_all_bookings().await;
        //         Ok(())
        //     }
        //     Err(err) => Err(err.to_string())
        // }
        
        Err("Not implemented yet".to_string())
    }

    // **Utility Methods**

    /// **Check if any bookings are loading**
    pub fn is_any_loading() -> Signal<bool> {
        Signal::derive(move || {
            let this = Self::get();
            this.loading_upcoming.get() || this.loading_completed.get() || this.loading_cancelled.get()
        })
    }

    /// **Get total bookings count**
    pub fn get_total_bookings_count() -> Signal<usize> {
        Signal::derive(move || {
            let bookings = Self::get().bookings.get();
            bookings.upcoming.len() + bookings.completed.len() + bookings.cancelled.len()
        })
    }

    /// **Check if user has any bookings**
    pub fn has_any_bookings() -> Signal<bool> {
        Signal::derive(move || {
            Self::get_total_bookings_count().get() > 0
        })
    }

    /// **Refresh all bookings**
    pub async fn refresh_bookings() {
        Self::fetch_all_bookings().await;
    }

    /// **Debug: Log current state (only in debug builds)**
    #[cfg(feature = "debug_log")]
    pub fn debug_log_state() {
        let this = Self::get();
        let bookings = this.bookings.get_untracked();
        crate::log!(
            "UIBookingsState Debug - Upcoming: {}, Completed: {}, Cancelled: {}",
            bookings.upcoming.len(),
            bookings.completed.len(),
            bookings.cancelled.len()
        );
    }
}