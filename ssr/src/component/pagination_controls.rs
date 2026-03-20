use crate::view_state_layer::ui_search_state::UIPaginationState;
use leptos::*;

#[component]
pub fn PaginationControls() -> impl IntoView {
    let handle_next_page = move |_| {
        UIPaginationState::go_to_next_page();
    };

    view! {
        <Show
            when=move || !UIPaginationState::is_next_button_disabled()
            fallback=move || view! { <></> }
        >
            <div class="flex items-center justify-center space-x-4 py-3">
                <button
                    on:click=handle_next_page
                    class="flex items-center px-4 py-2 bg-white-600 rounded-lg font-medium
                           transition-colors duration-200"
                >
                    "Load More"
                    <svg class="w-4 h-4 ml-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"></path>
                    </svg>
                </button>
            </div>
        </Show>
    }
}

#[component]
pub fn PaginationInfo() -> impl IntoView {
    let pagination_state: UIPaginationState = expect_context();

    view! {
        <div class="text-center text-sm text-gray-600 pb-4">
            {move || {
                pagination_state.pagination_meta.get()
                    .map(|meta| {
                        let start = (meta.page - 1) * meta.page_size + 1;
                        let end = meta.page * meta.page_size;

                        view! {
                            <span>
                                "Showing "
                                <span class="font-semibold">{start.to_string()}</span>
                                " - "
                                <span class="font-semibold">{end.to_string()}</span>
                                " results"
                                {meta.total_results
                                    .map(|total| format!(" of {}", total))
                                    .unwrap_or_default()}
                            </span>
                        }.into_view()
                    })
                    .unwrap_or_else(|| view! { <span></span> }.into_view())
            }}
        </div>
    }
}
