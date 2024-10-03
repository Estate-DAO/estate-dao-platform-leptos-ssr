use leptos::*;

use crate::component::{Divider, HSettingIcon};
use leptos_icons::*;
use crate::page::NumberCounter;

/// Guest quantity component (button)
#[component]
pub fn GuestQuantity() -> impl IntoView {
    let (is_open, set_is_open) = create_signal(false);

    view! {
        <div class="absolute inset-y-0 left-2 flex items-center text-2xl">
            <Icon icon=icondata::BsPerson class="text-black font-light" />
        </div>

        <button
            id="guestsDropdown"
            class="w-full flex-0 py-2 pl-10 text-left text-gray-700 text-sm font-light bg-transparent rounded-full focus:outline-none"
            on:blur=move |_| set_is_open.set(false)
            on:click=move |_| set_is_open.update(|open| *open = !*open)
        >
            "0 adult • 0 children"
        </button>

        <div class="absolute inset-y-2 text-xl right-3 flex items-center">
            <Icon icon=icondata::BiChevronDownRegular class="text-black" />
        </div>

        <Show when=move || is_open()>
            <SortOptions/>
        </Show>
    }
}

#[component]
fn SortOptions() -> impl IntoView {
    let selected_adults: RwSignal<i32> = create_rw_signal(0);
    let selected_children: RwSignal<i32> = create_rw_signal(0);

    view! {
        <form class="p-4">
            <div
            id="guestsDropdownContent"
            class="absolute right-0 w-48 bg-white rounded-md shadow-lg absolute mt-10 w-52 bg-white borderSortOptions border-gray-300 rounded-xl border border-gray-200"
            >
                <div class="px-4">
                    <SortOption name="Adults" value=selected_adults() selected=selected_adults/>
                    <Divider />
                    <SortOption name="Children" value=selected_children() selected=selected_children/>
                    <br />
                    <button type="button" class="w-full mb-4 bg-white border border-black-2 text-black py-2 rounded-full">
                        "Apply"
                    </button>
                </div>

            </div>
        </form>

    }
}

#[component]
fn SortOption(
    name: &'static str,
    value: i32,
    selected: RwSignal<i32>,
) -> impl IntoView {
    view! {
        <div class="flex flex-col">
            <NumberCounter label=name counter=selected class="mt-2" on:click=move |_| selected.set(value) />
        </div>
    }
}
