use estate_fe::{
    api::client_side_api::UpdateUserPrincipalEmailRequest, canister::backend::Result_,
    utils::admin::admin_canister,
};

use crate::server_functions_impl_custom_routes::generate_error_response;

use super::parse_json_request;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use estate_fe::view_state_layer::AppState;
use serde_json::json;

// this runs  on the server
#[tracing::instrument(skip(_state))]
pub async fn update_user_principal_email_mapping_in_canister_fn_route(
    State(_state): State<AppState>,
    body: String,
) -> Result<Response, Response> {
    // tracing::info!("EMAIL_PRINCIPAL_MAPPING: Starting update_user_principal_email_mapping_in_canister_fn_route");
    // tracing::info!("EMAIL_PRINCIPAL_MAPPING: Request body length: {}", body.len());

    let request: UpdateUserPrincipalEmailRequest = parse_json_request(&body)?;
    let principal = request.principal;
    let user_email = request.user_email;

    // tracing::info!("EMAIL_PRINCIPAL_MAPPING: Parsed request - principal: {}, email: {}", principal, user_email);

    // // Use admin canister to make the backend call
    // tracing::info!("EMAIL_PRINCIPAL_MAPPING: Getting admin canister instance");
    let adm_cans = admin_canister();

    // tracing::info!("EMAIL_PRINCIPAL_MAPPING: Getting backend canister connection");
    let backend_cans = adm_cans.backend_canister().await;

    // tracing::info!("EMAIL_PRINCIPAL_MAPPING: Making canister call to update_user_principal_email_index");
    let result = backend_cans
        .update_user_principal_email_index(principal, user_email.clone())
        .await
        .map_err(|e| {
            tracing::error!(
                "EMAIL_PRINCIPAL_MAPPING: Canister call failed with error: {}",
                e
            );
            generate_error_response(&e.to_string())
        })?;

    tracing::info!("EMAIL_PRINCIPAL_MAPPING: Canister call completed, processing result");

    match result {
        Result_::Ok(status) => {
            tracing::info!("EMAIL_PRINCIPAL_MAPPING: Success - status: {}", status);
            Ok((StatusCode::OK, status).into_response())
        }
        Result_::Err(e) => {
            tracing::error!("EMAIL_PRINCIPAL_MAPPING: Backend returned error: {}", e);
            Err(generate_error_response(&e))
        }
    }
}
