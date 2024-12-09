use crate::canister::backend::Booking;
use crate::utils::admin::admin_canister;
use leptos::logging::log;
use leptos::*;

#[server(GreetBackend)]
pub async fn get_user_booking_backend(
    email: String,
) -> Result<Option<Vec<Booking>>, ServerFnError> {
    let adm_cans = admin_canister();

    let backend_cans = adm_cans.backend_canister().await;

    let result = backend_cans.get_user_bookings(email).await?;

    log!("{result:#?}");

    Ok(result)
}
