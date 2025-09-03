use leptos::{server, ServerFnError, SignalGet, SignalGetUntracked};

use crate::{api::auth::auth_state::AuthState, canister::{backend::{Backend, Booking}, BACKEND_ID}, log, utils::admin::admin_canister};


#[server]
async fn get_user_booking_by_email(email: String) -> Result<Vec<Booking>, ServerFnError> {
    let admin_cans =  admin_canister();
    admin_cans.backend_canister().await.get_user_bookings(email).await.map(|f| f.unwrap_or_default()).map_err(|e| {
        log!("ERROR: get_user_booking_by_email failed - {e}");
        ServerFnError::ServerError(format!("Failed to get failed - {e}"))
    })
}


pub async fn user_get_my_bookings(email: String) -> Result<Vec<Booking>, ServerFnError> {
    get_user_booking_by_email(email).await
}
