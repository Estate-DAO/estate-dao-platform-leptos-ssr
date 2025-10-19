use base64::{engine::general_purpose, Engine as _};
use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_query_map};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use url::form_urlencoded;

// ParamsMap is just a HashMap<String, String> in leptos_router
pub type ParamsMap = HashMap<String, String>;

#[derive(Error, Debug)]
pub enum QueryParamError {
    #[error("Failed to parse parameter '{param}': {reason}")]
    ParseError { param: String, reason: String },
    #[error("Missing required parameter: {param}")]
    MissingParam { param: String },
    #[error("Invalid filter format: {filter}")]
    InvalidFilter { filter: String },
    #[error("Base64 decode error: {0}")]
    Base64Error(#[from] base64::DecodeError),
    #[error("UTF8 error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// New trait for base64-encoded query parameter synchronization
pub trait QueryParamsSync<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Clone,
{
    /// Parse from URL query params (base64 encoded)
    fn from_url_params(query_map: &HashMap<String, String>) -> Option<T> {
        query_map
            .get("state")
            .and_then(|encoded| decode_state(encoded).ok())
    }

    /// Encode state to URL query params  
    fn to_url_params(&self) -> HashMap<String, String>
    where
        Self: Serialize,
    {
        let mut params = HashMap::new();
        params.insert("state".to_string(), encode_state(self));
        params
    }

    /// Apply parsed params to application state (signals, contexts)
    fn sync_to_app_state(&self);
}

/// Base64 encode state for URL storage
pub fn encode_state<T: Serialize + ?Sized>(state: &T) -> String {
    match serde_json::to_string(state) {
        Ok(json) => general_purpose::URL_SAFE_NO_PAD.encode(json),
        Err(_) => String::new(), // Fallback to empty string on error
    }
}

/// Base64 decode state from URL
pub fn decode_state<T: for<'de> Deserialize<'de>>(encoded: &str) -> Result<T, QueryParamError> {
    let json_bytes = general_purpose::URL_SAFE_NO_PAD.decode(encoded)?;
    let json_str = String::from_utf8(json_bytes)?;
    Ok(serde_json::from_str(&json_str)?)
}

/// Helper function to update URL with state
pub fn update_url_with_state<T>(state: &T)
where
    T: QueryParamsSync<T> + Serialize + for<'de> Deserialize<'de> + Clone,
{
    let navigate = use_navigate();
    let params = state.to_url_params();

    let query_string = form_urlencoded::Serializer::new(String::new())
        .extend_pairs(&params)
        .finish();

    if let Some(window) = web_sys::window() {
        if let Ok(current_path) = window.location().pathname() {
            let new_url = if query_string.is_empty() {
                current_path
            } else {
                format!("{}?{}", current_path, query_string)
            };
            navigate(&new_url, Default::default());
        }
    }
}

/// Comparison operators for filtering (used in base64 encoded state)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ComparisonOp {
    /// Exact match
    Eq(String),
    /// Greater than
    Gt(f64),
    /// Greater than or equal
    Gte(f64),
    /// Less than
    Lt(f64),
    /// Less than or equal
    Lte(f64),
    /// Contains any of
    In(Vec<String>),
    /// Contains all of
    All(Vec<String>),
    /// Near location: lat, lng, radius
    Near(f64, f64, f64),
}

// ComparisonOp implementation is now much simpler since we use base64 encoding
// No need for complex parsing - serde handles serialization/deserialization

/// Collection of filters for a query (used in base64 encoded state)
pub type FilterMap = HashMap<String, ComparisonOp>;

/// Sort direction for query parameters
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SortDirection {
    Asc,
    Desc,
}

impl SortDirection {
    pub fn to_string(&self) -> &'static str {
        match self {
            SortDirection::Asc => "asc",
            SortDirection::Desc => "desc",
        }
    }
}

// TODO: Implement reactive query params hook in the future
// For now, we'll use the QueryParamsHandler trait directly

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_encoding_decoding() {
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct TestState {
            destination: String,
            adults: u32,
        }

        let state = TestState {
            destination: "NYC".to_string(),
            adults: 2,
        };

        let encoded = encode_state(&state);
        assert!(!encoded.is_empty());

        let decoded: TestState = decode_state(&encoded).unwrap();
        assert_eq!(decoded, state);
    }

    #[test]
    fn test_comparison_op_serialization() {
        let op = ComparisonOp::Gte(100.0);
        let json = serde_json::to_string(&op).unwrap();
        let deserialized: ComparisonOp = serde_json::from_str(&json).unwrap();
        assert_eq!(op, deserialized);
    }
}
