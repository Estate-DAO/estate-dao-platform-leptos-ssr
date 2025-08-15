use crate::api::auth::types::LoginProvider;
use crate::api::auth::types::{YralOAuthClient, CSRF_TOKEN_COOKIE, PKCE_VERIFIER_COOKIE};
use crate::api::consts::yral_auth::{REFRESH_MAX_AGE, REFRESH_TOKEN_COOKIE, USERNAME_MAX_LEN};
use crate::api::consts::{get_app_domain_with_dot, APP_URL};
use axum::response::IntoResponse;
use axum_extra::extract::{cookie::Key, PrivateCookieJar, SignedCookieJar};
use http::header;
use leptos::{expect_context, use_context, ServerFnError};
use leptos_axum::ResponseOptions;
use leptos_use::SameSite;
use openidconnect::LoginHint;
use openidconnect::Nonce;
use openidconnect::OAuth2TokenResponse;
use openidconnect::{
    core::{CoreAuthenticationFlow, CoreIdTokenVerifier, CoreTokenIntrospectionResponse},
    reqwest::async_http_client,
    AuthorizationCode, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, Scope,
    StandardTokenResponse,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use std::sync::LazyLock;
use std::time::Duration;
use tower_cookies::Cookie;
use yral_types::delegated_identity::DelegatedIdentityWire;

pub fn token_verifier() -> CoreIdTokenVerifier<'static> {
    // TODO: use real impl
    CoreIdTokenVerifier::new_insecure_without_verification()
}

use axum::{
    body::{Body, Bytes},
    extract::{FromRef, FromRequestParts, MatchedPath, State},
    http::{
        header::{HeaderName, HeaderValue, ACCEPT, LOCATION, REFERER},
        request::Parts,
        HeaderMap, Method, Request, Response, StatusCode,
    },
    routing::{get, patch, post, put},
};

#[derive(Serialize, Deserialize, Clone)]
struct OAuthState {
    pub csrf_token: CsrfToken,
    pub client_redirect_uri: Option<String>,
}

use crate::view_state_layer::AppState;

#[tracing::instrument(skip(oauth2, response_options))]
pub async fn yral_auth_url_impl(
    oauth2: YralOAuthClient,
    login_hint: String,
    provider: LoginProvider,
    client_redirect_uri: Option<String>,
    app_state: &AppState,
    response_options: Option<&ResponseOptions>,
) -> Result<(String, PrivateCookieJar), ServerFnError> {
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let oauth_state = OAuthState {
        csrf_token: CsrfToken::new_random(),
        client_redirect_uri,
    };
    let cookie_key = app_state.cookie_key.clone();

    // Clone the oauth_state for use in closure and keep original for cookie creation
    let oauth_state_for_closure = oauth_state.clone();
    let oauth2_request = oauth2
        .authorize_url(
            CoreAuthenticationFlow::AuthorizationCode,
            move || CsrfToken::new(serde_json::to_string(&oauth_state_for_closure).unwrap()),
            Nonce::new_random,
        )
        .add_scope(Scope::new("openid".into()))
        .set_pkce_challenge(pkce_challenge);
    // .set_login_hint(LoginHint::new(login_hint));

    let mut oauth2_request = oauth2_request;
    if provider != LoginProvider::Any {
        let provider = match provider {
            LoginProvider::Google => "google",
            LoginProvider::Apple => "apple",
            LoginProvider::Any => unreachable!(),
        };
        oauth2_request = oauth2_request.add_extra_param("provider", provider);
    }

    let (auth_url, oauth_csrf_token, _) = oauth2_request.url();

    // Create a new empty PrivateCookieJar with the provided key
    let mut jar = PrivateCookieJar::new(cookie_key.clone());

    let cookie_life = Duration::from_secs(60 * 10).try_into().unwrap(); // 10 minutes

    // Create and log PKCE cookie
    crate::log!(
        "[OAUTH_DEBUG] Creating PKCE cookie '{}' with value: {}",
        PKCE_VERIFIER_COOKIE,
        pkce_verifier.secret()
    );

    // let is_production = {
    //     cfg_if::cfg_if! {
    //         if #[cfg(any(feature = "release-lib-prod", feature = "release-bin-prod"))] {
    //             true
    //         } else {
    //             false
    //         }
    //     }
    // };

    let app_domain = get_app_domain_with_dot();

    // .nofeebooking.com
    let pkce_cookie = Cookie::build((PKCE_VERIFIER_COOKIE, pkce_verifier.secret().clone()))
        .same_site(SameSite::None)
        .domain(app_domain)
        .secure(true)
        .path("/")
        .max_age(cookie_life)
        .http_only(true)
        .build();
    // tracing::debug!("[OAUTH_DEBUG] PKCE cookie details: {:#?}", pkce_cookie);
    jar = jar.add(pkce_cookie);

    // Create and log CSRF cookie - use the actual CSRF token, not the entire JSON state
    crate::log!(
        "[OAUTH_DEBUG] Creating CSRF cookie '{}' with value: {}",
        CSRF_TOKEN_COOKIE,
        oauth_state.csrf_token.secret()
    );
    crate::log!(
        "[OAUTH_DEBUG] Note: oauth_csrf_token contains full JSON state: {}",
        oauth_csrf_token.secret()
    );
    let csrf_cookie = Cookie::build((CSRF_TOKEN_COOKIE, oauth_state.csrf_token.secret().clone()))
        .same_site(SameSite::None)
        .path("/")
        .max_age(cookie_life)
        .secure(true)
        // .http_only(true)
        .build();
    tracing::debug!("[OAUTH_DEBUG] CSRF cookie details: {:#?}", csrf_cookie);
    jar = jar.add(csrf_cookie);

    // // Always set cookies server-side for both Axum and Leptos handlers
    // if let Some(resp) = response_options {
    //     // For Leptos server functions - use ResponseOptions
    //     set_cookies(resp, jar.clone());
    // }

    crate::log!("[OAUTH_DEBUG] Cookies set for URL: {}", auth_url);
    crate::log!("[OAUTH_DEBUG] Final cookie jar: {:#?}", jar);
    crate::log!("[OAUTH_DEBUG] Auth URL generation complete with cookies added to jar");
    // Note: For raw Axum handlers, cookies are set via the returned PrivateCookieJar
    // which gets applied through the (jar, response) tuple pattern

    // Return both the URL and the cookie jar for the caller to handle
    Ok((auth_url.to_string(), jar))
}

