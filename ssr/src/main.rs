#![allow(unused_variables)]
#![allow(unused_imports)]

use cfg_if::cfg_if;
use estate_fe::{
    api::{consts::EnvVarConfig, Provab},
    init::{get_provab_client, initialize_provab_client, AppStateBuilder},
    ssr_booking::{
        mock_handler::MockStep, payment_handler::GetPaymentStatusFromPaymentProvider,
        pipeline::process_pipeline, pipeline_lock::PipelineLockManager, SSRBookingPipelineStep,
    },
    utils::{admin::AdminCanisters, notifier::Notifier, sort_json::sort_json},
};

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use axum::{
            body::Body as AxumBody,
            extract::{Path, State,ConnectInfo},
            http::Request,
            response::{IntoResponse, Response, sse::{Event, Sse, KeepAlive}},
        };
        use axum::{routing::get, Router, routing::post};

        use leptos::*;
        use leptos::{get_configuration, logging::log, provide_context};
        use leptos_axum::handle_server_fns_with_context;
        use leptos_axum::{generate_route_list, LeptosRoutes};

        use estate_fe::app::*;
        use estate_fe::fallback::file_and_error_handler;
        use estate_fe::state::AppState;
        use axum::{http::{StatusCode, HeaderMap} };
        use axum::body::Bytes;
        use serde_json::Value;
        use hmac::{Hmac, Mac};
        use sha2::Sha512;
        use tracing::{info, error, debug};
        type HmacSha512 = Hmac<Sha512>;
        use std::net::{IpAddr, SocketAddr};
        use futures::stream::{self, Stream};
        use std::{convert::Infallible, path::PathBuf, time::Duration};
        use tokio_stream::StreamExt as _;
        use tokio::sync::broadcast;
        use std::sync::Arc;
        use estate_fe::ssr_booking::ServerSideBookingEvent;

        pub async fn server_fn_handler(
            State(app_state): State<AppState>,
            path: Path<String>,
            request: Request<AxumBody>,
        ) -> impl IntoResponse {
            log!("{:?}", path);

            handle_server_fns_with_context(
                move || {
                    provide_context(app_state.env_var_config.clone());
                    provide_context(AdminCanisters::from_env());

                    // provide a single instance of provab client so that connection pooling can be used
                    // creating a new client for each reqwest causes new TCP connection each time
                    // This results in TCP handshake, slow-start, causing considerable latency!
                    provide_context(app_state.provab_client.clone());
                },
                request,
            )
            .await
        }

        pub async fn leptos_routes_handler(
            State(app_state): State<AppState>,
            req: Request<AxumBody>,
        ) -> Response {
            let handler = leptos_axum::render_route_with_context(
                app_state.leptos_options.clone(),
                app_state.routes.clone(),
                move || {
                                        // provide_context(app_state.canisters.clone());
                    // #[cfg(feature = "backend-admin")]
                    // provide_context(app_state.admin_canisters.clone());
                    // #[cfg(feature = "cloudflare")]
                    // provide_context(app_state.cloudflare.clone());
                    // provide_context(app_state.kv.clone());
                    // provide_context(app_state.cookie_key.clone());
                    // #[cfg(feature = "oauth-ssr")]
                    // provide_context(app_state.google_oauth_clients.clone());

                    // #[cfg(feature = "ga4")]
                    // provide_context(app_state.grpc_offchain_channel.clone());
                    provide_context(app_state.env_var_config.clone());
                },
                App,
            );
            handler(req).await.into_response()
        }

        // async fn sse_handler(
        //     State(state): State<AppState>,
        // ) -> Sse<impl Stream<Item = Result<Event, axum::BoxError>>> {
        //     let mut count_rx = state.count_tx.subscribe();

        //     let stream = async_stream::stream! {
        //         // Send the initial count
        //         let initial_count = get_server_count().await.unwrap_or(0);
        //         yield Ok(Event::default().data(initial_count.to_string()));

        //         // Listen for count updates
        //         while let Ok(count) = count_rx.recv().await {
        //             yield Ok(Event::default().data(count.to_string()));
        //         }
        //     };

        //     Sse::new(stream).keep_alive(KeepAlive::default())
        // }


        async fn start_ssr_booking_processing_pipeline(payload: &Value, state: &AppState) {
            let payment_id = payload["payment_id"].as_str();
            let order_id = payload["order_id"].as_str().unwrap_or_default();
            // let order_description = payload["order_description"].as_str().unwrap_or_default().to_string();

            if !state.pipeline_lock_manager.try_acquire_lock(payment_id, order_id) {
                debug!("Pipeline already running for payment_id: {:?}, order_id: {}", payment_id, order_id);
                return;
            }

            let notifier = Notifier::with_bus();
            let payment_status_step = SSRBookingPipelineStep::PaymentStatus(GetPaymentStatusFromPaymentProvider);
            let mock_step = SSRBookingPipelineStep::Mock(MockStep::default());

            let event = ServerSideBookingEvent {
                payment_id: payment_id.map(String::from),
                order_id: order_id.to_string(),
                provider: "nowpayments".to_string(),
                user_email: String::new(),
                payment_status: None,
                backend_payment_status: Some(String::from("started_processing")),
            };

            debug!("ServerSideBookingEvent = {:?}", event);

            let lock_manager = state.pipeline_lock_manager.clone();
            let payment_id = payment_id.map(String::from);
            let order_id = order_id.to_string();

            tokio::spawn(async move {
                let result = process_pipeline(event, &[payment_status_step, mock_step], Some(&notifier)).await;

                lock_manager.release_lock(payment_id.as_deref(), &order_id);

                match result {
                    Ok(_) => debug!("Pipeline processed successfully"),
                    Err(e) => error!("Pipeline processing failed: {:?}", e),
                }
            });
        }

        async fn nowpayments_webhook(
            ConnectInfo(remote_addr): ConnectInfo<SocketAddr>,
            State(state): State<AppState>,
            headers: HeaderMap,
            body: Bytes,
        ) -> (StatusCode, &'static str) {
            debug!("Received NowPayments webhook request from {}", remote_addr);
            // todo see scratchpad_me.md for more security hardening

            let signature = match headers.get("x-nowpayments-sig") {
                Some(sig) => sig,
                None => {
                    error!("Missing x-nowpayments-sig header");
                    return (StatusCode::BAD_REQUEST, "Signature missing");
                }
            };
            let signature = match signature.to_str() {
                Ok(s) => s,
                Err(_) => {
                    error!("Invalid signature header format");
                    return (StatusCode::BAD_REQUEST, "Invalid signature format");
                }
            };

            let payload: Value = match serde_json::from_slice(&body) {
                Ok(val) => val,
                Err(e) => {
                    error!("Failed to parse JSON body: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, "JSON parsing error");
                }
            };
            info!("Parsed JSON payload: {:?}", payload);

            let sorted_payload = sort_json(&payload);
            let payload_str = serde_json::to_string(&sorted_payload)
                        .unwrap_or_default();

            start_ssr_booking_processing_pipeline(&payload, &state).await;
              // debug!("Computing HMAC-SHA512 signature for sorted payload: {}", payload_str);

            // 4. Compute HMAC-SHA512 signature
            // let mut mac = HmacSha512::new_from_slice(state.env_var_config.ipn_secret.as_bytes())
            //             .expect("HMAC key creation failed");
            // mac.update(payload_str.as_bytes());
            // let computed_hmac = mac.finalize().into_bytes();
            // let computed_hex = hex::encode(computed_hmac);

            // 5. Compare signatures
            // if computed_hex.eq(signature) {
            //     info!("NowPayments webhook signature verified successfully");
            //     (StatusCode::OK, "OK")
            // } else {
            //     error!("Signature verification failed: expected {}, got {}", computed_hex, signature);
            //     (StatusCode::BAD_REQUEST, "Invalid signature")
            // }
            //
            // if mac.verify_slice(provided_signature_bytes).is_err() {
            //     // Signature didn't match (constant-time check internally)
            //     tracing::error!("Invalid webhook signature");
            //     return StatusCode::BAD_REQUEST;
            // }
            (StatusCode::OK, "ok")
        }

        #[tokio::main]
        async fn main() {
            better_panic::install();
            tracing_subscriber::fmt::init();

            // initialize provab client
            initialize_provab_client();

            let conf = get_configuration(None).await.unwrap();
            let leptos_options = conf.leptos_options;
            let addr = leptos_options.site_addr;
            let routes = leptos_query::with_query_suppression(|| leptos_axum::generate_route_list(App));

            // let (count_tx, count_rx) = broadcast::channel(100);

            let res = AppStateBuilder::new(leptos_options, routes.clone(), get_provab_client())
            .build()
            .await;

            let trace_layer = tower_http::trace::TraceLayer::new_for_http().make_span_with(
                |request: &axum::extract::Request<_>| {
                    let uri = request.uri().to_string();
                    tracing::info_span!("http_request", method = ?request.method(), uri)
                },
            );

            let app = Router::new()
                .route(
                    "/api/*fn_name",
                    get(server_fn_handler).post(server_fn_handler),
                )
                .route("/ipn/webhook", post(nowpayments_webhook))
                .leptos_routes_with_handler(routes, get(leptos_routes_handler))
                .fallback(file_and_error_handler)
                .layer(trace_layer)
                .with_state(res);

            let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
            logging::log!("listening on http://{}", &addr);

            axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
                .await
                .unwrap();

        }

    }
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for a purely client-side app
    // see lib.rs for hydration function instead
}
