use leptos::LeptosOptions;
use leptos_router::RouteListing;
use tokio::sync::broadcast;

use crate::{api::consts::EnvVarConfig, ssr_booking::PipelineLockManager, state::AppState};

use crate::api::Provab;
use once_cell::sync::OnceCell;

static PROVAB_CLIENT: OnceCell<Provab> = OnceCell::new();

pub fn initialize_provab_client() {
    PROVAB_CLIENT
        .set(Provab::default())
        .expect("Failed to initialize Provab client");
}

pub fn get_provab_client() -> &'static Provab {
    PROVAB_CLIENT.get().expect("Failed to get Provab client")
}

pub struct AppStateBuilder {
    leptos_options: LeptosOptions,
    routes: Vec<RouteListing>,
    provab_client: &'static Provab,
    // broadcaster: broadcast::Sender<i32>,
}

impl AppStateBuilder {
    pub fn new(
        leptos_options: LeptosOptions,
        routes: Vec<RouteListing>,
        provab_client: &'static Provab,
        // broadcaster: broadcast::Sender<i32>,
    ) -> Self {
        Self {
            leptos_options,
            routes,
            provab_client,
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
        }
    }
}
