
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Response Status not 200")]
    ResponseNotOK(String),
    
    #[error("Provab response error")]
    ResponseError,  

    #[error("Decompression failed")]
    DecompressionFailed,

    #[error("HTTP request failed")]
    RequestFailed(#[from] reqwest::Error),
    #[error("JSON parsing failed")]
    JsonParseFailed(#[from] serde_json::Error),
    #[error("Invalid header Value")]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),
    #[error("Invalid header Name")]
    InvalidHeaderName(#[from] reqwest::header::InvalidHeaderName),
}

pub type ApiClientResult<T> = error_stack::Result<T, ApiError>;
