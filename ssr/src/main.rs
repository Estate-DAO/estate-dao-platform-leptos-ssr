#![allow(unused_variables)]
#![allow(unused_imports)]

use cfg_if::cfg_if;
use estate_fe::{
    api::consts::EnvVarConfig,
    init::AppStateBuilder,
    utils::{admin::AdminCanisters, sort_json::sort_json},
};

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use axum::{
            body::Body as AxumBody,
            extract::{Path, State},
            http::Request,
            response::{IntoResponse, Response},
        };
        use axum::{routing::get, Router};

        use leptos::*;
        use leptos::{get_configuration, logging::log, provide_context};
        use leptos_axum::handle_server_fns_with_context;
        use leptos_axum::{generate_route_list, LeptosRoutes};

        use estate_fe::app::*;
        use estate_fe::fallback::file_and_error_handler;
        use estate_fe::state::AppState;
        use axum::{ routing::post, http::{StatusCode, HeaderMap} };
        use axum::body::Bytes;
        use serde_json::Value;
        use hmac::{Hmac, Mac};
        use sha2::Sha512;
        use tracing::{info, error};
        type HmacSha512 = Hmac<Sha512>;
        use std::net::IpAddr;
        use axum::extract::ConnectInfo;

        // Define whitelist (could be a lazy_static or const once computed)
        static NOWPAYMENTS_ALLOWED_IPS: &[&str] = &[
            "51.89.194.21",
            "51.75.77.69",
            "138.201.172.58",
            "65.21.158.36",
        ];
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



        // todo see scratchpad_me.md for more security hardening
        async fn nowpayments_webhook(
            ConnectInfo(remote_addr): ConnectInfo<std::net::SocketAddr>,
            State(state): State<AppState>,
            headers: HeaderMap,
            body: Bytes,
        ) -> (StatusCode, &'static str) {
            let client_ip = remote_addr.ip();
            // Only allow if in whitelist
            let allowed = NOWPAYMENTS_ALLOWED_IPS.iter().any(|&ip| client_ip == ip.parse::<IpAddr>().unwrap());

            if !allowed {
                tracing::warn!("Rejected webhook from unauthorized IP: {}", client_ip);
                return (StatusCode::FORBIDDEN, "Forbidden");
            }
            // 1. Extract signature from headers
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

            // 2. Parse JSON body
            let payload: Value = match serde_json::from_slice(&body) {
                Ok(val) => val,
                Err(e) => {
                    error!("Failed to parse JSON body: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, "JSON parsing error");
                }
            };

            // 3. Sort JSON object
            let sorted_payload = sort_json(&payload);
            let payload_str = serde_json::to_string(&sorted_payload)
                .unwrap_or_default();

            // 4. Compute HMAC-SHA512 signature
            let mut mac = HmacSha512::new_from_slice(state.env_var_config.ipn_secret.as_bytes())
                .expect("HMAC key creation failed");
            mac.update(payload_str.as_bytes());
            let computed_hmac = mac.finalize().into_bytes();
            let computed_hex = hex::encode(computed_hmac);

            // 5. Compare signatures
            if computed_hex.eq(signature) {
                info!("NowPayments webhook signature verified successfully");
                (StatusCode::OK, "OK")
            } else {
                error!("Signature verification failed: expected {}, got {}", computed_hex, signature);
                (StatusCode::BAD_REQUEST, "Invalid signature")
            }
            //
            // if mac.verify_slice(provided_signature_bytes).is_err() {
            //     // Signature didn't match (constant-time check internally)
            //     tracing::error!("Invalid webhook signature");
            //     return StatusCode::BAD_REQUEST;
            // }
        }


    #[tokio::main]
    async fn main() {
        better_panic::install();
        // A minimal tracing middleware for request logging.
        tracing_subscriber::fmt::init();

        // Setting get_configuration(None) means we'll be using cargo-leptos's env values
        // For deployment these variables are:
        // <https://github.com/leptos-rs/start-axum#executing-a-server-on-a-remote-machine-without-the-toolchain>
        // Alternately a file can be specified such as Some("Cargo.toml")
        // The file would need to be included with the executable when moved to deployment
        let conf = get_configuration(None).await.unwrap();
        let leptos_options = conf.leptos_options;
        let addr = leptos_options.site_addr;
        // let routes = generate_route_list(App);
        let routes = leptos_query::with_query_suppression(|| leptos_axum::generate_route_list(App));

        let res = AppStateBuilder::new(leptos_options, routes.clone())
        .build()
        .await;

        let trace_layer = tower_http::trace::TraceLayer::new_for_http().make_span_with(
            |request: &axum::extract::Request<_>| {
                let uri = request.uri().to_string();
                tracing::info_span!("http_request", method = ?request.method(), uri)
            },
        );


        // build our application with a route
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
        axum::serve(listener, app.into_make_service())
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
