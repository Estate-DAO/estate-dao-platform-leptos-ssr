use crate::api::consts::USER_IDENTITY;
use crate::component::yral_auth_provider::YralAuthProvider;
use leptos::*;
use leptos_use::{use_cookie_with_options, UseCookieOptions};

#[component]
pub fn Navbar(#[prop(optional)] blue_header: bool) -> impl IntoView {
    let class = if blue_header {
        "bg-blue-600 flex justify-between items-center py-6 sm:py-8 md:py-10 px-4 sm:px-6 md:px-8 relative z-[1001]"
    } else {
        "flex justify-between items-center py-6 sm:py-8 md:py-10 px-4 sm:px-6 md:px-8 relative z-[1001]"
    };
    let logo = if blue_header {
        "/img/logo_white.svg"
    } else {
        "/img/nofeebooking.webp"
    };
    view! {
        // <!-- Improved mobile navbar with better padding and icon sizing -->
        <nav class=class>
            <div class="flex items-center text-xl">
                // <Icon icon=EstateDaoIcon />
                <a href="/" class="flex items-center">
                    <img
                        src=logo
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
                view! { <YralAuthProvider /> }.into_view()
            }}
        </nav>
    }
}
