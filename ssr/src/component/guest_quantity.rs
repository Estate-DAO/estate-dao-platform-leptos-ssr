use leptos::*;

use crate::component::{Divider, HSettingIcon};
use leptos_icons::*;
use crate::page::NumberCounter;
use web_sys::*;

#[derive(Debug, Clone)]
struct GuestSelection {
    adults: RwSignal<u32>,
    children: RwSignal<u32>,
}

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

    let guest_selection = GuestSelection {
        adults: create_rw_signal(0),
        children: create_rw_signal(0),
    };
    let guest_selection_clone = guest_selection.clone();

    provide_context(guest_selection);

    let guest_count_display = create_memo(move |_prev| {
        format!(
            "{} • {}",
            pluralize(guest_selection_clone.adults.get(), "adult", "adults"),
            pluralize(guest_selection_clone.children.get(), "child", "children")
        )
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
            {{ move || guest_count_display }}
        </button>

        <div class="absolute inset-y-2 text-xl right-3 flex items-center">
            <Icon icon=icon class="text-black" />
        </div>

        <Show when=move || is_open()>
            <PeopleOptions set_is_open=set_is_open.into() />
        </Show>
    }
}


#[component]
fn PeopleOptions(set_is_open: WriteSignal<bool>) -> impl IntoView {

    let guest_selection = use_context::<GuestSelection>().unwrap();

    let apply_selection = move |_| {
        log::info!("Adults: {}, Children: {}", guest_selection.adults.get(), guest_selection.children.get());
        web_sys::console::log_1(&format!(
            "Adults: {}, Children: {}",
            guest_selection.adults.get(),
            guest_selection.children.get()
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
                <NumberCounter label="Adults" counter=guest_selection.adults class="mt-2" />
                <Divider />
                <NumberCounter label="Children" counter=guest_selection.children class="mt-2" />
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

fn pluralize(count: u32, singular: &str, plural: &str) -> String {
    if count == 1 {
        format!("{} {}", count, singular)
    } else {
        format!("{} {}", count, plural)
    }
}