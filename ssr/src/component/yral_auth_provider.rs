use codee::string::{FromToStringCodec, JsonSerdeCodec};
use ic_agent::identity::DelegatedIdentity;
use leptos::*;
// use leptos::{ev, prelude::*, component, view, server, ServerFnError, expect_context, create_action, window, IntoView, Children, Oco};
use leptos_use::{
    storage::{use_local_storage, use_local_storage_with_options, UseStorageOptions},
    use_cookie, use_cookie_with_options, use_event_listener, use_interval_fn, use_window,
    UseCookieOptions,
};
use web_sys::Window;

use crate::{
    api::{
        auth::{
            auth_state::{AuthState, AuthStateSignal},
            types::OidcUser,
        },
        client_side_api::ClientSideApiClient,
        consts::{get_host, APP_URL, USER_IDENTITY},
    },
    app::AppRoutes,
    page::AccountTabs,
};

#[server]
pub async fn get_app_url_server() -> Result<String, ServerFnError> {
    let env_url = std::env::var("APP_URL")
        .map_err(|e| ServerFnError::new(format!("Failed to read APP_URL from env: {}", e)))?;
    Ok(env_url)
}

#[component]
pub fn YralAuthProvider() -> impl IntoView {
    let profile_details = Resource::local(
        move || AuthStateSignal::auth_state().get(),
        move |auth| async move {
            if auth.is_authenticated() {
                return Some(auth);
            }

            let url = format!("/api/user-info");
            match gloo_net::http::Request::get(&url).send().await {
                Ok(response) => {
                    if response.status() == 200 {
                        if let Ok(user_data) = response.json::<AuthState>().await {
                            logging::log!("Fetched user info: {:?}", user_data);
                            AuthStateSignal::auth_set(user_data.clone());
                            return Some(user_data);
                        }
                    }
                }
                Err(e) => {
                    logging::log!("Failed to fetch user info: {:?}", e);
                }
            }
            None
        },
    );

    let wishlist_details = Resource::local(
        move || AuthStateSignal::auth_state().get(),
        move |auth| async move {
            if auth.is_authenticated() {
                return Some(auth);
            }

            let url = format!("/api/user-wishlist");
            match gloo_net::http::Request::get(&url).send().await {
                Ok(response) => {
                    if response.status() == 200 {
                        if let Ok(user_data) = response.json::<Vec<String>>().await {
                            logging::log!("Fetched wishlist: {:?}", user_data);
                            AuthStateSignal::wishlist_set(Some(user_data));
                        }
                    }
                }
                Err(e) => {
                    logging::log!("Failed to fetch wishlist: {:?}", e);
                }
            }
            None
        },
    );

    // Cookie-based check for session
    let (session_cookie, _) = use_cookie::<String, FromToStringCodec>("session");

    Effect::new(move |_| {
        if session_cookie.get().is_some() {
            logging::log!("Session cookie detected, fetching user info");
            wasm_bindgen_futures::spawn_local(async move {
                let url = format!("/api/user-info");
                match gloo_net::http::Request::get(&url).send().await {
                    Ok(resp) if resp.status() == 200 => {
                        if let Ok(user_data) = resp.json::<AuthState>().await {
                            logging::log!("User info from cookie: {:?}", user_data);
                            AuthStateSignal::auth_set(user_data);
                        }
                    }
                    _ => {
                        logging::log!("Failed to fetch user info from cookie check");
                    }
                }
            });
        }
    });

    view! {
        {
            move || wishlist_details.get().map(|_| {})
        }
        {move || profile_details.get().flatten().map(|user| {
            view! {
                <div>
                    <UserAvatar user />
                </div>
            }
        }).or_else(|| view! {
            <div>
                <LoginButton />
            </div>
        }.into())}
    }
}

