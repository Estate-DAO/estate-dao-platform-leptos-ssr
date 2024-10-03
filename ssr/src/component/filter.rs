use leptos::*;

use crate::component::{Divider, HSettingIcon};
use leptos_icons::*;

/// Filter component (button)
#[component]
pub fn Filter() -> impl IntoView {
    let (is_open, set_is_open) = create_signal(false);

    view! {
        <div class="relative">
            <button 
            class="bg-white text-black px-4 py-2 rounded-lg flex items-center border border-gray-300"
            on:click=move |_| set_is_open.update(|open| *open = !*open)
            >
                <Icon class="w-5 h-5 mr-2" icon=HSettingIcon />
                Filters
            </button>

            <Show when=move || is_open()>
                <div class="absolute mt-2 w-52 bg-white borderSortOptions border-gray-300 rounded-xl shadow-lg">
                    <SortOptions/>
                </div>
            </Show>
        </div>
    }
}

#[component]
fn SortOptions() -> impl IntoView {
    let selected = create_rw_signal("Name");

    view! {
        <form class="p-4">
            <div class="space-y-4">
                <SortOption name="sort" value="Price Low to high" selected=selected/>
        <Divider />

                <SortOption name="sort" value="Popularity" selected=selected/>
        <Divider />

                <SortOption name="sort" value="Rating high to low" selected=selected/>
            </div>
            <button type="button" class="w-full mt-4 bg-white border border-black-2 text-black px-4 py-2 rounded-full">
                "Apply"
            </button>
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
            <input
                type="radio"
                name=name
                value=value
                checked=move || selected.get() == value
                on:change=move |_| selected.set(value)
            />
            <span>{value}</span>
        </label>
    }
}