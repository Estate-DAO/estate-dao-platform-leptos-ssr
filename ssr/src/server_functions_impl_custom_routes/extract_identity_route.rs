use axum::response::Response;
use axum::{body::to_bytes, response::IntoResponse};
use axum::{body::Body, extract::State, http::Request};
use axum_extra::extract::SignedCookieJar;
use estate_fe::{
    api::{
        auth::{
            auth_url::{no_op_nonce_verifier, token_verifier},
            types::{NewIdentity, YralOAuthClient},
        },
        consts::yral_auth::REFRESH_TOKEN_COOKIE,
    },
    view_state_layer::AppState,
};
use http::StatusCode;
use oauth2::{reqwest::async_http_client, RefreshToken};
use serde_json::json;
use yral_types::delegated_identity::DelegatedIdentityWire;

fn generate_error_response(error: &str) -> Response {
    let error_response = json!({
        "error": error
    });
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        error_response.to_string(),
    )
        .into_response()
}

pub async fn extract_identity_impl_server_fn_route(
    State(state): State<AppState>,
    request: Request<Body>,
) -> Result<Response, Response> {
    // Split the request into parts and body
    let (mut parts, body) = request.into_parts();

    // Extract body as string
    let body_bytes = to_bytes(body, usize::MAX).await.map_err(|e| {
        tracing::error!("Failed to read request body: {:?}", e);
        (StatusCode::BAD_REQUEST, "Failed to read request body").into_response()
    })?;
    let body = String::from_utf8(body_bytes.to_vec()).map_err(|e| {
        tracing::error!("Invalid UTF-8 in request body: {:?}", e);
        (StatusCode::BAD_REQUEST, "Invalid request body encoding").into_response()
    })?;

    // Extract cookie jars from the request using helper
    let cookie_key = state.cookie_key.clone();
    let (private_jar, signed_jar) =
        extract_cookie_jars(&mut parts, &cookie_key)
            .await
            .map_err(|(status, msg)| {
                tracing::error!("[OAUTH_DEBUG] Cookie extraction failed: {}", msg);
                (status, msg).into_response()
            })?;

    let oauth_client = state.yral_oauth_client.clone();
    let identity = extract_identity_impl(&signed_jar, oauth_client)
        .await
        .map_err(|e| {
            tracing::error!("[OAUTH_DEBUG] OAuth implementation failed: {:?}", e);
            generate_error_response(&format!("OAuth authentication failed: {}", e))
        })?
        .ok_or({
            tracing::error!("Identity not found in cookie. no refresh token set?");
            generate_error_response("Identity not found in cookie. no refresh token set?")
        })?;

    // Create NewIdentity response
    let new_identity = NewIdentity {
        id_wire: identity,
        // todo (2025-08-10): do the username logic via regex. see how it is done in the auth_url.rs
        fallback_username: Some(String::from("username")),
    };

    // Return JSON response
    let response = serde_json::to_string(&new_identity)
        .map_err(|e| generate_error_response(&format!("Response serialization failed: {}", e)))?;

    Ok((StatusCode::OK, response).into_response())
}

use leptos::ServerFnError;

use crate::server_functions_impl_custom_routes::extract_cookie_jars;
pub async fn extract_identity_impl(
    jar: &SignedCookieJar,
    yral_oauth_client: YralOAuthClient,
) -> Result<Option<DelegatedIdentityWire>, ServerFnError> {
    let Some(refresh_token) = jar.get(REFRESH_TOKEN_COOKIE) else {
        return Ok(None);
    };

    let token_res = yral_oauth_client
        .exchange_refresh_token(&RefreshToken::new(refresh_token.value().to_string()))
        .request_async(async_http_client)
        .await?;

    let id_token = token_res
        .extra_fields()
        .id_token()
        .expect("Yral Auth V2 must return an ID token");
    let id_claims = id_token.claims(&token_verifier(), no_op_nonce_verifier)?;
    let identity = id_claims.additional_claims().ext_delegated_identity.clone();

    Ok(Some(identity))
}
