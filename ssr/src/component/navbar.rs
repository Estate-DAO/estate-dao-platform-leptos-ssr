use crate::api::{auth::types::NewIdentity, consts::USER_IDENTITY};
use crate::component::{profile_component::ProfileComponent, yral_auth_provider::YralAuthProvider};
use codee::string::JsonSerdeCodec;
use leptos::SignalGetUntracked;
use leptos::*;
use leptos_use::{use_cookie_with_options, UseCookieOptions};

#[component]
pub fn Navbar() -> impl IntoView {
    // <!-- Check if user is logged in by reading USER_IDENTITY cookie -->
    let (stored_identity, _) = use_cookie_with_options::<NewIdentity, JsonSerdeCodec>(
        USER_IDENTITY,
        UseCookieOptions::default()
            .path("/")
            .same_site(leptos_use::SameSite::Lax)
            .http_only(false)
            .secure(false),
    );

    crate::log!(
        "AUTH_FLOW: navbar - USER_IDENTITY cookie check - is_some: {}, data: {:?}",
        stored_identity.get_untracked().is_some(),
        stored_identity.get_untracked()
    );

    view! {
        // <!-- Improved mobile navbar with better padding and icon sizing -->
        <nav class="flex justify-between items-center py-6 sm:py-8 md:py-10 px-4 sm:px-6 md:px-8">
            <div class="flex items-center text-xl">
                // <Icon icon=EstateDaoIcon />
                <a href="/" class="flex items-center">
                    <img
                        src="/img/nofeebooking.webp"
                        alt="NoFeeBooking Logo"
                        class="h-8 w-24 sm:h-10 sm:w-32 md:h-12 md:w-48 object-contain"
                    />
                </a>
            </div>
            // <div class="flex space-x-8">
                // <a href="#" class="text-gray-700 hover:text-gray-900">
                //     Whitepaper
                // </a>
                // <a href="#" class="text-gray-700 hover:text-gray-900">
                //     About us
                // </a>

                // <button />
            // </div>
            // <!-- Conditional rendering based on login state -->
            {move || {
                if stored_identity.get().is_some() {
                    // <!-- User is logged in, show profile component -->
                    view! { <ProfileComponent /> }.into_view()
                } else {
                    // <!-- User is not logged in, show auth provider -->
                    view! { <YralAuthProvider /> }.into_view()
                }
            }}
        </nav>
    }
}
