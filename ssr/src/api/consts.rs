pub const PROVAB_BASE_URL_PROD: &str =
    "https://prod.services.travelomatix.com/webservices/index.php/hotel_v3/service";

pub const PROVAB_BASE_URL_TEST: &str =
    "https://abctravel.elixirpinging.xyz/webservices/index.php/hotel_v3/service";

pub const AGENT_URL_LOCAL: &str = "http://localhost:4943";

// pub const AGENT_URL_LOCAL: &str = "https://ic0.app";
pub const AGENT_URL_REMOTE: &str = "https://a4gq6-oaaaa-aaaab-qaa4q-cai.raw.ic0.app";
pub const LOCALHOST_DEV: &str = "http://127.0.0.1:3000";
pub const PROD_URL: &str = "https://estatefe.fly.dev";

// CONST FOR LOCAL STORAGE
pub const PAYMENT_ID: &str = "estatedao_payment_id";
pub const PAYMENT_STATUS: &str = "estatedao_payment_status";
pub const BOOKING_ID: &str = "estatedao_booking_id";
pub const BOOK_ROOM_RESPONSE: &str = "estatedao_book_room_response";

use crate::{app::AppRoutes, utils::route::join_base_and_path_url};
use cfg_if::cfg_if;
use dotenvy::dotenv;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::Deserialize;
use std::collections::HashMap;
use std::env::VarError;
use thiserror::Error;

pub fn get_payments_url(status: &str) -> String {
    cfg_if! {
        if #[cfg(feature = "local-bin")] {
            let base_url = LOCALHOST_DEV;
        } else {
            let base_url = PROD_URL;
        }
    }
    join_base_and_path_url(base_url, &AppRoutes::Confirmation.to_string()).unwrap_or_else(|e| {
        eprintln!("Error joining URL: {}", e);
        format!("{}?payment={}", base_url, status) // Fallback to simpler construction if joining fails.
    })
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct EnvVarConfig {
    #[serde(default = "provab_base_url_default")]
    pub provab_base_url: String,
    provab_headers: HashMap<String, String>,
    pub nowpayments_api_key: String,
    pub admin_private_key: String,
}

impl EnvVarConfig {
    pub fn try_from_env() -> Self {
        let provab_headers = env_or_panic("PROVAB_HEADERS");

        let pv_hashmap: HashMap<String, String> = serde_json::from_str(&provab_headers).unwrap();

        let value = Self {
            provab_headers: pv_hashmap,
            provab_base_url: env_w_default("PROVAB_BASE_URL", PROVAB_BASE_URL_TEST).unwrap(),
            nowpayments_api_key: env_or_panic("NOW_PAYMENTS_USDC_ETHEREUM_API_KEY"),
            admin_private_key: env_or_panic(
                "ESTATE_DAO_SNS_PROPOSAL_SUBMISSION_IDENTITY_PRIVATE_KEY",
            ),
        };

        value
    }

    pub fn get_headers(&self) -> HeaderMap {
        transform_headers(&self.provab_headers)
    }
}

fn env_w_default(key: &str, default: &str) -> Result<String, EstateEnvConfigError> {
    match std::env::var(key) {
        Ok(val) => Ok(val),
        Err(VarError::NotPresent) => Ok(default.to_string()),
        Err(e) => Err(EstateEnvConfigError::EnvVarError(format!(
            "missing {key}: {e}"
        ))),
    }
}

fn env_wo_default(key: &str) -> Result<Option<String>, EstateEnvConfigError> {
    match std::env::var(key) {
        Ok(val) => Ok(Some(val)),
        Err(VarError::NotPresent) => Ok(None),
        Err(e) => Err(EstateEnvConfigError::EnvVarError(format!("{key}: {e}"))),
    }
}

fn env_or_panic(key: &str) -> String {
    match std::env::var(key) {
        Ok(val) => val,
        Err(e) => panic!("missing {key}: {e}"),
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
