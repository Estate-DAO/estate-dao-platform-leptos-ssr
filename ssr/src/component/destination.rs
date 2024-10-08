use leptos::*;

use crate::component::{Divider, HSettingIcon};
use leptos_icons::*;

/// Destination component (input)
#[component]
pub fn Destination() -> impl IntoView {
    let (is_open, set_is_open) = create_signal(false);

    view! {
        <div class="absolute inset-y-0 left-2 text-xl flex items-center">
            <Icon icon=icondata::BsMap class="text-black" />
        </div>

        <input
            type="text"
            placeholder="Destination"
            class="w-full ml-2 py-2 pl-8 text-gray-800 bg-transparent border-none focus:outline-none text-sm"
            on:blur=move |_| set_is_open.set(false)
            on:click=move |_| set_is_open.update(|open| *open = !*open)
        />

        <Show when=move || is_open()>
            <div class="absolute mt-6 w-80 bg-white borderSortOptions border border-gray-200 rounded-xl shadow-lg">
                <SortOptions />
            </div>
        </Show>
    }
}

#[component]
fn SortOptions() -> impl IntoView {
    let selected = create_rw_signal("Name");

    view! {
        <form class="p-4">
            <div class="space-y-4">
                <label class="flex items-center space-x-2 cursor-pointer">
                    <span on:click=move |_| selected.set("Italy")>"Italy"</span>
                </label>
                <Divider />

                <label class="flex items-center space-x-2 cursor-pointer">
                    <span on:click=move |_| selected.set("Spain")>"Spain"</span>
                </label>
                <Divider />

                <label class="flex items-center space-x-2 cursor-pointer">
                    <span on:click=move |_| selected.set("France")>"France"</span>
                </label>
                <Divider />

                <label class="flex items-center space-x-2 cursor-pointer">
                    <span on:click=move |_| selected.set("Athens")>"Athens"</span>
                </label>
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
