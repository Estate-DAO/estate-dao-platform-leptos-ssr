use crate::api::consts::SEARCH_COMPONENT_ROOMS_DEFAULT;
use crate::api::RoomGuest;
use crate::state::input_group_state::{InputGroupState, OpenDialogComponent};
// use crate::page::NumberCounter;
use crate::utils::pluralize;
use crate::{
    component::{Divider, HSettingIcon, NumberCounter},
    state::search_state::SearchCtx,
};
use ev::{InputEvent, MouseEvent};
// use leptos::logging::log;
use crate::log;
use leptos::*;
use leptos_icons::*;
use std::ops::Index;

#[derive(Debug, Clone)]
pub struct GuestSelection {
    pub adults: RwSignal<u32>,
    pub children: RwSignal<u32>,
    pub rooms: RwSignal<u32>,
    pub children_ages: ChildrenAges,
}

impl Default for GuestSelection {
    fn default() -> Self {
        Self {
            adults: RwSignal::new(2),
            children: RwSignal::new(0),
            rooms: RwSignal::new(SEARCH_COMPONENT_ROOMS_DEFAULT), // Set default value for rooms to 1
            children_ages: ChildrenAges::default(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ChildrenAges(RwSignal<Vec<u32>>);

impl ChildrenAges {
    pub fn get_untracked(&self) -> Vec<u32> {
        self.0.get_untracked().iter().map(|age| *age).collect()
    }

    pub fn get_value_at(&self, index: u32) -> u32 {
        *self.0.get_untracked().get(index as usize).unwrap_or(&5)
    }

    pub fn update_children_ages(&self, index: u32, age: u32) {
        self.0.update(|f| f[index as usize] = age);
    }

    pub fn push_children_ages(&self) {
        self.0.update(|f| f.push(10));
    }

    pub fn pop_children_ages(&self) {
        self.0.update(|f| {
            let _a = f.pop();
        });
    }
}

impl GuestSelection {
    pub fn get_room_guests(search_ctx: &SearchCtx) -> Vec<RoomGuest> {
        let guest_selection = search_ctx.guests;

        let no_of_adults = guest_selection.get_untracked().adults.get_untracked();
        let no_of_child = guest_selection.get_untracked().children.get_untracked();
        let children_ages: Vec<String> = guest_selection
            .get_untracked()
            .children_ages
            .get_untracked()
            .iter()
            .map(|age| age.to_string())
            .collect();

        let child_age = if no_of_child > 0 {
            Some(children_ages)
        } else {
            None
        };

        vec![RoomGuest {
            no_of_adults,
            no_of_child,
            child_age,
        }]
    }

    // pub fn reactive_length(&self) {
    //     let no_of_child = self.children.get_untracked();
    //     if no_of_child > 0 {
    //         self.children_ages.push_children_ages();
    //     } else {
    //         self.children_ages.pop_children_ages();
    //     }
    // }
}

/// Guest quantity component (button)
#[component]
pub fn GuestQuantity() -> impl IntoView {
    let is_open = create_memo(move |_| InputGroupState::is_guest_open());

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
            pluralize(
                guest_selection_clone.get().children.get(),
                "child",
                "children"
            )
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
            on:click=move |_| InputGroupState::toggle_dialog(OpenDialogComponent::GuestComponent)
        >
            {{ move || guest_count_display }}
        </button>

        <div class="absolute inset-y-2 text-xl right-3 flex items-center">
            <Icon icon=icon class="text-black" />
        </div>

        <Show when=move || is_open()>
            <PeopleOptions />
        </Show>
    }
}

#[component]
fn PeopleOptions() -> impl IntoView {
    let search_ctx: SearchCtx = expect_context();

    let guest_selection = search_ctx.guests;
    let apply_selection = move |ev: MouseEvent| {
        // log::info!("Apply selection- {ev:?}");
        ev.prevent_default();

        log::info!(
            "Adults: {}, Children: {}, Ages: {:?}",
            guest_selection.get().adults.get_untracked(),
            guest_selection.get().children.get_untracked(),
            guest_selection.get().children_ages.get_untracked()
        );

        SearchCtx::log_state();
        InputGroupState::toggle_dialog(OpenDialogComponent::GuestComponent)
    };

    create_effect(move |_| {
        let children_ages = guest_selection.get().children_ages;

        let num_children = children_ages.0.get().len() as u32;

        let push_signal = num_children < guest_selection.get().children.get();
        let should_not_act = num_children == guest_selection.get().children.get();

        if should_not_act {
            return;
        } else {
            if push_signal {
                children_ages.push_children_ages();
            } else {
                children_ages.pop_children_ages();
            }
        }
        // log::info!("Push signal: {:?}", push_signal);
        // log::info!("Children ages: {:?}", children_ages.0.get());
        // log::info!("Children: {:?}", guest_selection.get().children.get());
    });

    view! {
        <div class="p-4">
            <div
                id="guestsDropdownContent"
                class="fixed inset-0 bg-white z-[100] md:absolute md:right-0 md:bg-white md:shadow-lg md:mt-10 md:borderSortOptions md:border-gray-300 md:rounded-xl md:border md:px-4"
            >
                // <div class="flex items-center p-4 border-b border-gray-200 md:hidden">
                //     <button
                //         class="text-gray-800 mr-4"
                //         on:click=move |_| InputGroupState::toggle_dialog(OpenDialogComponent::None)
                //     >
                //         <Icon icon=icondata::BsArrowLeft class="text-xl" />
                //     </button>
                //     <h2 class="text-lg font-medium">Select Guests</h2>
                // </div>

                <div class="p-4 md:p-0">
                    <NumberCounter
                        label="Adults"
                        counter=guest_selection.get().adults
                        class="mt-2"
                        on_increment=move || {
                            guest_selection.get().adults.update(|n| *n += 1);
                        }
                    />
                    <Divider />

                    <NumberCounter
                        label="Children"
                        counter=guest_selection.get().children
                        class="mt-2"
                        on_increment=move || {
                            guest_selection.get().children.update(|n| *n += 1);
                        }
                    />
                    <div class="flex flex-wrap">
                        // Add number input fields for children ages
                        {move || {
                            (0..guest_selection.get().children.get())
                                .map(|i| {
                                    view! {
                                        <input
                                            type="number"
                                            min=1
                                            max=18
                                            class="mt-2 ml-3 p-2 border border-gray-300 w-16"
                                            name=format!("child_age[{}]", i)
                                            value=move || {
                                                guest_selection.get().children_ages.get_value_at(i)
                                            }
                                            placeholder="Age"
                                            on:input=move |e| {
                                                let age = event_target_value(&e);
                                                log!("{}",age);
                                                guest_selection
                                                    .get()
                                                    .children_ages
                                                    .update_children_ages(i as u32, age.parse().unwrap_or(10));
                                            }
                                        />
                                    }
                                })
                                .collect::<Vec<_>>()
                                .into_view()
                        }}
                    </div>
                    <Divider />

                    <NumberCounter
                        label="Rooms"
                        counter=guest_selection.get().rooms
                        class="mt-2"
                        on_increment=move || {
                            guest_selection.get().rooms.update(|n| *n += 1);
                        }
                    />
                    // <!-- Modified button to be more prominent on mobile -->
                    <button
                        type="button"
                        class="w-full mt-6 mb-4 bg-blue-500 md:bg-white text-white md:text-black md:border md:border-black-2 py-3 md:py-2 rounded-full"
                        on:click=apply_selection
                    >
                        "Apply"
                    </button>
                </div>
            </div>
        </div>
    }
}
