use axum::{
    body::{to_bytes, Body},
    extract::State,
    http::{Request, StatusCode},
    response::{IntoResponse, Response},
};
use estate_fe::api::auth::types::LoginProvider;
use estate_fe::api::{
    auth::auth_url::{perform_yral_auth_impl, yral_auth_url_impl},
    auth::types::NewIdentity,
    client_side_api::YralAuthLoginUrlRequest,
};
use estate_fe::page::OAuthQuery;
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
        state,
        None, // No ResponseOptions for raw Axum handler
    )
    .await
}

#[cfg_attr(feature = "debug_log", axum::debug_handler)]
#[tracing::instrument(skip(state))]
pub async fn initiate_auth_axum_handler(
    State(state): State<AppState>,
    request: Request<Body>,
) -> Result<Response, Response> {
    // Split the request into parts and body
    let (parts, body) = request.into_parts();

    // Extract body as string
    let body_bytes = to_bytes(body, usize::MAX).await.map_err(|e| {
        tracing::error!("Failed to read request body: {:?}", e);
        (StatusCode::BAD_REQUEST, "Failed to read request body").into_response()
    })?;
    let body = String::from_utf8(body_bytes.to_vec()).map_err(|e| {
        tracing::error!("Invalid UTF-8 in request body: {:?}", e);
        (StatusCode::BAD_REQUEST, "Invalid request body encoding").into_response()
    })?;

    // Parse JSON request body
    let auth_query_params: YralAuthLoginUrlRequest = parse_json_request(&body)?;

    // Log browser information for debugging cookie issues
    if let Some(user_agent) = parts.headers.get("user-agent") {
        crate::log!("[COOKIE_DEBUG] User-Agent: {:?}", user_agent);
        let user_agent_str = user_agent.to_str().unwrap_or("unknown");
        if user_agent_str.contains("Chrome") {
            crate::log!(
                "[COOKIE_DEBUG] Chrome browser detected - requires Secure=true for SameSite=None"
            );
        } else if user_agent_str.contains("Safari") {
            crate::log!("[COOKIE_DEBUG] Safari browser detected");
        } else if user_agent_str.contains("Firefox") {
            crate::log!("[COOKIE_DEBUG] Firefox browser detected");
        }
    }

    crate::log!("Received auth query params: {:?}", auth_query_params);

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

    tracing::debug!("cookie_jar: {:#?}", cookie_jar);
    tracing::debug!("auth_url: {:#?}", auth_url);
    tracing::debug!("response: {:#?}", response);

    // Create response and log Set-Cookie headers for debugging
    let response_with_cookies =
        (cookie_jar, (StatusCode::OK, response.to_string())).into_response();

    // Log Set-Cookie headers being sent to browser
    if let Some(set_cookie_headers) = response_with_cookies
        .headers()
        .get_all("set-cookie")
        .iter()
        .next()
    {
        crate::log!(
            "[COOKIE_DEBUG] Set-Cookie headers being sent: {:?}",
            set_cookie_headers
        );
    }
    for cookie_header in response_with_cookies.headers().get_all("set-cookie") {
        crate::log!("[COOKIE_DEBUG] Set-Cookie: {:?}", cookie_header);
    }

    Ok(response_with_cookies)
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

use axum::extract::FromRequestParts;
use axum_extra::extract::cookie::Key;
use axum_extra::extract::{PrivateCookieJar, SignedCookieJar};

// Helper function to extract cookie jars from request parts using the app's cookie key
pub async fn extract_cookie_jars(
    parts: &mut axum::http::request::Parts,
    cookie_key: &Key,
) -> Result<(PrivateCookieJar<Key>, SignedCookieJar<Key>), (StatusCode, &'static str)> {
    // tracing::debug!("[COOKIE_DEBUG] Starting cookie jar extraction");
    // tracing::debug!("[COOKIE_DEBUG] Cookie key length: {}", cookie_key.signing().len());
    // tracing::debug!("[COOKIE_DEBUG] Available request headers: {:#?}", parts.headers);

    // Log raw cookie header if present
    if let Some(cookie_header) = parts.headers.get("cookie") {
        tracing::debug!("[COOKIE_DEBUG] Raw cookie header: {:?}", cookie_header);
    } else {
        tracing::warn!("[COOKIE_DEBUG] No cookie header found in request");
    }

    let private_jar = PrivateCookieJar::from_request_parts(parts, cookie_key)
        .await
        .map_err(|e| {
            // tracing::error!("[COOKIE_DEBUG] Failed to extract private cookies: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to extract private cookies",
            )
        })?;

    // tracing::debug!("[COOKIE_DEBUG] Private jar extracted successfully");
    // tracing::debug!("[COOKIE_DEBUG] Private jar is empty: {}", private_jar.iter().count() == 0);

    let signed_jar = SignedCookieJar::from_request_parts(parts, cookie_key)
        .await
        .map_err(|e| {
            tracing::error!("[COOKIE_DEBUG] Failed to extract signed cookies: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to extract signed cookies",
            )
        })?;

    // tracing::debug!("[COOKIE_DEBUG] Signed jar extracted successfully");
    // tracing::debug!("[COOKIE_DEBUG] Signed jar is empty: {}", signed_jar.iter().count() == 0);

    // // Log individual cookies in private jar
    // for cookie in private_jar.iter() {
    //     tracing::debug!("[COOKIE_DEBUG] Private cookie name: {}", cookie.name());
    // }

    // // Log individual cookies in signed jar
    // for cookie in signed_jar.iter() {
    //     tracing::debug!("[COOKIE_DEBUG] Signed cookie name: {}", cookie.name());
    // }

    Ok((private_jar, signed_jar))
}

