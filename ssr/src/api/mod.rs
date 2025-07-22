use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "ssr")] {
        pub mod api_client;
        pub mod auth;
    }
}

pub mod client_side_api;

pub mod provab;
pub use provab::{DeserializableInput, ProvabReq, ProvabReqMeta};

cfg_if! {
    if #[cfg(feature = "mock-provab")]{
pub mod mock;
    }
}

mod types;
pub use types::*;

pub mod consts;

pub mod payments;
pub use payments::ports::{FailureGetPaymentStatusResponse, SuccessGetPaymentStatusResponse};

pub mod canister;

pub mod liteapi;
