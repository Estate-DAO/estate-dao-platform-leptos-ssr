// CONST FOR LOCAL STORAGE
pub const PAYMENT_ID: &str = "estatedao_payment_id";
pub const PAYMENT_STATUS: &str = "estatedao_payment_status";
pub const BOOKING_ID: &str = "estatedao_booking_id";
pub const BOOK_ROOM_RESPONSE: &str = "estatedao_book_room_response";
pub const USER_IDENTITY: &str = "estatedao_user_identity";
pub const USER_EMAIL_MAPPING_SYNCED: &str = "estatedao_user_email_mapping_synced";

// PROVAB_BASE_URL options
pub const PROVAB_PROD_OLD_PROXY: &str =
    "http://5.75.246.9:8001/prod/webservices/index.php/hotel_v3/service";

pub const PROVAB_TEST_OLD_PROXY: &str =
    "http://5.75.246.9:8001/test/webservices/index.php/hotel_v3/service";

pub const PROVAB_PROD_ESTATEFLY_PROXY: &str =
    // "http://estate-static-ip-egress-proxy.internal/prod/webservices/index.php/hotel_v3/service";
    "http://5.75.246.9:8001/prod/webservices/index.php/hotel_v3/service";

// APP_URL
const LOCALHOST_APP_URL: &str = "http://localhost:3002/";
// const LOCALHOST_APP_URL: &str = "https://louse-musical-hideously.ngrok-free.app";
// const STAGING_APP_URL: &str = "https://pr-111-estate.fly.dev";
// const STAGING_APP_URL: &str = "https://estatefe.fly.dev";
const PROD_APP_URL: &str = "https://nofeebooking.com";

// common consts
const AGENT_URL_REMOTE: &str = "https://a4gq6-oaaaa-aaaab-qaa4q-cai.raw.ic0.app";

const BASE_URL: &str = crate::canister::APP_URL;
// const for local environment
const AGENT_URL_LOCAL: &str = "http://localhost:4943";

pub const GTAG_MEASUREMENT_ID: Lazy<&str> = Lazy::new(|| "G-BPRVSPTP2T");

pub fn get_host() -> String {
    #[cfg(feature = "hydrate")]
    {
        leptos_use::use_window()
            .as_ref()
            .unwrap()
            .location()
            .host()
            .unwrap()
            .to_string()
    }

    #[cfg(not(feature = "hydrate"))]
    {
        use leptos::prelude::*;

        use axum::http::request::Parts;
        let parts: Option<Parts> = use_context();
        if parts.is_none() {
            return "".to_string();
        }
        let headers = parts.unwrap().headers;
        headers
            .get("Host")
            .map(|h| h.to_str().unwrap_or_default().to_string())
            .unwrap_or_default()
    }
}

cfg_if! {
    if #[cfg(feature = "local-consts")] {
        pub static APP_URL: Lazy<String> = Lazy::new(|| {
            env_w_default("NGROK_LOCALHOST_URL", LOCALHOST_APP_URL).unwrap().to_string()
        });
        pub const AGENT_URL: &str = AGENT_URL_LOCAL;
        pub const SEARCH_COMPONENT_ROOMS_DEFAULT: u32 = 4;
    }
    else if #[cfg(feature = "prod-consts")] {
        pub static APP_URL: Lazy<String> = Lazy::new(||   env_w_default("APP_URL", PROD_APP_URL).unwrap().to_string());
        pub const AGENT_URL: &str = AGENT_URL_REMOTE;
        pub const SEARCH_COMPONENT_ROOMS_DEFAULT: u32 = 1;
    }
    else {
        pub static APP_URL: Lazy<String> = Lazy::new(|| BASE_URL.to_string());
        pub const AGENT_URL: &str = AGENT_URL_REMOTE;
        pub const SEARCH_COMPONENT_ROOMS_DEFAULT: u32 = 1;
    }
}

// Get the default PROVAB_BASE_URL based on the current environment
pub fn get_default_provab_base_url() -> &'static str {
    cfg_if::cfg_if! {
        if #[cfg(feature = "local-consts")] {
            PROVAB_TEST_OLD_PROXY
        }
        else if #[cfg(feature = "prod-consts")] {
            PROVAB_PROD_ESTATEFLY_PROXY
        }
        else {
            PROVAB_PROD_OLD_PROXY
        }
    }
}

