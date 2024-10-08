use leptos::*;

use crate::component::{Divider, HSettingIcon};
use leptos_icons::*;
use crate::page::NumberCounter;
use web_sys::*;

/// Guest quantity component (button)
#[component]
pub fn GuestQuantity() -> impl IntoView {
    let (is_open, set_is_open) = create_signal(false);

    let icon = create_memo(move |_| {
        if is_open() {
            icondata::BiChevronUpRegular
        } else {
            icondata::BiChevronDownRegular
        }
    });

    view! {
        <div class="absolute inset-y-0 left-2 flex items-center text-2xl">
            <Icon icon=icondata::BsPerson class="text-black font-light" />
        </div>

        <button
            id="guestsDropdown"
            class="w-full flex-0 py-2 pl-10 text-left text-gray-700 text-sm font-light bg-transparent rounded-full focus:outline-none"
            // on:blur=move |_| set_is_open.set(false)
            on:click=move |_| set_is_open.update(|open| *open = !*open)
        >
            "0 adult • 0 children"
        </button>

        <div class="absolute inset-y-2 text-xl right-3 flex items-center">
            <Icon icon=icon class="text-black" />
        </div>

        <Show when=move || is_open()>
            <SortOptions set_is_open=set_is_open.into() />
        </Show>
    }
}

#[component]
fn SortOptions(set_is_open: WriteSignal<bool>) -> impl IntoView {
    let selected_adults: RwSignal<i32> = create_rw_signal(0);
    let selected_children: RwSignal<i32> = create_rw_signal(0);

    let apply_selection = move |_| {
        log::info!("Adults: {}, Children: {}", selected_adults(), selected_children());
        web_sys::console::log_1(&format!(
            "Adults: {}, Children: {}",
            selected_adults(),
            selected_children()
        )
        .into());
        set_is_open.set(false);
    };

    view! {
        <div class="p-4">
            <div
                id="guestsDropdownContent"
                class="absolute right-0 bg-white rounded-md shadow-lg mt-10 borderSortOptions border-gray-300 rounded-xl border border-gray-200 px-4"
            >
                <NumberCounter label="Adults" counter=selected_adults class="mt-2" />
                <Divider />
                <NumberCounter label="Children" counter=selected_children class="mt-2" />
                <br />
                <button
                    type="button"
                    class="w-full mb-4 bg-white border border-black-2 text-black py-2 rounded-full"
                    on:click=apply_selection
                >
                    "Apply"
                </button>
            </div>
        </div>
    }
}
