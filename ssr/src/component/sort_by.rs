use leptos::*;

use leptos_icons::*;

/// Filter component (button)
#[component]
pub fn SortBy() -> impl IntoView {
    view! {
        <button class="bg-white text-black px-4 py-2 rounded-lg flex items-start border border-gray-300">
            Sort by <Icon icon=icondata::BiChevronDownRegular class="w-6 h-6 ml-2" />
        </button>
    }
}
