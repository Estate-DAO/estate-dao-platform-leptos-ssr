#![allow(unused_variables)]
#![allow(unused_imports)]

use cfg_if::cfg_if;
use estate_fe::{api::consts::EnvVarConfig, init::AppStateBuilder, utils::admin::AdminCanisters};
cfg_if! {
    if #[cfg(feature = "ssr")] {
        use axum::{
            body::Body as AxumBody,
            extract::{Path, State},
            http::Request,
            response::{IntoResponse, Response, sse::{Event, Sse, KeepAlive}},
            routing::get, 
            Router
        };
        
        use axum_extra::TypedHeader;
        use futures::stream::{self, Stream};
        use std::{convert::Infallible, path::PathBuf, time::Duration};
        use tokio_stream::StreamExt as _;
        use tokio::sync::broadcast;
        use std::sync::Arc;

        use leptos::*;
        use leptos::{get_configuration, logging::log, provide_context};
        use leptos_axum::handle_server_fns_with_context;
        use leptos_axum::{generate_route_list, LeptosRoutes};

        use estate_fe::app::*;
        use estate_fe::fallback::file_and_error_handler;
        use estate_fe::state::AppState;


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
        
        // #[get("/api/events")]
        // async fn events() -> impl Responder {
        //     use crate::counters::ssr_imports::*;
        //     use futures::StreamExt;
        
        //     let stream = futures::stream::once(async {
        //         crate::counters::get_server_count().await.unwrap_or(0)
        //     })
        //     .chain(COUNT_CHANNEL.clone())
        //     .map(|value| {
        //         Ok(web::Bytes::from(format!(
        //             "event: message\ndata: {value}\n\n"
        //         ))) as Result<web::Bytes>
        //     });
        //     HttpResponse::Ok()
        //         .insert_header(("Content-Type", "text/event-stream"))
        //         .streaming(stream)
        // }
        
        async fn sse_handler(
            State(state): State<Arc<AppState>>,
        ) -> Sse<impl Stream<Item = Result<Event, axum::BoxError>>> {
            let mut count_rx = state.count_tx.subscribe();
        
            let stream = async_stream::stream! {
                // Send the initial count
                let initial_count = get_server_count().await.unwrap_or(0);
                yield Ok(Event::default().data(initial_count.to_string()));
        
                // Listen for count updates
                while let Ok(count) = count_rx.recv().await {
                    yield Ok(Event::default().data(count.to_string()));
                }
            };
        
            Sse::new(stream).keep_alive(KeepAlive::default())
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
									
			let (count_tx, _) = broadcast::channel(16);
	
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
				.route("/sse/events", get(sse_handler))
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

// async fn sse_handler(
//     TypedHeader(user_agent): TypedHeader<headers::UserAgent>,
// ) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
//     println!("`{}` connected", user_agent.as_str());

//     // A `Stream` that repeats an event every second
//     //
//     // You can also create streams from tokio channels using the wrappers in
//     // https://docs.rs/tokio-stream
//     let stream = stream::repeat_with(|| Event::default().data("hi!"))
//         .map(Ok)
//         .throttle(Duration::from_secs(1));
    
//     let stream = stream::unfold(0, |state| async move {
//         match state {
//             0 => {
//                 // Simulate Task 1
//                 sleep(Duration::from_secs(1)).await;
//                 Some((Ok(Event::default().data("Task 1 completed")), 1))
//             }
//             1 => {
//                 // Simulate Task 2
//                 sleep(Duration::from_secs(2)).await;
//                 Some((Ok(Event::default().data("Task 2 completed")), 2))
//             }
//             2 => {
//                 // Simulate Task 3
//                 sleep(Duration::from_secs(1)).await;
//                 Some((Ok(Event::default().data("Task 3 completed")), 3))
//             }
//             _ => None, // End the stream after 3 events
//         }
//     });

//     Sse::new(stream).keep_alive(KeepAlive::default())
//     // Sse::new(stream).keep_alive(
//     //     axum::response::sse::KeepAlive::new()
//     //         .interval(Duration::from_secs(1))
//     //         .text("keep-alive-text"),
//     // )
// }

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for a purely client-side app
    // see lib.rs for hydration function instead
}
