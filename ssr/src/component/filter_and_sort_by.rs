use leptos::*;

use crate::component::{Filter, SortBy};

/// Filter component (button)
#[component]
pub fn FilterAndSortBy() -> impl IntoView {
    view! {
        <div class="flex space-x-4 p-6">
            <Filter />
            <SortBy />
        </div>
    }
}
