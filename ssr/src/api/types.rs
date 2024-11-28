use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("API ResponseError - NotOk - \n{0}")]
    ResponseNotOK(String),

    #[error("Provab response error")]
    ResponseError,

    #[error("Decompression failed - {0}")]
    DecompressionFailed(String),

    #[error("HTTP request failed - {0}")]
    RequestFailed(#[from] reqwest::Error),
    #[error("JSON parsing failed `{0}`")]
    JsonParseFailed(String),
    #[error("Invalid header Value - {0}")]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),
    #[error("Invalid header Name - {0}")]
    InvalidHeaderName(#[from] reqwest::header::InvalidHeaderName),
}

pub type ApiClientResult<T> = error_stack::Result<T, ApiError>;
