#![allow(unused_variables)]
#![allow(unused_imports)]

use cfg_if::cfg_if;
use estate_fe::{
    api::provab::Provab,
    api::{
        consts::EnvVarConfig,
        payments::{service::PaymentServiceImpl, NowPayments},
    },
    init::{get_provab_client, initialize_provab_client, AppStateBuilder},
    ssr_booking::{
        booking_handler::MakeBookingFromBookingProvider,
        email_handler::SendEmailAfterSuccessfullBooking,
        get_booking_from_backend::GetBookingFromBackend, mock_handler::MockStep,
        payment_handler::GetPaymentStatusFromPaymentProvider, pipeline::process_pipeline,
        pipeline_lock::PipelineLockManager, SSRBookingPipelineStep,
    },
    utils::{
        admin::AdminCanisters,
        app_reference::BookingId,
        booking_id,
        estate_tracing,
        event_stream::event_stream_handler,
        notifier::Notifier,
        sort_json::sort_json, // notification_system::{NOTIFICATION_SYSTEM, Notification},
                              // event_stream::{event_stream_handler, counter_events},
    },
};

/// Detect payment provider from payment ID (sync version for main.rs)
fn detect_payment_provider_sync(payment_id: &Option<String>) -> String {
    if let Some(ref pid) = payment_id {
        if let Ok(provider) = PaymentServiceImpl::detect_provider_from_payment_id(pid) {
            return provider.as_str().to_string();
        }
    }

    // Fallback for unknown providers
    "unknown".to_string()
}

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use estate_fe::api::SsrCityApiProvider;

        use axum::{
            body::Body as AxumBody,
            extract::{Path, State,ConnectInfo},
            http::Request,
            response::{IntoResponse, Response, sse::{Event, Sse, KeepAlive}},
        };
        use axum::{middleware, routing::get, Router, routing::post};
        use tower_http::cors::{CorsLayer, Any};

        use leptos::*;
        use leptos::{get_configuration, logging::log, provide_context};
        use leptos_axum::handle_server_fns_with_context;
        use leptos_axum::{generate_route_list, LeptosRoutes};

        use estate_fe::app::*;
        use estate_fe::fallback::file_and_error_handler;
        use estate_fe::view_state_layer::AppState;
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
        use tokio::sync::{broadcast, mpsc};
        use std::sync::Arc;
        use estate_fe::ssr_booking::ServerSideBookingEvent;

        use tracing::{instrument, Instrument};
        use tracing::Level;
        mod sitemap;

        use sitemap::sitemap_handler;

        mod basic_auth;
        use basic_auth::*;
        mod server_functions_impl_custom_routes;

        use server_functions_impl_custom_routes::api_routes;
        use estate_fe::oauth::*;

        // Helper: verify NowPayments HMAC-SHA512 signature (gated by feature)
        #[cfg(not(feature = "debug_log"))]
        fn verify_ipn_signature(state: &AppState, payload: &serde_json::Value, signature: &str) -> bool {
            let sorted = sort_json(payload);
            let s = match serde_json::to_string(&sorted) {
                Ok(s) => s,
                Err(e) => { error!("JSON serialization error: {}", e); return false; }
            };
            let mut mac = match HmacSha512::new_from_slice(state.env_var_config.ipn_secret.as_bytes()) {
                Ok(m) => m,
                Err(e) => { error!("HMAC key creation failed: {}", e); return false; }
            };

            mac.update(s.as_bytes());
            let computed_hex = hex::encode(mac.finalize().into_bytes());

            // 5. Compare signatures using constant-time comparison to prevent timing attacks
            computed_hex.eq(signature)
        }

        #[cfg(feature = "debug_log")]
        fn verify_ipn_signature(_state: &AppState, _payload: &serde_json::Value, _signature: &str) -> bool {
            debug!("Skipping signature verification (debug_log)");
            true
        }

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
                    provide_context(app_state.cookie_key.clone());

                    // provide a single instance of provab client so that connection pooling can be used
                    // creating a new client for each reqwest causes new TCP connection each time
                    // This results in TCP handshake, slow-start, causing considerable latency!
                    // provide_context(app_state.provab_client.clone());
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
                    provide_context(app_state.cookie_key.clone());
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


        #[instrument(skip(state))]
        async fn start_ssr_booking_processing_pipeline(payload: &Value, state: &AppState) {
            let span = tracing::span!(Level::DEBUG, "start_ssr_booking_processing_pipeline");
            let _enter = span.enter();

            let payload_str = serde_json::to_string_pretty(payload).unwrap();
            debug!("payload_str: {payload_str}");


            let payment_id = payload.get("payment_id").and_then(|v| v.as_u64()).map(|id| id.to_string());
            let order_id = payload.get("order_id").and_then(|v| v.as_str()) ;

            // let order_description = payload["order_description"].as_str().unwrap_or_default().to_string();

            debug!("[ssr_booking] payment_id: {:?}, order_id: {:?}", payment_id, order_id);
            if order_id.is_none() {
                debug!("[ssr_booking] No order_id provided for payment_id: {:?}", payment_id);
                return;
            }
            let order_id = order_id.unwrap();

            if !state.pipeline_lock_manager.try_acquire_lock(payment_id.as_deref(), order_id) {
                debug!("[ssr_booking] Pipeline already running for payment_id: {:?}, order_id: {}", payment_id, order_id);
                return;
            }

            // start this notifier in app_state so that you have only one event bus with notifier running
            let notifier = state.notifier_for_pipeline;
            let payment_status_step = SSRBookingPipelineStep::PaymentStatus(GetPaymentStatusFromPaymentProvider);
            let book_room_step = SSRBookingPipelineStep::BookRoom(MakeBookingFromBookingProvider);
            let send_email_step = SSRBookingPipelineStep::SendEmail(SendEmailAfterSuccessfullBooking);
            let get_booking_step = SSRBookingPipelineStep::GetBookingFromBackend(GetBookingFromBackend);
            let mock_step = SSRBookingPipelineStep::Mock(MockStep::default());

            let booking_id_result = BookingId::from_order_id(order_id);

            // let user_email = if let Some(booking_id) = booking_id_result {
            //     booking_id.email.to_string()
            // } else {
            //     // current flow assumes the existence of user email to lookup payment / booking details in backend
            //     error!("Failed to extract user_email from order_id: {}", order_id);
            //     String::new()
            // };


            // current flow assumes the existence of user email to lookup payment / booking details in backend
            if booking_id_result.is_none() {
                error!("Failed to extract user_email from order_id: {}", order_id);
                return;
            }

            let booking_id = booking_id_result.unwrap();
            let user_email = booking_id.email.to_string();

            let event = ServerSideBookingEvent {
                payment_id: payment_id.clone(),
                order_id: order_id.to_string(),
                provider: detect_payment_provider_sync(&payment_id),
                user_email,
                payment_status: None,
                backend_payment_status: Some(String::from("started_processing")),
                backend_booking_status: None,
                backend_booking_struct: None,
            };

            let lock_manager = state.pipeline_lock_manager.clone();
            let order_id = order_id.to_string();

            let current_span = tracing::Span::current();
            tokio::spawn(async move {
                let result = process_pipeline(event, &[payment_status_step, book_room_step, get_booking_step, send_email_step, mock_step], Some(notifier))
                    .instrument(current_span)
                    .await;

                lock_manager.release_lock(payment_id.as_deref(), &order_id);

                match result {
                    Ok(val) => debug!("[ssr_booking] Pipeline processed successfully - {val:#?}"),
                    Err(e) => error!("[ssr_booking] Pipeline processing failed - {e:#?}"),
                }
            });
        }


        #[instrument(skip(state))]
        async fn nowpayments_webhook(
            ConnectInfo(remote_addr): ConnectInfo<SocketAddr>,
            State(state): State<AppState>,
            headers: HeaderMap,
            body: Bytes,
        ) -> (StatusCode, &'static str) {
            debug!("Received NowPayments webhook request from {}", remote_addr);

            // 1. Extract and validate signature from headers
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

            // 2. Parse and validate JSON payload
            let payload: Value = match serde_json::from_slice(&body) {
                Ok(val) => val,
                Err(e) => {
                    error!("Failed to parse JSON body: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, "JSON parsing error");
                }
            };
            debug!("Parsed JSON payload: {:?}", payload);

            // 3-5. Verify signature (skipped if debug_log)
            if !verify_ipn_signature(&state, &payload, &signature) {
                error!("Invalid signature");
                return (StatusCode::BAD_REQUEST, "Invalid signature");
            }
            info!("NowPayments webhook signature verified successfully");
            // 6. Process pipeline
            start_ssr_booking_processing_pipeline(&payload, &state).await;
            (StatusCode::OK, "OK")
        }

        #[tokio::main]
        async fn main() -> anyhow::Result<()> {
            better_panic::install();
            // estate_tracing::init_tracing();

        use telemetry_axum;
        #[cfg(feature = "debug_log")]
        let config_telemetry = telemetry_axum::Config{
            exporter: telemetry_axum::Exporter::All,
            propagate: false,
            ..Default::default()
        };

        // remove otlp endpoint in prod
        #[cfg(not(feature = "debug_log"))]
        let config_telemetry = telemetry_axum::Config{
            exporter: telemetry_axum::Exporter::Stdout,
            propagate: false,
            ..Default::default()
        };

        let (logger_provider, tracer_provider, metrics_provider) =
            telemetry_axum::init_telemetry(&config_telemetry)
                .map_err(|e| anyhow::Error::new(e))?;


            estate_fe::utils::debug_local_env();

            let conf = get_configuration(None).await.unwrap();
            let leptos_options = conf.leptos_options;
            let addr = leptos_options.site_addr;
            let routes = leptos_query::with_query_suppression(|| leptos_axum::generate_route_list(App));

            let res = AppStateBuilder::new(leptos_options, routes.clone())
            .build()
            .await;

            // Start cities polling as an actor with custom intervals
            #[cfg(not(target_arch = "wasm32"))]
            let city_actor_ref = {
                let api_provider = SsrCityApiProvider::new();
                let current_span = tracing::Span::current();
                
                let actor_ref = bg_ractor::start_cities_polling_with_secs(
                    api_provider,
                    60,  // Update cities every 60 seconds
                    3,   // Heartbeat every 3 seconds
                    "city.json".to_string(),
                )
                .instrument(current_span)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to start cities polling: {}", e))?;
                
                info!("Cities polling background actor started with 60s update and 3s heartbeat intervals");
                actor_ref
            };

            let trace_layer = tower_http::trace::TraceLayer::new_for_http()
                .make_span_with(|request: &axum::extract::Request<_>| {
                    let method = request.method();
                    let uri = request.uri();
                    let route = request
                        .extensions()
                        .get::<axum::extract::MatchedPath>()
                        .map(|path| path.as_str())
                        .unwrap_or_else(|| uri.path());

                    tracing::info_span!(
                        "http_request",
                        method = %method,
                        uri = %uri,
                        route = route,
                        // OpenTelemetry semantic conventions
                        otel.name = format!("{} {}", method, route),
                        otel.kind = "server",
                        http.method = %method,
                        http.url = %uri,
                        http.route = route,
                        service.name = "estate_fe",
                    )
                })
                .on_response(|response: &axum::response::Response, latency: std::time::Duration, _span: &tracing::Span| {
                    tracing::info!(
                        status_code = response.status().as_u16(),
                        latency_ms = latency.as_millis(),
                        "request completed"
                    );
                });

            let cors = CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any);


            let app = Router::new()
                .route(
                    "/api/*fn_name",
                    get(server_fn_handler).post(server_fn_handler),
                )
                .route("/auth/google", get(google_auth))
                .route("/app", get(get_app_url))
                .route("/api/user-info", get(api_user_info))
                .route("/auth/google/callback", get(google_callback))
                .route("/auth/logout", get(logout))
                .route("/ipn/webhook", post(nowpayments_webhook))
                .route("/stream/events", get(event_stream_handler))
                .route("/sitemap-index.xml", get(sitemap_handler))
                .nest("/server_fn_api", api_routes())
                .leptos_routes_with_handler(routes, get(leptos_routes_handler))
                .fallback(file_and_error_handler)
                .layer(cors)
                .layer(trace_layer)
                // Protect admin routes with browser challenge
                .layer(
                    middleware::from_fn_with_state(res.clone(),selective_auth_middleware)
                )
                .with_state(res);



                let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
                logging::log!("listening on http://{}", &addr);

                axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
                    .await
                    .unwrap();

                // Stop the city updater actor
                #[cfg(not(target_arch = "wasm32"))]
                {
                    info!("Stopping city updater actor");
                    city_actor_ref.stop(None);
                }
                
                info!("Application shutting down");

                //   cleanup tracing config.

                if let Some(logger_provider) = logger_provider {
                    if let Err(e) = logger_provider.shutdown() {
                        println!("error shutting down logger provider: {e}");
                    }
                }
                if let Err(e) = tracer_provider.shutdown() {
                    println!("error shutting down tracer provider: {e}");
                }
                if let Some(metrics_provider) = metrics_provider {
                    if let Err(e) = metrics_provider.shutdown() {
                        println!("error shutting down metrics provider: {e}");
                    }
                }

                info!("shut down");

            Ok(())
        }

    }
}

#[cfg(not(feature = "ssr"))]
pub fn main() -> anyhow::Result<()> {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for a purely client-side app
    // see lib.rs for hydration function instead
    Ok(())
}
