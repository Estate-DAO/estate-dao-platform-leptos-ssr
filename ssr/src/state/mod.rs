pub mod search_state;
pub mod view_state;

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use axum::extract::FromRef;
        use leptos::LeptosOptions;
        use leptos_router::RouteListing;

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
            // pub cookie_key: Key,
            // #[cfg(feature = "oauth-ssr")]
            // pub google_oauth_clients: crate::auth::core_clients::CoreClients,
            // #[cfg(feature = "ga4")]
            // pub grpc_offchain_channel: tonic::transport::Channel,
        }
    }
}
