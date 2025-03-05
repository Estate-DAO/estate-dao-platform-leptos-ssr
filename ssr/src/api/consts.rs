// CONST FOR LOCAL STORAGE
pub const PAYMENT_ID: &str = "estatedao_payment_id";
pub const PAYMENT_STATUS: &str = "estatedao_payment_status";
pub const BOOKING_ID: &str = "estatedao_booking_id";
pub const BOOK_ROOM_RESPONSE: &str = "estatedao_book_room_response";

// PROVAB_BASE_URL
const PROVAB_PROD_OLD_PROXY: &str =
    // "https://abctravel.elixirpinging.xyz/prod/webservices/index.php/hotel_v3/service";
    "http://5.75.246.9:8001/prod/webservices/index.php/hotel_v3/service";

const PROVAB_TEST_OLD_PROXY: &str =
    // "https://abctravel.elixirpinging.xyz/webservices/index.php/hotel_v3/service";
    // "http://5.75.246.9:8001/prod/webservices/index.php/hotel_v3/service";
    "http://5.75.246.9:8001/test/webservices/index.php/hotel_v3/service";

// const PROVAB_PROD_ESTATEFLY_PROXY: &str =
//     "http://estate-static-ip-egress-proxy.internal/produrl/webservices/index.php/hotel_v3/service";

const PROVAB_PROD_ESTATEFLY_PROXY: &str =
    // "http://estate-static-ip-egress-proxy.internal:8080/webservices/index.php/hotel_v3/service";
    "http://estate-static-ip-egress-proxy.internal/prod/webservices/index.php/hotel_v3/service";

// const PROVAB_TEST_ESTATEFLY_PROXY: &str =
//     "http://estate-static-ip-egress-proxy.internal:8001/webservices/index.php/hotel_v3/service";

// APP_URL
const LOCALHOST_APP_URL: &str = "http://localhost:3000";
const STAGING_APP_URL: &str = "https://estatefe.fly.dev";
const PROD_APP_URL: &str = "https://nofeebooking.com";

// common consts
const AGENT_URL_REMOTE: &str = "https://a4gq6-oaaaa-aaaab-qaa4q-cai.raw.ic0.app";

// const for local environment
const AGENT_URL_LOCAL: &str = "http://localhost:4943";

cfg_if! {
    if #[cfg(feature = "local-consts")] {
        pub const APP_URL: &str = LOCALHOST_APP_URL;
        pub const AGENT_URL: &str = AGENT_URL_LOCAL;
        pub const PROVAB_BASE_URL: &str = PROVAB_TEST_OLD_PROXY;
    }
    else  if #[cfg(feature = "prod-consts")] {
        pub const APP_URL: &str = PROD_APP_URL;
        pub const AGENT_URL: &str = AGENT_URL_REMOTE;
        pub const PROVAB_BASE_URL: &str = PROVAB_PROD_ESTATEFLY_PROXY;
        // pub const PROVAB_BASE_URL: &str = PROVAB_TEST_ESTATEFLY_PROXY;
    }
    else {
        pub const APP_URL: &str = STAGING_APP_URL;
        pub const AGENT_URL: &str = AGENT_URL_REMOTE;
        // pub const PROVAB_BASE_URL: &str = PROVAB_TEST_OLD_PROXY;
        pub const PROVAB_BASE_URL: &str = PROVAB_PROD_OLD_PROXY;

    }
}

use crate::{app::AppRoutes, utils::route::join_base_and_path_url};
use cfg_if::cfg_if;
use colored::Colorize;
// use dotenvy::dotenv;
// use leptos::logging::log;
use crate::log;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::Deserialize;
use std::collections::HashMap;
use std::env::VarError;
use thiserror::Error;

pub fn get_payments_url(status: &str) -> String {
    let base_url = APP_URL;
    let url = join_base_and_path_url(base_url, &AppRoutes::Confirmation.to_string())
        .unwrap_or_else(|e| {
            eprintln!("Error joining URL: {}", e);
            format!("{}?payment={}", base_url, status) // Fallback to simpler construction if joining fails.
        });

    println!(
        "{}",
        format!("get_payments_url - {}", url).bright_green().bold()
    );
    url
}

pub fn get_price_amount_based_on_env(price: u32) -> u32 {
    cfg_if::cfg_if! {
          if #[cfg(feature = "prod-consts")] {
            price
        }
        else {
            20_u32
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct EnvVarConfig {
    #[serde(default = "provab_base_url_default")]
    pub provab_base_url: String,
    provab_headers: HashMap<String, String>,
    pub nowpayments_api_key: String,
    pub admin_private_key: String,
    pub ipn_secret: String, // skip the payment on localhost using environment variable
                            // pub payment_skip_local: String
}

impl EnvVarConfig {
    pub fn try_from_env() -> Self {
        log_other_consts();
        let provab_headers = env_or_panic("PROVAB_HEADERS");

        let pv_hashmap: HashMap<String, String> = parse_provab_headers(&provab_headers);

        let value = Self {
            provab_headers: pv_hashmap,
            provab_base_url: env_w_default("PROVAB_BASE_URL", PROVAB_BASE_URL).unwrap(),
            nowpayments_api_key: env_or_panic("NOW_PAYMENTS_USDC_ETHEREUM_API_KEY"),
            admin_private_key: env_or_panic(
                "ESTATE_DAO_SNS_PROPOSAL_SUBMISSION_IDENTITY_PRIVATE_KEY",
            ),
            // todo add secret when available in gh actions
            ipn_secret: env_w_default("NOWPAYMENTS_IPN_SECRET", "dummy-secret-for-now").unwrap(),
            // ipn_secret: env_or_panic("NOWPAYMENTS_IPN_SECRET"), // payment_skip_local: env_w_default("PAYMENTS_SKIP_LOCAL", "false").unwrap()
        };

        println!("{}", value.provab_base_url);
        value
    }

    pub fn get_headers(&self) -> HeaderMap {
        transform_headers(&self.provab_headers)
    }
}

fn log_other_consts() {
    log!("APP_URL: {}", APP_URL);
    log!("AGENT_URL: {}", AGENT_URL);
    log!("AGENT_URL: {}", AGENT_URL);
}
fn parse_provab_headers(provab_headers: &str) -> HashMap<String, String> {
    if provab_headers.starts_with('{') {
        // log!("got provab_headers - {{");
        serde_json::from_str(provab_headers).unwrap()
    } else {
        let trimmed_headers = provab_headers
            .trim_start_matches('\'')
            .trim_end_matches('\'');
        // log!("need to trim provab_headers - ' ");
        serde_json::from_str(trimmed_headers).unwrap()
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
    PROVAB_BASE_URL.to_string()
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
