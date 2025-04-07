pub mod api_error_state;
pub mod confirmation_results_state;
pub mod hotel_details_state;
pub mod input_group_state;
pub mod local_storage;
pub mod search_state;
pub mod view_state;
// pub mod pricing_book_now;

pub trait GlobalStateForLeptos: Clone + Default + 'static + Sized {
    fn get() -> Self {
        let this = use_context::<Self>();
        match this {
            Some(x) => x,
            None => {
                Self::set_global();
                Self::default()
            }
        }
    }

    fn set_global() {
        provide_context(Self::default());
    }
}

use cfg_if::cfg_if;
use leptos::{provide_context, use_context};

use crate::api::consts::EnvVarConfig;

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use axum::extract::FromRef;
        use crate::api::Provab;
        use leptos::LeptosOptions;
        use leptos_router::RouteListing;
        use crate::ssr_booking::PipelineLockManager;
        // use tokio::sync::broadcast;

        #[derive(FromRef, Clone)]
        pub struct AppState {
            pub leptos_options: LeptosOptions,
            // pub canisters: Canisters<false>,
            // #[cfg(feature = "backend-admin")]
            // pub admin_canisters: super::admin_canisters::AdminCanisters,
            // #[cfg(feature = "cloudflare")]
            // pub cloudflare: gob_cloudflare::CloudflareAuth,
            // pub kv: KVStoreImpl,
            pub routes: Vec<RouteListing>,
            pub env_var_config: EnvVarConfig,
            // pub count_tx: broadcast::Sender<i32>,
            pub pipeline_lock_manager: PipelineLockManager,
            pub provab_client: &'static Provab,
            // pub cookie_key: Key,
            // #[cfg(feature = "oauth-ssr")]
            // pub google_oauth_clients: crate::auth::core_clients::CoreClients,
            // #[cfg(feature = "ga4")]
            // pub grpc_offchain_channel: tonic::transport::Channel,
        }
    }
}
