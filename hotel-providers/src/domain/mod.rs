//! Domain module - Provider-agnostic types for hotel operations
//!
//! These types represent the core business domain and are independent of any specific provider.

mod booking_types;
mod hotel_search_types;

pub use booking_types::*;
pub use hotel_search_types::*;
