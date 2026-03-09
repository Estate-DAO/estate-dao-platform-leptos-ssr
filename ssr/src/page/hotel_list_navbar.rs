use leptos::*;

use crate::{
    component::{yral_auth_provider::YralAuthProvider, CurrencySelectorModal},
    page::InputGroupContainer,
};

#[component]
pub fn HotelListNavbar() -> impl IntoView {
    view! {
        // Fixed top bar only
        <nav class="hidden lg:block p-2 fixed top-0 left-0 right-0 z-50 bg-white shadow-sm">
            <div class="h-14 md:h-16 flex items-center gap-8 justify-between px-4 sm:px-6">
                {/* Left: Logo */}
                <a href="/" class="flex items-center space-x-2">
                    <img src="/img/nofeebooking.webp" alt="NoFeeBooking Logo" class="h-9 sm:h-10 w-auto" />
                </a>

                {/* Center: Search (desktop only) */}
                <div class="hidden lg:flex flex-1 justify-center">
                    <div class="w-full max-w-3xl">
                        <InputGroupContainer
                        default_expanded=true
                        given_disabled=false
                        allow_outside_click_collapse=false
                        size="small"
                        auto_search_on_place_select=true
                        />
                    </div>
                </div>

                {/* Right: Currency + Auth */}
                <div class="flex items-center space-x-3">
                    <CurrencySelectorModal />
                    <YralAuthProvider />
                </div>
            </div>
        </nav>

        // Mobile Header (lg:hidden)
        <nav class="lg:hidden fixed top-0 left-0 right-0 z-[1001] bg-white shadow-sm h-14 flex items-center justify-between px-4">
            <a href="/" class="flex items-center">
                <img src="/img/nofeebooking.webp" alt="NoFeeBooking" class="h-9 w-auto" />
            </a>

            <div class="flex items-center gap-2">
                <CurrencySelectorModal />
                <YralAuthProvider />
            </div>
        </nav>
    }
}
