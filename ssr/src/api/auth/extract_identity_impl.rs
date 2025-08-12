use yral_types::delegated_identity::DelegatedIdentityWire;

use crate::api::{auth::types::NewIdentity, client_side_api::ClientSideApiClient};
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
// #[server(ExtractNewIdentity)]
pub async fn extract_new_identity() -> Result<Option<NewIdentity>, ServerFnError> {
    let client = ClientSideApiClient::new();
    client
        .extract_new_identity()
        .await
        .map_err(|e| ServerFnError::new(e))
}
