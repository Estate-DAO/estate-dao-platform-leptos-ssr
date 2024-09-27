use leptos::*;

use crate::component::HSettingIcon;
use leptos_icons::*;

/// Filter component (button)
#[component]
pub fn Filter() -> impl IntoView {
    view! {
        <button class="bg-white text-black px-4 py-2 rounded-lg flex items-center border border-gray-300">
            <Icon class="w-5 h-5 mr-2" icon=HSettingIcon />
            Filters
        </button>
    }
}
