use axum::response::IntoResponse;
use http::header;
use leptos_axum::{extract_with_state, ResponseOptions};
use leptos_use::SameSite;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tower_cookies::Cookie;

use crate::api::auth::types::LoginProvider;
use crate::api::auth::types::{YralOAuthClient, CSRF_TOKEN_COOKIE, PKCE_VERIFIER_COOKIE};
use axum_extra::extract::{cookie::Key, PrivateCookieJar};
use leptos::{expect_context, ServerFnError};
use openidconnect::LoginHint;
use openidconnect::Nonce;
use openidconnect::{
    core::{CoreAuthenticationFlow, CoreIdTokenVerifier, CoreTokenIntrospectionResponse},
    CsrfToken, PkceCodeChallenge, Scope, StandardTokenResponse,
};

pub fn token_verifier() -> CoreIdTokenVerifier<'static> {
    // TODO: use real impl
    CoreIdTokenVerifier::new_insecure_without_verification()
}

#[derive(Serialize, Deserialize)]
struct OAuthState {
    pub csrf_token: CsrfToken,
    pub client_redirect_uri: Option<String>,
}

pub async fn yral_auth_url_impl(
    oauth2: YralOAuthClient,
    login_hint: String,
    provider: LoginProvider,
    client_redirect_uri: Option<String>,
) -> Result<String, ServerFnError> {
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let oauth_state = OAuthState {
        csrf_token: CsrfToken::new_random(),
        client_redirect_uri,
    };

    let oauth2_request = oauth2
        .authorize_url(
            CoreAuthenticationFlow::AuthorizationCode,
            move || CsrfToken::new(serde_json::to_string(&oauth_state).unwrap()),
            Nonce::new_random,
        )
        .add_scope(Scope::new("openid".into()))
        .set_pkce_challenge(pkce_challenge)
        .set_login_hint(LoginHint::new(login_hint));

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

    let key: Key = expect_context();
    let mut jar: PrivateCookieJar = extract_with_state(&key).await?;

    let cookie_life = Duration::from_secs(60 * 10).try_into().unwrap(); // 10 minutes
    let pkce_cookie = Cookie::build((PKCE_VERIFIER_COOKIE, pkce_verifier.secret().clone()))
        .same_site(SameSite::None)
        .path("/")
        .max_age(cookie_life)
        .build();
    jar = jar.add(pkce_cookie);

    let csrf_cookie = Cookie::build((CSRF_TOKEN_COOKIE, oauth_csrf_token.secret().clone()))
        .same_site(SameSite::None)
        .path("/")
        .max_age(cookie_life)
        .build();
    jar = jar.add(csrf_cookie);

    let resp: ResponseOptions = expect_context();
    set_cookies(&resp, jar);

    Ok(auth_url.to_string())
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
