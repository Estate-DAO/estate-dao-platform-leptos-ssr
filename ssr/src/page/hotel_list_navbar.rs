use leptos::*;
use leptos_icons::*;

use crate::{component::yral_auth_provider::YralAuthProvider, page::InputGroupContainer};

#[component]
pub fn HotelListNavbar() -> impl IntoView {
    view! {
        // Fixed top bar only
        <nav class="p-2 fixed top-0 left-0 right-0 z-50 bg-white shadow-sm">
            <div class="h-14 md:h-16 flex items-center gap-8 justify-between px-4 sm:px-6">
                {/* Left: Logo */}
                <a href="/" class="flex items-center space-x-2">
                    <img src="/img/nofeebooking.webp" alt="NoFeeBooking Logo" class="h-8 sm:h-9 w-auto" />
                </a>

                {/* Center: Search (desktop only) */}
                <div class="hidden lg:flex flex-1 justify-center">
                    <div class="w-full max-w-3xl">
                        <InputGroupContainer
                        default_expanded=true
                        given_disabled=false
                        allow_outside_click_collapse=false
                        size="small"
                        />
                    </div>
                </div>

                {/* Right: Auth */}
                <div class="flex items-center space-x-3">
                    <YralAuthProvider />
                </div>
            </div>
        </nav>

        // Mobile search as a normal flow element (NOT fixed)
        <div class="lg:hidden bg-white px-4 pt-16 pb-3">
            // pt-16 matches the fixed bar height (h-14 ≈ 56px -> 14 * 4px = 56; we add a tad more to be safe)
            <InputGroupContainer
                default_expanded=false
                given_disabled=false
                allow_outside_click_collapse=false
            />
        </div>
    }
}
