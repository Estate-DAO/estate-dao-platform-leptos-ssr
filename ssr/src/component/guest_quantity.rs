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
pub struct ChildrenAges(RwSignal<Vec<u32>>);

impl ChildrenAges {
    pub fn new() -> Self {
        Self(RwSignal::new(Vec::new()))
    }

    pub fn get_untracked(&self) -> Vec<u32> {
        self.0.get_untracked()
    }

    pub fn get_value_at(&self, index: u32) -> u32 {
        self.0
            .get_untracked()
            .get(index as usize)
            .copied()
            .unwrap_or(5)
    }

    pub fn update_children_ages(&mut self, index: u32, age: u32) {
        log!(
            "[children_signal] Updating child age at index {} to {}",
            index,
            age
        );
        self.0.update(|vec| {
            if let Some(existing_age) = vec.get_mut(index as usize) {
                *existing_age = age;
            }
        });
    }

    pub fn push_children_ages(&mut self) {
        log!("[children_signal] Pushing new child age (10) to vector");
        self.0.update(|vec| vec.push(10));
    }

    pub fn pop_children_ages(&mut self) {
        log!("[children_signal] Popping child age from vector");
        self.0.update(|vec| {
            vec.pop();
        });
    }

    pub fn set_ages(&mut self, ages: Vec<u32>) {
        log!("[children_signal] Setting children ages to: {:?}", ages);
        self.0.set(ages);
    }

    pub fn len(&self) -> usize {
        self.0.get_untracked().len()
    }

    pub fn set_vec(&self, ages: Vec<u32>) {
        log!(
            "[children_signal] Setting children ages vector to: {:?}",
            ages
        );
        self.0.set(ages);
    }

    pub fn get_signal(&self) -> RwSignal<Vec<u32>> {
        self.0
    }
}

impl IntoIterator for ChildrenAges {
    type Item = u32;
    type IntoIter = std::vec::IntoIter<u32>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.get_untracked().into_iter()
    }
}

impl From<Vec<u32>> for ChildrenAges {
    fn from(vec: Vec<u32>) -> Self {
        Self(RwSignal::new(vec))
    }
}

impl From<ChildrenAges> for Vec<u32> {
    fn from(ages: ChildrenAges) -> Self {
        ages.0.get_untracked()
    }
}

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
            children_ages: ChildrenAges::new(),
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

    pub fn increment_adults() {
        let search_ctx: UISearchCtx = expect_context();
        let this = search_ctx.guests;
        this.adults.update(|n| *n += 1);
        log!(
            "[adults_signal] Incrementing adults count to: {}",
            this.adults.get_untracked()
        );
        log!("[adults_signal] Incrementing adults count");
    }
    pub fn decrement_adults() {
        let search_ctx: UISearchCtx = expect_context();
        let this = search_ctx.guests;
        this.adults.update(|n| *n = n.saturating_sub(1));
        log!(
            "[adults_signal] Decrementing adults count to: {}",
            this.adults.get_untracked()
        );
        log!("[adults_signal] Decrementing adults count");
    }

    pub fn increment_rooms() {
        let search_ctx: UISearchCtx = expect_context();
        let this = search_ctx.guests;
        this.rooms.update(|n| *n += 1);
        log!(
            "[rooms_signal] Incrementing rooms count to: {}",
            this.rooms.get_untracked()
        );
        log!("[rooms_signal] Incrementing rooms count");
    }
    pub fn decrement_rooms() {
        let search_ctx: UISearchCtx = expect_context();
        let this = search_ctx.guests;
        this.rooms.update(|n| *n = n.saturating_sub(1));
        log!(
            "[rooms_signal] Decrementing rooms count to: {}",
            this.rooms.get_untracked()
        );
        log!("[rooms_signal] Decrementing rooms count");
    }

    pub fn increment_children() {
        let search_ctx: UISearchCtx = expect_context();
        let this = search_ctx.guests;
        this.children.update(|n| *n += 1);
        log!(
            "[children_signal] Incrementing children count to: {}",
            this.children.get_untracked()
        );
        log!("[children_signal] Incrementing children count");
        this.children_ages.get_signal().update(|vec| vec.push(10));
    }

    pub fn decrement_children() {
        let search_ctx: UISearchCtx = expect_context();
        let this = search_ctx.guests;
        this.children.update(|n| *n = n.saturating_sub(1));
        log!(
            "[children_signal] Decrementing children count to: {}",
            this.children.get_untracked()
        );
        log!("[children_signal] Decrementing children count");
        this.children_ages.get_signal().update(|vec| {
            vec.pop();
        });
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

    //  fn reactive_length(&self) {
    //     let no_of_child = self.children.get_untracked();
    //     if no_of_child > 0 {
    //         self.children_ages.push_children_ages();
    //     } else {
    //         self.children_ages.pop_children_ages();
    //     }
    // }
}

