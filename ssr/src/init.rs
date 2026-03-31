use std::str::FromStr;
use std::sync::{Arc, Mutex, RwLock};

use axum_extra::extract::cookie::Key;
use axum_extra::extract::PrivateCookieJar;
use base64::{engine::general_purpose, Engine as _};
use leptos::LeptosOptions;
use leptos_router::RouteListing;

use crate::api::consts::LITEAPI_ROOM_MAPPING;
use crate::{
    api::consts::EnvVarConfig, ssr_booking::email_handler::EmailClient,
    ssr_booking::PipelineLockManager, utils::currency::resolve_currency_code,
    utils::error_alerts::ErrorAlertService, utils::notifier::Notifier, view_state_layer::AppState,
};

use hotel_providers::{PlaceProviderPort, ProviderRegistry, ProviderRegistryBuilder};
use once_cell::sync::OnceCell;

use hotel_providers::booking::BookingDriver;
use hotel_providers::liteapi::{LiteApiClient, LiteApiDriver};

static LITEAPI_DRIVER: OnceCell<LiteApiDriver> = OnceCell::new();
static BOOKING_DRIVER: OnceCell<BookingDriver> = OnceCell::new();
static PROVIDER_REGISTRY: OnceCell<RwLock<Arc<ProviderRegistry>>> = OnceCell::new();
static CURRENT_HOTEL_PROVIDER: OnceCell<RwLock<PrimaryHotelProvider>> = OnceCell::new();
static NOTIFIER: OnceCell<Notifier> = OnceCell::new();
static ERROR_ALERT_SERVICE: OnceCell<ErrorAlertService> = OnceCell::new();

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
enum PrimaryHotelProvider {
    LiteApi,
    Booking,
}

// Compile-time default. Can be overridden at runtime via admin endpoint.
const PRIMARY_HOTEL_PROVIDER: PrimaryHotelProvider = PrimaryHotelProvider::LiteApi;

impl PrimaryHotelProvider {
    fn as_key(self) -> &'static str {
        match self {
            PrimaryHotelProvider::LiteApi => "liteapi",
            PrimaryHotelProvider::Booking => "booking",
        }
    }
}

impl FromStr for PrimaryHotelProvider {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let normalized: String = value
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .flat_map(|c| c.to_lowercase())
            .collect();

