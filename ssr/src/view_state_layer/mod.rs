pub mod api_error_state;
pub mod booking_context_state;
pub mod booking_conversions;
pub mod confirmation_results_state;
pub mod email_verification_state;
pub mod hotel_details_state;
pub mod input_group_state;
pub mod local_storage;
pub mod search_state;
pub mod ui_block_room;
pub mod ui_confirmation_page;
pub mod ui_confirmation_page_v2;
pub mod ui_hotel_details;
pub mod ui_search_state;
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

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use axum::extract::FromRef;
        use crate::api::provab::Provab;
        use crate::api::liteapi::LiteApiHTTPClient;
        use leptos::LeptosOptions;
        use leptos_router::RouteListing;
        use crate::ssr_booking::PipelineLockManager;
        // use tokio::sync::broadcast;
        use crate::{api::consts::EnvVarConfig, utils::notifier::Notifier};
        use std::sync::Arc;
        use tokio::sync::RwLock;
        use tracing::{debug, info};
        use futures::StreamExt;
        use tokio::task;
        use crate::{
            utils::{notifier_event::NotifierEvent, tokio_event_bus::Event as BusEvent},
        };
        use tracing::instrument;

        #[derive(FromRef, Clone, Debug)]
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
            pub liteapi_client: &'static LiteApiHTTPClient,
            pub notifier_for_pipeline: &'static Notifier,
            // pub cookie_key: Key,
            // #[cfg(feature = "oauth-ssr")]
            // pub google_oauth_clients: crate::auth::core_clients::CoreClients,
            // #[cfg(feature = "ga4")]
            // pub grpc_offchain_channel: tonic::transport::Channel,
        }

        #[cfg(feature = "debug_log")]
        impl AppState {
            #[instrument(skip(self))]
            pub async fn setup_debug_event_subscriber(&self) {
                // info!("[DEBUG_EVENT_BUS] Setting up debug event subscriber");
                if let Some(bus) = &self.notifier_for_pipeline.bus {
                info!("[DEBUG_EVENT_BUS] subscribed to eventbus");

                let subscribe_to_all_events = NotifierEvent::subscribe_all_steps_pattern();
                info!("[DEBUG_EVENT_BUS] subscribing to all events pattern  - {subscribe_to_all_events}");
                    let (_, receiver) = bus.subscribe(subscribe_to_all_events).await;

                    // Spawn a task to log all events
                    task::spawn(async move {
                        let mut stream = receiver;
                        // info!("[DEBUG_EVENT_BUS] waiting for event from eventbus");

                        while let Some(event) = stream.recv().await {
                            let BusEvent { topic, payload: NotifierEvent { event_type, step_name, order_id, email, .. } } = event;
                            info!(
                                "[DEBUG_EVENT_BUS] Received event - Topic: {}, Type: {:?}, Step: {:?}, Order: {}, Email: {}",
                                topic, event_type, step_name, order_id, email
                            );
                        }
                    });
                }
            }
        }
    }
}
