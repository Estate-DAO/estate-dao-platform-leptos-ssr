//! Shared domain types and port traits for hotel booking services
//!
//! This crate contains provider-agnostic domain types and interface
//! contracts (ports) used by both the SSR application and hotel-providers crate.

mod booking_types;
pub mod grouped_rooms;
mod hotel_search_types;
pub mod ports;

pub use booking_types::*;
pub use grouped_rooms::*;
pub use hotel_search_types::*;
