use crate::domain::{DomainHotelDetails, DomainRoomData};
use leptos::*;
use std::collections::HashMap;

use super::GlobalStateForLeptos;

#[derive(Debug, Clone, Default)]
pub struct HotelDetailsUIState {
    pub hotel_details: RwSignal<Option<DomainHotelDetails>>,
    pub loading: RwSignal<bool>,
    pub error: RwSignal<Option<String>>,
    pub available_rooms: RwSignal<Vec<DomainRoomData>>,
    pub selected_rooms: RwSignal<HashMap<String, u32>>, // room_type -> quantity
    pub room_loading: RwSignal<bool>,
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
        this.available_rooms.set(vec![]);
        this.selected_rooms.set(HashMap::new());
        this.room_loading.set(false);
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

    // <!-- Room selection methods -->
    pub fn set_available_rooms(rooms: Vec<DomainRoomData>) {
        let this: Self = expect_context();
        this.available_rooms.set(rooms);
    }

    pub fn get_available_rooms() -> Vec<DomainRoomData> {
        let this: Self = expect_context();
        this.available_rooms.get()
    }

    pub fn set_room_loading(loading: bool) {
        let this: Self = expect_context();
        this.room_loading.set(loading);
    }

    pub fn is_room_loading() -> bool {
        let this: Self = expect_context();
        this.room_loading.get()
    }

    pub fn increment_room_counter(room_type: String) {
        let this: Self = expect_context();
        this.selected_rooms.update(|rooms| {
            let current = rooms.get(&room_type).copied().unwrap_or(0);
            rooms.insert(room_type, current + 1);
        });
        Self::update_total_price();
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

    // <!-- Helper method to update total price when room selection changes -->
    fn update_total_price() {
        let this: Self = expect_context();

        // Calculate total price based on selected rooms and available room data
        let selected_rooms = this.selected_rooms.get();
        let available_rooms = this.available_rooms.get();

        let total = selected_rooms
            .iter()
            .fold(0.0, |acc, (room_type, &quantity)| {
                // Find the room data for this room type to get price
                // Note: This is a simplified calculation - in real implementation,
                // you would need to match room_type to actual room pricing from hotel details
                acc + (quantity as f64 * 100.0) // Placeholder price calculation
            });

        this.total_price.set(total);
    }
}

impl GlobalStateForLeptos for HotelDetailsUIState {}
