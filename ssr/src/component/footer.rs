use leptos::*;
use leptos_icons::*;

#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <div class="py-16 px-20 flex items-center justify-between">
            <div class="flex items-center space-x-6">
                <div class="font-semibold text-xl">hello@estatedao.com</div>
                <div class="text-xl">
                    <Icon icon=icondata::IoLogoInstagram />
                </div>
                <div class="text-xl">
                    <Icon icon=icondata::BiLinkedin />
                </div>

            </div>
            <div class="text-gray-400 font-semibold">
                "Copyright © 2025 EstateDao. All Rights Reserved."
            </div>
        </div>
    }
}
