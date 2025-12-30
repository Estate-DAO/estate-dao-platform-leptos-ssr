//! Adapters module - provider implementations
//!
//! Contains concrete implementations of the provider ports.

#[cfg(feature = "liteapi")]
pub mod liteapi;

#[cfg(feature = "mock")]
pub mod mock;
