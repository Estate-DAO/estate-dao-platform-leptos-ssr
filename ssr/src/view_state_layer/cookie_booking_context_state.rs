use crate::utils::{
    app_reference::BookingId, booking_id::PaymentIdentifiers, cookie_storage::CookieBookingStorage,
};
use crate::view_state_layer::GlobalStateForLeptos;
use leptos::*;

/// **Cookie-based Booking Context State**
///
/// **Purpose**: Loads and manages booking context information from client-side cookies
/// using leptos_use, enabling SSR compatibility and better reliability.
///
/// **Benefits over localStorage version**:
/// - Works in SSR environments
/// - No hydration issues
/// - Cross-tab persistence
/// - Simple client-side API
/// - No server functions needed
///
/// **Pattern**: Follows established GlobalStateForLeptos pattern for reactive state management
/// **Integration**: Drop-in replacement for the localStorage-based BookingContextState

#[derive(Debug, Clone, Default)]
pub struct CookieBookingContextState {
    // **Core Booking Data from cookies**
    pub booking_id: RwSignal<Option<BookingId>>,
    pub app_reference: RwSignal<Option<String>>,

    // **Derived Data** - Computed from booking_id
    pub order_id: RwSignal<Option<String>>,
    pub email: RwSignal<Option<String>>,

    // **State Management**
    pub is_loaded: RwSignal<bool>,
    pub load_error: RwSignal<Option<String>>,
}

impl GlobalStateForLeptos for CookieBookingContextState {}

impl CookieBookingContextState {
    /// **Initialize by loading data from cookies**
    /// This loads data from cookies synchronously using leptos_use
    pub fn initialize() {
        let this = Self::get();

        // Reset state
        this.is_loaded.set(false);
        this.load_error.set(None);

        // Load data from cookies synchronously
        let cookie_booking_id = CookieBookingStorage::get_booking_id_untracked();
        Self::load_booking_data(cookie_booking_id);
    }

    /// **Initialize with explicitly provided data**
    /// Useful for testing or when data is available from other sources
    pub fn initialize_with_data(booking_id: Option<BookingId>, app_reference: Option<String>) {
        let this = Self::get();

        // Reset state
        this.is_loaded.set(false);
        this.load_error.set(None);

        // Process the provided data immediately
        Self::load_booking_data(booking_id.clone());

        // If app_reference is provided but booking_id is None, try to derive booking_id
        if booking_id.is_none() && app_reference.is_some() {
            // Handle legacy case where only app_reference is available
            this.app_reference.set(app_reference.clone());

            if let Some(app_ref) = &app_reference {
                // Try to parse as order_id format first
                if let Some(fallback_booking_id) = BookingId::from_order_id(app_ref) {
                    this.email.set(Some(fallback_booking_id.email));
                    this.order_id.set(Some(app_ref.clone()));
                } else {
                    // Last resort: use app_ref as order_id with empty email
                    this.order_id.set(Some(app_ref.clone()));
                    this.email.set(Some("".to_string()));
                }
            }
        }

        this.is_loaded.set(true);
    }

    /// **Internal method to process BookingId data**
    fn load_booking_data(booking_id: Option<BookingId>) {
        let this = Self::get();

        // Set the booking_id
        this.booking_id.set(booking_id.clone());

        // Compute derived values
        if let Some(booking_id) = &booking_id {
            // Set app_reference from BookingId
            this.app_reference
                .set(Some(booking_id.app_reference.clone()));

            // Set email from BookingId
            this.email.set(Some(booking_id.email.clone()));

            // Generate order_id using PaymentIdentifiers
            let order_id = PaymentIdentifiers::order_id_from_app_reference(
                &booking_id.app_reference,
                &booking_id.email,
            );
            this.order_id.set(Some(order_id));

            leptos::logging::log!(
                "CookieBookingContextState loaded - app_reference: {}, email: {}, order_id: {}",
                booking_id.app_reference,
                booking_id.email,
                PaymentIdentifiers::order_id_from_app_reference(
                    &booking_id.app_reference,
                    &booking_id.email
                )
            );
        } else {
            // Clear derived values if no booking_id
            this.app_reference.set(None);
            this.email.set(None);
            this.order_id.set(None);
        }

        this.is_loaded.set(true);
    }

    // **Getters for reactive UI access**

    /// **Get BookingId struct**
    pub fn get_booking_id() -> Signal<Option<BookingId>> {
        Self::get().booking_id.into()
    }

    /// **Get app_reference string**
    pub fn get_app_reference() -> Signal<Option<String>> {
        Self::get().app_reference.into()
    }

    /// **Get derived order_id**
    pub fn get_order_id() -> Signal<Option<String>> {
        Self::get().order_id.into()
    }

    /// **Get email from booking context**
    pub fn get_email() -> Signal<Option<String>> {
        Self::get().email.into()
    }

    /// **Check if context is loaded**
    pub fn is_loaded() -> Signal<bool> {
        Self::get().is_loaded.into()
    }

    /// **Get load error if any**
    pub fn get_load_error() -> Signal<Option<String>> {
        Self::get().load_error.into()
    }

    // **Non-reactive getters for use in async contexts**

    /// **Get order_id non-reactively for API calls**
    pub fn get_order_id_untracked() -> Option<String> {
        Self::get().order_id.get_untracked()
    }

    /// **Get email non-reactively for API calls**
    pub fn get_email_untracked() -> Option<String> {
        Self::get().email.get_untracked()
    }

    /// **Get app_reference non-reactively**
    pub fn get_app_reference_untracked() -> Option<String> {
        Self::get().app_reference.get_untracked()
    }

    /// **Get BookingId non-reactively**
    pub fn get_booking_id_untracked() -> Option<BookingId> {
        Self::get().booking_id.get_untracked()
    }

    // **Convenience methods**

    /// **Check if booking context is available**
    pub fn has_booking_context() -> Signal<bool> {
        Signal::derive(move || {
            let this = Self::get();
            this.is_loaded.get()
                && (this.booking_id.get().is_some() || this.app_reference.get().is_some())
        })
    }

    /// **Get order_id and email as tuple (non-reactive)**
    /// Useful for API calls that need both values
    pub fn get_order_details_untracked() -> (Option<String>, Option<String>) {
        let this = Self::get();
        (this.order_id.get_untracked(), this.email.get_untracked())
    }

    /// **Get formatted guest info if available**
    pub fn get_guest_info_untracked() -> Option<(String, String)> {
        let booking_id = Self::get_booking_id_untracked()?;
        Some((booking_id.email, booking_id.app_reference))
    }

    /// **Debug: Log current state (only in debug builds)**
    #[cfg(feature = "debug_log")]
    pub fn debug_log_state() {
        let this = Self::get();
        leptos::logging::log!(
            "CookieBookingContextState Debug - BookingId: {:?}, AppRef: {:?}, OrderId: {:?}, Email: {:?}",
            this.booking_id.get_untracked(),
            this.app_reference.get_untracked(),
            this.order_id.get_untracked(),
            this.email.get_untracked()
        );
    }
}
