//! Provider port types - re-exports from hotel-types
//!
//! This module re-exports provider error types from hotel-types
//! for use throughout the SSR application.

// Re-export core types from hotel-types
pub use hotel_types::ports::{
    ProviderError, ProviderErrorDetails, ProviderErrorKind, ProviderSteps,
};
