pub const PROVAB_BASE_URL_PROD: &str =
    "https://prod.services.travelomatix.com/webservices/index.php/hotel_v3/service";

pub const PROVAB_BASE_URL_TEST: &str = "http://5.75.246.9/webservices/index.php/hotel_v3/service";

// pub const PROVAB_BASE_URL_LOCAL: &str = "https://api.cloudflare.com/client/v4/";
use serde_json::Value;
use std::collections::HashMap;

use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Client, Url,
};
pub fn get_provab_base_url_from_env() -> &'static str {
    match (option_env!("LOCAL"), option_env!("PROVAB_BASE_URL")) {
        (Some("true"), _) => &PROVAB_BASE_URL_TEST,
        (_, Some(val)) => val,
        _ => &PROVAB_BASE_URL_PROD,
    }
}

pub fn is_local_env() -> bool {
    Some("true") == option_env!("LOCAL")
}

pub fn get_headers_from_env() -> HeaderMap {
    let headers: Value =
        serde_json::from_str(option_env!("PROVAB_HEADERS").expect("PROVAB_HEADERS must be set"))
            .expect("Failed to deserialize PROVAB_HEADERS");

    let headers = headers
        .as_object()
        .expect("PROVAB_HEADERS must be a JSON object")
        .iter()
        .map(|(key, value)| (key.clone(), value.as_str().unwrap().to_string()))
        .collect();

    transform_headers(&headers)
}

pub fn transform_headers(headers: &HashMap<String, String>) -> HeaderMap {
    let mut header_map = HeaderMap::new();
    for (key, value) in headers {
        let header_name = HeaderName::from_bytes(key.as_bytes()).unwrap();

        let header_value = HeaderValue::from_str(&value).unwrap();

        header_map.insert(header_name, header_value);
    }
    header_map
}
