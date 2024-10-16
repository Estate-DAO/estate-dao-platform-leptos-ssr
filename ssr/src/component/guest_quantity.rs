use leptos::*;
use leptos::logging::log;
use crate::{component::{Divider, HSettingIcon}, state::search_state::SearchCtx};
use leptos_icons::*;
use crate::page::NumberCounter;
// use web_sys::*;

// use crate::page::location_search::InputBox;

#[derive(Debug, Clone, Default)]
pub struct GuestSelection {
    pub adults: RwSignal<u32>,
    pub children: RwSignal<u32>,
    pub children_ages:  RwSignal<Vec<u32>>,
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

    let search_ctx: SearchCtx = expect_context();
    let guest_selection = search_ctx.guests;

    let guest_selection_clone = guest_selection.clone();

    let guest_count_display = create_memo(move |_prev| {
        format!(
            "{} • {}",
            pluralize(guest_selection_clone.get().adults.get(), "adult", "adults"),
            pluralize(guest_selection_clone.get().children.get(), "child", "children")
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

    let search_ctx: SearchCtx = expect_context();

    let guest_selection = search_ctx.guests; 
    let apply_selection = move |ev| {
        log::info!("Apply selection- {ev:?}");
        log::info!("Adults: {}, Children: {}, Ages: {:?}", guest_selection.get().adults.get_untracked(), guest_selection.get().children.get_untracked(), guest_selection.get().children_ages.get_untracked());
       
        // apply the changes to guest_selection_orig
        SearchCtx::set_adults(guest_selection.get().adults.get());
        SearchCtx::set_children(guest_selection.get().children.get());
        SearchCtx::set_children_ages(guest_selection.get().children_ages.get());
        set_is_open.set(false);
    };


    view! {
        <div class="p-4">
            <div
                id="guestsDropdownContent"
                class="absolute right-0 bg-white rounded-md shadow-lg mt-10 borderSortOptions border-gray-300 rounded-xl border border-gray-200 px-4"
            >
                <NumberCounter label="Adults" counter=guest_selection.get().adults class="mt-2" />
                <Divider />
                <NumberCounter
                    label="Children"
                    counter=guest_selection.get().children
                    class="mt-2"
                />
                <div class="flex flex-wrap">
                    // Add number input fields for children ages
                    {move || {
                        (0..guest_selection.get().children.get())
                            .map(|i| {
                                view! {
                                    <input
                                        type="number"
                                        class="mt-2 ml-3 p-2 border border-gray-300 w-16"
                                        name=format!("child_age[{}]", i)
                                        placeholder="Age"
                                        on:input=move |e| {
                                            let age = event_target_value(&e);
                                            log!("{}",age);
                                        }
                                    />
                                }
                            })
                            .collect::<Vec<_>>()
                            .into_view()
                    }}
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