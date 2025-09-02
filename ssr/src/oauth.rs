use axum::{extract::State, response::IntoResponse, Json};
use axum_extra::extract::cookie::{Cookie, SignedCookieJar};
use codee::string::JsonSerdeCodec;
use http::StatusCode;
use leptos_use::use_cookie;
use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, RedirectUrl,
    Scope, TokenResponse, TokenUrl,
};
use tower_cookies::cookie::time::Duration;

use crate::{
    api::{
        auth::types::{AppUrl, OidcUser},
        consts::APP_URL,
    },
    view_state_layer::AppState,
};

const CSRF_COOKIE: &str = "g_csrf";
const SESSION_COOKIE: &str = "session"; // signed session cookie

pub async fn get_app_url() -> Json<AppUrl> {
    let env_url = std::env::var("APP_URL").unwrap_or_else(|_| "http://localhost:3002/".into());
    let const_url = APP_URL.to_string();
    Json(AppUrl { env_url, const_url })
}

fn build_google_client() -> BasicClient {
    let client_id = std::env::var("GOOGLE_CLIENT_ID").expect("GOOGLE_CLIENT_ID not set");
    let client_secret =
        std::env::var("GOOGLE_CLIENT_SECRET").expect("GOOGLE_CLIENT_SECRET not set");
    let redirect = std::env::var("GOOGLE_REDIRECT_URL").expect("GOOGLE_REDIRECT_URL not set");

    BasicClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
        AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string()).unwrap(),
        Some(TokenUrl::new("https://oauth2.googleapis.com/token".to_string()).unwrap()),
    )
    .set_redirect_uri(RedirectUrl::new(redirect).unwrap())
}

