use crate::api::consts::SEARCH_COMPONENT_ROOMS_DEFAULT;
// todo - change this to view state layer
// use crate::api::provab::RoomGuest;
use crate::component::NumberCounterV2;
use crate::utils::pluralize;
use crate::view_state_layer::input_group_state::{InputGroupState, OpenDialogComponent};
use crate::view_state_layer::GlobalStateForLeptos;
use crate::{
    component::{Divider, HSettingIcon},
    view_state_layer::ui_search_state::UISearchCtx,
};
use ev::{InputEvent, MouseEvent};
// use leptos::logging::log;
use crate::log;
use leptos::*;
use leptos_icons::*;
use std::ops::Index;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ChildrenAges(Vec<u32>);

impl ChildrenAges {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn get_untracked(&self) -> Vec<u32> {
        self.0.clone()
    }

    pub fn get_value_at(&self, index: u32) -> u32 {
        self.0.get(index as usize).copied().unwrap_or(5)
    }

    pub fn update_children_ages(&mut self, index: u32, age: u32) {
        if let Some(existing_age) = self.0.get_mut(index as usize) {
            *existing_age = age;
        }
    }

    pub fn push_children_ages(&mut self) {
        self.0.push(10);
    }

    pub fn pop_children_ages(&mut self) {
        self.0.pop();
    }

