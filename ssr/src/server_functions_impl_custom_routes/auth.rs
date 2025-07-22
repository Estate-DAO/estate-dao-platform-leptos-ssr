use leptos::ServerFnError::{Deserialization, ServerError};
use leptos::*;
use serde::{de::DeserializeOwned, Serialize};

// Define custom error type for ServerFnError
type NoCustomError = ();
type ServerFnResult<T> = Result<T, ServerFnError<NoCustomError>>;
use anyhow::Result;
use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect},
};
use estate_fe::{
    api::auth::{is_token_expired, YralOAuthClient, YralTokenResponse},
    log,
    view_state_layer::{user_auth_state::UserInfo, AppState},
};
use serde::{Deserialize, Serialize};
use tower_cookies::cookie::SameSite;
use tower_cookies::{Cookie, Cookies};

#[derive(Deserialize)]
pub struct AuthCallbackParams {
    pub code: String,
    pub state: String,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

/// Initiate OAuth2 flow with YRAL
#[server(InitiateAuth, "/api")]
pub async fn initiate_auth() -> ServerFnResult<String> {
    let state = expect_context::<AppState>();

    let oauth_client = YralOAuthClient::new(state.env_var_config.clone())
        .map_err(|e| ServerError(format!("OAuth client error: {}", e)))?;

    let (auth_url, code_verifier, csrf_state) = oauth_client.get_authorization_url();

    // Store code_verifier and state in secure HTTP-only cookies
    let cookies = expect_context::<Cookies>();

    let code_verifier_cookie = Cookie::build(("auth_code_verifier", code_verifier))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .max_age(tower_cookies::cookie::time::Duration::minutes(10)) // Short expiry
        .build();

    let state_cookie = Cookie::build(("auth_state", csrf_state))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .max_age(tower_cookies::cookie::time::Duration::minutes(10)) // Short expiry
        .build();

    cookies.add(code_verifier_cookie);
    cookies.add(state_cookie);

    log!("Generated auth URL: {}", auth_url);
    Ok(auth_url)
}

/// Handle OAuth2 callback from YRAL
pub async fn auth_callback_handler(
    Query(params): Query<AuthCallbackParams>,
    State(app_state): State<AppState>,
    cookies: Cookies,
) -> impl IntoResponse {
    // Check for OAuth errors first
    if let Some(error) = params.error {
        let error_desc = params.error_description.unwrap_or_default();
        log!("OAuth error: {} - {}", error, error_desc);
        return Redirect::to(&format!("/?auth=error&message={}", error)).into_response();
    }

    // Verify state parameter for CSRF protection
    let stored_state = cookies.get("auth_state").map(|c| c.value().to_string());

    if stored_state.as_ref() != Some(&params.state) {
        log!(
            "State mismatch: expected {:?}, got {}",
            stored_state,
            params.state
        );
        return Redirect::to("/?auth=error&message=invalid_state").into_response();
    }

    // Get code verifier
    let code_verifier = cookies
        .get("auth_code_verifier")
        .map(|c| c.value().to_string());

    let Some(verifier) = code_verifier else {
        log!("Missing code verifier in cookies");
        return Redirect::to("/?auth=error&message=missing_verifier").into_response();
    };

    // Exchange code for tokens
    let oauth_client = match YralOAuthClient::new(app_state.env_var_config.clone()) {
        Ok(client) => client,
        Err(e) => {
            log!("OAuth client creation failed: {}", e);
            return Redirect::to("/?auth=error&message=config_error").into_response();
        }
    };

    let token_response = match oauth_client
        .exchange_code_for_tokens(params.code, verifier)
        .await
    {
        Ok(response) => response,
        Err(e) => {
            log!("Token exchange failed: {}", e);
            return Redirect::to("/?auth=error&message=token_exchange_failed").into_response();
        }
    };

    // Extract user info from token response
    let user_info = match oauth_client.extract_user_info(&token_response) {
        Ok(info) => info,
        Err(e) => {
            log!("Failed to extract user info: {}", e);
            return Redirect::to("/?auth=error&message=user_info_extraction_failed")
                .into_response();
        }
    };

    // Store tokens in secure cookies
    let expires_at = chrono::Utc::now().timestamp() + token_response.expires_in as i64;

    let access_token_cookie = Cookie::build(("access_token", token_response.access_token.clone()))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .max_age(tower_cookies::cookie::time::Duration::seconds(
            token_response.expires_in as i64,
        ))
        .build();

    let refresh_token_cookie =
        Cookie::build(("refresh_token", token_response.refresh_token.clone()))
            .http_only(true)
            .secure(true)
            .same_site(SameSite::Lax)
            .max_age(tower_cookies::cookie::time::Duration::days(30))
            .build();

    let user_info_cookie = Cookie::build((
        "user_info",
        serde_json::to_string(&user_info).unwrap_or_default(),
    ))
    .http_only(true)
    .secure(true)
    .same_site(SameSite::Lax)
    .max_age(tower_cookies::cookie::time::Duration::days(7))
    .build();

    cookies.add(access_token_cookie);
    cookies.add(refresh_token_cookie);
    cookies.add(user_info_cookie);

    // Clean up temporary cookies
    cookies.remove(Cookie::build("auth_code_verifier").build());
    cookies.remove(Cookie::build("auth_state").build());

    log!(
        "Authentication successful for user: {}",
        user_info.principal
    );
    Redirect::to("/?auth=success").into_response()
}

/// Logout user and clear authentication cookies
#[server(Logout, "/api")]
pub async fn logout() -> ServerFnResult<()> {
    let cookies = expect_context::<Cookies>();

    cookies.remove(Cookie::build("access_token").build());
    cookies.remove(Cookie::build("refresh_token").build());
    cookies.remove(Cookie::build("user_info").build());

    log!("User logged out successfully");
    Ok(())
}

/// Get current user information from cookies
#[server(GetUserInfo, "/api")]
pub async fn get_user_info() -> ServerFnResult<Option<UserInfo>> {
    let cookies = expect_context::<Cookies>();

    let user_info_str = cookies.get("user_info").map(|c| c.value().to_string());

    if let Some(info_str) = user_info_str {
        match serde_json::from_str::<UserInfo>(&info_str) {
            Ok(user_info) => {
                // Check if access token is still valid
                if let Some(access_token) =
                    cookies.get("access_token").map(|c| c.value().to_string())
                {
                    if !is_token_expired(&access_token) {
                        return Ok(Some(user_info));
                    }
                }
                // Token expired, try to refresh
                if let Some(refresh_token) =
                    cookies.get("refresh_token").map(|c| c.value().to_string())
                {
                    // Try to refresh the token
                    match refresh_user_token(refresh_token).await {
                        Ok(_) => Ok(Some(user_info)),
                        Err(_) => {
                            // Refresh failed, clear cookies
                            let _ = logout().await;
                            Ok(None)
                        }
                    }
                } else {
                    Ok(None)
                }
            }
            Err(e) => {
                log!("Failed to parse user info from cookies: {}", e);
                Ok(None)
            }
        }
    } else {
        Ok(None)
    }
}

/// Refresh access token using refresh token
#[server(RefreshToken, "/api")]
pub async fn refresh_user_token(refresh_token: String) -> ServerFnResult<()> {
    let state = expect_context::<AppState>();
    let cookies = expect_context::<Cookies>();

    let oauth_client = YralOAuthClient::new(state.env_var_config.clone())
        .map_err(|e| ServerError(format!("OAuth client error: {}", e)))?;

    let token_response = oauth_client
        .refresh_token(refresh_token)
        .await
        .map_err(|e| ServerError(format!("Token refresh failed: {}", e)))?;

    // Update access token cookie
    let access_token_cookie = Cookie::build(("access_token", token_response.access_token.clone()))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .max_age(tower_cookies::cookie::time::Duration::seconds(
            token_response.expires_in as i64,
        ))
        .build();

    cookies.add(access_token_cookie);

    log!("Token refreshed successfully");
    Ok(())
}
