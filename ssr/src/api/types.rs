// use thiserror::Error;

// #[derive(Error, Debug)]
// pub enum ApiError {
//     #[error("Response Status not 200 - {0}")]
//     ResponseNotOK(String),

//     #[error("Provab response error - Got: {0}")]
//     ResponseError(String),

//     #[error("Decompression failed - {0}")]
//     DecompressionFailed(String),

//     #[error("HTTP request failed - {0}")]
//     RequestFailed(#[from] reqwest::Error),

//     #[error("JSON parsing failed `{0}`")]
//     JsonParseFailed(String),

//     #[error("Invalid header Value")]
//     InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),

//     #[error("Invalid header Name")]
//     InvalidHeaderName(#[from] reqwest::header::InvalidHeaderName),

//     #[error("Other error: {0}")]
//     Other(String),
// }

// pub type ApiClientResult<T> = anyhow::Result<T, ApiError>;
// // pub type ApiClientResult<T> = error_stack::Result<T, ApiError>;

use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum ApiError {
    #[error("Response Status not 200 - {0}")]
    ResponseNotOK(String),

    #[error("Provider response error - Got: {0}")]
    ResponseError(String),

    #[error("Decompression failed - {0}")]
    DecompressionFailed(String),

    #[error("HTTP request failed - {0}")]
    RequestFailed(#[from] Arc<reqwest::Error>),

    #[error("JSON parsing failed `{0}`")]
    JsonParseFailed(String),

    #[error("Invalid header Value")]
    InvalidHeaderValue(#[from] Arc<reqwest::header::InvalidHeaderValue>),

    #[error("Invalid header Name")]
    InvalidHeaderName(#[from] Arc<reqwest::header::InvalidHeaderName>),

    #[error("Other error: {0}")]
    Other(String),
}

// Manual From implementations since thiserror can't handle Arc wrapping
impl From<reqwest::Error> for ApiError {
    fn from(err: reqwest::Error) -> Self {
        ApiError::RequestFailed(Arc::new(err))
    }
}

impl From<reqwest::header::InvalidHeaderValue> for ApiError {
    fn from(err: reqwest::header::InvalidHeaderValue) -> Self {
        ApiError::InvalidHeaderValue(Arc::new(err))
    }
}

impl From<reqwest::header::InvalidHeaderName> for ApiError {
    fn from(err: reqwest::header::InvalidHeaderName) -> Self {
        ApiError::InvalidHeaderName(Arc::new(err))
    }
}

impl PartialEq for ApiError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ApiError::ResponseNotOK(a), ApiError::ResponseNotOK(b)) => a == b,
            (ApiError::ResponseError(a), ApiError::ResponseError(b)) => a == b,
            (ApiError::DecompressionFailed(a), ApiError::DecompressionFailed(b)) => a == b,
            (ApiError::JsonParseFailed(a), ApiError::JsonParseFailed(b)) => a == b,
            (ApiError::Other(a), ApiError::Other(b)) => a == b,

            // For Arc<reqwest::Error>, compare by string representation
            (ApiError::RequestFailed(a), ApiError::RequestFailed(b)) => {
                // Compare by string representation and status code if available
                a.to_string() == b.to_string() && a.status() == b.status()
            }

            // For Arc<InvalidHeaderValue>, compare by string representation
            (ApiError::InvalidHeaderValue(a), ApiError::InvalidHeaderValue(b)) => {
                a.to_string() == b.to_string()
            }

            // For Arc<InvalidHeaderName>, compare by string representation
            (ApiError::InvalidHeaderName(a), ApiError::InvalidHeaderName(b)) => {
                a.to_string() == b.to_string()
            }

            // Different variants are never equal
            _ => false,
        }
    }
}

pub type ApiClientResult<T> = anyhow::Result<T, ApiError>;
