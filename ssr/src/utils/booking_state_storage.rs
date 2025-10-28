use chrono::{DateTime, Utc};
use codee::string::JsonSerdeCodec;
use leptos::*;
use leptos_use::{use_cookie_with_options, UseCookieOptions};
use serde::{Deserialize, Serialize};

use crate::component::SelectedDateRange;
use crate::domain::{DomainHotelDetails, DomainRoomData};
use crate::view_state_layer::ui_block_room::{AdultDetail, ChildDetail, RoomSelectionSummary};

/// Cookie name for booking state persistence
const BOOKING_STATE_COOKIE: &str = "estatedao_booking_state";

/// Persistent booking state data that survives browser navigation
/// This contains all essential information needed to restore the booking flow
/// after returning from external payment providers like Stripe
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PersistedBookingState {
    // Hotel and booking context
    pub hotel_context: Option<DomainHotelDetails>,
    pub date_range: Option<SelectedDateRange>,
    pub room_selection_summary: Vec<RoomSelectionSummary>,

    // Guest information
    pub adults: Vec<AdultDetail>,
    pub children: Vec<ChildDetail>,
    pub adults_count: usize,
    pub children_count: usize,

    // Pricing information
    pub room_price: f64,
    pub total_price: f64,
    pub num_nights: u32,

    // Block room response data (if available)
    pub block_room_id: Option<String>,
    pub block_room_called: bool,

    // Form state
    pub terms_accepted: bool,

    // Timestamp for expiration
    pub created_at: i64, // Unix timestamp in seconds
}

impl Default for PersistedBookingState {
    fn default() -> Self {
        Self {
            hotel_context: None,
            date_range: None,
            room_selection_summary: vec![],
            adults: vec![],
            children: vec![],
            adults_count: 0,
            children_count: 0,
            room_price: 0.0,
            total_price: 0.0,
            num_nights: 0,
            block_room_id: None,
            block_room_called: false,
            terms_accepted: false,
            created_at: Utc::now().timestamp(),
        }
    }
}

impl PersistedBookingState {
    /// Check if the persisted state is expired (older than 8 hours)
    pub fn is_expired(&self) -> bool {
        let now = Utc::now().timestamp();
        let eight_hours = 8 * 60 * 60; // 8 hours in seconds
        now > self.created_at + eight_hours
    }

    /// Check if the state has essential booking information
    pub fn has_essential_data(&self) -> bool {
        self.hotel_context.is_some()
            && !self.room_selection_summary.is_empty()
            && !self.adults.is_empty()
            && self.total_price > 0.0
    }

    /// Update the timestamp to current time
    pub fn refresh_timestamp(&mut self) {
        self.created_at = Utc::now().timestamp();
    }
}

/// Cookie-based booking state storage for persistence across navigation
pub struct BookingStateStorage;

impl BookingStateStorage {
    /// Get cookie domain configuration (environment-aware)
    fn get_cookie_domain() -> Option<String> {
        // Use the same domain logic as CookieBookingStorage for consistency
        use crate::api::consts::{get_app_domain_with_dot, APP_URL};

        let domain_with_dot = get_app_domain_with_dot();

        // Skip domain setting for localhost to avoid cookie issues
        if domain_with_dot.contains("localhost") || domain_with_dot.contains("127.0.0.1") {
            None
        } else {
            Some(domain_with_dot)
        }
    }

    /// Get or create a cookie store for booking state
    fn use_booking_state_cookie() -> (
        Signal<Option<PersistedBookingState>>,
        WriteSignal<Option<PersistedBookingState>>,
    ) {
        let mut cookie_options = UseCookieOptions::default()
            .path("/")
            .same_site(leptos_use::SameSite::Lax);

        // Apply domain configuration if needed (same as CookieBookingStorage)
        if let Some(domain) = Self::get_cookie_domain() {
            cookie_options = cookie_options.domain(&domain);
        }

        use_cookie_with_options::<PersistedBookingState, JsonSerdeCodec>(
            BOOKING_STATE_COOKIE,
            cookie_options,
        )
    }

