use leptos::*;
use leptos_icons::*;

use crate::{component::yral_auth_provider::YralAuthProvider, page::InputGroupContainer};

#[component]
pub fn HotelListNavbar() -> impl IntoView {
    view! {
        // Fixed top bar only
        <nav class="hidden lg:block p-2 fixed top-0 left-0 right-0 z-50 bg-white shadow-sm">
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

        // Mobile Header (lg:hidden)
        <nav class="lg:hidden fixed top-0 left-0 right-0 z-[1001] bg-white shadow-sm h-14 flex items-center justify-between px-4">
            <div class="flex items-center">
                <button
                    class="p-2 -ml-2 text-gray-700"
                    on:click=move |_| {
                        // navigate("/", Default::default()); // Or history back?
                        // For now, let's just go home as a safe default or use window history if possible.
                         if let Some(w) = web_sys::window() {
                             let _ = w.history().unwrap().back();
                         }
                    }
                >
                    <Icon icon=icondata::BsChevronLeft class="w-6 h-6" />
                </button>
            </div>

            <div class="absolute left-1/2 transform -translate-x-1/2">
                 <a href="/" class="flex items-center">
                    <img src="/img/nofeebooking.webp" alt="NoFeeBooking" class="h-8 w-auto" />
                </a>
            </div>

            <div class="flex items-center">
                <YralAuthProvider />
            </div>
        </nav>
    }
}
