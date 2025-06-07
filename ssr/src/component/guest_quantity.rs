use crate::api::consts::SEARCH_COMPONENT_ROOMS_DEFAULT;
use crate::api::RoomGuest;
use crate::component::NumberCounterV2;
use crate::state::input_group_state::{InputGroupState, OpenDialogComponent};
use crate::state::GlobalStateForLeptos;
use crate::utils::pluralize;
use crate::{
    component::{Divider, HSettingIcon},
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

impl GlobalStateForLeptos for GuestSelection {}

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

    pub fn increment_children() {
        let this = Self::get();
        this.children.update(|n| *n += 1);
    }

    pub fn increment_rooms() {
        let this = Self::get();
        this.rooms.update(|n| *n += 1);
    }

    pub fn get_adults() -> u32 {
        let this = Self::get();
        this.adults.get_untracked()
    }

    pub fn get_children() -> u32 {
        let this = Self::get();
        this.children.get_untracked()
    }
    pub fn get_children_reactive() -> u32 {
        let this = Self::get();
        this.children.get()
    }

    pub fn get_children_ages() -> Vec<u32> {
        let this = Self::get();
        this.children_ages.get_untracked()
    }
    pub fn get_children_age_at(i: u32) -> u32 {
        let this = Self::get();
        this.children_ages.get_value_at(i)
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
    let guest_selection = search_ctx.guests.get_untracked();

    // Get direct references to the signals
    let adults_signal = guest_selection.adults;
    let children_signal = guest_selection.children;
    let rooms_signal = guest_selection.rooms;
    let children_ages = guest_selection.children_ages;

    let guest_count_display = create_memo(move |_prev| {
        format!(
            "{} • {}",
            pluralize(adults_signal.get(), "adult", "adults"),
            pluralize(children_signal.get(), "child", "children")
        )
    });

    view! {
        <div class="relative w-full h-full">
            <div class="absolute inset-y-0 left-1 flex items-center text-2xl">
                <Icon icon=icondata::BsPerson class="text-black font-light" />
            </div>

            <div class="flex items-center h-full">
            <button
                id="guestsDropdown"
                class="w-full flex-0 py-2 pl-10 text-left text-gray-700 text-sm font-light bg-transparent rounded-full focus:outline-none"
                on:click=move |_| InputGroupState::toggle_dialog(OpenDialogComponent::GuestComponent)
            >
                {{ move || guest_count_display }}
            </button>

            </div>
            <div class="absolute inset-y-2 text-xl right-3 flex items-center">
                <Icon icon=icon class="text-black" />
            </div>

            <Show when=move || is_open()>
                // !<-- Main Modal Container -->
                <div
                    class="fixed inset-0 z-[9999]"
                    on:click=move |_| InputGroupState::toggle_dialog(OpenDialogComponent::GuestComponent)
                >
                    // !<-- Content Container - Centered on desktop, bottom sheet on mobile -->
                    <div
                        class="fixed bottom-0 left-0 right-0 top-auto md:absolute md:top-full md:right-0 md:left-auto md:bottom-auto md:max-w-[400px] md:w-[400px] z-[9999]"
                        on:click=|e| e.stop_propagation()
                    >
                        <div class="relative md:border-1 md:border-gray-400 md:rounded-2xl md:shadow-xl md:z-[9999]">
                            <div class="bg-white px-4 py-6 md:rounded-2xl">
                                // !<-- Guest Selection Section -->
                                <div class="space-y-6">
                                    <NumberCounterV2
                                        label="Adults"
                                        counter=adults_signal
                                        class="flex justify-between items-center p-2 rounded-lg hover:bg-gray-50 transition-colors"
                                        on_increment=move || {
                                            adults_signal.update(|n| *n += 1);
                                        }
                                        min=1_u32
                                    />

                                    <NumberCounterV2
                                        label="Children"
                                        counter=children_signal
                                        class="flex justify-between items-center p-2 rounded-lg hover:bg-gray-50 transition-colors"
                                        on_increment={
                                            let children_ages = children_ages.clone();
                                            move || {
                                                children_signal.update(|n| *n += 1);
                                                children_ages.push_children_ages();
                                            }
                                        }
                                        on_decrement=Box::new({
                                            let children_ages = children_ages.clone();
                                            move || {
                                                children_signal.update(|n| *n = n.saturating_sub(1));
                                                children_ages.pop_children_ages();
                                            }
                                        })
                                        min=0_u32
                                    />

                                    // !<-- Children Ages Grid - Responsive grid layout -->
                                    <div class="grid grid-cols-4 md:grid-cols-5 gap-2">
                                        {
                                            let children_ages = children_ages.clone();
                                            move || {
                                                (0..children_signal.get())
                                                    .map(|i| {
                                                        let children_ages = children_ages.clone();
                                                        view! {
                                                            <input
                                                                type="number"
                                                                min=1
                                                                max=18
                                                                class="p-2 border border-gray-300 w-full rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                                                name=format!("child_age[{}]", i)
                                                                value={
                                                                    let children_ages = children_ages.clone();
                                                                    move || {
                                                                        children_ages.get_value_at(i)
                                                                    }
                                                                }
                                                                placeholder="Age"
                                                                on:input={
                                                                    let children_ages = children_ages.clone();
                                                                    move |e| {
                                                                        let age = event_target_value(&e);
                                                                        log!("{}",age);
                                                                        children_ages.update_children_ages(i as u32, age.parse().unwrap_or(10));
                                                                    }
                                                                }
                                                            />
                                                        }
                                                    })
                                                    .collect::<Vec<_>>()
                                                    .into_view()
                                            }
                                        }
                                    </div>

                                    <NumberCounterV2
                                        label="Rooms"
                                        counter=rooms_signal
                                        class="flex justify-between items-center p-2 rounded-lg hover:bg-gray-50 transition-colors"
                                        on_increment=move || {
                                            rooms_signal.update(|n| *n += 1);
                                        }
                                        min=1_u32
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
                                                    GuestSelection::get_adults(),
                                                    GuestSelection::get_children(),
                                                    GuestSelection::get_children_ages()
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
