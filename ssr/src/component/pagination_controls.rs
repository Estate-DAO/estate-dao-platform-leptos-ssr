use crate::view_state_layer::ui_search_state::UIPaginationState;
use leptos::html::Div;
use leptos::*;
use leptos_use::{use_intersection_observer_with_options, UseIntersectionObserverOptions};

/// Infinite-scroll pagination.
///
/// Renders a sentinel div at the end of the hotel list. When the sentinel
/// enters the extended viewport (rootMargin ≈ 5 card heights ahead), the
/// next page is fetched automatically.
///
/// When `use_button=true` (map view), falls back to a manual "Load More" button.
#[component]
pub fn PaginationControls(#[prop(optional, default = false)] use_button: bool) -> impl IntoView {
    let pagination_state: UIPaginationState = expect_context();

    // Guard: true while a page fetch has been triggered but new meta hasn't arrived yet.
    let is_loading = create_rw_signal(false);

    // Reset is_loading whenever pagination_meta changes (new data arrived).
    {
        let pagination_state = pagination_state.clone();
        create_effect(move |prev: Option<_>| {
            let meta = pagination_state.pagination_meta.get();
            if prev.is_some() {
                is_loading.set(false);
            }
            meta
        });
    }

    let sentinel_ref: NodeRef<Div> = create_node_ref();

    // Infinite-scroll observer — only active when not in button mode.
    if !use_button {
        use_intersection_observer_with_options(
            sentinel_ref,
            move |entries, _| {
                if let Some(entry) = entries.first() {
                    if entry.is_intersecting()
                        && !is_loading.get_untracked()
                        && !UIPaginationState::is_next_button_disabled()
                    {
                        is_loading.set(true);
                        UIPaginationState::go_to_next_page();
                    }
                }
            },
            UseIntersectionObserverOptions::default()
                // Pre-trigger ~5 card heights (≈250 px each) before sentinel reaches viewport.
                .root_margin("0px 0px 1250px 0px".to_string()),
        );
    }

    let handle_next_page = move |_| {
        if !is_loading.get_untracked() && !UIPaginationState::is_next_button_disabled() {
            is_loading.set(true);
            UIPaginationState::go_to_next_page();
        }
    };

    view! {
        {move || {
            let meta = pagination_state.pagination_meta.get();
            let has_next = meta.as_ref().map_or(false, |m| m.has_next_page);
            let has_loaded = meta.is_some();

            if use_button {
                // Map-view: keep manual button to avoid rootMargin quirks in nested scroll containers.
                if has_next {
                    view! {
                        <div class="flex items-center justify-center space-x-4 py-3">
                            <button
                                on:click=handle_next_page
                                class="flex items-center px-4 py-2 bg-white rounded-lg border border-gray-200
                                       shadow-sm font-medium text-gray-700 hover:bg-gray-50
                                       transition-colors duration-200 disabled:opacity-50"
                                disabled=move || is_loading.get()
                            >
                                {move || if is_loading.get() {
                                    view! {
                                        <svg class="w-4 h-4 mr-2 animate-spin" fill="none" viewBox="0 0 24 24">
                                            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"/>
                                            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"/>
                                        </svg>
                                    }.into_view()
                                } else {
                                    view! { <></> }.into_view()
                                }}
                                "Load More"
                                <svg class="w-4 h-4 ml-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
                                </svg>
                            </button>
                        </div>
                    }.into_view()
                } else if has_loaded {
                    view! {
                        <div class="flex items-center justify-center py-4 text-xs text-gray-400">
                            "All properties loaded"
                        </div>
                    }.into_view()
                } else {
                    view! { <></> }.into_view()
                }
            } else if has_next {
                // Infinite-scroll mode: sentinel div observed by the IntersectionObserver above.
                view! {
                    <div
                        node_ref=sentinel_ref
                        class="flex items-center justify-center py-6 min-h-[48px]"
                    >
                        <Show when=move || is_loading.get() fallback=move || view! { <></> }>
                            <div class="flex items-center gap-2 text-sm text-gray-400">
                                <svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
                                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"/>
                                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"/>
                                </svg>
                                "Finding more properties..."
                            </div>
                        </Show>
                    </div>
                }.into_view()
            } else if has_loaded {
                // End of results message.
                view! {
                    <div class="flex flex-col items-center justify-center py-10 text-center gap-2">
                        <div class="w-10 h-10 rounded-full bg-slate-100 flex items-center justify-center mb-1">
                            <svg class="w-5 h-5 text-slate-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                    d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"/>
                            </svg>
                        </div>
                        <p class="text-sm font-medium text-gray-500">"That's everything we've got!"</p>
                        <p class="text-xs text-gray-400">"Try different dates or a nearby destination for more options."</p>
                    </div>
                }.into_view()
            } else {
                view! { <></> }.into_view()
            }
        }}
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
