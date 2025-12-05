use crate::{
    domain::{DomainHotelDetails, DomainHotelStaticDetails, DomainRoomData, DomainRoomOption},
    error, log, warn,
};
use leptos::*;
use std::collections::{HashMap, HashSet};

use super::GlobalStateForLeptos;

#[derive(Debug, Clone, Default)]
pub struct HotelDetailsUIState {
    pub hotel_details: RwSignal<Option<DomainHotelDetails>>,
    pub static_details: RwSignal<Option<DomainHotelStaticDetails>>,
    pub rates: RwSignal<Option<Vec<DomainRoomOption>>>,
    pub loading: RwSignal<bool>,
    pub rates_loading: RwSignal<bool>,
    pub error: RwSignal<Option<String>>,
    pub selected_rooms: RwSignal<HashMap<String, u32>>, // rate_key -> quantity
    // this is the aggregate price of all the rooms selected
    pub total_price: RwSignal<f64>,
}

impl HotelDetailsUIState {
    pub fn from_leptos_context() -> Self {
        expect_context()
    }

    pub fn set_hotel_details(details: Option<DomainHotelDetails>) {
        let this: Self = expect_context();
        this.hotel_details.set(details);
    }

    pub fn set_static_details(details: Option<DomainHotelStaticDetails>) {
        let this: Self = expect_context();
        this.static_details.set(details);
    }

    pub fn set_rates(rates: Option<Vec<DomainRoomOption>>) {
        let this: Self = expect_context();
        let previous = this.rates.get_untracked();
        let cloned = rates.clone();
        this.rates.set(rates);
        Self::sync_selected_rooms_with_rates(
            cloned.as_ref().map(|r| r.as_slice()),
            previous.as_ref().map(|r| r.as_slice()),
        );
    }

    pub fn set_loading(loading: bool) {
        let this: Self = expect_context();
        this.loading.set(loading);
    }

    pub fn set_rates_loading(loading: bool) {
        let this: Self = expect_context();
        this.rates_loading.set(loading);
    }

    pub fn set_error(error: Option<String>) {
        let this: Self = expect_context();
        this.error.set(error);
    }

    pub fn reset() {
        let this: Self = expect_context();
        this.hotel_details.set(None);
        this.static_details.set(None);
        this.rates.set(None);
        this.loading.set(false);
        this.rates_loading.set(false);
        this.error.set(None);
        this.selected_rooms.set(HashMap::new());
        this.total_price.set(0.0);
    }

    pub fn get_hotel_details() -> Option<DomainHotelStaticDetails> {
        let this: Self = expect_context();
        this.static_details.get()
    }

    pub fn is_loading() -> bool {
        let this: Self = expect_context();
        this.loading.get()
    }

    pub fn get_error() -> Option<String> {
        let this: Self = expect_context();
        this.error.get()
    }

    // <!-- Room selection methods using consolidated DomainHotelDetails.all_rooms -->
    pub fn get_available_room_options() -> Vec<DomainRoomOption> {
        let this: Self = expect_context();
        if let Some(rates) = this.rates.get() {
            rates
        } else {
            vec![]
        }
    }

    pub fn get_available_rooms() -> Vec<DomainRoomData> {
        // Legacy method for compatibility - extracts room_data from all_rooms
        Self::get_available_room_options()
            .into_iter()
            .map(|room_option| room_option.room_data)
            .collect()
    }

    pub fn increment_room_counter(room_type: String) {
        let this: Self = expect_context();

        // Check if incrementing would exceed the search limit
        if Self::can_increment_room_selection() {
            this.selected_rooms.update(|rooms| {
                let current = rooms.get(&room_type).copied().unwrap_or(0);
                rooms.insert(room_type, current + 1);
            });
            Self::update_total_price();
        }
    }

    pub fn decrement_room_counter(room_type: String) {
        let this: Self = expect_context();
        this.selected_rooms.update(|rooms| {
            let current = rooms.get(&room_type).copied().unwrap_or(0);
            if current > 0 {
                rooms.insert(room_type, current - 1);
            }
        });
        Self::update_total_price();
    }

    pub fn total_room_price() -> f64 {
        let this: Self = expect_context();
        this.total_price.get()
    }

    pub fn total_selected_rooms() -> u32 {
        let this: Self = expect_context();
        this.selected_rooms.get().values().sum()
    }

    pub fn get_selected_room_ids() -> Vec<String> {
        let this: Self = expect_context();
        this.selected_rooms
            .get()
            .iter()
            .filter(|(_, &quantity)| quantity > 0)
            .map(|(room_type, _)| room_type.clone())
            .collect()
    }

    /// Replace the current selection with a single room type and quantity.
    /// Used when single-room-type booking is enforced to keep selections consistent.
    pub fn set_single_room_selection(room_type: String, quantity: u32) {
        let this: Self = expect_context();
        let mut rooms = HashMap::new();
        if quantity > 0 {
            rooms.insert(room_type, quantity);
        }
        this.selected_rooms.set(rooms);
        Self::update_total_price();
    }

