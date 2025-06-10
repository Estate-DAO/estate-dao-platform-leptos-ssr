use cfg_if::cfg_if;

pub mod api_client;
pub mod client_side_api;

pub mod provab;

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