#[cfg_attr(feature = "debug_log", axum::debug_handler)]
#[tracing::instrument(skip(state))]
pub async fn perform_yral_oauth_api_server_fn_route(
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

    // Log request headers and cookies for debugging
    crate::log!("[OAUTH_DEBUG] Request method: {}", parts.method);
    crate::log!("[OAUTH_DEBUG] Request URI: {}", parts.uri);
    crate::log!("[OAUTH_DEBUG] Request headers: {:#?}", parts.headers);

    // Log browser information for cookie debugging
    if let Some(user_agent) = parts.headers.get("user-agent") {
        crate::log!("[OAUTH_DEBUG] User-Agent: {:?}", user_agent);
        let user_agent_str = user_agent.to_str().unwrap_or("unknown");
        if user_agent_str.contains("Chrome") {
            crate::log!("[OAUTH_DEBUG] Chrome browser detected - checking strict cookie policies");
        }
    }

    // Parse JSON request body
    let oauth_query: OAuthQuery = parse_json_request(&body)?;
    tracing::info!(
        "[OAUTH_DEBUG] Received OAuth callback: code={}, state={}",
        oauth_query.code,
        oauth_query.state
    );

    // Parse and log state parameter to extract CSRF token
    if let Ok(state_json) = serde_json::from_str::<serde_json::Value>(&oauth_query.state) {
        tracing::debug!("[OAUTH_DEBUG] Parsed state parameter: {:#?}", state_json);
        if let Some(csrf_token) = state_json.get("csrf_token").and_then(|v| v.as_str()) {
            tracing::info!(
                "[OAUTH_DEBUG] CSRF token from state parameter: {}",
                csrf_token
            );
        } else {
            tracing::warn!("[OAUTH_DEBUG] No csrf_token found in state parameter");
        }
    } else {
        tracing::warn!(
            "[OAUTH_DEBUG] Failed to parse state parameter as JSON: {}",
            oauth_query.state
        );
    }

    // Extract cookie jars from the request using helper
    let cookie_key = state.cookie_key.clone();
    let (private_jar, signed_jar) =
        extract_cookie_jars(&mut parts, &cookie_key)
            .await
            .map_err(|(status, msg)| {
                tracing::error!("[OAUTH_DEBUG] Cookie extraction failed: {}", msg);
                (status, msg).into_response()
            })?;

    // // Log cookies available in jars for debugging
    // tracing::debug!("[OAUTH_DEBUG] Private jar cookies: {:#?}", private_jar);
    // tracing::debug!("[OAUTH_DEBUG] Signed jar cookies: {:#?}", signed_jar);

    // Call the OAuth implementation function
    let oauth_client = state.yral_oauth_client.clone();

    let (claims_from_yral_auth, updated_jar) = perform_yral_auth_impl(
        oauth_query.state,
        oauth_query.code,
        oauth_client,
        private_jar,
        signed_jar,
        None, // No ResponseOptions in raw Axum handler
    )
    .await
    .map_err(|e| {
        tracing::error!("[OAUTH_DEBUG] OAuth implementation failed: {:?}", e);
        let error_response = json!({
            "error": format!("OAuth authentication failed: {}", e)
        });
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            error_response.to_string(),
        )
            .into_response()
    })?;

    let identity = claims_from_yral_auth.identity;
    let username = claims_from_yral_auth.username;
    let email = claims_from_yral_auth.email;

    // Create NewIdentity response
    let new_identity = NewIdentity {
        id_wire: identity,
        fallback_username: username,
        email: email,
    };

    // Return JSON response
    let response = serde_json::to_string(&new_identity).map_err(|e| {
        tracing::error!("Failed to serialize NewIdentity: {:?}", e);
        let error_response = json!({
            "error": format!("Response serialization failed: {}", e)
        });
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            error_response.to_string(),
        )
            .into_response()
    })?;

    // Return with updated cookie jar containing refresh token
    Ok((updated_jar, (StatusCode::OK, response)).into_response())
}
