use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
    Json,
};
use axum_extra::extract::cookie::{Cookie, SignedCookieJar};
use http::StatusCode;
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
    canister::backend::HotelId,
    utils::admin::admin_canister,
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
    // Use APP_URL to construct the redirect URL dynamically
    let app_url = std::env::var("APP_URL").unwrap_or_else(|_| "http://localhost:3002/".into());
    let redirect = format!("{}/auth/google/callback", app_url.trim_end_matches('/'));

    BasicClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
        AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string()).unwrap(),
        Some(TokenUrl::new("https://oauth2.googleapis.com/token".to_string()).unwrap()),
    )
    .set_redirect_uri(RedirectUrl::new(redirect).expect("Invalid redirect URL"))
}

/// GET /auth/google
pub async fn google_auth(
    State(_state): State<AppState>,
    jar: SignedCookieJar,
) -> impl IntoResponse {
    let client = build_google_client();
    let (pkce_challenge, pkce_verifier) = oauth2::PkceCodeChallenge::new_random_sha256();
    let (auth_url, csrf) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("openid".into()))
        .add_scope(Scope::new("email".into()))
        .add_scope(Scope::new("profile".into()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    // Determine cookie domain based on APP_URL
    let app_url = std::env::var("APP_URL").unwrap_or_else(|_| "http://localhost:3002/".into());
    let cookie_domain = if app_url.contains("nofeebooking.com") {
        Some(".nofeebooking.com".to_string()) // Covers nofeebooking.com and www.nofeebooking.com
    } else {
        None
    };

    // Build CSRF cookie
    let mut csrf_cookie_builder = Cookie::build((CSRF_COOKIE, csrf.secret().to_string()))
        .path("/")
        .http_only(true)
        .same_site(axum_extra::extract::cookie::SameSite::Lax)
        .secure(app_url.starts_with("https://"));

    if let Some(domain) = cookie_domain.clone() {
        csrf_cookie_builder = csrf_cookie_builder.domain(domain);
    }
    let csrf_cookie = csrf_cookie_builder.build();

    let jar = jar.add(csrf_cookie);

    // Build PKCE cookie
    let pkce_key = format!("{CSRF_COOKIE}_pkce");
    let mut pkce_cookie_builder =
        Cookie::build((pkce_key.clone(), pkce_verifier.secret().to_string()))
            .path("/")
            .http_only(true)
            .same_site(axum_extra::extract::cookie::SameSite::Lax)
            .secure(app_url.starts_with("https://"));

    if let Some(domain) = cookie_domain.clone() {
        pkce_cookie_builder = pkce_cookie_builder.domain(domain);
    }
    let pkce_cookie = pkce_cookie_builder.build();

    let jar = if jar.get(&pkce_key.clone()).is_none() {
        jar.add(pkce_cookie)
    } else {
        jar
    };

    // tracing::debug!("Setting CSRF cookie: {:?}", csrf_cookie);
    // tracing::debug!("Setting PKCE cookie: {:?}", pkce_cookie);
    tracing::debug!("Redirecting to: {}", auth_url.as_ref());

    (jar, axum::response::Redirect::to(auth_url.as_ref()))
}

/// GET /auth/google/callback?code=...&state=...
pub async fn google_callback(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    tracing::debug!("Received cookies: {:?}", jar.iter().collect::<Vec<_>>());
    tracing::debug!("Query params: {:?}", params);

    let code = match params.get("code") {
        Some(c) => c,
        None => return (StatusCode::BAD_REQUEST, "missing code").into_response(),
    };
    let state_param = match params.get("state") {
        Some(s) => s,
        None => return (StatusCode::BAD_REQUEST, "missing state").into_response(),
    };

    let Some(csrf_cookie) = jar.get(CSRF_COOKIE) else {
        tracing::error!("Missing CSRF cookie in callback");
        return (StatusCode::BAD_REQUEST, "missing CSRF cookie").into_response();
    };
    if csrf_cookie.value() != state_param {
        tracing::error!(
            "CSRF token mismatch: cookie={}, state={}",
            csrf_cookie.value(),
            state_param
        );
        return (StatusCode::BAD_REQUEST, "invalid CSRF").into_response();
    };

    let pkce_key = format!("{CSRF_COOKIE}_pkce");
    let Some(pkce_cookie) = jar.get(&pkce_key) else {
        tracing::error!("Missing PKCE cookie in callback");
        return (StatusCode::BAD_REQUEST, "missing PKCE").into_response();
    };
    let pkce_verifier = oauth2::PkceCodeVerifier::new(pkce_cookie.value().to_string());

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

    let app_url = std::env::var("APP_URL").unwrap_or_else(|_| "http://localhost:3002/".into());
    let cookie_domain = if app_url.contains("nofeebooking.com") {
        Some(".nofeebooking.com".to_string())
    } else {
        None
    };
    let is_secure = app_url.starts_with("https://");

    let session_json = match serde_json::to_string(&userinfo) {
        Ok(json) => json,
        Err(e) => {
            tracing::error!("session serialization error: {e:?}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "internal server error").into_response();
        }
    };

    let mut session_cookie_builder = Cookie::build((SESSION_COOKIE, session_json))
        .path("/")
        .http_only(true)
        .same_site(axum_extra::extract::cookie::SameSite::Lax)
        .secure(is_secure);

    if let Some(domain) = cookie_domain.clone() {
        session_cookie_builder = session_cookie_builder.domain(domain);
    }
    let session_cookie = session_cookie_builder.build();

    // Remove CSRF and PKCE cookies with matching attributes
    let mut csrf_removal_builder = Cookie::build((CSRF_COOKIE, ""))
        .path("/")
        .http_only(true)
        .secure(is_secure)
        .max_age(Duration::seconds(0));

    if let Some(domain) = cookie_domain.clone() {
        csrf_removal_builder = csrf_removal_builder.domain(domain);
    }
    let csrf_removal = csrf_removal_builder.build();

    let mut pkce_removal_builder = Cookie::build((pkce_key.clone(), ""))
        .path("/")
        .http_only(true)
        .secure(is_secure)
        .max_age(Duration::seconds(0));

    if let Some(domain) = cookie_domain.clone() {
        pkce_removal_builder = pkce_removal_builder.domain(domain);
    }
    let pkce_removal = pkce_removal_builder.build();

    let jar = jar
        .remove(csrf_removal.clone())
        .remove(pkce_removal.clone())
        .add(session_cookie.clone());

    tracing::debug!("Setting session cookie: {:?}", session_cookie);
    tracing::debug!("Removing CSRF cookie: {:?}", csrf_removal);
    tracing::debug!("Removing PKCE cookie: {:?}", pkce_removal);

    let script = Html(
        r#"
            <script>
            window.opener.postMessage("oauth-success", window.location.origin);
            window.close();
            </script>
        "#,
    );

    (jar, script).into_response()
}

/// POST /auth/logout
pub async fn logout(State(_state): State<AppState>, jar: SignedCookieJar) -> impl IntoResponse {
    // Determine cookie domain and secure attribute based on APP_URL
    let app_url = std::env::var("APP_URL").unwrap_or_else(|_| "http://localhost:3002/".into());
    let cookie_domain = if app_url.contains("nofeebooking.com") {
        Some(".nofeebooking.com".to_string())
    } else {
        None
    };
    let is_secure = app_url.starts_with("https://");

    // Create a removal cookie for session with matching attributes
    let mut session_removal_builder = Cookie::build((SESSION_COOKIE, ""))
        .path("/")
        .http_only(true)
        .same_site(axum_extra::extract::cookie::SameSite::Lax)
        .secure(is_secure)
        .max_age(Duration::seconds(0)); // Expire immediately

    if let Some(domain) = cookie_domain.clone() {
        session_removal_builder = session_removal_builder.domain(domain);
    }
    let session_removal = session_removal_builder.build();

    // Create a removal cookie for CSRF with matching attributes
    let mut csrf_removal_builder = Cookie::build((CSRF_COOKIE, ""))
        .path("/")
        .http_only(true)
        .secure(is_secure)
        .max_age(Duration::seconds(0));

    if let Some(domain) = cookie_domain.clone() {
        csrf_removal_builder = csrf_removal_builder.domain(domain);
    }
    let csrf_removal = csrf_removal_builder.build();

    // Create a removal cookie for PKCE with matching attributes
    let pkce_key = format!("{CSRF_COOKIE}_pkce");
    let mut pkce_removal_builder = Cookie::build((pkce_key, ""))
        .path("/")
        .http_only(true)
        .secure(is_secure)
        .max_age(Duration::seconds(0));

    if let Some(domain) = cookie_domain.clone() {
        pkce_removal_builder = pkce_removal_builder.domain(domain);
    }
    let pkce_removal = pkce_removal_builder.build();

    // Add removal cookies to the jar
    let jar = jar
        .add(session_removal.clone())
        .add(csrf_removal.clone())
        .add(pkce_removal.clone());

    tracing::debug!("Removing session cookie: {:?}", session_removal);
    tracing::debug!("Removing CSRF cookie: {:?}", csrf_removal);
    tracing::debug!("Removing PKCE cookie: {:?}", pkce_removal);

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

async fn get_wishlist_by_email(email: String) -> Result<Vec<String>, String> {
    // Fetch wishlist from backend canister using email
    // Placeholder implementation
    let admin_cans = admin_canister();
    admin_cans
        .backend_canister()
        .await
        .get_wishlist_by_email(email)
        .await
        .map(|f| f.iter().map(|f| f.hotel_code.clone()).collect())
        .map_err(|f| f.to_string())
}

async fn add_to_wishlist_by_email(email: String, hotel_code: String) -> Result<(), String> {
    // Fetch wishlist from backend canister using email
    // Placeholder implementation
    let admin_cans = admin_canister();
    let res = admin_cans
        .backend_canister()
        .await
        .add_to_wishlist_by_email(email, HotelId { hotel_code })
        .await
        .map_err(|f| f.to_string())?;

    match res {
        crate::canister::backend::Result_::Ok(_) => Ok(()),
        crate::canister::backend::Result_::Err(e) => Err(e),
    }
}

async fn remove_from_wishlist_by_email(email: String, hotel_code: String) -> Result<(), String> {
    // Fetch wishlist from backend canister using email
    // Placeholder implementation
    let admin_cans = admin_canister();
    let res = admin_cans
        .backend_canister()
        .await
        .remove_from_wishlist_by_email(email, HotelId { hotel_code })
        .await
        .map_err(|f| f.to_string())?;

    match res {
        crate::canister::backend::Result_::Ok(_) => Ok(()),
        crate::canister::backend::Result_::Err(e) => Err(e),
    }
}

pub async fn add_to_user_wishlist(
    Path(hotel_code): Path<String>,
    jar: SignedCookieJar,
) -> impl IntoResponse {
    let err = (StatusCode::UNAUTHORIZED, "Not authenticated");
    let user = match get_current_user(jar).await {
        Some(user) => user,
        None => return (StatusCode::UNAUTHORIZED, "Not authenticated").into_response(),
    };
    if let Some(email) = user.email {
        match add_to_wishlist_by_email(email, hotel_code).await {
            Ok(w) => Json(w).into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
        }
    } else {
        (StatusCode::UNAUTHORIZED, "Not authenticated").into_response()
    }
}

pub async fn remove_from_user_wishlist(
    Path(hotel_code): Path<String>,
    jar: SignedCookieJar,
) -> impl IntoResponse {
    let err = (StatusCode::UNAUTHORIZED, "Not authenticated");
    let user = match get_current_user(jar).await {
        Some(user) => user,
        None => return (StatusCode::UNAUTHORIZED, "Not authenticated").into_response(),
    };
    if let Some(email) = user.email {
        match remove_from_wishlist_by_email(email, hotel_code).await {
            Ok(w) => Json(w).into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
        }
    } else {
        (StatusCode::UNAUTHORIZED, "Not authenticated").into_response()
    }
}

pub async fn get_user_wishlist(jar: SignedCookieJar) -> impl IntoResponse {
    let err = (StatusCode::UNAUTHORIZED, "Not authenticated");
    let user = match get_current_user(jar).await {
        Some(user) => user,
        None => return (StatusCode::UNAUTHORIZED, "Not authenticated").into_response(),
    };
    if let Some(email) = user.email {
        match get_wishlist_by_email(email).await {
            Ok(w) => Json(w).into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
        }
    } else {
        (StatusCode::UNAUTHORIZED, "Not authenticated").into_response()
    }
}
