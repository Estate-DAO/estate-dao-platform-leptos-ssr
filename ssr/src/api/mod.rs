use cfg_if::cfg_if;

mod api_client;

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
