use codee::string::{FromToStringCodec, JsonSerdeCodec};
use ic_agent::identity::DelegatedIdentity;
use leptos::*;
// use leptos::{ev, prelude::*, component, view, server, ServerFnError, expect_context, create_action, window, IntoView, Children, Oco};
use leptos_use::{
    storage::{use_local_storage, use_local_storage_with_options, UseStorageOptions},
    use_cookie_with_options, use_event_listener, use_interval_fn, use_window, UseCookieOptions,
};
use web_sys::Window;

use crate::api::{
    auth::types::{LoginProvider, NewIdentity, OidcUser, ProviderKind, YralAuthMessage},
    client_side_api::ClientSideApiClient,
    consts::{APP_URL, USER_IDENTITY},
};

// use super::auth::{NewIdentity, LoginProvider, ProviderKind, YralAuthMessage, YralOAuthClient, yral_auth_url_impl};
// use super::{LoginProvButton, LoginProvCtx, ProviderKind};

#[derive(Clone, Copy)]
pub struct LoginProvCtx {
    /// Setting processing should only be done on login cancellation
    /// and inside [LoginProvButton]
    /// stores the current provider handling the login
    pub processing: RwSignal<Option<ProviderKind>>,
    pub set_processing: RwSignal<Option<ProviderKind>>,
    pub login_complete: RwSignal<Option<NewIdentity>>,
}

impl Default for LoginProvCtx {
    fn default() -> Self {
        Self {
            processing: create_rw_signal(None),
            set_processing: create_rw_signal(None),
            login_complete: create_rw_signal(None),
        }
    }
}

/// Login providers must use this button to trigger the login action
/// automatically sets the processing state to true
// #[component]
// fn LoginProvButton<Cb: Fn(ev::MouseEvent) + 'static>(
//     prov: ProviderKind,
//     #[prop(into)] class: Oco<'static, str>,
//     on_click: Cb,
//     #[prop(optional, into)] disabled: Signal<bool>,
//     children: Children,
// ) -> impl IntoView {
//     let ctx: LoginProvCtx = expect_context();

//     // let click_action = Action::new(move |()| async move {
//     //     // LoginMethodSelected.send_event(prov);
//     // });

//     view! {
//         <button
//             disabled=move || ctx.processing.get().is_some() || disabled()
//             class=class
//             on:click=move |ev| {
//                 ctx.set_processing.set(Some(prov));
//                 on_click(ev);
//                 // click_action.dispatch(());
//             }
//         >

//             {children()}
//         </button>
//     }
// }

// #[server]
// async fn yral_auth_login_url(
//     login_hint: String,
//     provider: LoginProvider,
// ) -> Result<String, ServerFnError> {
//     let oauth2: YralOAuthClient = expect_context();
//     let url = yral_auth_url_impl(oauth2, login_hint, provider, None).await?;
//     Ok(url)
// }

// todo(2025-08-08): LoginProvCtx = on page load (root) -> check cookie and set values in LoginProvCtx
//  --> if login_complete is Some, then navbar -> user profile icon
// ssr/src/component/base_route.rs from this reference file, we want to pick the following items
// 1. extract user_principal and use that to make any api calls - to backend via client_side_api.rs
// 2. routes to /logout

#[component]
pub fn YralAuthProvider() -> impl IntoView {
    let ctx: LoginProvCtx = expect_context();
    let signing_in = move || ctx.processing.get() == Some(ProviderKind::YralAuth);
    let signing_in_provider = create_rw_signal(LoginProvider::Google);

    // State for user data and authentication status
    // let (user, set_user) = create_signal::<Option<OidcUser>>(None);
    // let (loading, set_loading) = create_signal(true);

    let profile_details = Resource::local(
        || (),
        move |_| async move {
            let app_url = APP_URL.clone();
            let url = format!("{app_url}api/user-info");
            match reqwest::get(&url).await {
                Ok(response) => {
                    if response.status().is_success() {
                        if let Ok(user_data) = response.json::<OidcUser>().await {
                            return Some(user_data);
                        }
                    }
                }
                Err(_) => {
                    logging::log!("Failed to fetch user info");
                }
            }
            None
        },
    );

    view! {
        // <Show when=move || !loading.get() fallback=|| view! {
        //     <div class="flex justify-center items-center w-10 h-10">
        //         <div class="animate-spin rounded-full h-6 w-6 border-b-2 border-blue-600"></div>
        //     </div>
        // }>
            {move || profile_details.get().flatten().map(|user|{
                view! {
                    <div>
                        <UserAvatar user />
                    </div>
                }
            }).or_else(|| view!{
                <div>
                                <LoginButton
                                    signing_in=signing_in
                                    signing_in_provider=signing_in_provider
                                />
                </div>
            }.into())
        }
        // </Show>
    }
}

/// Login button component (shown when not authenticated)
#[component]
fn LoginButton(
    signing_in: impl Fn() -> bool + 'static + Clone,
    signing_in_provider: RwSignal<LoginProvider>,
) -> impl IntoView {
    let signing_in_clone = signing_in.clone();
    view! {
        <button
            class="flex gap-2 justify-center items-center px-4 py-3 sm:px-6 sm:py-3 md:gap-3 font-medium text-sm md:text-base text-gray-700 bg-white border border-gray-300 rounded-lg shadow-sm hover:bg-gray-50 hover:border-gray-400 hover:shadow-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed min-h-[44px]"
            on:click=move |ev| {
                ev.prevent_default();
                let window = web_sys::window().expect("no global window exists");
                let _ = window.location().set_href("/auth/google");
            }
            disabled=move || signing_in() && signing_in_provider.get() == LoginProvider::Google
        >
            <img class="w-5 h-5 md:w-5 md:h-5" src="/img/google.svg" alt="Google logo" />
            <span>
                {move || format!(
                    "{}Google",
                    if signing_in_clone() && signing_in_provider.get() == LoginProvider::Google {
                        "Logging in with "
                    } else {
                        "Login with "
                    },
                )}
            </span>
        </button>
    }
}

/// User avatar component (shown when authenticated)
#[component]
fn UserAvatar(user: OidcUser) -> impl IntoView {
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
