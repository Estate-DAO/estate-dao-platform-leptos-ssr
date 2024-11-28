use cfg_if::cfg_if;
use estate_fe::{api::consts::EnvVarConfig, init::AppStateBuilder};

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use axum::{
            body::Body as AxumBody,
            extract::{Path, State},
            http::Request,
            http::header::HeaderMap,
            response::{IntoResponse, Response},
        };
        use axum::extract::{rejection::JsonRejection, Query};

        use axum::{routing::{get, post}, Router, Json};

        use tracing::info;
        use serde_json::{Value,json};
        use serde::Deserialize;


        use leptos::*;
        use leptos::{get_configuration, logging::log, provide_context};
        use leptos_axum::handle_server_fns_with_context;
        use leptos_axum::{generate_route_list, LeptosRoutes};

        use estate_fe::app::*;
        use estate_fe::fallback::file_and_error_handler;
        use estate_fe::state::AppState;

        #[derive(Debug, Deserialize)]
        pub struct QueryParams {
            // Add specific fields if known
            #[serde(flatten)]
            pub extra: std::collections::HashMap<String, String>,
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


        pub async fn handle_json(
            headers: HeaderMap,
            Query(query): Query<QueryParams>,
            Json(payload): Json<Value>
        ) -> Result<Json<Value>, JsonRejection>{

            // Convert headers to a more loggable format
            let headers: std::collections::HashMap<_, _> = headers
            .iter()
            .map(|(k, v)| (k.as_str(), v.to_str().unwrap_or("Invalid header value")))
            .collect();


            println!("Query parameters: {:?}", query);
            println!("Received payload: {}", payload);
            println!("Received headers: {:#?}", headers);

             // Structured logging with tracing
            //  info!(
            //     query = ?query,
            //     payload = ?payload,
            //     headers = ?headers,
            //     "Received request"
            // );


            Ok(Json(json!({
                "status": 1,
                "message": "payload_received"
            })))
        }


    #[tokio::main]
    async fn main() {
        better_panic::install();

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

        // build our application with a route
        let app = Router::new()
            .route(
                "/api/*fn_name",
                get(server_fn_handler).post(server_fn_handler),
            )
            .route("/webhooks", post(handle_json))
            .leptos_routes_with_handler(routes, get(leptos_routes_handler))
            .fallback(file_and_error_handler)
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
