use std::sync::{Arc, Mutex};

use axum_extra::extract::cookie::Key;
use axum_extra::extract::PrivateCookieJar;
use base64::{engine::general_purpose, Engine as _};
use leptos::LeptosOptions;
use leptos_router::RouteListing;

use crate::{
    api::consts::EnvVarConfig, ssr_booking::email_handler::EmailClient,
    ssr_booking::PipelineLockManager, utils::error_alerts::ErrorAlertService,
    utils::notifier::Notifier, view_state_layer::AppState,
};

use crate::api::liteapi::LiteApiHTTPClient;
use once_cell::sync::OnceCell;

static LITEAPI_CLIENT: OnceCell<LiteApiHTTPClient> = OnceCell::new();
static NOTIFIER: OnceCell<Notifier> = OnceCell::new();
static ERROR_ALERT_SERVICE: OnceCell<ErrorAlertService> = OnceCell::new();

pub fn initialize_liteapi_client() {
    LITEAPI_CLIENT
        .set(LiteApiHTTPClient::default())
        .expect("Failed to initialize LiteAPI client");
}

pub fn get_liteapi_client() -> &'static LiteApiHTTPClient {
    LITEAPI_CLIENT.get().expect("Failed to get LiteAPI client")
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

pub struct AppStateBuilder {
    leptos_options: LeptosOptions,
    routes: Vec<RouteListing>,
    liteapi_client: &'static LiteApiHTTPClient,
    notifier_for_pipeline: &'static Notifier,
}

impl AppStateBuilder {
    pub fn new(leptos_options: LeptosOptions, routes: Vec<RouteListing>) -> Self {
        initialize_liteapi_client();
        initialize_notifier();

        Self {
            leptos_options,
            routes,
            liteapi_client: get_liteapi_client(),
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

        let app_state = AppState {
            leptos_options: self.leptos_options,
            routes: self.routes,
            env_var_config,
            pipeline_lock_manager: PipelineLockManager::new(),
            liteapi_client: self.liteapi_client,
            notifier_for_pipeline: self.notifier_for_pipeline,
            cookie_key: cookie_key.clone(),
            error_alert_service,
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
