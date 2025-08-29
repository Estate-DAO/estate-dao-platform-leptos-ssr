use crate::ports::hotel_provider_port::ProviderError;
use thiserror::Error;

/// Provider-agnostic domain errors for hotel search operations
#[derive(Debug, Error, Clone, PartialEq)]
pub enum DomainHotelSearchError {
    #[error("The search criteria provided were invalid: {0}")]
    InvalidCriteria(String),

    #[error("The hotel provider failed or is unavailable: {0}")]
    ProviderFailure(ProviderError), // Will hold the translated adapter error message

    #[error("No hotels were found for the given criteria.")]
    NoResults,

    #[error("An unknown internal error occurred.")]
    Unknown,
}

/// Domain errors for hotel details operations  
#[derive(Debug, Error, Clone, PartialEq)]
pub enum DomainHotelDetailsError {
    #[error("Invalid hotel token provided: {0}")]
    InvalidToken(String),

    #[error("Hotel details not found for the given token")]
    NotFound,

    #[error("The hotel provider failed or is unavailable: {0}")]
    ProviderFailure(ProviderError),

    #[error("An unknown internal error occurred.")]
    Unknown,
}

/// Domain errors for booking operations
#[derive(Debug, Error, Clone, PartialEq)]
pub enum DomainBookingError {
    #[error("Invalid booking criteria: {0}")]
    InvalidCriteria(String),

    #[error("Room is no longer available")]
    RoomUnavailable,

    #[error("Payment processing failed: {0}")]
    PaymentFailed(String),

    #[error("The booking provider failed: {0}")]
    ProviderFailure(ProviderError),

    #[error("An unknown internal error occurred.")]
    Unknown,
}