use crate::{app::AppRoutes, utils::route::join_base_and_path_url};
use cfg_if::cfg_if;
use colored::Colorize;
use leptos::use_context;
use once_cell::sync::Lazy;
// use dotenvy::dotenv;
// use leptos::logging::log;
use crate::log;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env::VarError;
use thiserror::Error;

/// Extract domain from URL and add dot prefix (e.g., ".nofeebooking.com")
fn extract_domain_with_dot(url: &str) -> String {
    // Remove protocol (http:// or https://)
    let without_protocol = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);

    // Remove trailing slash if present
    let domain = without_protocol.trim_end_matches('/');

    // Remove port if present (e.g., localhost:3002 -> localhost)
    let domain_without_port = domain.split(':').next().unwrap_or(domain);

    format!(".{}", domain_without_port)
}

/// Get the domain with a dot prefix (e.g., ".nofeebooking.com")
/// Extracts domain from APP_URL and adds dot prefix
pub fn get_app_domain_with_dot() -> String {
    extract_domain_with_dot(APP_URL.as_str())
}

#[cfg(feature = "mock-provab")]
pub fn get_payments_url(_status: &str) -> String {
    let base_url = APP_URL.as_str();
    let url = join_base_and_path_url(base_url, &AppRoutes::Confirmation.to_string())
        .unwrap_or_else(|e| {
            eprintln!("Error joining URL: {}", e);
            format!("{}?NP_id={}", base_url, _status) // Fallback to simpler construction if joining fails.
        });
    // http://localhost:3000/confirmation?NP_id=4766973829
    format!("{}?NP_id={}", url, "4766973829")
}

