use crate::{
    application_services::filter_types::UISortOptions, log,
    view_state_layer::ui_search_state::UISearchCtx,
};
use leptos::*;
use leptos_icons::*;

#[component]
pub fn SortBy() -> impl IntoView {
    let (is_open, set_is_open) = create_signal(false);
    let search_ctx: UISearchCtx = expect_context();
    let current_sort = search_ctx.sort_options;

    let icon = create_memo(move |_| {
        if is_open() {
            icondata::BiChevronUpRegular
        } else {
            icondata::BiChevronDownRegular
        }
    });

    let current_sort_display = Signal::derive(move || {
        let text = current_sort.get().get_display_name();
        if text.is_empty() {
            "Select".to_string()
        } else {
            text
        }
    });

    view! {
        <div class="relative inline-flex items-center gap-2">
            <span class="text-gray-700 font-medium text-xs sm:text-sm whitespace-nowrap">"Sort By:"</span>
            <div class="relative">
                <button
                    class="flex items-center justify-between w-32 sm:w-44 bg-white border border-gray-300 text-gray-700 rounded-md px-2 sm:px-3 py-2 text-xs sm:text-sm font-medium hover:border-blue-500 focus:border-blue-500 focus:ring-2 focus:ring-blue-100 transition-all"
                    on:click=move |_| set_is_open.update(|open| *open = !*open)
                >
                    <span class="truncate">{current_sort_display}</span>
                    <Icon icon=icon class="flex-shrink-0 w-4 h-4 text-gray-500" />
                </button>

                <Show when=move || is_open()>
                    <div
                        class="absolute right-0 mt-1 w-32 sm:w-44 bg-white border border-gray-200 rounded-md shadow-lg z-50 overflow-hidden"
                    >
                        <SortOptions
                            set_is_open=set_is_open.into()
                            current_sort=current_sort
                        />
                    </div>
                </Show>
            </div>
        </div>
    }
}

#[component]
fn SortOptions(
    set_is_open: WriteSignal<bool>,
    current_sort: RwSignal<UISortOptions>,
) -> impl IntoView {
    let sort_options = vec![
        ("our_top_picks", "Top Picks"),
        ("price_low_to_high", "Price ( Low to High )"),
        ("price_high_to_low", "Price ( High to Low )"),
        ("distance_from_center", "Distance from center"),
        ("rating_high_to_low", "Rating ( High to Low )"),
        ("rating_low_to_high", "Rating ( Low to High )"),
    ];

    let selected_sort_key = Signal::derive(move || current_sort.get().to_string());

    let apply_sort = move |sort_key: String| {
        log!("Applying sort: {}", sort_key);
        let new_sort = UISortOptions::from_string(&sort_key);
        UISearchCtx::set_sort_options(new_sort);
        set_is_open.set(false);
    };

    view! {
        <div class="py-1">
            {sort_options.into_iter().map(|(key, label)| {
                let sort_key = key.to_string();
                let sort_key_clone = sort_key.clone();
                view! {
                    <SortOption
                        key=sort_key
                        label=label
                        selected=selected_sort_key
                        on_select=move |_| apply_sort(sort_key_clone.clone())
                    />
                }
            }).collect_view()}
        </div>
    }
}

#[component]
fn SortOption(
    key: String,
    label: &'static str,
    selected: Signal<String>,
    on_select: impl Fn(()) + 'static + Clone,
) -> impl IntoView {
    let is_selected = Signal::derive(move || selected.get() == key);

    view! {
        <button
            type="button"
            class={move || format!(
                "block w-full text-left px-4 py-2 text-sm transition-all {}",
                if is_selected() {
                    "bg-blue-50 text-blue-700 font-semibold"
                } else {
                    "text-gray-700 hover:bg-gray-100"
                }
            )}
            on:click=move |_| on_select(())
        >
            {label}
        </button>
    }
}
