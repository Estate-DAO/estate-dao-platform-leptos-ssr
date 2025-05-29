use crate::canister::backend::{self, BookingSummary};
use crate::utils::admin::admin_canister;
// use leptos::logging::log;
use crate::log;
use leptos::*;

#[server(GetAllBookingsBackend)]
pub async fn get_all_bookings_backend() -> Result<Vec<BookingSummary>, ServerFnError> {
    let adm_cans = admin_canister();

    let backend_cans = adm_cans.backend_canister().await;

    let result = backend_cans.get_all_bookings().await?;

    log!("get_all_bookings_backend - {result:#?}");

    Ok(result)
}
