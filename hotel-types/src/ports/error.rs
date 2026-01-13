//! Provider error types

use std::fmt;
use std::sync::Arc;

/// Steps in the provider workflow where errors can occur
#[derive(Debug, Clone, PartialEq)]
pub enum ProviderSteps {
    PlaceSearch,
    PlaceDetails,
    HotelSearch,
    HotelDetails,
    HotelRate,
    HotelBlockRoom,
    HotelBookRoom,
    GetBookingDetails,
}

/// Categories of provider errors for fallback decision making
#[derive(Debug, Clone, PartialEq)]
pub enum ProviderErrorKind {
    /// Network-related errors (timeouts, connection failures)
    Network,
    /// Provider service is unavailable
    ServiceUnavailable,
    /// Authentication/authorization failures
    Auth,
    /// Rate limiting
    RateLimited,
    /// Invalid request parameters
    InvalidRequest,
    /// Resource not found
    NotFound,
    /// Provider returned unexpected data
    ParseError,
    /// Internal provider error
    InternalError,
    /// Other/unknown errors
    Other,
}

/// Details about a provider error
#[derive(Debug, Clone)]
pub struct ProviderErrorDetails {
    pub provider_name: String,
    pub error_kind: ProviderErrorKind,
    pub error_step: ProviderSteps,
    pub message: String,
    pub retryable: bool,
}

/// Provider error wrapper with Arc for cheap cloning
#[derive(Debug, Clone)]
pub struct ProviderError(pub Arc<ProviderErrorDetails>);

impl ProviderError {
    /// Create a new provider error
    pub fn new(
        provider_name: impl Into<String>,
        error_kind: ProviderErrorKind,
        error_step: ProviderSteps,
        message: impl Into<String>,
    ) -> Self {
        let retryable = matches!(
            error_kind,
            ProviderErrorKind::Network
                | ProviderErrorKind::ServiceUnavailable
                | ProviderErrorKind::RateLimited
        );

        ProviderError(Arc::new(ProviderErrorDetails {
            provider_name: provider_name.into(),
            error_kind,
            error_step,
            message: message.into(),
            retryable,
        }))
    }

    /// Create a network error
    pub fn network(
        provider_name: impl Into<String>,
        error_step: ProviderSteps,
        message: impl Into<String>,
    ) -> Self {
        Self::new(
            provider_name,
            ProviderErrorKind::Network,
            error_step,
            message,
        )
    }

    /// Create a service unavailable error
    pub fn service_unavailable(
        provider_name: impl Into<String>,
        error_step: ProviderSteps,
        message: impl Into<String>,
    ) -> Self {
        Self::new(
            provider_name,
            ProviderErrorKind::ServiceUnavailable,
            error_step,
            message,
        )
    }

    /// Create a not found error
    pub fn not_found(
        provider_name: impl Into<String>,
        error_step: ProviderSteps,
        message: impl Into<String>,
    ) -> Self {
        Self::new(
            provider_name,
            ProviderErrorKind::NotFound,
            error_step,
            message,
        )
    }

    /// Create a parse error
    pub fn parse_error(
        provider_name: impl Into<String>,
        error_step: ProviderSteps,
        message: impl Into<String>,
    ) -> Self {
        Self::new(
            provider_name,
            ProviderErrorKind::ParseError,
            error_step,
            message,
        )
    }

    /// Create a generic other error
    pub fn other(
        provider_name: impl Into<String>,
        error_step: ProviderSteps,
        message: impl Into<String>,
    ) -> Self {
        Self::new(provider_name, ProviderErrorKind::Other, error_step, message)
    }

    /// Check if this error should trigger a fallback to another provider
    pub fn should_fallback(&self) -> bool {
        self.0.retryable
    }

    /// Get the error kind
    pub fn kind(&self) -> &ProviderErrorKind {
        &self.0.error_kind
    }

    /// Get the provider name
    pub fn provider_name(&self) -> &str {
        &self.0.provider_name
    }

    /// Get the error step
    pub fn step(&self) -> &ProviderSteps {
        &self.0.error_step
    }
}

impl fmt::Display for ProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let error_message = match self.0.error_step {
            ProviderSteps::PlaceSearch => {
                "There was a problem searching for the place. Please try again."
            }
            ProviderSteps::PlaceDetails => {
                "Could not retrieve details for the selected place. Please try again."
            }
            ProviderSteps::HotelSearch => {
                "There was a problem searching for hotels. Please try again."
            }
            ProviderSteps::HotelDetails => {
                "Could not retrieve details for the selected hotel. Please try again."
            }
            ProviderSteps::HotelRate => {
                "This hotel is fully booked for your selected dates. Please choose different dates or another hotel."
            }
            ProviderSteps::HotelBlockRoom => {
                "We were unable to reserve the room. Please try again."
            }
            ProviderSteps::HotelBookRoom => {
                "There was a problem confirming your booking. Please try again."
            }
            ProviderSteps::GetBookingDetails => {
                "Could not retrieve booking details. Please try again."
            }
        };
        write!(f, "{}", error_message)
    }
}

impl std::error::Error for ProviderError {}

impl PartialEq for ProviderError {
    fn eq(&self, other: &Self) -> bool {
        self.0.provider_name == other.0.provider_name
            && self.0.error_kind == other.0.error_kind
            && self.0.error_step == other.0.error_step
    }
}
