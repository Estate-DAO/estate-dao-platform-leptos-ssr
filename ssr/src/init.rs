use leptos::LeptosOptions;
use leptos_router::RouteListing;

use crate::{api::consts::EnvVarConfig, state::AppState};

pub struct AppStateBuilder {
    leptos_options: LeptosOptions,
    routes: Vec<RouteListing>,
}

impl AppStateBuilder {
    pub fn new(leptos_options: LeptosOptions, routes: Vec<RouteListing>) -> Self {
        Self {
            leptos_options,
            routes,
        }
    }

    pub async fn build(self) -> AppState {
        AppState {
            leptos_options: self.leptos_options,
            routes: self.routes,
            env_var_config: EnvVarConfig::try_from_env(),
        }
    }
}
