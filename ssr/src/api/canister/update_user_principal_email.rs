use crate::{
    api::auth::{extract_identity_impl::extract_new_identity, types::NewIdentity},
    canister::backend::{self, Result_},
    utils::admin::admin_canister,
};
use candid::Principal;
use leptos::*;

// #[server(UpdateUserPrincipalEmailMapping)]
pub async fn update_user_principal_email_mapping_in_canister(
    user_email: String,
) -> Result<String, ServerFnError> {
    call_update_user_principal_email_mapping_in_canister(user_email)
        .await
        .map_err(ServerFnError::ServerError)
}

pub async fn call_update_user_principal_email_mapping_in_canister(
    user_email: String,
) -> Result<String, String> {
    // Extract the user identity from server-side cookies
    let id_wire = match extract_new_identity().await {
        Ok(Some(identity)) => identity.id_wire,
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

    // Use admin canister to make the backend call
    let adm_cans = admin_canister();
    let backend_cans = adm_cans.backend_canister().await;

    let result = backend_cans
        .update_user_principal_email_index(principal, user_email.clone())
        .await
        .map_err(|e| e.to_string())?;

    match result {
        Result_::Ok(status) => {
            crate::log!(
                "Successfully updated user principal email mapping: {}",
                status
            );
            Ok(status)
        }
        Result_::Err(e) => {
            crate::log!("Failed to update user principal email mapping: {}", e);
            Err(e)
        }
    }
}