    pub fn set_multi_room_selection(selections: Vec<(String, u32)>) {
        if selections.is_empty() {
            return;
        }

        let this: Self = expect_context();
        use crate::view_state_layer::ui_search_state::UISearchCtx;
        let ui_search_ctx: UISearchCtx = expect_context();
        let max_rooms = ui_search_ctx.guests.rooms.get();
        let remaining_rooms = max_rooms.saturating_sub(Self::total_selected_rooms());
        if remaining_rooms == 0 {
            return;
        }

        let mut remaining = remaining_rooms;
        this.selected_rooms.update(move |rooms| {
            for (room_type, quantity) in selections {
                if remaining == 0 {
                    break;
                }

                let key = room_type.clone();
                if quantity == 0 {
                    rooms.remove(&key);
                    continue;
                }

                let add = quantity.min(remaining);
                let entry = rooms.entry(key).or_insert(0);
                *entry = entry.saturating_add(add);
                remaining = remaining.saturating_sub(add);
            }
        });

        Self::update_total_price();
    }

    pub fn get_selected_rooms() -> HashMap<String, u32> {
        let this: Self = expect_context();
        this.selected_rooms.get()
    }

    // <!-- Validation methods for room selection limits -->
    pub fn can_increment_room_selection() -> bool {
        use crate::view_state_layer::ui_search_state::UISearchCtx;

        let ui_search_ctx: UISearchCtx = expect_context();
        let max_rooms = ui_search_ctx.guests.rooms.get();
        let current_total = Self::total_selected_rooms();

        current_total < max_rooms
    }

    pub fn is_at_room_selection_limit() -> bool {
        !Self::can_increment_room_selection()
    }

    // <!-- Helper method to get selected rooms with their data and pricing -->
    pub fn get_selected_rooms_with_data() -> Vec<(DomainRoomOption, u32)> {
        let selected_rooms = Self::get_selected_rooms();
        let available_room_options = Self::get_available_room_options();

        selected_rooms
            .into_iter()
            .filter(|(_, quantity)| *quantity > 0)
            .filter_map(|(room_id, quantity)| {
                available_room_options
                    .iter()
                    .find(|option| option.room_data.rate_key == room_id)
                    .map(|room_option| (room_option.clone(), quantity))
            })
            .collect()
    }

    // <!-- Calculate subtotal for given number of nights -->
    pub fn calculate_subtotal_for_nights() -> f64 {
        let selected_rooms_with_data = Self::get_selected_rooms_with_data();

        selected_rooms_with_data
            .iter()
            .fold(0.0, |acc, (room_option, quantity)| {
                acc + (room_option.price_excluding_included_taxes() * *quantity as f64)
            })
    }

    // <!-- Calculate line total for a specific room -->
    pub fn calculate_room_line_total(
        room_option: &DomainRoomOption,
        quantity: u32,
        nights: u32,
    ) -> f64 {
        room_option.price_excluding_included_taxes() * quantity as f64
    }

    // <!-- Format room breakdown text -->
    pub fn format_room_breakdown_text(room_name: &str, quantity: u32, nights: u32) -> String {
        format!(
            "{} × {} × {} night{}",
            room_name,
            quantity,
            nights,
            if nights != 1 { "s" } else { "" }
        )
    }

    // <!-- Helper method to update total price using helper functions -->
    fn update_total_price() {
        let this: Self = expect_context();

        // Use helper function to calculate total price per night (without nights multiplier)
        let selected_rooms_with_data = Self::get_selected_rooms_with_data();
        let total = selected_rooms_with_data
            .iter()
            .fold(0.0, |acc, (room_option, quantity)| {
                acc + (room_option.price_excluding_included_taxes() * *quantity as f64)
            });

        this.total_price.set(total);
    }

    fn sync_selected_rooms_with_rates(
        rates: Option<&[DomainRoomOption]>,
        previous_rates: Option<&[DomainRoomOption]>,
    ) {
        let this: Self = expect_context();
        match rates {
            Some(rate_list) if !rate_list.is_empty() => {
                let valid_ids: HashSet<_> = rate_list
                    .iter()
                    .map(|option| option.room_data.rate_key.clone())
                    .collect();
                let mut changed = false;
                this.selected_rooms.update(|rooms| {
                    let mut to_remove = Vec::new();
                    let mut to_rekey = Vec::new();

                    for room_id in rooms.keys().cloned().collect::<Vec<_>>() {
                        if valid_ids.contains(&room_id) {
                            continue;
                        }

                        if let Some(prev_rates) = previous_rates {
                            if let Some(prev_option) = prev_rates
                                .iter()
                                .find(|opt| opt.room_data.rate_key == room_id)
                            {
                                let mapped_id = prev_option.room_data.mapped_room_id;
                                if mapped_id != 0 {
                                    if let Some(new_option) =
                                        rate_list.iter().find(|opt| opt.mapped_room_id == mapped_id)
                                    {
                                        let new_id = new_option.room_data.rate_key.clone();
                                        to_rekey.push((room_id.clone(), new_id));
                                        continue;
                                    }
                                }
                            }
                        }

                        to_remove.push(room_id);
                    }

                    for room_id in to_remove {
                        rooms.remove(&room_id);
                        changed = true;
                    }

                    for (old_id, new_id) in to_rekey {
                        if let Some(qty) = rooms.remove(&old_id) {
                            let entry = rooms.entry(new_id).or_insert(0);
                            *entry += qty;
                            changed = true;
                        }
                    }
                });
                if changed {
                    Self::update_total_price();
                }
            }
            _ => {
                if !this.selected_rooms.get_untracked().is_empty() {
                    this.selected_rooms.set(HashMap::new());
                    this.total_price.set(0.0);
                }
            }
        }
    }
}

impl GlobalStateForLeptos for HotelDetailsUIState {}
