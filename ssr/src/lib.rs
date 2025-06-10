#![allow(unused_variables)]
#![allow(unused_imports)]

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
