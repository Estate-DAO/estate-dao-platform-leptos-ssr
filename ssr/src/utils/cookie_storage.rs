use codee::string::JsonSerdeCodec;
use leptos::*;
use leptos_use::{use_cookie_with_options, UseCookieOptions};

use crate::utils::app_reference::BookingId;

/// Cookie names for booking-related data
const BOOKING_ID_COOKIE: &str = "estatedao_booking_id";

/// **Client-side Cookie-based BookingId Storage**
///
/// **Purpose**: Store and retrieve BookingId data using client-side cookies
/// with leptos_use, enabling cross-tab persistence and SSR compatibility.
///
/// **Benefits**:
/// - Works in both SSR and CSR environments
/// - Cross-tab/window persistence
/// - Simple client-side API
/// - No server functions needed
/// - Compatible with existing patterns
pub struct CookieBookingStorage;

impl CookieBookingStorage {
    /// **Get cookie domain configuration (same as OAuth system)**
    fn get_cookie_domain() -> Option<String> {
        // Use same logic as oauth.rs for consistent domain handling
        let app_url = std::env::var("APP_URL").unwrap_or_else(|_| "http://localhost:3002/".into());
        if app_url.contains("nofeebooking.com") {
            Some(".nofeebooking.com".to_string()) // Covers nofeebooking.com and www.nofeebooking.com
        } else {
            None
        }
    }

    /// **Get or create a cookie store for BookingId**
    /// Returns (getter_signal, setter)
    pub fn use_booking_id_cookie() -> (Signal<Option<BookingId>>, WriteSignal<Option<BookingId>>) {
        let mut cookie_options = UseCookieOptions::default()
            .path("/")
            .same_site(leptos_use::SameSite::Lax);

        // Apply domain configuration if needed (same as OAuth system)
        if let Some(domain) = Self::get_cookie_domain() {
            cookie_options = cookie_options.domain(&domain);
        }

        use_cookie_with_options::<BookingId, JsonSerdeCodec>(BOOKING_ID_COOKIE, cookie_options)
    }

    /// **Store BookingId in cookie**
    pub fn store_booking_id(booking_id: &BookingId) {
        let (_, set_booking_id) = Self::use_booking_id_cookie();
        set_booking_id.set(Some(booking_id.clone()));
    }

    /// **Get BookingId from cookie**
    pub fn get_booking_id() -> Signal<Option<BookingId>> {
        let (booking_id, _) = Self::use_booking_id_cookie();
        booking_id
    }

    /// **Get BookingId from cookie (non-reactive)**
    pub fn get_booking_id_untracked() -> Option<BookingId> {
        let (booking_id, _) = Self::use_booking_id_cookie();
        booking_id.get_untracked()
    }

    /// **Remove BookingId from cookie**
    pub fn remove_booking_id() {
        let (_, set_booking_id) = Self::use_booking_id_cookie();
        set_booking_id.set(None);
    }

    /// **Extract app_reference from BookingId stored in cookies**
    pub fn get_app_reference() -> Signal<Option<String>> {
        Signal::derive(move || {
            Self::get_booking_id()
                .get()
                .map(|booking_id| booking_id.app_reference)
        })
    }

    /// **Extract app_reference from BookingId stored in cookies (non-reactive)**
    pub fn get_app_reference_untracked() -> Option<String> {
        Self::get_booking_id_untracked().map(|booking_id| booking_id.app_reference)
    }

    /// **Extract email from BookingId stored in cookies**
    pub fn get_email() -> Signal<Option<String>> {
        Signal::derive(move || {
            Self::get_booking_id()
                .get()
                .map(|booking_id| booking_id.email)
        })
    }

    /// **Extract email from BookingId stored in cookies (non-reactive)**
    pub fn get_email_untracked() -> Option<String> {
        Self::get_booking_id_untracked().map(|booking_id| booking_id.email)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_booking_id_serialization() {
        let booking_id = BookingId::new(
            "test@example.com".to_string(),
            "HB1234-56789-01234".to_string(),
        );

        let serialized = serde_json::to_string(&booking_id).unwrap();
        let deserialized: BookingId = serde_json::from_str(&serialized).unwrap();

        assert_eq!(booking_id.email, deserialized.email);
        assert_eq!(booking_id.app_reference, deserialized.app_reference);
    }
}