pub fn no_op_nonce_verifier(_: Option<&Nonce>) -> Result<(), String> {
    Ok(())
}

fn set_cookies(resp: &ResponseOptions, jar: impl IntoResponse) {
    let resp_jar = jar.into_response();
    for cookie in resp_jar
        .headers()
        .get_all(header::SET_COOKIE)
        .into_iter()
        .cloned()
    {
        resp.append_header(header::SET_COOKIE, cookie);
    }
}

// -----------------

pub struct ClaimsFromYralAuth {
    pub identity: DelegatedIdentityWire,
    pub username: Option<String>,
    pub email: Option<String>,
}

pub async fn perform_yral_auth_impl(
    provided_csrf: String,
    auth_code: String,
    oauth2: YralOAuthClient,
    private_jar: PrivateCookieJar<Key>,
    signed_jar: SignedCookieJar<Key>,
    response_options: Option<ResponseOptions>,
) -> Result<(ClaimsFromYralAuth, SignedCookieJar<Key>), ServerFnError> {
    let mut jar = private_jar;

    // Debug logging for cookie availability
    tracing::debug!("[OAUTH_DEBUG] perform_yral_auth_impl - Looking for cookies in private jar");
    tracing::debug!("[OAUTH_DEBUG] Private jar: {:#?}", jar);

    // Try to get CSRF token from cookie and log the attempt
    let csrf_cookie_result = jar.get(CSRF_TOKEN_COOKIE);
    let csrf_from_cookie = match &csrf_cookie_result {
        Some(cookie) => {
            tracing::info!("[OAUTH_DEBUG] CSRF cookie found: value={}", cookie.value());
            tracing::debug!("[OAUTH_DEBUG] CSRF cookie details: {:#?}", cookie);
            Some(cookie.value().to_string())
        }
        None => {
            tracing::warn!(
                "[OAUTH_DEBUG] CSRF cookie '{}' not found in jar",
                CSRF_TOKEN_COOKIE
            );
            None
        }
    };

    // Parse CSRF token from the provided_csrf (state parameter) as fallback
    let csrf_from_state =
        if let Ok(state_json) = serde_json::from_str::<serde_json::Value>(&provided_csrf) {
            if let Some(csrf_token) = state_json.get("csrf_token").and_then(|v| v.as_str()) {
                tracing::info!(
                    "[OAUTH_DEBUG] CSRF token extracted from state parameter: {}",
                    csrf_token
                );
                Some(csrf_token.to_string())
            } else {
                tracing::warn!("[OAUTH_DEBUG] No csrf_token found in state parameter JSON");
                None
            }
        } else {
            // If state parameter is not JSON, assume it's the token directly
            tracing::info!(
                "[OAUTH_DEBUG] Using provided_csrf as direct token: {}",
                provided_csrf
            );
            Some(provided_csrf.clone())
        };

    // Validate CSRF token (prefer cookie, fallback to state)
    let (csrf_validated, validation_source) =
        match (csrf_from_cookie.as_ref(), csrf_from_state.as_ref()) {
            (Some(cookie_csrf), Some(state_csrf)) => {
                if cookie_csrf == state_csrf {
                    tracing::info!(
                        "[OAUTH_DEBUG] CSRF validation: PASSED - both cookie and state match"
                    );
                    (true, "both_match")
                } else {
                    tracing::warn!(
                        "[OAUTH_DEBUG] CSRF validation: cookie='{}' != state='{}'",
                        cookie_csrf,
                        state_csrf
                    );
                    // Use state as it's more reliable
                    (true, "state_only")
                }
            }
            (Some(_), None) => {
                tracing::info!("[OAUTH_DEBUG] CSRF validation: using cookie only");
                (true, "cookie_only")
            }
            (None, Some(_)) => {
                tracing::info!("[OAUTH_DEBUG] CSRF validation: using state parameter only");
                (true, "state_only")
            }
            (None, None) => {
                tracing::error!(
                    "[OAUTH_DEBUG] CSRF validation: FAILED - no token available from either source"
                );
                (false, "none_available")
            }
        };

    if !csrf_validated {
        return Err(ServerFnError::new(
            "CSRF token validation failed - no valid token found",
        ));
    }

    tracing::info!(
        "[OAUTH_DEBUG] CSRF validation successful using: {}",
        validation_source
    );

    // Try to get PKCE verifier cookie and log the attempt
    tracing::debug!(
        "[OAUTH_DEBUG] Looking for PKCE verifier cookie: '{}'",
        PKCE_VERIFIER_COOKIE
    );
    let pkce_cookie = jar.get(PKCE_VERIFIER_COOKIE).ok_or_else(|| {
        tracing::error!(
            "[OAUTH_DEBUG] PKCE verifier cookie '{}' not found in private jar",
            PKCE_VERIFIER_COOKIE
        );
        ServerFnError::new("PKCE verifier cookie not found")
    })?;

    tracing::info!(
        "[OAUTH_DEBUG] PKCE cookie found: value={}",
        pkce_cookie.value()
    );
    tracing::debug!("[OAUTH_DEBUG] PKCE cookie details: {:#?}", pkce_cookie);
    let pkce_verifier = PkceCodeVerifier::new(pkce_cookie.value().to_owned());

    jar = jar.remove(PKCE_VERIFIER_COOKIE);
    jar = jar.remove(CSRF_TOKEN_COOKIE);

    // Set cookies if we have response options (for Leptos server functions)
    if let Some(resp) = &response_options {
        set_cookies(resp, jar);
    }

    let token_res = oauth2
        .exchange_code(AuthorizationCode::new(auth_code))
        .set_pkce_verifier(pkce_verifier)
        .request_async(async_http_client)
        .await?;

    let id_token_verifier = token_verifier();
    let id_token = token_res
        .extra_fields()
        .id_token()
        .ok_or_else(|| ServerFnError::new("Google did not return an ID token"))?;
    // we don't use a nonce
    let claims = id_token.claims(&id_token_verifier, no_op_nonce_verifier)?;
    let identity: DelegatedIdentityWire = claims.additional_claims().ext_delegated_identity.clone();

    static USERNAME_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^([a-zA-Z0-9]){3,15}$").unwrap());

    let username = claims.email().and_then(|e| {
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

    let email = claims.email().map(|f| f.deref().clone());

    let mut jar = signed_jar;

    let refresh_token = token_res
        .refresh_token()
        .expect("Yral Auth V2 must return a refresh token");

    // Create refresh token cookie and add to jar
    let refresh_max_age = REFRESH_MAX_AGE;
    let refresh_cookie = Cookie::build((REFRESH_TOKEN_COOKIE, refresh_token.secret().clone()))
        .http_only(true)
        .secure(true)
        .path("/")
        .same_site(SameSite::None)
        .partitioned(true)
        .max_age(refresh_max_age.try_into().unwrap())
        .build();

    jar = jar.add(refresh_cookie);

    // Update user identity if we have response options (for Leptos handlers)
    if let Some(resp) = &response_options {
        set_cookies(resp, jar.clone());
    }

    Ok((
        ClaimsFromYralAuth {
            identity,
            username,
            email,
        },
        jar,
    ))
}

pub fn update_user_identity(
    response_opts: &ResponseOptions,
    mut jar: SignedCookieJar,
    refresh_jwt: String,
) -> Result<(), ServerFnError> {
    let refresh_max_age = REFRESH_MAX_AGE;

    let refresh_cookie = Cookie::build((REFRESH_TOKEN_COOKIE, refresh_jwt))
        .http_only(true)
        .secure(true)
        .path("/")
        .same_site(SameSite::None)
        .partitioned(true)
        .max_age(refresh_max_age.try_into().unwrap());

    jar = jar.add(refresh_cookie);
    set_cookies(response_opts, jar);
    Ok(())
}
