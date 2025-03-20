pub mod api_error_state;
pub mod local_storage;
pub mod search_state;
pub mod view_state;

use cfg_if::cfg_if;

use crate::api::consts::EnvVarConfig;

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use axum::extract::FromRef;
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
            pub pipeline_lock_manager: PipelineLockManager
            // pub cookie_key: Key,
            // #[cfg(feature = "oauth-ssr")]
            // pub google_oauth_clients: crate::auth::core_clients::CoreClients,
            // #[cfg(feature = "ga4")]
            // pub grpc_offchain_channel: tonic::transport::Channel,
        }
    }
}