#[component]
fn LoginButton() -> impl IntoView {
    let _ = use_event_listener(use_window(), ev::message, move |msg| {
        if let Some(data) = msg.data().as_string() {
            if data == "oauth-success" {
                logging::log!("Received oauth-success message");
                wasm_bindgen_futures::spawn_local(async move {
                    let url = format!("/api/user-info");
                    match gloo_net::http::Request::get(&url).send().await {
                        Ok(resp) if resp.status() == 200 => {
                            if let Ok(user_data) = resp.json::<AuthState>().await {
                                logging::log!("User info from postMessage: {:?}", user_data);
                                AuthStateSignal::auth_set(user_data);
                            }
                        }
                        Err(e) => {
                            logging::log!("Failed to fetch user info from postMessage: {:?}", e);
                        }
                        _ => logging::log!("Unexpected response when fetching user info"),
                    }
                });
            }
        }
    });

    view! {
        <button
            class="flex gap-2 justify-center items-center px-4 py-3 sm:px-6 sm:py-3 md:gap-3 font-medium text-sm md:text-base text-gray-700 bg-white border border-gray-300 rounded-lg shadow-sm hover:bg-gray-50 hover:border-gray-400 hover:shadow-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed min-h-[44px]"
            on:click=move |ev| {
                ev.prevent_default();
                let window = web_sys::window().unwrap();
                let _ = window.open_with_url_and_target(
                    &format!("/auth/google"),
                    "oauthPopup"
                );
            }
        >
            <img class="w-5 h-5 md:w-5 md:h-5" src="/img/google.svg" alt="Google logo" />
            // <span>"Login with Google"</span>
        </button>
    }
}

/// User avatar component (shown when authenticated)
#[component]
fn UserAvatar(user: AuthState) -> impl IntoView {
    let picture_url = user
        .picture
        .unwrap_or_else(|| "/img/default-avatar.png".to_string());
    let user_name = user.name.unwrap_or_else(|| "User".to_string());

    view! {
        <div class="relative group">
            // Circular avatar image
            <img
                src=picture_url
                alt=format!("{} profile picture", user_name)
                class="w-10 h-10 rounded-full object-cover border-2 border-white shadow-sm cursor-pointer hover:border-blue-500 transition-all duration-200"
            />

            // Optional: Dropdown menu on hover/click
            <div class="absolute right-0 mt-2 w-48 bg-white rounded-md shadow-lg py-1 z-50 opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-all duration-200">
                <div class="px-4 py-2 border-b border-gray-100">
                    <p class="text-sm font-medium text-gray-900 truncate">{user_name}</p>
                    <p class="text-xs text-gray-500 truncate">{user.email.unwrap_or_default()}</p>
                </div>
                <div class="px-4 py-2 border-b border-gray-100">
                    <a href=AccountTabs::Booking.as_route() class="text-sm font-medium text-gray-900 truncate">My Bookings</a>
                </div>
                <div class="px-4 py-2 border-b border-gray-100">
                    <a href=AccountTabs::Wishlist.as_route() class="text-sm font-medium text-gray-900 truncate">My Wishlist</a>
                </div>
                <button
                    on:click=move |ev| {
                        ev.prevent_default();
                        let window = web_sys::window().expect("no global window exists");
                        let _ = window.location().set_href("/auth/logout");
                    }
                    class="block px-4 py-2 text-sm text-gray-700 hover:bg-gray-100"
                >
                    "Logout"
                </button>
            </div>
        </div>
    }
}

/// Alternative: Simple avatar without dropdown
#[component]
fn SimpleUserAvatar(user: OidcUser) -> impl IntoView {
    let picture_url = user
        .picture
        .unwrap_or_else(|| "/img/default-avatar.png".to_string());

    view! {
        <a href="/profile" class="block">
            <img
                src=picture_url
                alt="User profile"
                class="w-10 h-10 rounded-full object-cover border-2 border-white shadow-sm hover:border-blue-500 transition-all duration-200"
            />
        </a>
    }
}

/// Alternative: Avatar with tooltip
#[component]
fn UserAvatarWithTooltip(user: OidcUser) -> impl IntoView {
    let picture_url = user
        .picture
        .unwrap_or_else(|| "/img/default-avatar.png".to_string());
    let user_name = user.name.unwrap_or_else(|| "User".to_string());

    view! {
        <div class="relative group">
            <img
                src=picture_url
                alt="User profile"
                class="w-10 h-10 rounded-full object-cover border-2 border-white shadow-sm cursor-pointer"
            />
            // Tooltip
            <div class="absolute bottom-full left-1/2 transform -translate-x-1/2 mb-2 px-2 py-1 bg-gray-900 text-white text-xs rounded opacity-0 group-hover:opacity-100 transition-opacity duration-200">
                {user_name}
                <div class="absolute top-full left-1/2 transform -translate-x-1/2 border-4 border-transparent border-t-gray-900"></div>
            </div>
        </div>
    }
}
