use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use estate_fe::api::auth::types::LoginProvider;
use estate_fe::api::{
    auth::auth_url::yral_auth_url_impl, client_side_api::YralAuthLoginUrlRequest,
};
use estate_fe::view_state_layer::AppState;
use leptos::ServerFnError;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::json;

use anyhow::Result;

use super::parse_json_request;
// use axum::{
//     extract::{Query, State},
//     http::{HeaderMap, StatusCode},
//     response::{IntoResponse, Redirect},
// };
// use estate_fe::{
//     api::auth::types::YralOAuthClient,
//     log,
//     view_state_layer::{user_auth_state::UserInfo, AppState},
// };
// use tower_cookies::cookie::SameSite;
// use tower_cookies::{Cookie, Cookies};

// #[derive(Deserialize)]
// pub struct AuthCallbackParams {
//     pub code: String,
//     pub state: String,
//     pub error: Option<String>,
//     pub error_description: Option<String>,
// }

pub async fn initiate_auth(
    auth_query_params: YralAuthLoginUrlRequest,
    state: &AppState,
) -> Result<(String, axum_extra::extract::PrivateCookieJar), ServerFnError> {
    let oauth_client = state.yral_oauth_client.clone();
    let yral_auth_redirect_uri = state.env_var_config.yral_redirect_uri.clone();
    yral_auth_url_impl(
        oauth_client,
        "".to_string(),
        auth_query_params.provider,
        Some(yral_auth_redirect_uri),
        state.cookie_key.clone(),
        None, // No Leptos ResponseOptions in raw Axum handler
    )
    .await
}

#[cfg_attr(feature = "debug_log", axum::debug_handler)]
pub async fn initiate_auth_axum_handler(
    State(state): State<AppState>,
    body: String,
) -> Result<Response, Response> {
    // Parse JSON request body
    let auth_query_params: YralAuthLoginUrlRequest = parse_json_request(&body)?;
    tracing::info!("Received auth query params: {:?}", auth_query_params);

    // Call the auth function and get both URL and cookies
    let (auth_url, cookie_jar) = initiate_auth(auth_query_params, &state)
        .await
        .map_err(|e| {
            tracing::error!("Failed to generate auth URL: {:?}", e);
            let error_response = json!({
                "error": format!("Failed to generate auth URL: {}", e)
            });
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                error_response.to_string(),
            )
                .into_response()
        })?;

    // Create JSON response
    let response = json!({
        "auth_url": auth_url
    });

    // Use the standard Axum pattern: (cookies, status, body)
    Ok((cookie_jar, (StatusCode::OK, response.to_string())).into_response())
}

// /// Handle OAuth2 callback from YRAL
// pub async fn auth_callback_handler(
//     Query(params): Query<AuthCallbackParams>,
//     State(app_state): State<AppState>,
//     cookies: Cookies,
// ) -> impl IntoResponse {
//     // Check for OAuth errors first
//     if let Some(error) = params.error {
//         let error_desc = params.error_description.unwrap_or_default();
//         log!("OAuth error: {} - {}", error, error_desc);
//         return Redirect::to(&format!("/?auth=error&message={}", error)).into_response();
//     }

//     // Verify state parameter for CSRF protection
//     let stored_state = cookies.get("auth_state").map(|c| c.value().to_string());

//     if stored_state.as_ref() != Some(&params.state) {
//         log!(
//             "State mismatch: expected {:?}, got {}",
//             stored_state,
//             params.state
//         );
//         return Redirect::to("/?auth=error&message=invalid_state").into_response();
//     }

//     // Get code verifier
//     let code_verifier = cookies
//         .get("auth_code_verifier")
//         .map(|c| c.value().to_string());

//     let Some(verifier) = code_verifier else {
//         log!("Missing code verifier in cookies");
//         return Redirect::to("/?auth=error&message=missing_verifier").into_response();
//     };

//     // Exchange code for tokens
//     let oauth_client = match YralOAuthClient::new(app_state.env_var_config.clone()) {
//         Ok(client) => client,
//         Err(e) => {
//             log!("OAuth client creation failed: {}", e);
//             return Redirect::to("/?auth=error&message=config_error").into_response();
//         }
//     };

//     let token_response = match oauth_client
//         .exchange_code_for_tokens(params.code, verifier)
//         .await
//     {
//         Ok(response) => response,
//         Err(e) => {
//             log!("Token exchange failed: {}", e);
//             return Redirect::to("/?auth=error&message=token_exchange_failed").into_response();
//         }
//     };

