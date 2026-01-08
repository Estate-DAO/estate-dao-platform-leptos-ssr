use serde::{Deserialize, Serialize};
use thiserror::Error;

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ServiceBookingStatus {
    Draft,
    RoomBlocked,
    PendingPayment,
    Confirmed,
    Failed,
    Cancelled,
}

#[derive(Error, Debug)]
pub enum BookingError {
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Provider error: {0}")]
    ProviderError(String),
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Backend error: {0}")]
    BackendError(String),
}

impl BookingError {
    pub fn user_message(&self) -> String {
        match self {
            BookingError::ValidationError(msg) => format!("Please check your input: {}", msg),
            BookingError::NetworkError(_) => {
                "We are having trouble connecting to our services. Please try again.".to_string()
            }
            BookingError::BackendError(_) => {
                "There was an issue processing your booking with our backend.".to_string()
            }
            BookingError::ProviderError(_) => {
                "There was an issue with the hotel provider.".to_string()
            }
            BookingError::SerializationError(_) | BookingError::InternalError(_) => {
                "An internal error occurred.".to_string()
            }
        }
    }

    pub fn technical_details(&self) -> String {
        self.to_string()
    }

    pub fn category(&self) -> &'static str {
        match self {
            BookingError::ValidationError(_) => "Validation",
            BookingError::NetworkError(_) => "Network",
            BookingError::BackendError(_) => "Backend",
            BookingError::ProviderError(_) => "Provider",
            BookingError::SerializationError(_) => "Serialization",
            BookingError::InternalError(_) => "Internal",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BookingServiceConfig {
    pub provider_timeout_seconds: u64,
}
