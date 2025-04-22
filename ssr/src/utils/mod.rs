pub mod admin;
pub mod app_reference;
pub mod backend_default_impl;
pub mod booking_id;
mod colored_print;
pub mod date;
pub mod host;
pub mod ic;
pub mod icon;
pub mod parent_resource;
pub mod route;
pub mod sort_json;

cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {
        pub mod tokio_event_bus;
        pub mod uuidv7;
        pub mod notifier;
        pub mod notifier_event;
        pub mod estate_tracing;
        pub mod event_stream;
    }
}

pub use backend_default_impl::*;
pub use colored_print::*;

pub fn pluralize(count: u32, singular: &str, plural: &str) -> String {
    if count == 1 {
        format!("{} {}", count, singular)
    } else {
        format!("{} {}", count, plural)
    }
}

#[cfg(feature = "debug_log")]
pub fn debug_local_env() {
    use log::debug;

    use crate::api::payments::NowPayments;

    debug!("nowpayments - vars = {:#?}", NowPayments::try_from_env());
}

#[cfg(not(feature = "debug_log"))]
pub fn debug_local_env() {}