    /// Store booking state in cookie
    pub fn store_booking_state(state: &PersistedBookingState) {
        let mut state_with_timestamp = state.clone();
        state_with_timestamp.refresh_timestamp();

        leptos::logging::log!(
            "BookingStateStorage: Attempting to store booking state - hotel: {}, rooms: {}, total: ${:.2}, timestamp: {}",
            state_with_timestamp
                .hotel_context
                .as_ref()
                .map(|h| &h.hotel_name)
                .unwrap_or(&"None".to_string()),
            state_with_timestamp.room_selection_summary.len(),
            state_with_timestamp.total_price,
            state_with_timestamp.created_at
        );

        // Get the cookie getter/setter once
        let (getter, setter) = Self::use_booking_state_cookie();

        // Use setter to update the cookie
        setter.set(Some(state_with_timestamp.clone()));

        // Verify storage immediately after setting using the same getter
        if let Some(stored) = getter.get_untracked() {
            leptos::logging::log!(
                "BookingStateStorage: Successfully verified stored state - timestamp: {}, hotel: {}",
                stored.created_at,
                stored.hotel_context.as_ref().map(|h| &h.hotel_name).unwrap_or(&"None".to_string())
            );
        } else {
            leptos::logging::log!("BookingStateStorage: WARNING - Failed to verify stored state immediately after setting");
        }
    }

    /// Get booking state from cookie
    pub fn get_booking_state() -> Signal<Option<PersistedBookingState>> {
        let (getter, _) = Self::use_booking_state_cookie();
        getter
    }

    /// Get booking state from cookie (non-reactive)
    pub fn get_booking_state_untracked() -> Option<PersistedBookingState> {
        leptos::logging::log!(
            "BookingStateStorage: Attempting to retrieve booking state from cookie"
        );

        let (getter, _) = Self::use_booking_state_cookie();
        let stored_state = getter.get_untracked();

        match stored_state {
            Some(ref state) => {
                leptos::logging::log!(
                    "BookingStateStorage: Found stored state - timestamp: {}, hotel: {}, is_expired: {}",
                    state.created_at,
                    state.hotel_context.as_ref().map(|h| &h.hotel_name).unwrap_or(&"None".to_string()),
                    state.is_expired()
                );

                if state.is_expired() {
                    leptos::logging::log!(
                        "BookingStateStorage: Stored state is expired, removing and returning None"
                    );
                    Self::remove_booking_state();
                    return None;
                }

                stored_state
            }
            None => {
                leptos::logging::log!("BookingStateStorage: No stored state found in cookie");
                None
            }
        }
    }

    /// Remove booking state from cookie
    pub fn remove_booking_state() {
        let (_, setter) = Self::use_booking_state_cookie();
        setter.set(None);
        leptos::logging::log!("BookingStateStorage: Removed expired/invalid booking state");
    }

    /// Check if valid booking state exists
    pub fn has_valid_booking_state() -> bool {
        if let Some(state) = Self::get_booking_state_untracked() {
            !state.is_expired() && state.has_essential_data()
        } else {
            false
        }
    }

