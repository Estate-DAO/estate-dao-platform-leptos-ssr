use leptos::*;
use leptos::logging::log;
use crate::component::{Divider, HSettingIcon};
use leptos_icons::*;
use crate::page::NumberCounter;
use web_sys::*;

// use crate::page::location_search::InputBox;


#[derive(Debug, Clone)]
struct GuestSelection {
    adults: RwSignal<u32>,
    children: RwSignal<u32>,
    children_ages:  RwSignal<Vec<u32>>,
}

impl Default for GuestSelection {
    fn default() -> Self {
        GuestSelection {
            adults: create_rw_signal(0),
            children: create_rw_signal(0),
            children_ages: create_rw_signal(vec![]),
        }
    }
}

impl GuestSelection {
    pub fn update_children_ages(&mut self, ages: Vec<u32>) {
        if self.children.get() >= 1 && ages.len() <= self.children.get() as usize {
            self.children_ages.update(|ages_vec| ages_vec.extend(ages));
        }
    }


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

    let guest_selection = GuestSelection::default();

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
                <div class="flex flex-wrap">
                // Add number input fields for children ages
                {move || (0..guest_selection.children.get()).map(|i| view! {
                    <input
                        type="number"
                        class="mt-2 ml-3 p-2 border border-gray-300 w-16"
                        name={format!("child_age[{}]",i)}
                        placeholder=format!("Child {} Age", i + 1)
                        on:input=move |e| {
                            let age = event_target_value(&e);
                            log!("{}",age);
                            // guest_children.update_children_ages()
                        }
                    />

                    // <InputBox
                    // // heading=""
                    // placeholder="Where to?"
                    // updater=set_location
                    // validator=non_empty_string_validator
                    // initial_value=ctx.form_state.with_untracked(|f| f.location.clone()).unwrap_or_default()
                    // />
                }).collect::<Vec<_>>() }
                </div>
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