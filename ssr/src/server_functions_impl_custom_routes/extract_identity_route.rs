use crate::server_functions_impl_custom_routes::{extract_cookie_jars, generate_error_response};
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
        consts::yral_auth::{REFRESH_TOKEN_COOKIE, USERNAME_MAX_LEN},
    },
    view_state_layer::AppState,
};
use http::StatusCode;
use oauth2::{reqwest::async_http_client, RefreshToken};
use openidconnect::{core::CoreGenderClaim, IdTokenClaims};
use regex::Regex;
use serde_json::json;
use std::{ops::Deref, sync::LazyLock};
use yral_types::delegated_identity::DelegatedIdentityWire;

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

    // Extract identity and claims from refresh token
    let Some(refresh_token) = signed_jar.get(REFRESH_TOKEN_COOKIE) else {
        tracing::error!("No refresh token found in cookies");
        return Err(generate_error_response(
            "Identity not found in cookie. no refresh token set?",
        ));
    };

    let token_res = oauth_client
        .exchange_refresh_token(&RefreshToken::new(refresh_token.value().to_string()))
        .request_async(async_http_client)
        .await
        .map_err(|e| {
            tracing::error!("[OAUTH_DEBUG] Failed to exchange refresh token: {:?}", e);
            generate_error_response(&format!("OAuth token exchange failed: {}", e))
        })?;

    let id_token = token_res
        .extra_fields()
        .id_token()
        .expect("Yral Auth V2 must return an ID token");
    let id_claims = id_token
        .claims(&token_verifier(), no_op_nonce_verifier)
        .map_err(|e| {
            tracing::error!("[OAUTH_DEBUG] Failed to extract ID token claims: {:?}", e);
            generate_error_response(&format!("Failed to extract token claims: {}", e))
        })?;
    let identity = id_claims.additional_claims().ext_delegated_identity.clone();

    // Extract username and email from claims using the same pattern as auth_url.rs
    static USERNAME_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^([a-zA-Z0-9]){3,15}$").unwrap());

    let username = id_claims.email().and_then(|e| {
        let mail: String = e.deref().clone();
        let mut username = mail.split_once("@")?.0;
        username = username
            .char_indices()
            .nth(USERNAME_MAX_LEN)
            .map(|(i, _)| &username[..i])
            .unwrap_or(username);

        USERNAME_REGEX
            .is_match(username)
            .then(|| username.to_string())
    });

    let email = id_claims.email().map(|f| f.deref().clone());

    // Create NewIdentity response
    let new_identity = NewIdentity {
        id_wire: identity,
        fallback_username: username,
        email,
    };

    // Return JSON response
    let response = serde_json::to_string(&new_identity)
        .map_err(|e| generate_error_response(&format!("Response serialization failed: {}", e)))?;

    Ok((StatusCode::OK, response).into_response())
}