#[cfg(not(feature = "mock-provab"))]
pub fn get_payments_url(status: &str) -> String {
    let base_url = APP_URL.as_str();
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

/// Get payment URL with provider-specific parameters
/// For Stripe: Adds ?session_id={CHECKOUT_SESSION_ID}
/// For NowPayments: Adds ?payment=<status>
#[cfg(not(feature = "mock-provab"))]
pub fn get_payments_url_v2(status: &str, provider: PaymentProvider) -> String {
    let base_url = APP_URL.as_str();
    let confirmation_url = join_base_and_path_url(base_url, &AppRoutes::Confirmation.to_string())
        .unwrap_or_else(|e| {
            eprintln!("Error joining URL: {}", e);
            base_url.to_string() // Fallback to base URL if joining fails
        });

    let url = match provider {
        PaymentProvider::Stripe => {
            format!("{}?session_id={{CHECKOUT_SESSION_ID}}", confirmation_url)
        }
        PaymentProvider::NowPayments => format!("{}?payment={}", confirmation_url, status),
    };

    println!(
        "{}",
        format!("get_payments_url_v2 - {}", url)
            .bright_green()
            .bold()
    );
    url
}

#[cfg(feature = "mock-provab")]
pub fn get_payments_url_v2(_status: &str, provider: PaymentProvider) -> String {
    let base_url = APP_URL.as_str();
    let url = join_base_and_path_url(base_url, &AppRoutes::Confirmation.to_string())
        .unwrap_or_else(|e| {
            eprintln!("Error joining URL: {}", e);
            base_url.to_string() // Fallback to base URL if joining fails
        });

    // For mock,  return NP_id=4766973829 for nowpayments
    // for stripe, return session_id={CHECKOUT_SESSION_ID}
    match provider {
        PaymentProvider::Stripe => format!("{}?session_id={{CHECKOUT_SESSION_ID}}", url),
        PaymentProvider::NowPayments => format!("{}?NP_id={}", url, "4766973829"),
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PaymentProvider {
    Stripe,
    NowPayments,
}

/// ipn = Instant Payment Notification
#[cfg(not(feature = "mock-provab"))]
pub fn get_ipn_callback_url(provider: PaymentProvider) -> String {
    let base_url = APP_URL.as_str();
    match provider {
        PaymentProvider::Stripe => format!("{}/stripe/webhook", base_url),
        PaymentProvider::NowPayments => format!("{}/ipn/webhook", base_url),
    }
}

/// ipn = Instant Payment Notification
#[cfg(feature = "mock-provab")]
pub fn get_ipn_callback_url(provider: PaymentProvider) -> String {
    let base_url = APP_URL.as_str();
    match provider {
        PaymentProvider::Stripe => format!("{}/stripe/webhook", base_url),
        PaymentProvider::NowPayments => format!("{}/ipn/webhook", base_url),
    }
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
    pub liteapi_key: String,
    pub liteapi_prebook_base_url: String,
    pub nowpayments_api_key: String,
    pub admin_private_key: String,
    pub email_client_config: EmailConfig,
    pub ipn_secret: String, // skip the payment on localhost using environment variable
    // pub payment_skip_local: String
    pub stripe_secret_key: String,
    pub stripe_webhook_secret: String,

    // basic auth for the time being
    // todo(auth) - replace with yral auth when integrated
    pub basic_auth_username: String,
    pub basic_auth_password: String,

    // YRAL OAuth2 configuration
    pub yral_client_id: String,
    pub yral_client_secret: String,
    pub yral_redirect_uri: String,

    // Cookie encryption key (base64 encoded)
    pub cookie_key: String,
}

impl EnvVarConfig {
    pub fn expect_context_or_try_from_env() -> Self {
        let env_var_config = use_context::<EnvVarConfig>();
        match env_var_config {
            Some(env_var_config) => env_var_config,
            None => Self::try_from_env(),
        }
    }

    pub fn try_from_env() -> Self {
        // log_other_consts();
        let provab_headers = env_or_panic("PROVAB_HEADERS");

        let pv_hashmap: HashMap<String, String> = parse_provab_headers(&provab_headers);

        // Get PROVAB_BASE_URL from environment or use the default
        let provab_base_url = match std::env::var("PROVAB_BASE_URL") {
            Ok(url) => {
                // log!("Using PROVAB_BASE_URL from environment: {}", url);
                url
            }
            Err(_) => {
                let default_url = get_default_provab_base_url().to_string();
                // log!(
                //     "PROVAB_BASE_URL not found in environment, using default: {}",
                //     default_url
                // );
                default_url
            }
        };

        let value = Self {
            provab_headers: pv_hashmap,
            provab_base_url,
            liteapi_key: env_or_panic("LITEAPI_KEY"),
            liteapi_prebook_base_url: env_or_panic("LITEAPI_PREBOOK_BASE_URL"),
            nowpayments_api_key: env_or_panic("NOW_PAYMENTS_USDC_ETHEREUM_API_KEY"),
            admin_private_key: env_or_panic(
                "ESTATE_DAO_SNS_PROPOSAL_SUBMISSION_IDENTITY_PRIVATE_KEY",
            ),
            // todo add secret when available in gh actions
            ipn_secret: env_w_default("NOWPAYMENTS_IPN_SECRET", "dummy-secret-for-now").unwrap(),
            email_client_config: EmailConfig::from_env().unwrap(),
            stripe_secret_key: env_or_panic("STRIPE_SECRET_KEY"),
            stripe_webhook_secret: env_w_default("STRIPE_WEBHOOK_SECRET", "dummy-secret-for-now")
                .unwrap(),
            // ipn_secret: env_or_panic("NOWPAYMENTS_IPN_SECRET"), // payment_skip_local: env_w_default("PAYMENTS_SKIP_LOCAL", "false").unwrap()
            basic_auth_username: env_or_panic("BASIC_AUTH_USERNAME_FOR_LEPTOS_ROUTE"),
            basic_auth_password: env_or_panic("BASIC_AUTH_PASSWORD_FOR_LEPTOS_ROUTE"),

            // YRAL OAuth2 configuration
            yral_client_id: env_w_default("YRAL_AUTH_CLIENT_ID", "").unwrap(),
            yral_client_secret: env_w_default("YRAL_AUTH_CLIENT_SECRET", "").unwrap(),
            yral_redirect_uri: env_w_default(
                "YRAL_AUTH_REDIRECT_URL",
                &format!("{}/auth/callback", APP_URL.as_str()),
            )
            .unwrap(),

            // Cookie encryption key
            cookie_key: env_or_panic("COOKIE_KEY"),
        };

        // println!("Using PROVAB_BASE_URL: {}", value.provab_base_url);
        value
    }

    pub fn get_headers(&self) -> HeaderMap {
        transform_headers(&self.provab_headers)
    }

    /// Create a test configuration with default values for testing
    #[cfg(test)]
    pub fn for_testing() -> Self {
        use std::collections::HashMap;

        let mut provab_headers = HashMap::new();
        provab_headers.insert("Content-Type".to_string(), "application/json".to_string());
        provab_headers.insert("Authorization".to_string(), "Bearer test-token".to_string());

        Self {
            provab_base_url: "https://test-api.example.com".to_string(),
            provab_headers,
            liteapi_key: "test-liteapi-key".to_string(),
            liteapi_prebook_base_url: "https://test-liteapi.example.com".to_string(),
            nowpayments_api_key: "test-nowpayments-key".to_string(),
            admin_private_key: "test-admin-key".to_string(),
            email_client_config: EmailConfig {
                client_id: Some("test-email-client-id".to_string()),
                client_secret: Some("test-email-client-secret".to_string()),
                access_token: Some("test-email-access-token".to_string()),
                refresh_token: Some("test-email-refresh-token".to_string()),
                token_expiry: 1234567890,
            },
            ipn_secret: "test-ipn-secret".to_string(),
            stripe_secret_key: "sk_test_123456789".to_string(),
            stripe_webhook_secret: "whsec_test_123456789".to_string(),
            basic_auth_username: "test-admin".to_string(),
            basic_auth_password: "test-password".to_string(),

            // YRAL OAuth2 test configuration
            yral_client_id: "test-yral-client-id".to_string(),
            yral_client_secret: "test-yral-client-secret".to_string(),
            yral_redirect_uri: "http://localhost:3002/auth/callback".to_string(),

            // Test cookie key (base64 encoded dummy key)
            cookie_key: "dGVzdC1jb29raWUta2V5LWR1bW15LWRhdGE=".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmailConfig {
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub token_expiry: u64, // Store the token expiration timestamp
}

impl ConfigLoader for EmailConfig {
    fn from_env() -> Result<Self, String> {
        let client_id = env_or_panic("EMAIL_CLIENT_ID");
        let client_secret = env_or_panic("EMAIL_CLIENT_SECRET");
        let refresh_token = env_or_panic("EMAIL_REFRESH_TOKEN");
        let token_expiry = env_or_panic("EMAIL_TOKEN_EXPIRY");
        let access_token = env_or_panic("EMAIL_ACCESS_TOKEN");
        Ok(EmailConfig {
            client_id: Some(client_id),
            client_secret: Some(client_secret),
            access_token: Some(access_token),
            refresh_token: Some(refresh_token),
            token_expiry: token_expiry.parse().unwrap(),
        })
    }
}

fn log_other_consts() {
    log!("APP_URL: {}", APP_URL.as_str());
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

pub fn env_w_default(key: &str, default: &str) -> Result<String, EstateEnvConfigError> {
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
    get_default_provab_base_url().to_string()
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

/// A trait for loading configuration from the environment or another source.
pub trait ConfigLoader: Sized {
    /// Initialize the configuration by reading from the environment.
    ///
    /// # Errors
    ///
    /// Returns an `Err` if any required parameter is missing or invalid.
    fn from_env() -> Result<Self, String>;
}

#[cfg(test)]
mod tests {
    use super::extract_domain_with_dot;

    #[test]
    fn test_extract_domain_with_dot_nofeebooking() {
        let url = "https://nofeebooking.com";
        let result = extract_domain_with_dot(url);
        assert_eq!(result, ".nofeebooking.com");
    }

    #[test]
    fn test_extract_domain_with_dot_localhost() {
        let url = "http://localhost:3002";
        let result = extract_domain_with_dot(url);
        assert_eq!(result, ".localhost");
    }

    #[test]
    fn test_extract_domain_with_dot_with_trailing_slash() {
        let url = "https://nofeebooking.com/";
        let result = extract_domain_with_dot(url);
        assert_eq!(result, ".nofeebooking.com");
    }

    #[test]
    fn test_extract_domain_with_dot_localhost_with_trailing_slash() {
        let url = "http://localhost:3002/";
        let result = extract_domain_with_dot(url);
        assert_eq!(result, ".localhost");
    }

    #[test]
    fn test_extract_domain_with_dot_no_protocol() {
        let url = "nofeebooking.com";
        let result = extract_domain_with_dot(url);
        assert_eq!(result, ".nofeebooking.com");
    }
}
