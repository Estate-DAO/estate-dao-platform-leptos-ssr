use crate::ports::hotel_provider_port::ProviderError;
use serde::{Deserialize, Serialize};

/// Service-specific types for BookingService
/// These types wrap domain types with additional service layer concerns

#[derive(Debug, Clone)]
pub enum BookingError {
    ValidationError(String),
    ProviderError(ProviderError),
    BackendError(String),
    NetworkError(String),
    SerializationError(String),
}

impl From<ProviderError> for BookingError {
    fn from(error: ProviderError) -> Self {
        BookingError::ProviderError(error)
    }
}

impl From<serde_json::Error> for BookingError {
    fn from(error: serde_json::Error) -> Self {
        BookingError::SerializationError(error.to_string())
    }
}

impl std::fmt::Display for BookingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BookingError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            BookingError::ProviderError(err) => write!(f, "Provider error: {}", err),
            BookingError::BackendError(msg) => write!(f, "Backend error: {}", msg),
            BookingError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            BookingError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for BookingError {}

impl BookingError {
    /// Get user-friendly error message for display in UI
    pub fn user_message(&self) -> String {
        match self {
            BookingError::ValidationError(msg) => format!("Please check your information: {}", msg),
            BookingError::ProviderError(_) => {
                "Unable to process your booking request. Please try again.".to_string()
            }
            BookingError::BackendError(_) => {
                "Unable to save your booking. Please try again.".to_string()
            }
            BookingError::NetworkError(_) => {
                "Network connection issue. Please check your connection and try again.".to_string()
            }
            BookingError::SerializationError(_) => {
                "Data processing error. Please try again.".to_string()
            }
        }
    }

    /// Get technical error details for logging
    pub fn technical_details(&self) -> String {
        match self {
            BookingError::ValidationError(msg) => format!("Validation: {}", msg),
            BookingError::ProviderError(err) => format!("Provider: {}", err),
            BookingError::BackendError(msg) => format!("Backend: {}", msg),
            BookingError::NetworkError(msg) => format!("Network: {}", msg),
            BookingError::SerializationError(msg) => format!("Serialization: {}", msg),
        }
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            BookingError::ValidationError(_) => false, // Don't retry validation errors
            BookingError::ProviderError(_) => true,    // May succeed on retry
            BookingError::BackendError(_) => true,     // May succeed on retry
            BookingError::NetworkError(_) => true,     // Network issues are retryable
            BookingError::SerializationError(_) => false, // Don't retry serialization errors
        }
    }

    /// Get error category for analytics/monitoring
    pub fn category(&self) -> &'static str {
        match self {
            BookingError::ValidationError(_) => "validation",
            BookingError::ProviderError(_) => "provider",
            BookingError::BackendError(_) => "backend",
            BookingError::NetworkError(_) => "network",
            BookingError::SerializationError(_) => "serialization",
        }
    }
}

/// Service-level booking data that combines domain and backend concerns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceBookingData {
    pub booking_id: String,
    pub email: String,
    pub app_reference: String,
    pub block_room_id: Option<String>,
    pub payment_amount: f64,
    pub payment_currency: String,
    pub status: ServiceBookingStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceBookingStatus {
    Draft,
    RoomBlocked,
    PaymentPending,
    Confirmed,
    Failed,
    Cancelled,
}

/// Configuration for BookingService
#[derive(Debug, Clone)]
pub struct BookingServiceConfig {
    pub max_retry_attempts: u32,
    pub timeout_seconds: u64,
    pub auto_save_to_backend: bool,
}

impl Default for BookingServiceConfig {
    fn default() -> Self {
        Self {
            max_retry_attempts: 3,
            timeout_seconds: 30,
            auto_save_to_backend: true,
        }
    }
}