// Extension trait to add missing methods to ChildrenAges
pub trait ChildrenAgesSignalExt {
    fn get_value_at(&self, index: u32) -> u32;
    fn set_ages(&self, ages: Vec<u32>);
}

impl ChildrenAgesSignalExt for ChildrenAges {
    fn get_value_at(&self, index: u32) -> u32 {
        self.get_value_at(index)
    }

    fn set_ages(&self, ages: Vec<u32>) {
        self.0.set(ages);
    }
}

/// Guest quantity component (button)
#[component]
pub fn GuestQuantity(#[prop(optional, into)] h_class: MaybeSignal<String>) -> impl IntoView {
    let h_class = create_memo(move |_| {
        let class = h_class.get();
        if class.is_empty() {
            "h-full".to_string()
        } else {
            class
        }
    });
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
    let children_ages_signal = guest_selection.children_ages.get_signal();

    // **Robust Counter Logic - Inspired by RoomCounterV1**
    // Create local closures with boundary checks and safety validations

    // Adults counter logic (minimum 1) - directly use signals to avoid move issues

    // Children counter logic (minimum 0) - directly use signals to avoid move issues

    // Rooms counter logic (minimum 1) - directly use signals to avoid move issues

    // log!(
    //     "[children_signal] Initial children_signal value: {}",
    //     children_signal.get_untracked()
    // );
    // log!(
    //     "[children_signal] Initial children_ages_signal value: {:?}",
    //     children_ages_signal.get_untracked()
    // );

    let guest_count_display = create_memo(move |_prev| {
        let adults = adults_signal.get();
        let children = children_signal.get();
        // log!(
        //     "[children_signal] Updating guest_count_display: Adults = {}, Children = {}",
        //     adults,
        //     children
        // );
        format!(
            "{} • {}",
            pluralize(adults, "adult", "adults"),
            pluralize(children, "child", "children")
        )
    });

    view! {
        <div class="relative flex items-center w-full">
            <div class="absolute inset-y-0 left-1 flex items-center text-2xl">
                <Icon icon=icondata::BiUserRegular class="text-blue-500 font-extralight"/>
            </div>

            <button
                class=move || {
                    format!(
                        "w-full {} py-2 pl-10 text-black bg-transparent border-none focus:outline-none text-sm text-left flex items-center justify-around",
                        h_class(),
                    )
                }
                on:click=move |_| InputGroupState::toggle_dialog(OpenDialogComponent::GuestComponent)
            >
                <div class="text-black font-medium truncate">{guest_count_display}</div>
                <div>
                    <Icon icon=icon() class="text-gray-600 text-sm"/>
                </div>
            </button>

            <Show when=move || is_open()>
                // !<-- Mobile Overlay -->
                <div
                    class="md:hidden fixed inset-0 z-[9999] bg-black/50"
                    on:click=move |_| InputGroupState::toggle_dialog(OpenDialogComponent::GuestComponent)
                ></div>

                // !<-- Desktop: Positioned dropdown aligned to right edge, extending beyond section -->
                <div
                    class="absolute top-full right-0 mt-2 w-80 bg-white border border-gray-200 rounded-lg shadow-lg z-[60] hidden md:block"
                    style="transform: translateX(calc(100% - 320px));"
                    on:click=|e| e.stop_propagation()
                >
                    <div class="p-6 space-y-6">
                        // !<-- Guest Selection Section -->
                        <div class="space-y-4">
                            <h3 class="text-lg font-medium text-left">"Guests and Rooms"</h3>

                            // !<-- Adults Counter -->
                            <NumberCounterV2
                                label="Adults"
                                class="py-2"
                                counter=adults_signal
                                on_increment=GuestSelection::increment_adults
                                on_decrement=GuestSelection::decrement_adults
                                min=1u32
                            />

                            <Divider/>

                            // !<-- Children Counter -->
                            <NumberCounterV2
                                label="Children"
                                class="py-2"
                                counter=children_signal
                                on_increment=GuestSelection::increment_children
                                on_decrement=GuestSelection::decrement_children
                            />

                            // !<-- Children Ages Grid -->
                            <Show when=move || { children_signal.get() > 0 }>
                                <div class="pl-4 space-y-3">
                                    <p class="text-sm text-gray-600 font-medium">
                                        "Ages of children"
                                    </p>
                                    <div class="grid grid-cols-2 md:grid-cols-3 gap-3">
                                        {move || {
                                            let children_count = children_signal.get();
                                            (0..children_count)
                                                .map(| i | {
                                                    // Create a reactive age value that updates with the signal
                                                    let age_value = move || {
                                                        children_ages_signal
                                                            .get()
                                                            .get(i as usize)
                                                            .cloned()
                                                            .unwrap_or(10)
                                                    };
                                                    view! {
                                                        <div class="flex flex-col items-center space-y-2">
                                                            <label class="text-xs text-gray-500 font-medium">
                                                                {format!("Child {}", i + 1)}
                                                            </label>
                                                            <select
                                                                class="w-16 h-10 text-center border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent bg-white text-sm"
                                                                on:change=move | ev | {
                                                                    let value = event_target_value(&ev)
                                                                        .parse()
                                                                        .unwrap_or(10);
                                                                    // Update the children_ages state directly
                                                                    children_ages_signal
                                                                        .update(| ages | {
                                                                            if let Some(age) = ages
                                                                                .get_mut(i as usize)
                                                                            {
                                                                                *age = value;
                                                                            }
                                                                        });
                                                                }
                                                                prop:value=move || {
                                                                    age_value().to_string()
                                                                }
                                                            >
                                                                {(0..=17)
                                                                    .map(| age | {
                                                                        view! {
                                                                            <option value=age
                                                                                .to_string()>
                                                                                {age.to_string()}
                                                                            </option>
                                                                        }
                                                                    })
                                                                    .collect::<Vec<_>>()}
                                                            </select>
                                                        </div>
                                                    }
                                                })
                                                .collect::<Vec<_>>()
                                        }}
                                    </div>
                                    <p class="text-xs text-gray-500">"Ages 0-17"</p>
                                </div>
                            </Show>

                            <Divider/>

                            // !<-- Rooms Counter -->
                            <NumberCounterV2
                                label="Rooms"
                                class="py-2"
                                counter=rooms_signal
                                on_increment=GuestSelection::increment_rooms
                                on_decrement=GuestSelection::decrement_rooms
                                min=1u32
                            />
                        </div>

                        // !<-- Apply Button Section -->
                        <div class="flex justify-center pt-4">
                            <button
                                type="button"
                                class="w-48 bg-blue-500 text-white py-2 rounded-full hover:bg-blue-600 transition-colors text-sm font-medium"
                                on:click=move |_| {
                                    InputGroupState::toggle_dialog(OpenDialogComponent::None)
                                }
                            >
                                "Apply"
                            </button>
                        </div>
                    </div>
                </div>

                // !<-- Mobile: Bottom sheet -->
                <div
                    class="md:hidden fixed bottom-0 left-0 right-0 z-[9999] bg-white rounded-t-lg"
                    on:click=|e| e.stop_propagation()
                >
                    <div class="p-6 space-y-6">
                        // !<-- Guest Selection Section -->
                        <div class="space-y-4">
                            <h3 class="text-lg font-medium text-center">"Guests and Rooms"</h3>

                            // !<-- Adults Counter -->
                            <NumberCounterV2
                                label="Adults"
                                class="py-2"
                                counter=adults_signal
                                on_increment=GuestSelection::increment_adults
                                on_decrement=GuestSelection::decrement_adults
                                min=1u32
                            />

                            <Divider/>

                            // !<-- Children Counter -->
                            <NumberCounterV2
                                label="Children"
                                class="py-2"
                                counter=children_signal
                                on_increment=GuestSelection::increment_children
                                on_decrement=GuestSelection::decrement_children
                            />

                            // !<-- Children Ages Grid -->
                            <Show when=move || { children_signal.get() > 0 }>
                                <div class="pl-4 space-y-3">
                                    <p class="text-sm text-gray-600 font-medium">
                                        "Ages of children"
                                    </p>
                                    <div class="grid grid-cols-2 gap-3">
                                        {move || {
                                            let children_count = children_signal.get();
                                            (0..children_count)
                                                .map(| i | {
                                                    // Create a reactive age value that updates with the signal
                                                    let age_value = move || {
                                                        children_ages_signal
                                                            .get()
                                                            .get(i as usize)
                                                            .cloned()
                                                            .unwrap_or(10)
                                                    };
                                                    view! {
                                                        <div class="flex flex-col items-center space-y-2">
                                                            <label class="text-xs text-gray-500 font-medium">
                                                                {format!("Child {}", i + 1)}
                                                            </label>
                                                            <select
                                                                class="w-16 h-10 text-center border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent bg-white text-sm"
                                                                on:change=move | ev | {
                                                                    let value = event_target_value(&ev)
                                                                        .parse()
                                                                        .unwrap_or(10);
                                                                    // Update the children_ages state directly
                                                                    children_ages_signal
                                                                        .update(| ages | {
                                                                            if let Some(age) = ages
                                                                                .get_mut(i as usize)
                                                                            {
                                                                                *age = value;
                                                                            }
                                                                        });
                                                                }
                                                                prop:value=move || {
                                                                    age_value().to_string()
                                                                }
                                                            >
                                                                {(0..=17)
                                                                    .map(| age | {
                                                                        view! {
                                                                            <option value=age
                                                                                .to_string()>
                                                                                {age.to_string()}
                                                                            </option>
                                                                        }
                                                                    })
                                                                    .collect::<Vec<_>>()}
                                                            </select>
                                                        </div>
                                                    }
                                                })
                                                .collect::<Vec<_>>()
                                        }}
                                    </div>
                                    <p class="text-xs text-gray-500">"Ages 0-17"</p>
                                </div>
                            </Show>

                            <Divider/>

                            // !<-- Rooms Counter -->
                            <NumberCounterV2
                                label="Rooms"
                                class="py-2"
                                counter=rooms_signal
                                on_increment=GuestSelection::increment_rooms
                                on_decrement=GuestSelection::decrement_rooms
                                min=1u32
                            />
                        </div>

                        // !<-- Apply Button Section -->
                        <div class="flex justify-center pt-4">
                            <button
                                type="button"
                                class="w-full bg-blue-500 text-white py-3 rounded-full hover:bg-blue-600 transition-colors text-sm font-medium"
                                on:click=move |_| {
                                    InputGroupState::toggle_dialog(OpenDialogComponent::None)
                                }
                            >
                                "Apply"
                            </button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
