use codee::string::JsonSerdeCodec;
use leptos::*;
use leptos_use::{use_cookie_with_options, UseCookieOptions};

use crate::{
    api::{
        auth::{auth_state::auth_state, types::NewIdentity},
        consts::USER_IDENTITY,
    },
    component::{profile_dropdown::ProfileDropdown, user_profile_icon::UserProfileIcon},
};

#[component]
pub fn ProfileComponent() -> impl IntoView {
    let auth = auth_state();

    let (stored_identity, _) = use_cookie_with_options::<NewIdentity, JsonSerdeCodec>(
        USER_IDENTITY,
        UseCookieOptions::default()
            .path("/")
            .same_site(leptos_use::SameSite::Lax)
            .http_only(false)
            .secure(false),
    );

    // <!-- Display user principal with dropdown -->
    view! {
        <ProfileDropdown>
            <div class="flex items-center gap-2 px-3 py-2 bg-gray-100 rounded-md hover:bg-gray-200 transition-colors">
                // <!-- Dynamic profile icon based on first letter of username, fallback to principal -->
                {move || {
                    let icon_letter = if let Some(identity) = stored_identity.get() {
                        // <!-- Priority: first letter of username -> first letter of principal -->
                        if let Some(username) = &identity.fallback_username {
                            if !username.is_empty() {
                                username.chars().next().unwrap_or('U').to_string()
                            } else {
                                let principal = candid::Principal::self_authenticating(&identity.id_wire.from_key);
                                let principal_text = principal.to_text();
                                principal_text.chars().next().unwrap_or('U').to_string()
                            }
                        } else {
                            let principal = candid::Principal::self_authenticating(&identity.id_wire.from_key);
                            let principal_text = principal.to_text();
                            principal_text.chars().next().unwrap_or('U').to_string()
                        }
                    } else {
                        "U".to_string()
                    };

                    view! {
                        <UserProfileIcon letter=icon_letter size=32 />
                    }
                }}
                <div class="text-sm">
                    <div class="font-medium text-gray-900">
                        {move || {
                            if let Some(identity) = stored_identity.get() {
                                // <!-- Priority: fallback_username -> email -> "User" -->
                                if let Some(username) = identity.fallback_username {
                                    username
                                } else if let Some(email) = identity.email {
                                    email
                                } else {
                                    "User".to_string()
                                }
                            } else {
                                "User".to_string()
                            }
                        }}
                    </div>
                    <div class="text-gray-500 text-xs font-mono">
                        {move || {
                            if let Some(identity) = stored_identity.get() {
                                let principal = candid::Principal::self_authenticating(&identity.id_wire.from_key);
                                let principal_text = principal.to_text();
                                // <!-- Show truncated principal -->
                                if principal_text.len() > 16 {
                                    format!("{}...{}", &principal_text[..8], &principal_text[principal_text.len()-8..])
                                } else {
                                    principal_text
                                }
                            } else {
                                "Not logged in".to_string()
                            }
                        }}
                    </div>
                </div>
                // <!-- Down arrow indicator -->
                <svg class="w-4 h-4 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                </svg>
            </div>
        </ProfileDropdown>
    }
}
