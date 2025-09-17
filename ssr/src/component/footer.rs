use leptos::*;
use leptos_icons::*;

use crate::{app::AppRoutes, page::AccountTabs};

#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer class="bg-neutral-900 text-gray-300 py-16 px-6 sm:px-12">
            <div class="max-w-8xl mx-auto grid grid-cols-1 lg:grid-cols-2 gap-12">
                {/* Left section (Logo + Info) */}
                <div class="space-y-6 order-2 lg:order-1">
                    <div class="flex items-center space-x-2">
                        <img src="/img/nofeebooking.webp" alt="NoFee Booking Logo" class="h-8 w-auto" />
                        <span class="text-xl font-semibold text-white">"NoFee Booking"</span>
                    </div>

                    <p class="text-sm">
                        "First decentralised booking platform powered by ICP."
                    </p>

                    <img src="/img/icp.svg" alt="Internet Computer Logo" class="h-6 w-auto" />

                    <p class="text-xs text-gray-400">
                        "Copyright © 2024 EstateDao. All Rights Reserved."
                    </p>
                </div>

                {/* Right section (Company + Support) */}
                <div class="grid grid-cols-2 gap-8 order-1 lg:order-2">
                    <div class="space-y-3">
                        <h3 class="text-white font-semibold">"Company"</h3>
                        <ul class="space-y-2 text-sm">
                            <li><a href="/about" class="hover:text-white">"About Us"</a></li>
                            <li><a href=AppRoutes::MyBookings.to_string() class="hover:text-white">"My Trips"</a></li>
                        </ul>
                    </div>

                    <div class="space-y-3">
                        <h3 class="text-white font-semibold">"Support"</h3>
                        <ul class="space-y-2 text-sm">
                            <li><a href=AccountTabs::Support.as_route() class="hover:text-white">"Help Centre"</a></li>
                            <li><a href=AccountTabs::Support.as_route() class="hover:text-white">"FAQ"</a></li>
                            <li><a href=AccountTabs::Privacy.as_route() class="hover:text-white">"Privacy Policy"</a></li>
                            <li><a href=AccountTabs::Terms.as_route() class="hover:text-white">"Terms & Conditions"</a></li>
                        </ul>

                        <div class="flex space-x-4 pt-2">
                            <a href="https://x.com/estatedao_icp?s=11" target="_blank" class="text-blue-400 hover:text-white">
                                <Icon icon=icondata::BiTwitter />
                            </a>
                            <a href="https://www.facebook.com/profile.php?id=61576590939204" target="_blank" class="text-blue-400 hover:text-white">
                                <Icon icon=icondata::BiFacebook />
                            </a>
                            <a href="https://www.instagram.com/estatedao_/" target="_blank" class="text-blue-400 hover:text-white">
                                <Icon icon=icondata::IoLogoInstagram />
                            </a>
                        </div>
                    </div>
                </div>
            </div>
        </footer>
    }
}
