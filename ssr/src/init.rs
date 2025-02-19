use leptos::LeptosOptions;
use leptos_router::RouteListing;
use tokio::sync::broadcast;

use crate::{api::consts::EnvVarConfig, ssr_booking::PipelineLockManager, state::AppState};

pub struct AppStateBuilder {
    leptos_options: LeptosOptions,
    routes: Vec<RouteListing>,
    // broadcaster: broadcast::Sender<i32>,
}

impl AppStateBuilder {
    pub fn new(
        leptos_options: LeptosOptions,
        routes: Vec<RouteListing>,
        // broadcaster: broadcast::Sender<i32>,
    ) -> Self {
        Self {
            leptos_options,
            routes,
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
        }
    }
}
