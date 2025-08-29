use crate::utils::browser;
use crate::view_state_layer::ui_search_state::UIPaginationState;
use leptos::*;

#[component]
pub fn PaginationControls() -> impl IntoView {
    let pagination_state: UIPaginationState = expect_context();

    let handle_previous_page = move |_| {
        UIPaginationState::go_to_previous_page();

        // Scroll to top after pagination
        browser::scroll_to_top();
    };

    let handle_next_page = move |_| {
        // crate::log!("[PAGINATION-DEBUG] 🔄 Next button clicked!");
        UIPaginationState::go_to_next_page();
        // crate::log!("[PAGINATION-DEBUG] 🔄 go_to_next_page() called");

        // Scroll to top after pagination
        browser::scroll_to_top();
    };

    view! {
        <div class="flex items-center justify-center space-x-4 py-6">
            <button
                on:click=handle_previous_page
                disabled=move || UIPaginationState::is_previous_button_disabled()
                class="flex items-center px-4 py-2 bg-blue-600 text-white rounded-lg font-medium
                       hover:bg-blue-700 disabled:bg-gray-300 disabled:cursor-not-allowed 
                       transition-colors duration-200"
            >
                <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7"></path>
                </svg>
                "Previous"
            </button>

            <div class="flex items-center space-x-2">
                <span class="text-gray-600">
                    "Page "
                    <span class="font-semibold text-gray-800">
                        {move || pagination_state.current_page.get().to_string()}
                    </span>
                </span>

                // {move || {
                //     pagination_state.pagination_meta.get()
                //         .map(|meta| {
                //             view! {
                //                 <span class="text-gray-500 text-sm">
                //                     "(" {meta.page_size} " per page)"
                //                 </span>
                //             }.into_view()
                //         })
                //         .unwrap_or_else(|| view! { <span></span> }.into_view())
                // }}
            </div>

            <button
                on:click=handle_next_page
                disabled=move || UIPaginationState::is_next_button_disabled()
                class="flex items-center px-4 py-2 bg-blue-600 text-white rounded-lg font-medium
                       hover:bg-blue-700 disabled:bg-gray-300 disabled:cursor-not-allowed 
                       transition-colors duration-200"
            >
                "Next"
                <svg class="w-4 h-4 ml-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"></path>
                </svg>
            </button>
        </div>
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