//     // Extract user info from token response
//     let user_info = match oauth_client.extract_user_info(&token_response) {
//         Ok(info) => info,
//         Err(e) => {
//             log!("Failed to extract user info: {}", e);
//             return Redirect::to("/?auth=error&message=user_info_extraction_failed")
//                 .into_response();
//         }
//     };

//     // Store tokens in secure cookies
//     let expires_at = chrono::Utc::now().timestamp() + token_response.expires_in as i64;

//     let access_token_cookie = Cookie::build(("access_token", token_response.access_token.clone()))
//         .http_only(true)
//         .secure(true)
//         .same_site(SameSite::Lax)
//         .max_age(tower_cookies::cookie::time::Duration::seconds(
//             token_response.expires_in as i64,
//         ))
//         .build();

//     let refresh_token_cookie =
//         Cookie::build(("refresh_token", token_response.refresh_token.clone()))
//             .http_only(true)
//             .secure(true)
//             .same_site(SameSite::Lax)
//             .max_age(tower_cookies::cookie::time::Duration::days(30))
//             .build();

//     let user_info_cookie = Cookie::build((
//         "user_info",
//         serde_json::to_string(&user_info).unwrap_or_default(),
//     ))
//     .http_only(true)
//     .secure(true)
//     .same_site(SameSite::Lax)
//     .max_age(tower_cookies::cookie::time::Duration::days(7))
//     .build();

//     cookies.add(access_token_cookie);
//     cookies.add(refresh_token_cookie);
//     cookies.add(user_info_cookie);

//     // Clean up temporary cookies
//     cookies.remove(Cookie::build("auth_code_verifier").build());
//     cookies.remove(Cookie::build("auth_state").build());

//     log!(
//         "Authentication successful for user: {}",
//         user_info.principal
//     );
//     Redirect::to("/?auth=success").into_response()
// }

// /// Logout user and clear authentication cookies
// #[server(Logout, "/api")]
// pub async fn logout() -> Result<(), ServerFnError> {
//     let cookies = expect_context::<Cookies>();

//     cookies.remove(Cookie::build("access_token").build());
//     cookies.remove(Cookie::build("refresh_token").build());
//     cookies.remove(Cookie::build("user_info").build());

//     log!("User logged out successfully");
//     Ok(())
// }

// /// Get current user information from cookies
// #[server(GetUserInfo, "/api")]
// pub async fn get_user_info() -> Result<Option<UserInfo>, ServerFnError> {
//     let cookies = expect_context::<Cookies>();

//     let user_info_str = cookies.get("user_info").map(|c| c.value().to_string());

//     if let Some(info_str) = user_info_str {
//         match serde_json::from_str::<UserInfo>(&info_str) {
//             Ok(user_info) => {
//                 // Check if access token is still valid
//                 if let Some(access_token) =
//                     cookies.get("access_token").map(|c| c.value().to_string())
//                 {
//                     if !is_token_expired(&access_token) {
//                         return Ok(Some(user_info));
//                     }
//                 }
//                 // Token expired, try to refresh
//                 if let Some(refresh_token) =
//                     cookies.get("refresh_token").map(|c| c.value().to_string())
//                 {
//                     // Try to refresh the token
//                     match refresh_user_token(refresh_token).await {
//                         Ok(_) => Ok(Some(user_info)),
//                         Err(_) => {
//                             // Refresh failed, clear cookies
//                             let _ = logout().await;
//                             Ok(None)
//                         }
//                     }
//                 } else {
//                     Ok(None)
//                 }
//             }
//             Err(e) => {
//                 log!("Failed to parse user info from cookies: {}", e);
//                 Ok(None)
//             }
//         }
//     } else {
//         Ok(None)
//     }
// }

// /// Refresh access token using refresh token
// #[server(RefreshToken, "/api")]
// pub async fn refresh_user_token(refresh_token: String) -> Result<(), ServerFnError> {
//     let state = expect_context::<AppState>();
//     let cookies = expect_context::<Cookies>();

//     let oauth_client = YralOAuthClient::new(state.env_var_config.clone())
//         .map_err(|e| ServerError(format!("OAuth client error: {}", e)))?;

//     let token_response = oauth_client
//         .refresh_token(refresh_token)
//         .await
//         .map_err(|e| ServerError(format!("Token refresh failed: {}", e)))?;

//     // Update access token cookie
//     let access_token_cookie = Cookie::build(("access_token", token_response.access_token.clone()))
//         .http_only(true)
//         .secure(true)
//         .same_site(SameSite::Lax)
//         .max_age(tower_cookies::cookie::time::Duration::seconds(
//             token_response.expires_in as i64,
//         ))
//         .build();

//     cookies.add(access_token_cookie);

//     log!("Token refreshed successfully");
//     Ok(())
// }
