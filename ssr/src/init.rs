use std::sync::{Arc, Mutex};

use axum_extra::extract::cookie::Key;
use axum_extra::extract::PrivateCookieJar;
use base64::{engine::general_purpose, Engine as _};
use leptos::LeptosOptions;
use leptos_router::RouteListing;

use crate::api::consts::LITEAPI_ROOM_MAPPING;
use crate::{
    api::consts::EnvVarConfig, ssr_booking::email_handler::EmailClient,
    ssr_booking::PipelineLockManager, utils::error_alerts::ErrorAlertService,
    utils::notifier::Notifier, view_state_layer::AppState,
};

use crate::adapters::LiteApiProviderBridge;
use hotel_providers::ProviderRegistry;
use once_cell::sync::OnceCell;

use hotel_providers::liteapi::{LiteApiClient, LiteApiDriver};

static LITEAPI_DRIVER: OnceCell<LiteApiDriver> = OnceCell::new();
static PROVIDER_REGISTRY: OnceCell<Arc<ProviderRegistry>> = OnceCell::new();
static NOTIFIER: OnceCell<Notifier> = OnceCell::new();
static ERROR_ALERT_SERVICE: OnceCell<ErrorAlertService> = OnceCell::new();

pub fn initialize_liteapi_driver() {
    // Load config from environment directly for driver initialization
    let api_key = std::env::var("LITEAPI_KEY").unwrap_or_else(|_| "".to_string());

    if api_key.is_empty() {
        tracing::warn!("LITEAPI_KEY environment variable is empty or not set!");
    } else {
        let key_prefix = if api_key.len() > 8 {
            &api_key[..8]
        } else {
            &api_key
        };
        tracing::info!(
            "LiteAPI driver initialized with API key prefix: {}...",
            key_prefix
        );
    }

    let client = LiteApiClient::new(api_key, None);
    let driver = LiteApiDriver::new(client, *LITEAPI_ROOM_MAPPING);

    LITEAPI_DRIVER
        .set(driver)
        .expect("Failed to initialize LiteAPI driver");
}

pub fn get_liteapi_driver() -> LiteApiDriver {
    LITEAPI_DRIVER
        .get()
        .expect("Failed to get LiteAPI driver")
        .clone()
}

pub fn initialize_provider_registry() {
    // Create bridge directly from driver
    let driver = get_liteapi_driver();
    let bridge = LiteApiProviderBridge::new(driver);

    let registry = ProviderRegistry::builder()
        .with_hotel_provider(bridge.clone())
        .with_place_provider(bridge)
        .build();

    PROVIDER_REGISTRY
        .set(Arc::new(registry))
        .expect("Failed to initialize provider registry");
}

pub fn get_provider_registry() -> Arc<ProviderRegistry> {
    PROVIDER_REGISTRY
        .get()
        .expect("Failed to get provider registry")
        .clone()
}

/// Get a LiteApiProviderBridge from the global driver.
/// This is the preferred way to access the LiteAPI provider.
pub fn get_liteapi_bridge() -> LiteApiProviderBridge {
    LiteApiProviderBridge::new(get_liteapi_driver())
}

/// Alias for backward compatibility - returns bridge instead of adapter.
#[deprecated(note = "Use get_liteapi_bridge() instead")]
pub fn get_liteapi_adapter() -> LiteApiProviderBridge {
    get_liteapi_bridge()
}

pub fn initialize_notifier() {
    NOTIFIER
        .set(Notifier::with_bus())
        .expect("Failed to initialize Notifier");
}

pub fn get_notifier() -> &'static Notifier {
    NOTIFIER.get().expect("Failed to get Notifier")
}

pub fn initialize_error_alert_service(env_config: &EnvVarConfig) {
    let email_client = EmailClient::new(env_config.email_client_config.clone());
    let service = ErrorAlertService::new(email_client);
    ERROR_ALERT_SERVICE
        .set(service)
        .expect("Failed to initialize ErrorAlertService");
}

pub fn get_error_alert_service() -> &'static ErrorAlertService {
    ERROR_ALERT_SERVICE
        .get()
        .expect("Failed to get ErrorAlertService")
}

// ...
pub struct AppStateBuilder {
    leptos_options: LeptosOptions,
    routes: Vec<RouteListing>,
    liteapi_driver: LiteApiDriver, // Replaced client with driver
    notifier_for_pipeline: &'static Notifier,
}

impl AppStateBuilder {
    pub fn new(leptos_options: LeptosOptions, routes: Vec<RouteListing>) -> Self {
        initialize_liteapi_driver(); // Initialize the new driver
        initialize_notifier();
        initialize_provider_registry(); // Registry now uses the driver via adapter

        Self {
            leptos_options,
            routes,
            liteapi_driver: get_liteapi_driver(), // Get driver
            notifier_for_pipeline: get_notifier(),
        }
    }

    pub async fn build(self) -> AppState {
        let env_var_config = EnvVarConfig::try_from_env();

        println!("env_var_config = {:#?}", env_var_config);
        let cookie_key_bytes = general_purpose::STANDARD
            .decode(&env_var_config.cookie_key)
            .expect("COOKIE_KEY must be valid base64");
        let cookie_key = Key::from(&cookie_key_bytes);

        // Initialize error alert service
        initialize_error_alert_service(&env_var_config);
        let error_alert_service = get_error_alert_service();

        // Start background flush task for error alerts
        {
            let service_arc = Arc::new(error_alert_service.clone());
            service_arc.start_background_flush();
            tracing::info!("ErrorAlertService background flush started (5-minute interval)");
        }

        // Initialize GeoIP database for IP location lookups
        crate::utils::geoip_service::init_geoip("ip_db.mmdb");

        // Initialize place search cache (max 200 entries, 5-minute TTL)
        let place_search_cache = moka::future::Cache::builder()
            .max_capacity(200)
            .time_to_live(std::time::Duration::from_secs(300)) // 5 minutes
            .build();
        tracing::info!("PlaceSearchCache initialized (200 entries max, 5-minute TTL)");

        let app_state = AppState {
            leptos_options: self.leptos_options,
            routes: self.routes,
            env_var_config,
            pipeline_lock_manager: PipelineLockManager::new(),
            liteapi_driver: self.liteapi_driver, // Pass driver
            notifier_for_pipeline: self.notifier_for_pipeline,
            cookie_key: cookie_key.clone(),
            error_alert_service,
            place_search_cache,
            // private_cookie_jar: Arc::new(Mutex::new(PrivateCookieJar::new(cookie_key)))
        };

        let app_state_clone = app_state.clone();

        #[cfg(feature = "debug_log")]
        {
            use crate::log;
            // print env vars
            log!("app_state.env_var_config = {:#?}", app_state.env_var_config);

            // Setup debug event subscriber
            let debug_state = app_state_clone;
            tokio::spawn(async move {
                debug_state.setup_debug_event_subscriber().await;
            });
        }

        app_state
    }
}