/// GET /auth/google
pub async fn google_auth(
    State(_state): State<AppState>,
    jar: SignedCookieJar,
) -> impl IntoResponse {
    let client = build_google_client();

    // PKCE (recommended with public clients)
    let (pkce_challenge, pkce_verifier) = oauth2::PkceCodeChallenge::new_random_sha256();

    let (auth_url, csrf) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("openid".into()))
        .add_scope(Scope::new("email".into()))
        .add_scope(Scope::new("profile".into()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    // Persist CSRF token in a signed cookie
    let csrf_cookie = Cookie::build((CSRF_COOKIE, csrf.secret().to_string()))
        .path("/")
        .http_only(true)
        .finish();

    let jar = jar.add(csrf_cookie);

    // Persist the PKCE verifier
    let pkce_cookie = Cookie::build((
        format!("{CSRF_COOKIE}_pkce"),
        pkce_verifier.secret().to_string(),
    ))
    .path("/")
    .http_only(true)
    .finish();

    let jar = jar.add(pkce_cookie);

    (jar, axum::response::Redirect::to(auth_url.as_ref()))
}

/// GET /auth/google/callback?code=...&state=...
pub async fn google_callback(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let code = match params.get("code") {
        Some(c) => c,
        None => return (StatusCode::BAD_REQUEST, "missing code").into_response(),
    };
    let state_param = match params.get("state") {
        Some(s) => s,
        None => return (StatusCode::BAD_REQUEST, "missing state").into_response(),
    };

    // Verify CSRF
    let Some(csrf_cookie) = jar.get(CSRF_COOKIE) else {
        return (StatusCode::BAD_REQUEST, "missing CSRF cookie").into_response();
    };
    if csrf_cookie.value() != state_param {
        return (StatusCode::BAD_REQUEST, "invalid CSRF").into_response();
    }

    // Fetch PKCE verifier
    let pkce_key = format!("{CSRF_COOKIE}_pkce");
    let Some(pkce_cookie) = jar.get(&pkce_key) else {
        return (StatusCode::BAD_REQUEST, "missing PKCE").into_response();
    };
    let pkce_verifier = oauth2::PkceCodeVerifier::new(pkce_cookie.value().to_string());

    // Exchange code
    let client = build_google_client();
    let token_res = match client
        .exchange_code(oauth2::AuthorizationCode::new(code.to_string()))
        .set_pkce_verifier(pkce_verifier)
        .request_async(oauth2::reqwest::async_http_client)
        .await
    {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("token exchange error: {e:?}");
            return (StatusCode::BAD_REQUEST, "token exchange failed").into_response();
        }
    };

    // Get userinfo from OIDC endpoint
    let access_token = token_res.access_token().secret();
    let userinfo: OidcUser = match reqwest::Client::new()
        .get("https://openidconnect.googleapis.com/v1/userinfo")
        .bearer_auth(access_token)
        .send()
        .await
        .and_then(|r| r.error_for_status())
    {
        Ok(res) => match res.json().await {
            Ok(user) => user,
            Err(e) => {
                tracing::error!("userinfo parse error: {e:?}");
                return (StatusCode::BAD_REQUEST, "failed to parse userinfo").into_response();
            }
        },
        Err(e) => {
            tracing::error!("userinfo error: {e:?}");
            return (StatusCode::BAD_REQUEST, "failed to fetch userinfo").into_response();
        }
    };

    // Create a signed session cookie
    let session_json = match serde_json::to_string(&userinfo) {
        Ok(json) => json,
        Err(e) => {
            tracing::error!("session serialization error: {e:?}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "internal server error").into_response();
        }
    };

    let session_cookie = Cookie::build((SESSION_COOKIE, session_json))
        .path("/")
        .http_only(true)
        .same_site(axum_extra::extract::cookie::SameSite::Lax)
        // Optional: set max age for session persistence
        // .max_age(time::Duration::days(7))
        .finish();

    let jar = jar
        // Remove CSRF cookie with same attributes as when set
        .remove(
            Cookie::build((CSRF_COOKIE, ""))
                .path("/")
                .http_only(true)
                .finish(),
        )
        // Remove PKCE cookie with same attributes as when set
        .remove(
            Cookie::build((pkce_key.clone(), ""))
                .path("/")
                .http_only(true)
                .finish(),
        )
        .add(session_cookie);

    // Redirect to home or dashboard
    (jar, axum::response::Redirect::to("/")).into_response()
}

/// POST /auth/logout
pub async fn logout(State(_state): State<AppState>, jar: SignedCookieJar) -> impl IntoResponse {
    // Create a removal cookie with the same attributes as the session cookie
    let removal_cookie = Cookie::build((SESSION_COOKIE, ""))
        .path("/")
        .http_only(true)
        .same_site(axum_extra::extract::cookie::SameSite::Lax)
        .max_age(Duration::seconds(0)) // Expire immediately
        .finish();

    let jar = jar.add(removal_cookie);

    // Also clear any other auth-related cookies
    let csrf_removal = Cookie::build((CSRF_COOKIE, ""))
        .path("/")
        .http_only(true)
        .max_age(Duration::seconds(0))
        .finish();

    let jar = jar.add(csrf_removal);

    (jar, axum::response::Redirect::to("/"))
}

/// Helper function to get current user from session
async fn get_current_user(jar: SignedCookieJar) -> Option<OidcUser> {
    jar.get(SESSION_COOKIE)
        .and_then(|cookie| serde_json::from_str(cookie.value()).ok())
}

pub async fn api_user_info(jar: SignedCookieJar) -> impl IntoResponse {
    match get_current_user(jar).await {
        Some(user) => (StatusCode::OK, Json(user)).into_response(),
        None => (StatusCode::UNAUTHORIZED, "Not authenticated").into_response(),
    }
}

// use leptos::*;
// use leptos_axum::extract;

// #[server]
// pub async fn get_current_user_server() -> Result<Option<OidcUser>, ServerFnError> {
//     let jar = extract::<SignedCookieJar>()
//         .await
//         .map_err(|e| ServerFnError::new(format!("Failed to extract cookies: {}", e)))?;

//     Ok(get_current_user(jar).await)
// }
