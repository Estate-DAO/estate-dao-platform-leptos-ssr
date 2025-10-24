use base64::{engine::general_purpose, Engine as _};
use chrono::Datelike;
use leptos::*;
use leptos_router::{use_navigate, use_query_map};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use url::form_urlencoded;

// ParamsMap is the type returned by use_query_map().get() - it's a BTreeMap
pub type ParamsMap = std::collections::BTreeMap<String, String>;

/// Helper functions for parsing individual query parameters (non-base64)
pub mod individual_params {
    use chrono::{Datelike, NaiveDate};

    /// Parse a date string in YYYY-MM-DD format to (year, month, day) tuple
    pub fn parse_date(date_str: &str) -> Option<(u32, u32, u32)> {
        NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .ok()
            .map(|date| (date.year() as u32, date.month(), date.day()))
    }

    /// Format a date tuple (year, month, day) to YYYY-MM-DD string
    pub fn format_date(date: (u32, u32, u32)) -> String {
        format!("{:04}-{:02}-{:02}", date.0, date.1, date.2)
    }

    /// Parse comma-separated string to Vec<String>
    pub fn parse_comma_separated(s: &str) -> Vec<String> {
        if s.trim().is_empty() {
            Vec::new()
        } else {
            s.split(',')
                .map(|item| item.trim().to_string())
                .filter(|item| !item.is_empty())
                .collect()
        }
    }

    /// Parse comma-separated string to Vec<u32>
    pub fn parse_comma_separated_u32(s: &str) -> Vec<u32> {
        if s.trim().is_empty() {
            Vec::new()
        } else {
            s.split(',')
                .filter_map(|item| item.trim().parse::<u32>().ok())
                .collect()
        }
    }

    /// Join Vec<String> to comma-separated string
    pub fn join_comma_separated(vec: &[String]) -> String {
        vec.join(",")
    }

    /// Join Vec<u32> to comma-separated string
    pub fn join_comma_separated_u32(vec: &[u32]) -> String {
        vec.iter()
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join(",")
    }

    /// Get optional string parameter from ParamsMap
    pub fn get_param(params: &leptos_router::ParamsMap, key: &str) -> Option<String> {
        params.get(key).cloned()
    }

    /// Get optional u32 parameter from ParamsMap
    pub fn get_param_u32(params: &leptos_router::ParamsMap, key: &str) -> Option<u32> {
        params.get(key).and_then(|s| s.parse().ok())
    }

    /// Get optional f64 parameter from ParamsMap
    pub fn get_param_f64(params: &leptos_router::ParamsMap, key: &str) -> Option<f64> {
        params.get(key).and_then(|s| s.parse().ok())
    }

    /// Get optional bool parameter from ParamsMap
    pub fn get_param_bool(params: &leptos_router::ParamsMap, key: &str) -> Option<bool> {
        params
            .get(key)
            .and_then(|s| match s.to_lowercase().as_str() {
                "true" | "1" | "yes" => Some(true),
                "false" | "0" | "no" => Some(false),
                _ => None,
            })
    }
}

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

/// Helper function to update URL with state (base64 encoded)
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

/// Helper function to update URL with individual query parameters (non-base64)
pub fn update_url_with_params(path: &str, params: &HashMap<String, String>) {
    let navigate = use_navigate();

    let query_string = form_urlencoded::Serializer::new(String::new())
        .extend_pairs(params.iter().filter(|(_, v)| !v.is_empty()))
        .finish();

    let new_url = if query_string.is_empty() {
        path.to_string()
    } else {
        format!("{}?{}", path, query_string)
    };

    navigate(&new_url, Default::default());
}

/// Helper function to build query string from params
pub fn build_query_string(params: &HashMap<String, String>) -> String {
    form_urlencoded::Serializer::new(String::new())
        .extend_pairs(params.iter().filter(|(_, v)| !v.is_empty()))
        .finish()
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

    #[test]
    fn test_parse_date() {
        use super::individual_params::*;

        let date = parse_date("2025-01-15");
        assert_eq!(date, Some((2025, 1, 15)));

        let invalid = parse_date("invalid");
        assert_eq!(invalid, None);
    }

    #[test]
    fn test_format_date() {
        use super::individual_params::*;

        let formatted = format_date((2025, 1, 15));
        assert_eq!(formatted, "2025-01-15");
    }

    #[test]
    fn test_parse_comma_separated() {
        use super::individual_params::*;

        let items = parse_comma_separated("wifi,pool,parking");
        assert_eq!(items, vec!["wifi", "pool", "parking"]);

        let empty = parse_comma_separated("");
        assert_eq!(empty.len(), 0);

        let with_spaces = parse_comma_separated("wifi, pool , parking");
        assert_eq!(with_spaces, vec!["wifi", "pool", "parking"]);
    }

    #[test]
    fn test_parse_comma_separated_u32() {
        use super::individual_params::*;

        let ages = parse_comma_separated_u32("8,10,12");
        assert_eq!(ages, vec![8, 10, 12]);

        let empty = parse_comma_separated_u32("");
        assert_eq!(empty.len(), 0);

        let with_invalid = parse_comma_separated_u32("8,invalid,10");
        assert_eq!(with_invalid, vec![8, 10]);
    }

    #[test]
    fn test_build_query_string() {
        let mut params = HashMap::new();
        params.insert("adults".to_string(), "2".to_string());
        params.insert("children".to_string(), "1".to_string());
        params.insert("rooms".to_string(), "1".to_string());

        let query_string = build_query_string(&params);
        assert!(query_string.contains("adults=2"));
        assert!(query_string.contains("children=1"));
        assert!(query_string.contains("rooms=1"));
    }
}
