use crate::api::{
    auth::{extract_identity_impl::extract_new_identity, types::NewIdentity},
    client_side_api::{self, ClientSideApiClient, UpdateUserPrincipalEmailRequest},
};
use crate::log;
use candid::{types::principal, Principal};
use leptos::*;

pub async fn update_user_principal_email_mapping_in_canister(
    user_email: String,
) -> Result<String, ServerFnError> {
    log!("update_user_principal_email_mapping_in_canister - Updating user principal email mapping - user_email: {}", user_email);
    call_update_user_principal_email_mapping_in_canister(user_email)
        .await
        .map_err(ServerFnError::ServerError)
}

pub async fn call_update_user_principal_email_mapping_in_canister(
    user_email: String,
) -> Result<String, String> {
    // Extract the user identity from server-side cookies
    let id_wire = match extract_new_identity().await {
        Ok(Some(identity)) => {
            log!(
                "update_user_principal_email_mapping_in_canister - Extracted identity: {:#?}",
                identity
            );
            identity.id_wire
        }
        Ok(None) => return Err("User not authenticated - no identity found".to_string()),
        Err(e) => return Err(format!("Failed to extract identity: {}", e)),
    };

    // Extract principal from the identity
    let principal = Principal::self_authenticating(&id_wire.from_key);

    crate::log!(
        "Updating user principal email mapping - principal: {}, email: {}",
        principal,
        user_email
    );

    let client_side_api = ClientSideApiClient::new();
    let result = client_side_api
        .update_user_principal_email_mapping_in_canister_client_side_fn(principal, user_email)
        .await;

    match result {
        Ok(status) => {
            crate::log!(
                "Successfully updated user principal email mapping: {}",
                status
            );
            Ok(status)
        }
        Err(e) => {
            crate::log!("Failed to update user principal email mapping: {}", e);
            Err(e)
        }
    }
}
