use codee::string::JsonSerdeCodec;
use leptos::*;
use leptos_use::{use_cookie_with_options, UseCookieOptions};

use crate::{
    api::{
        auth::{auth_state::auth_state, types::NewIdentity},
        consts::USER_IDENTITY,
    },
    component::user_profile_icon::UserProfileIcon,
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

    // <!-- Display user principal -->
    view! {
        <div class="flex items-center gap-3">
            <div class="flex items-center gap-2 px-3 py-2 bg-gray-100 rounded-md">
                // <!-- Dynamic profile icon based on first letter of principal -->
                {move || {
                    let icon_letter = if let Some(identity) = stored_identity.get() {
                        let principal = candid::Principal::self_authenticating(&identity.id_wire.from_key);
                        let principal_text = principal.to_text();
                        // <!-- Extract first character of principal for icon -->
                        principal_text.chars().next().unwrap_or('U').to_string()
                    } else {
                        "U".to_string()
                    };

                    view! {
                        <UserProfileIcon letter=icon_letter size=32 />
                    }
                }}
                <div class="text-sm">
                    <div class="font-medium text-gray-900">User</div>
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
            </div>
            // <!-- Logout button -->
            <a
                href="/logout"
                class="px-3 py-2 text-sm font-medium text-gray-700 hover:text-gray-900 hover:bg-gray-50 rounded-md transition-colors"
            >
                Logout
            </a>
        </div>
    }
}
