use crate::view_state_layer::ui_search_state::UIPaginationState;
use leptos::*;
use leptos_use::use_intersection_observer;

#[component]
pub fn PaginationControls() -> impl IntoView {
    let observer_target = create_node_ref::<html::Div>();

    use_intersection_observer(observer_target, move |entries, _| {
        if let Some(entry) = entries.first() {
            if entry.is_intersecting() && !UIPaginationState::is_next_button_disabled() {
                UIPaginationState::go_to_next_page();
            }
        }
    });

    view! {
        <div class="flex flex-col items-center justify-center space-y-4 py-6">
            <Show
                when=move || !UIPaginationState::is_next_button_disabled()
                fallback=move || view! {
                    <div class="text-gray-500 text-sm font-medium py-4">
                        "That's all the properties we have for now ✨"
                    </div>
                }
            >
                <div class="flex flex-col items-center">
                    <div class="w-6 h-6 border-2 border-blue-500 border-t-transparent rounded-full animate-spin mb-2"></div>
                    <span class="text-sm text-gray-500">"Loading more properties..."</span>
                    // Invisible target for intersection observer
                    <div node_ref=observer_target class="h-10 w-full pointer-events-none absolute -bottom-40"></div>
                </div>
            </Show>
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