        match normalized.as_str() {
            "liteapi" => Ok(PrimaryHotelProvider::LiteApi),
            "booking" | "bookingcom" => Ok(PrimaryHotelProvider::Booking),
            _ => Err(format!(
                "Unsupported hotel provider '{}'. Valid values: liteapi, booking",
                value
            )),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PrimaryPlaceProvider {
    LiteApi,
}

// Compile-time place provider selection. Extend this enum as more place providers are added.
const PRIMARY_PLACE_PROVIDER: PrimaryPlaceProvider = PrimaryPlaceProvider::LiteApi;

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

pub fn get_liteapi_driver_with_currency(currency: Option<&str>) -> LiteApiDriver {
    let api_key = std::env::var("LITEAPI_KEY").unwrap_or_else(|_| "".to_string());
    let resolved_currency = resolve_currency_code(currency);
    let client = LiteApiClient::with_currency(api_key, None, resolved_currency);
    LiteApiDriver::new(client, *LITEAPI_ROOM_MAPPING)
}

pub fn initialize_booking_driver() {
    let api_token = std::env::var("BOOKING_API_TOKEN").unwrap_or_default();
    let affiliate_id = std::env::var("BOOKING_AFFILIATE_ID")
        .ok()
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(0);
    let base_url = std::env::var("BOOKING_BASE_URL")
        .unwrap_or_else(|_| "https://demandapi.booking.com/3.1".to_string());
    let currency = std::env::var("BOOKING_CURRENCY").unwrap_or_else(|_| "USD".to_string());
    let use_mock = std::env::var("BOOKING_USE_MOCK")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(api_token.is_empty());

    if api_token.is_empty() {
        tracing::warn!("BOOKING_API_TOKEN environment variable is empty or not set!");
    }

    let driver = if use_mock {
        BookingDriver::new_mock(currency)
    } else {
        BookingDriver::new_real(api_token, affiliate_id, base_url, currency)
    };

    BOOKING_DRIVER
        .set(driver)
        .expect("Failed to initialize Booking.com driver");
}

pub fn get_booking_driver() -> BookingDriver {
    BOOKING_DRIVER
        .get()
        .expect("Failed to get Booking.com driver")
        .clone()
}

fn configure_hotel_providers(
    builder: ProviderRegistryBuilder,
    primary_hotel_provider: PrimaryHotelProvider,
    liteapi_driver: &LiteApiDriver,
    booking_driver: &BookingDriver,
) -> ProviderRegistryBuilder {
    match primary_hotel_provider {
        PrimaryHotelProvider::Booking => builder
            .with_hotel_provider(booking_driver.clone())
            .with_hotel_provider(liteapi_driver.clone()),
        PrimaryHotelProvider::LiteApi => builder
            .with_hotel_provider(liteapi_driver.clone())
            .with_hotel_provider(booking_driver.clone()),
    }
}

fn configure_place_provider(
    builder: ProviderRegistryBuilder,
    liteapi_driver: &LiteApiDriver,
) -> ProviderRegistryBuilder {
    builder.with_place_provider_arc(select_primary_place_provider(liteapi_driver))
}

fn select_primary_place_provider(liteapi_driver: &LiteApiDriver) -> Arc<dyn PlaceProviderPort> {
    match PRIMARY_PLACE_PROVIDER {
        PrimaryPlaceProvider::LiteApi => Arc::new(liteapi_driver.clone()),
    }
}

fn build_provider_registry(
    primary_hotel_provider: PrimaryHotelProvider,
    liteapi_driver: &LiteApiDriver,
    booking_driver: &BookingDriver,
) -> ProviderRegistry {
    configure_place_provider(
        configure_hotel_providers(
            ProviderRegistry::builder(),
            primary_hotel_provider,
            liteapi_driver,
            booking_driver,
        ),
        liteapi_driver,
    )
    .build()
}

pub fn initialize_provider_registry() {
    let liteapi_driver = get_liteapi_driver();
    let booking_driver = get_booking_driver();

    let primary_hotel_provider = PRIMARY_HOTEL_PROVIDER;
    let registry = Arc::new(build_provider_registry(
        primary_hotel_provider,
        &liteapi_driver,
        &booking_driver,
    ));

    CURRENT_HOTEL_PROVIDER
        .set(RwLock::new(primary_hotel_provider))
        .expect("Failed to initialize current hotel provider state");

    PROVIDER_REGISTRY
        .set(RwLock::new(registry))
        .expect("Failed to initialize provider registry");
}

pub fn get_provider_registry() -> Arc<ProviderRegistry> {
    PROVIDER_REGISTRY
        .get()
        .expect("Provider registry not initialized")
        .read()
        .expect("Provider registry lock poisoned")
        .clone()
}

pub fn get_primary_hotel_provider() -> String {
    CURRENT_HOTEL_PROVIDER
        .get()
        .expect("Current hotel provider state not initialized")
        .read()
        .expect("Current hotel provider lock poisoned")
        .as_key()
        .to_string()
}

pub fn update_primary_hotel_provider(provider_key: &str) -> Result<String, String> {
    let selected_provider = PrimaryHotelProvider::from_str(provider_key)?;
    let liteapi_driver = get_liteapi_driver();
    let booking_driver = get_booking_driver();
    let registry = Arc::new(build_provider_registry(
        selected_provider,
        &liteapi_driver,
        &booking_driver,
    ));

    let registry_lock = PROVIDER_REGISTRY
        .get()
        .ok_or_else(|| "Provider registry not initialized".to_string())?;
    {
        let mut write_guard = registry_lock
            .write()
            .map_err(|_| "Provider registry lock poisoned".to_string())?;
        *write_guard = registry;
    }

    let provider_lock = CURRENT_HOTEL_PROVIDER
        .get()
        .ok_or_else(|| "Current hotel provider state not initialized".to_string())?;
    {
        let mut write_guard = provider_lock
            .write()
            .map_err(|_| "Current hotel provider lock poisoned".to_string())?;
        *write_guard = selected_provider;
    }

    tracing::info!(
        "Primary hotel provider updated at runtime: {}",
        selected_provider.as_key()
    );
    Ok(selected_provider.as_key().to_string())
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
        initialize_booking_driver();
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
