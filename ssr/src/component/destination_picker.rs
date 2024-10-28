use leptos::*;

use crate::{
    component::{Divider, HSettingIcon},
    state::search_state::SearchCtx,
    api::Destination
};
use leptos_icons::*;

#[component]
pub fn DestinationPicker() -> impl IntoView {
    let (is_open, set_is_open) = create_signal(false);
    let search_ctx: SearchCtx = expect_context();
    
    let destinations = vec![
        Destination {
            city: "Goa".to_string(),
            country: "India".to_string(),
            city_id: 1254,
        },
        Destination {
            city: "Barcelona".to_string(),
            country: "Spain".to_string(),
            city_id: 2101,
        },
        Destination {
            city: "Lucknow".to_string(),
            country: "India".to_string(),
            city_id: 1432,
        },
        Destination {
            city: "Delhi".to_string(),
            country: "India".to_string(),
            city_id: 1193,
        },
        Destination {
            city: "Tokyo".to_string(),
            country: "Japan".to_string(),
            city_id: 3301,
        },
        Destination {
            city: "Paris".to_string(),
            country: "France".to_string(),
            city_id: 4201,
        },
    ];

    let display_value = create_memo(move |_| {
        search_ctx.destination.get()
            .map(|d| format!("{}, {}", d.city, d.country))
            .unwrap_or_else(|| "Where to?".to_string())
    });

    view! {
        <div class="relative w-full">
            <div class="w-full" on:click=move |_| set_is_open.update(|open| *open = !*open)>
                <input
                    type="text"
                    placeholder="Where to?"
                    class="w-full ml-2 py-2 pl-8 text-gray-800 bg-transparent border-none focus:outline-none text-sm"
                    prop:value=display_value
                    readonly=true
                />
            </div>

            <Show when=move || is_open()>
                <div class="absolute mt-2 w-80 bg-white borderSortOptions border border-gray-200 rounded-xl shadow-lg z-50">
                    <div class="p-4">
                        <div class="space-y-4">
                            {destinations.iter().map(|dest| {
                                let dest_for_click = dest.clone();
                                let dest_for_display = dest.clone();
                                view! {
                                    <div
                                        class="cursor-pointer hover:bg-gray-50 p-2 rounded"
                                        on:click=move |_| {
                                            SearchCtx::set_destination(dest_for_click.clone());
                                            set_is_open.set(false);
                                        }
                                    >
                                        <span class="text-gray-800">
                                            {format!("{}, {}", dest_for_display.city, dest_for_display.country)}
                                        </span>
                                    </div>
                                    <Divider />
                                }
                            }).collect_view()}
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}

#[component]
fn SortOptions(
    destinations: Vec<Destination>,
    set_is_open: WriteSignal<bool>
) -> impl IntoView {
    let search_ctx: SearchCtx = expect_context();

    view! {
        <form class="p-4">
            <div class="space-y-4">
                {destinations.into_iter().map(|dest| {
                    let dest_clone = dest.clone();
                    view! {
                        <label class="flex items-center space-x-2 cursor-pointer">
                            <span 
                                on:click=move |_| {
                                    SearchCtx::set_destination(dest_clone.clone());
                                    set_is_open.set(false);  // Close dropdown after selection
                                }
                            >
                                {format!("{}, {}", dest.city, dest.country)}
                            </span>
                        </label>
                        <Divider />
                    }
                }).collect_view()}
            </div>
        </form>
    }
}

#[component]
fn SortOption(
    name: &'static str,
    value: &'static str,
    selected: RwSignal<&'static str>,
) -> impl IntoView {
    view! {
        <label class="flex items-center space-x-2 cursor-pointer">
            <span on:click=move |_| selected.set(value)>{value}</span>
        </label>
    }
}
