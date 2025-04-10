use leptos::LeptosOptions;
use leptos_router::RouteListing;
use tokio::sync::broadcast;

use crate::{
    api::consts::EnvVarConfig, ssr_booking::PipelineLockManager, state::AppState,
    utils::notifier::Notifier,
};

use crate::api::Provab;
use once_cell::sync::OnceCell;

static PROVAB_CLIENT: OnceCell<Provab> = OnceCell::new();
static NOTIFIER: OnceCell<Notifier> = OnceCell::new();

pub fn initialize_provab_client() {
    PROVAB_CLIENT
        .set(Provab::default())
        .expect("Failed to initialize Provab client");
}

pub fn get_provab_client() -> &'static Provab {
    PROVAB_CLIENT.get().expect("Failed to get Provab client")
}

pub fn initialize_notifier() {
    NOTIFIER
        .set(Notifier::with_bus())
        .expect("Failed to initialize Notifier");
}

pub fn get_notifier() -> &'static Notifier {
    NOTIFIER.get().expect("Failed to get Notifier")
}

pub struct AppStateBuilder {
    leptos_options: LeptosOptions,
    routes: Vec<RouteListing>,
    provab_client: &'static Provab,
    notifier_for_pipeline: &'static Notifier,
    // broadcaster: broadcast::Sender<i32>,
}

impl AppStateBuilder {
    pub fn new(
        leptos_options: LeptosOptions,
        routes: Vec<RouteListing>,
        // broadcaster: broadcast::Sender<i32>,
    ) -> Self {
        initialize_provab_client();
        initialize_notifier();

        Self {
            leptos_options,
            routes,
            provab_client: get_provab_client(),
            notifier_for_pipeline: get_notifier(),
            // broadcaster,
        }
    }

    pub async fn build(self) -> AppState {
        AppState {
            leptos_options: self.leptos_options,
            routes: self.routes,
            env_var_config: EnvVarConfig::try_from_env(),
            // count_tx: self.broadcaster,
            pipeline_lock_manager: PipelineLockManager::new(),
            provab_client: self.provab_client,
            notifier_for_pipeline: self.notifier_for_pipeline,
        }
    }
}
