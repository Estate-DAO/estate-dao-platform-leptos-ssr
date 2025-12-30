//! Core travel types - Re-exports from hotel-types plus SSR-specific extensions
//!
//! Domain types are now defined in the shared `hotel-types` crate.
//! This module re-exports them and adds SSR-specific error types.

// Re-export all domain types from the shared hotel-types crate
pub use hotel_types::*;

// SSR-specific error types
mod errors;
pub use errors::*;
