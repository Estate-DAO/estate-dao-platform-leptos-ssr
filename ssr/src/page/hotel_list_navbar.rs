use leptos::*;
use leptos_icons::*;

use crate::{component::yral_auth_provider::YralAuthProvider, page::InputGroupContainer};

#[component]
pub fn HotelListNavbar() -> impl IntoView {
    view! {
        <nav class="w-full bg-white shadow-sm fixed top-0 left-0 z-50">
            <div class="max-w-7xl mx-auto flex items-center justify-between px-4 sm:px-6 lg:px-8 py-3">

                {/* --- Left: Logo --- */}
                <a href="/" class="flex items-center space-x-2">
                    <img
                        src="/img/nofeebooking.webp"
                        alt="NoFeeBooking Logo"
                        class="h-8 sm:h-9 w-auto"
                    />
                </a>

                {/* --- Center: Dynamic Search Input --- */}
                <div class="hidden md:flex flex-1 justify-center">
                    // <div class="w-full">
                        <InputGroupContainer
                            default_expanded=false
                            given_disabled=false
                            allow_outside_click_collapse=false
                        />
                    // </div>
                </div>

                {/* --- Right: Auth / Avatar --- */}
                <div class="flex items-center space-x-3">
                    {/* Mobile Search Icon */}
                    // <button class="md:hidden p-2 rounded-full hover:bg-gray-100 transition">
                    //     <Icon icon=icondata::BsSearch class="text-gray-700 text-xl" />
                    // </button>

                    {/* YralAuthProvider shows either LoginButton or UserAvatar */}
                    <YralAuthProvider />
                </div>
            </div>

            {/* --- Mobile Collapsible Search --- */}
            <div class="block md:hidden border-t border-gray-100 px-4 pb-3">
                <InputGroupContainer
                    default_expanded=false
                    given_disabled=false
                    allow_outside_click_collapse=false
                />
            </div>
        </nav>
    }
}
