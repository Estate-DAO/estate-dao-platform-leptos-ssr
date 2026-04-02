//! Provider ports module - re-exports from hotel-types
//!
//! This module re-exports the port traits and types from hotel-types
//! for backwards compatibility.

pub use hotel_types::ports::{
    HotelProviderPort, PlaceProviderPort, ProviderError, ProviderErrorDetails, ProviderErrorKind,
    ProviderKeys, ProviderNames, ProviderSteps, UISearchFilters,
};
