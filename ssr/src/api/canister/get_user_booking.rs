use crate::canister::backend::{self, Booking};
use crate::utils::admin::admin_canister;
// use leptos::logging::log;
use crate::log;
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

#[server(GetBookingById)]
pub async fn get_booking_by_id_backend(
    booking_id: backend::BookingId,
) -> Result<Option<Booking>, ServerFnError> {
    let adm_cans = admin_canister();

    let backend_cans = adm_cans.backend_canister().await;

    let result = backend_cans.get_booking_by_id(booking_id).await?;

    log!("get_booking_by_id_backend - {result:#?}");

    Ok(result)
}