    pub fn set_ages(&mut self, ages: Vec<u32>) {
        self.0 = ages;
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl IntoIterator for ChildrenAges {
    type Item = u32;
    type IntoIter = std::vec::IntoIter<u32>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl From<Vec<u32>> for ChildrenAges {
    fn from(vec: Vec<u32>) -> Self {
        Self(vec)
    }
}

impl From<ChildrenAges> for Vec<u32> {
    fn from(ages: ChildrenAges) -> Self {
        ages.0
    }
}

#[derive(Debug, Clone)]
pub struct GuestSelection {
    pub adults: RwSignal<u32>,
    pub children: RwSignal<u32>,
    pub rooms: RwSignal<u32>,
    pub children_ages: RwSignal<ChildrenAges>,
}

impl Default for GuestSelection {
    fn default() -> Self {
        Self {
            adults: RwSignal::new(2),
            children: RwSignal::new(0),
            rooms: RwSignal::new(SEARCH_COMPONENT_ROOMS_DEFAULT), // Set default value for rooms to 1
            children_ages: RwSignal::new(ChildrenAges::new()),
        }
    }
}

impl GlobalStateForLeptos for GuestSelection {}

impl GuestSelection {
    // pub fn get_room_guests(search_ctx: &UISearchCtx) -> Vec<RoomGuest> {
    //     let guest_selection = search_ctx.guests;

    //     let no_of_adults = guest_selection.get_untracked().adults.get_untracked();
    //     let no_of_child = guest_selection.get_untracked().children.get_untracked();
    //     let children_ages: Vec<String> = guest_selection
    //         .get_untracked()
    //         .children_ages
    //         .get_untracked()
    //         .iter()
    //         .map(|age| age.to_string())
    //         .collect();

    //     let child_age = if no_of_child > 0 {
    //         Some(children_ages)
    //     } else {
    //         None
    //     };

    //     vec![RoomGuest {
    //         no_of_adults,
    //         no_of_child,
    //         child_age,
    //     }]
    // }

    pub fn increment_children() {
        let this = Self::get();
        this.children.update(|n| *n += 1);
        this.children_ages.update(|ages| ages.push_children_ages());
    }

    pub fn decrement_children() {
        let this = Self::get();
        this.children.update(|n| *n = n.saturating_sub(1));
        this.children_ages.update(|ages| ages.pop_children_ages());
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
        this.children_ages.get_untracked().get_untracked()
    }
    pub fn get_children_age_at(i: u32) -> u32 {
        let this = Self::get();
        this.children_ages.get_untracked().get_value_at(i)
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

// Extension trait to add missing methods to RwSignal<ChildrenAges>
pub trait ChildrenAgesSignalExt {
    fn get_value_at(&self, index: u32) -> u32;
    fn set_ages(&self, ages: Vec<u32>);
}

impl ChildrenAgesSignalExt for RwSignal<ChildrenAges> {
    fn get_value_at(&self, index: u32) -> u32 {
        self.get_untracked().get_value_at(index)
    }

    fn set_ages(&self, ages: Vec<u32>) {
        self.update(|children_ages| {
            children_ages.set_ages(ages);
        });
    }
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

    let search_ctx: UISearchCtx = expect_context();
    let guest_selection = search_ctx.guests;

    // Get direct references to the signals
    let adults_signal = guest_selection.adults;
    let children_signal = guest_selection.children;
    let rooms_signal = guest_selection.rooms;
    let children_ages_signal = guest_selection.children_ages;

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

                                    <div class="flex justify-between items-center p-2 rounded-lg hover:bg-gray-50 transition-colors">
                                        <p>"Children"</p>
                                        <div class="flex items-center space-x-1">
                                            {
                                                // Create reactive validation signals inspired by RoomCounterV1
                                                let is_at_minimum = create_memo(move |_| children_signal.get() == 0);
                                                let is_at_maximum = create_memo(move |_| children_signal.get() >= 10); // Max 10 children

                                                // Button event handlers inspired by both RoomCounterV1 and NumberCounterV2
                                                let increment_children = move |_| {
                                                    if children_signal.get() < 10 { // Guard against maximum
                                                        GuestSelection::increment_children();
                                                    }
                                                };
                                                let decrement_children = move |_| {
                                                    if children_signal.get() > 0 { // Guard against minimum
                                                        GuestSelection::decrement_children();
                                                    }
                                                };

                                                view! {
                                                    <button
                                                        class=move || format!(
                                                            "ps-2 py-1 text-2xl {}",
                                                            if is_at_minimum() { "opacity-50 cursor-not-allowed" } else { "" }
                                                        )
                                                        disabled=is_at_minimum
                                                        on:click=decrement_children
                                                    >
                                                        {"\u{2003}\u{2003}\u{2003}\u{2003}-"}
                                                    </button>
                                                    <p class="text-center w-6">{move || children_signal.get()}</p>
                                                    <button
                                                        class=move || format!(
                                                            "py-1 text-2xl {}",
                                                            if is_at_maximum() { "opacity-50 cursor-not-allowed" } else { "" }
                                                        )
                                                        disabled=is_at_maximum
                                                        on:click=increment_children
                                                    >
                                                        "+"
                                                    </button>
                                                }
                                            }
                                        </div>
                                    </div>

                                    // !<-- Children Ages Grid - Responsive grid layout -->
                                    <div class="grid grid-cols-4 md:grid-cols-5 gap-2">
                                        {move || {
                                            (0..children_signal.get())
                                                .map(|i| {
                                                    let children_ages_signal = children_ages_signal.clone();
                                                    view! {
                                                        <input
                                                            type="number"
                                                            min=1
                                                            max=18
                                                            class="p-2 border border-gray-300 w-full rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                                            name=format!("child_age[{}]", i)
                                                            prop:value={
                                                                let children_ages_signal = children_ages_signal.clone();
                                                                move || {
                                                                    children_ages_signal.get().get_value_at(i as u32).to_string()
                                                                }
                                                            }
                                                            placeholder="Age"
                                                            on:input={
                                                                let children_ages_signal = children_ages_signal.clone();
                                                                move |e| {
                                                                    let age = event_target_value(&e);
                                                                    log!("Setting child {} age to: {}", i, age);
                                                                    children_ages_signal.update(|ages| {
                                                                        ages.update_children_ages(i as u32, age.parse().unwrap_or(10));
                                                                    });
                                                                }
                                                            }
                                                        />
                                                    }
                                                })
                                                .collect_view()
                                        }}
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

                                                UISearchCtx::log_state();
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
