//! Provider ports module - defines the interface for hotel providers
//!
//! This module contains the traits that providers must implement,
//! as well as error types for provider operations.

mod error;
mod hotel_port;
mod place_port;

pub use error::{ProviderError, ProviderErrorKind, ProviderSteps};
pub use hotel_port::{HotelProviderPort, UISearchFilters};
pub use place_port::PlaceProviderPort;
