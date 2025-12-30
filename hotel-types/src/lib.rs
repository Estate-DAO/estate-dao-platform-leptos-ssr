//! Shared domain types for hotel booking services
//!
//! This crate contains provider-agnostic domain types used by both
//! the SSR application and hotel-providers crate.

mod booking_types;
mod hotel_search_types;

pub use booking_types::*;
pub use hotel_search_types::*;
