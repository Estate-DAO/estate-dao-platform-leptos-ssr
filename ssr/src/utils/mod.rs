pub mod admin;
pub mod app_reference;
pub mod backend_default_impl;
pub mod backend_integration_helpers;
pub mod booking_backend_conversions;
pub mod booking_id;
mod colored_print;
pub mod date;
pub mod host;
pub mod ic;
pub mod icon;
pub mod parent_resource;
pub mod query_params;
pub mod responsive;
pub mod route;
pub mod search_action;
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

use std::future::Future;

pub use backend_default_impl::*;
pub use backend_integration_helpers::*;
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

#[cfg(not(feature = "hydrate"))]
pub fn send_wrap<Fut: Future + Send>(
    t: Fut,
) -> impl Future<Output = <Fut as Future>::Output> + Send {
    t
    // send_wrapper::SendWrapper::new(t)
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
