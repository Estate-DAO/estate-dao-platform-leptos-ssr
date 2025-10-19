use crate::{
    domain::{DomainHotelDetails, DomainRoomData, DomainRoomOption},
    error, log, warn,
};
use leptos::prelude::*;
use std::collections::HashMap;

use super::GlobalStateForLeptos;

#[derive(Debug, Clone, Default)]
pub struct HotelDetailsUIState {
    pub hotel_details: RwSignal<Option<DomainHotelDetails>>,
    pub loading: RwSignal<bool>,
    pub error: RwSignal<Option<String>>,
    pub selected_rooms: RwSignal<HashMap<String, u32>>, // room_unique_id -> quantity
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

    pub fn set_loading(loading: bool) {
        let this: Self = expect_context();
        this.loading.set(loading);
    }

    pub fn set_error(error: Option<String>) {
        let this: Self = expect_context();
        this.error.set(error);
    }

    pub fn reset() {
        let this: Self = expect_context();
        this.hotel_details.set(None);
        this.loading.set(false);
        this.error.set(None);
        this.selected_rooms.set(HashMap::new());
        this.total_price.set(0.0);
    }

    pub fn get_hotel_details() -> Option<DomainHotelDetails> {
        let this: Self = expect_context();
        this.hotel_details.get()
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
        if let Some(hotel_details) = this.hotel_details.get() {
            hotel_details.all_rooms
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
                    .find(|option| option.room_data.room_unique_id == room_id)
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
                acc + (room_option.price.room_price * *quantity as f64)
                // acc + (room_option.price.room_price * *quantity as f64 * nights as f64)
            })
    }

    // <!-- Calculate line total for a specific room -->
    pub fn calculate_room_line_total(
        room_option: &DomainRoomOption,
        quantity: u32,
        nights: u32,
    ) -> f64 {
        room_option.price.room_price * quantity as f64
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
                acc + (room_option.price.room_price * *quantity as f64)
            });

        this.total_price.set(total);
    }
}

impl GlobalStateForLeptos for HotelDetailsUIState {}
