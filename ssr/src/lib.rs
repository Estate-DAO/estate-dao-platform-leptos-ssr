#![allow(unused_variables)]
#![allow(unused_imports)]

use std::future::Future;

pub mod api;
pub mod app;
pub mod canister;
pub mod component;
pub mod error_template;
pub mod logging;
pub mod page;
pub mod utils;
pub mod view_state_layer;

pub mod adapters;
pub mod application_services;
pub mod domain;
pub mod ports;
pub mod web_api_translator;

#[cfg(feature = "ga4")]
pub mod event_streaming;

cfg_if::cfg_if! {
    if #[cfg(feature =   "ssr")]{
        pub mod fallback;
        pub mod init;
        pub mod ssr_booking;
        pub mod oauth;
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "hydrate")] {
        #[wasm_bindgen::prelude::wasm_bindgen]
        pub fn hydrate() {
            use crate::app::*;
            // initializes logging using the `log` crate
            _ = console_log::init_with_level(log::Level::Debug);
            console_error_panic_hook::set_once();
            leptos::mount_to_body(App);
        }

    }
}

#[cfg(not(feature = "hydrate"))]
pub fn send_wrap<Fut: Future + Send>(
    t: Fut,
) -> impl Future<Output = <Fut as Future>::Output> + Send {
    t
}

/// Wraps a specific future that is not `Send` when `hydrate` feature is enabled
/// the future must be `Send` when `ssr` is enabled
/// use only when necessary (usually inside resources)
/// if you get a Send related error inside an Action, it probably makes more
/// sense to use `Action::new_local` or `Action::new_unsync`
#[cfg(feature = "hydrate")]
pub fn send_wrap<Fut: Future>(t: Fut) -> impl Future<Output = <Fut as Future>::Output> + Send {
    send_wrapper::SendWrapper::new(t)
}
