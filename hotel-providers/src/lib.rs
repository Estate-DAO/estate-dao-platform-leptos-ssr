//! Hotel Providers Crate
//!
//! This crate provides a unified interface for hotel booking providers with fallback support.
//!
//! # Architecture
//!
//! - **Domain**: Provider-agnostic types for hotel search, booking, etc.
//! - **Ports**: Traits defining the provider interface (`HotelProviderPort`, `PlaceProviderPort`)
//! - **Adapters**: Concrete implementations (LiteAPI, Mock, etc.)
//! - **Registry**: Provider registration and composite provider with fallback

pub mod domain;
pub mod ports;

#[cfg(any(feature = "liteapi", feature = "mock"))]
pub mod adapters;

mod composite;
mod registry;

pub use composite::{CompositeHotelProvider, CompositePlaceProvider, FallbackStrategy};
pub use ports::{HotelProviderPort, PlaceProviderPort, ProviderError, ProviderErrorKind};
pub use registry::{ProviderRegistry, ProviderRegistryBuilder};
