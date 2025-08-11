use leptos::{ServerFnError, SignalGet, SignalGetUntracked};

use crate::{api::auth::auth_state::AuthState, canister::backend::Booking, log};

pub async fn user_get_my_bookings(auth_state: AuthState) -> Result<Vec<Booking>, ServerFnError> {
    log!("AUTH_FLOW: user_get_my_bookings called");

    let canister_store = auth_state.get_canisters();
    log!(
        "AUTH_FLOW: user_get_my_bookings - Canister store check - is_some: {}",
        canister_store.is_some()
    );

    // Also check other auth state fields for debugging
    log!("AUTH_FLOW: user_get_my_bookings - Auth state debug - logged_in: {:?}, user_identity: {:?}, user_principal: {:?}", 
        auth_state.is_logged_in_with_oauth().get_untracked(),
        auth_state.user_identity.get_untracked().is_some(),
        auth_state.user_principal_if_available().is_some());

    let canisters = canister_store.ok_or_else(|| {
        log!("AUTH_FLOW: user_get_my_bookings - ERROR - Canister store is empty, user not authenticated");
        ServerFnError::new("User not authenticated - canister store is empty")
    })?;

    let backend_cans = canisters.backend_canister().await;

    let result = backend_cans.my_bookings().await?;

    log!("called - backend_canister.my_bookings() - {result:#?}");

    Ok(result)
}