    /// Create persisted state from current UI state
    pub fn create_from_ui_state() -> Option<PersistedBookingState> {
        use crate::view_state_layer::ui_block_room::BlockRoomUIState;
        use crate::view_state_layer::ui_search_state::UISearchCtx;

        leptos::logging::log!("BookingStateStorage: create_from_ui_state() called");

        // Try to get UI contexts
        let block_room_state =
            match std::panic::catch_unwind(|| BlockRoomUIState::from_leptos_context()) {
                Ok(state) => {
                    leptos::logging::log!(
                        "BookingStateStorage: Successfully got BlockRoomUIState context"
                    );
                    state
                }
                Err(_) => {
                    leptos::logging::log!(
                        "BookingStateStorage: BlockRoomUIState context not available"
                    );
                    return None;
                }
            };

        let ui_search_ctx = match std::panic::catch_unwind(|| expect_context::<UISearchCtx>()) {
            Ok(ctx) => {
                leptos::logging::log!("BookingStateStorage: Successfully got UISearchCtx context");
                ctx
            }
            Err(_) => {
                leptos::logging::log!("BookingStateStorage: UISearchCtx context not available");
                return None;
            }
        };

        // Extract data from UI state (untracked to avoid reactivity issues)
        let adults = block_room_state.adults.get_untracked();
        let children = block_room_state.children.get_untracked();
        let hotel_context = block_room_state.hotel_context.get_untracked();
        let room_selection_summary = block_room_state.room_selection_summary.get_untracked();
        let room_price = block_room_state.room_price.get_untracked();
        let total_price = block_room_state.total_price.get_untracked();
        let num_nights = block_room_state.num_nights.get_untracked();
        let block_room_id = block_room_state.block_room_id.get_untracked();
        let block_room_called = block_room_state.block_room_called.get_untracked();
        let terms_accepted = block_room_state.terms_accepted.get_untracked();

        // Get date range from search context
        let date_range = ui_search_ctx.date_range.get_untracked();
        let adults_count = ui_search_ctx.guests.adults.get_untracked() as usize;
        let children_count = ui_search_ctx.guests.children.get_untracked() as usize;

        // Validate essential data
        leptos::logging::log!(
            "BookingStateStorage: Validating data - adults: {}, hotel_context: {}, room_selection: {}, total_price: {}",
            adults.len(),
            hotel_context.is_some(),
            room_selection_summary.len(),
            total_price
        );

        if adults.is_empty() || hotel_context.is_none() || room_selection_summary.is_empty() {
            leptos::logging::log!(
                "BookingStateStorage: Essential data missing, cannot create persisted state"
            );
            return None;
        }

        let mut state = PersistedBookingState {
            hotel_context,
            date_range: Some(date_range),
            room_selection_summary,
            adults: adults.clone(),
            children: children.clone(),
            adults_count,
            children_count,
            room_price,
            total_price,
            num_nights,
            block_room_id,
            block_room_called,
            terms_accepted,
            created_at: 0, // Will be set by refresh_timestamp
        };

        state.refresh_timestamp();

        Some(state)
    }

    /// Store current UI state (wrapper for easier API)
    pub fn store_current_state() {
        leptos::logging::log!("BookingStateStorage: store_current_state() called");

        if let Some(state) = Self::create_from_ui_state() {
            leptos::logging::log!(
                "BookingStateStorage: Created state from UI - hotel: {}, adults: {}, total: ${:.2}",
                state
                    .hotel_context
                    .as_ref()
                    .map(|h| &h.hotel_name)
                    .unwrap_or(&"None".to_string()),
                state.adults.len(),
                state.total_price
            );
            Self::store_booking_state(&state);
        } else {
            leptos::logging::log!(
                "BookingStateStorage: Cannot store current state - failed to create from UI state"
            );
        }
    }

    /// Restore booking state from cookie (wrapper for easier API)
    pub fn restore_state() -> Option<PersistedBookingState> {
        Self::get_booking_state_untracked()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_persisted_state_default() {
        let state = PersistedBookingState::default();
        assert!(!state.has_essential_data());
        assert_eq!(state.total_price, 0.0);
        assert_eq!(state.adults.len(), 0);
    }

    #[test]
    fn test_persisted_state_expiration() {
        let mut state = PersistedBookingState::default();
        state.created_at = 0; // Very old timestamp
        assert!(state.is_expired());

        // Fresh timestamp should not be expired
        state.refresh_timestamp();
        assert!(!state.is_expired());
    }

    #[test]
    fn test_essential_data_check() {
        let mut state = PersistedBookingState::default();
        assert!(!state.has_essential_data());

        // Add minimal essential data
        state.hotel_context = Some(DomainHotelDetails::default());
        state.room_selection_summary.push(RoomSelectionSummary {
            room_id: "test".to_string(),
            room_name: "Test Room".to_string(),
            quantity: 1,
            price_per_night: 100.0,
            room_data: DomainRoomData::default(),
        });
        state.adults.push(AdultDetail::default());
        state.total_price = 100.0;

        assert!(state.has_essential_data());
    }
}
