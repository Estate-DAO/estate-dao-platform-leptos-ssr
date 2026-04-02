//! Provider ports module - defines the interface for hotel providers
//!
//! This module contains the traits that providers must implement,
//! as well as error types for provider operations.

mod error;
mod hotel_port;
mod place_port;

pub use error::{ProviderError, ProviderErrorDetails, ProviderErrorKind, ProviderSteps};
pub use hotel_port::{HotelProviderPort, UISearchFilters};
pub use place_port::PlaceProviderPort;

#[allow(non_upper_case_globals)]
pub struct ProviderKeys;

impl ProviderKeys {
    #[allow(non_upper_case_globals)]
    pub const LiteApi: &'static str = "liteapi";
    #[allow(non_upper_case_globals)]
    pub const Booking: &'static str = "booking";
    #[allow(non_upper_case_globals)]
    pub const Amadeus: &'static str = "amadeus";
    #[allow(non_upper_case_globals)]
    pub const Composite: &'static str = "composite";
    #[allow(non_upper_case_globals)]
    pub const Mock: &'static str = "mock";
}

#[allow(non_upper_case_globals)]
pub struct ProviderNames;

impl ProviderNames {
    #[allow(non_upper_case_globals)]
    pub const LiteApi: &'static str = "LiteAPI";
    #[allow(non_upper_case_globals)]
    pub const Booking: &'static str = "Booking.com";
}
