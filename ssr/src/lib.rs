pub mod app;
pub mod component;
pub mod error_template;
pub mod page;
pub mod state;
pub mod utils;
pub mod api;

#[cfg(feature = "ssr")]
pub mod fallback;

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
