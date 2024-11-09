pub const PROVAB_BASE_URL_PROD: &str =
    "https://prod.services.travelomatix.com/webservices/index.php/hotel_v3/service";

pub const PROVAB_BASE_URL_TEST: &str =
    "https://abctravel.elixirpinging.xyz/webservices/index.php/hotel_v3/service";

use serde::Deserialize;
use std::collections::HashMap;

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct EnvVarConfig {
    #[serde(default = "provab_base_url_default")]
    pub provab_base_url: String,
    provab_headers: HashMap<String, String>,
}

impl EnvVarConfig {
    pub fn try_from_env() -> Self {
        let provab_headers = env_w_default(
            "PROVAB_HEADERS",
            std::option_env!("PROVAB_HEADERS",).unwrap_or_default(),
        )
        .unwrap();

        if provab_headers.is_empty() {
            panic!(" PROVAB_HEADERS  = {provab_headers:?}");
        };
        let pv_hashmap: HashMap<String, String> = serde_json::from_str(&provab_headers).unwrap();

        let value = Self {
            provab_headers: pv_hashmap,
            provab_base_url: env_w_default("PROVAB_BASE_URL", PROVAB_BASE_URL_TEST).unwrap(),
        };

        value
    }

    pub fn get_headers(&self) -> HeaderMap {
        transform_headers(&self.provab_headers)
    }
}

use std::env::VarError;

fn env_w_default(key: &str, default: &str) -> Result<String, EstateEnvConfigError> {
    match std::env::var(key) {
        Ok(val) => Ok(val),
        Err(VarError::NotPresent) => Ok(default.to_string()),
        Err(e) => Err(EstateEnvConfigError::EnvVarError(format!(
            "missing {key}: {e}"
        ))),
    }
}

fn provab_base_url_default() -> String {
    PROVAB_BASE_URL_TEST.to_string()
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

use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum EstateEnvConfigError {
    // #[error("Cargo.toml not found in package root")]
    // ConfigNotFound,
    // #[error("package.metadata.leptos section missing from Cargo.toml")]
    // ConfigSectionNotFound,
    #[error("Failed to get Estate Environment. Did you set environment vairables?")]
    EnvError,
    // #[error("Config Error: {0}")]
    // ConfigError(String),
    #[error("Config Error: {0}")]
    EnvVarError(String),
}
