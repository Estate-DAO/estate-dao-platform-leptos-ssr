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
        <div class="relative">
            <div class="absolute inset-y-0 left-2 flex items-center text-2xl">
                <Icon icon=icondata::BsPerson class="text-black font-light" />
            </div>

            <button
                id="guestsDropdown"
                class="w-full flex-0 py-2 pl-10 text-left text-gray-700 text-sm font-light bg-transparent rounded-full focus:outline-none"
                on:click=move |_| InputGroupState::toggle_dialog(OpenDialogComponent::GuestComponent)
            >
                {{ move || guest_count_display }}
            </button>

            <div class="absolute inset-y-2 text-xl right-3 flex items-center">
                <Icon icon=icon class="text-black" />
            </div>

            <Show when=move || is_open()>
                // !<-- Main Modal Container -->
                <div class="fixed inset-0 z-[9999]">
                    // !<-- Backdrop - Transparent overlay -->
                    <div class="absolute inset-0 bg-black/20 backdrop-blur-sm"></div>

                    // !<-- Content Container - Centered on desktop, bottom sheet on mobile -->
                    <div class="fixed bottom-0 left-0 right-0 top-auto md:top-1/2 md:left-1/2 md:-translate-x-1/2 md:-translate-y-1/2 md:max-w-[400px] md:w-[400px] z-[9999]">
                        <div class="relative md:border-1 md:border-gray-400 md:rounded-2xl md:shadow-xl md:z-[9999]">
                            <div class="bg-white px-4 py-6 md:rounded-2xl">
                                // !<-- Guest Selection Section -->
                                <div class="space-y-6">
                                    <NumberCounter
                                        label="Adults"
                                        counter=guest_selection.get().adults
                                        class="flex justify-between items-center p-2 rounded-lg hover:bg-gray-50 transition-colors"
                                        on_increment=move || {
                                            guest_selection.get().adults.update(|n| *n += 1);
                                        }
                                    />

                                    <NumberCounter
                                        label="Children"
                                        counter=guest_selection.get().children
                                        class="flex justify-between items-center p-2 rounded-lg hover:bg-gray-50 transition-colors"
                                        on_increment=move || {
                                            guest_selection.get().children.update(|n| *n += 1);
                                        }
                                    />

                                    // !<-- Children Ages Grid - Responsive grid layout -->
                                    <div class="grid grid-cols-4 md:grid-cols-5 gap-2">
                                        {move || {
                                            (0..guest_selection.get().children.get())
                                                .map(|i| {
                                                    view! {
                                                        <input
                                                            type="number"
                                                            min=1
                                                            max=18
                                                            class="p-2 border border-gray-300 w-full rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
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

                                    <NumberCounter
                                        label="Rooms"
                                        counter=guest_selection.get().rooms
                                        class="flex justify-between items-center p-2 rounded-lg hover:bg-gray-50 transition-colors"
                                        on_increment=move || {
                                            guest_selection.get().rooms.update(|n| *n += 1);
                                        }
                                    />
                                </div>

                                // !<-- Apply Button Section - Responsive styling -->
                                <div class="bg-white px-2 py-2 mt-6">
                                    <div class="flex justify-center">
                                        <button
                                            type="button"
                                            class="w-full text-sm md:w-48 bg-blue-500 text-white md:hover:bg-blue-600 py-3 md:py-2.5 rounded-full transition-colors shadow-sm hover:shadow-md"
                                            on:click=move |ev| {
                                                ev.prevent_default();

                                                log::info!(
                                                    "Adults: {}, Children: {}, Ages: {:?}",
                                                    guest_selection.get().adults.get_untracked(),
                                                    guest_selection.get().children.get_untracked(),
                                                    guest_selection.get().children_ages.get_untracked()
                                                );

                                                SearchCtx::log_state();
                                                InputGroupState::toggle_dialog(OpenDialogComponent::GuestComponent)
                                            }
                                        >
                                            "Apply"
                                        </button>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
