use yral_types::delegated_identity::DelegatedIdentityWire;

use crate::api::{
    auth::{
        auth_state::{auth_state, AuthStateSignal},
        types::NewIdentity,
    },
    client_side_api::ClientSideApiClient,
};
use crate::log;
use leptos::*;

/// Extract the identity from refresh token,
// /// returns None if refresh token doesn't exist
// #[server(endpoint = "extract_identity", input = Json, output = Json)]
// pub async fn extract_identity() -> Result<Option<DelegatedIdentityWire>, ServerFnError> {
//     let client = ClientSideApiClient::new();
//     client
//         .extract_identity()
//         .await
//         .map_err(|e| ServerFnError::new(e))
// }

/// Extract the full NewIdentity (with username and email) from refresh token
/// First checks auth_state.user_identity_cookie, then falls back to client API call
pub async fn extract_new_identity() -> Result<Option<NewIdentity>, ServerFnError> {
    let auth_state_signal: AuthStateSignal = expect_context();
    let auth_state = auth_state_signal.get();
    // Check if identity exists in auth state cookie first
    if let Some(identity) = auth_state.get_user_identity_cookie() {
        log!("auth_state.get_user_identity_cookie(): {:?} ", identity);
        return Ok(Some(identity));
    }

    // Fallback to client API call if not in cookie
    let client = ClientSideApiClient::new();
    client
        .extract_new_identity()
        .await
        .map_err(|e| ServerFnError::new(e))
}
