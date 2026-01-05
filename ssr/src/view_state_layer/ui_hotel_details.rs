use crate::{
    domain::{
        DomainHotelDetails, DomainHotelStaticDetails, DomainRoomData, DomainRoomGroup,
        DomainRoomVariant,
    },
    error, log, warn,
};
use leptos::*;
use std::collections::{HashMap, HashSet};

use super::GlobalStateForLeptos;

#[derive(Debug, Clone, Default)]
pub struct HotelDetailsUIState {
    pub hotel_details: RwSignal<Option<DomainHotelDetails>>,
    pub static_details: RwSignal<Option<DomainHotelStaticDetails>>,
    pub rates: RwSignal<Option<Vec<DomainRoomGroup>>>,
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

    pub fn set_rates(rates: Option<Vec<DomainRoomGroup>>) {
        let this: Self = expect_context();
        let previous = this.rates.get_untracked();
        let cloned = rates.clone();
        this.rates.set(rates);
        this.selected_rooms.set(HashMap::new());
        Self::sync_selected_rooms_with_rates(cloned, previous);
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
    pub fn get_available_room_variants() -> Vec<DomainRoomVariant> {
        let this: Self = expect_context();
        if let Some(groups) = this.rates.get() {
            groups
                .into_iter()
                .flat_map(|group| group.room_types)
                .collect()
        } else {
            vec![]
        }
    }

    // Deprecated or adapted?
    pub fn get_available_room_options() -> Vec<DomainRoomVariant> {
        Self::get_available_room_variants()
    }

    // Adapted to use simplified data (partially, since DomainRoomData no longer matches exactly)
    // We retain this if used elsewhere, but maybe we don't need it.
    // pub fn get_available_rooms() -> Vec<DomainRoomData> { ... }

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
    pub fn get_selected_rooms_with_data() -> Vec<(DomainRoomVariant, u32)> {
        let selected_rooms = Self::get_selected_rooms();
        let available_variants = Self::get_available_room_variants();

        selected_rooms
            .into_iter()
            .filter(|(_, quantity)| *quantity > 0)
            .filter_map(|(rate_key, quantity)| {
                available_variants
                    .iter()
                    .find(|option| option.rate_key == rate_key)
                    .map(|variant| (variant.clone(), quantity))
            })
            .collect()
    }

    // <! -- Calculate subtotal for given number of nights -->
    pub fn calculate_subtotal_for_nights() -> f64 {
        use crate::view_state_layer::ui_search_state::UISearchCtx;
        let ui_search_ctx: UISearchCtx = expect_context();
        let nights = {
            let n = ui_search_ctx.date_range.get().no_of_nights();
            if n == 0 {
                1.0
            } else {
                n as f64
            }
        };

        let selected_rooms_with_data = Self::get_selected_rooms_with_data();

        let total = selected_rooms_with_data
            .iter()
            .fold(0.0, |acc, (variant, quantity)| {
                // Calculate per-night price and round to 2 decimals to match display
                let total_price = variant.price_per_room_excluding_taxes;
                let price_per_night = total_price / nights;
                let rounded_price = (price_per_night * 100.0).round() / 100.0;
                let line_total = rounded_price * *quantity as f64 * nights;
                acc + line_total
            });
        // Round final total to 2 decimals to ensure consistency with line items
        (total * 100.0).round() / 100.0
    }

    // <!-- Calculate line total for a specific room -->
    pub fn calculate_room_line_total(
        variant: &DomainRoomVariant,
        quantity: u32,
        _nights: u32, // Unused param but kept for signature if needed
    ) -> f64 {
        variant.price_per_room_excluding_taxes * quantity as f64
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
            .fold(0.0, |acc, (variant, quantity)| {
                acc + (variant.price_per_room_excluding_taxes * *quantity as f64)
            });

        this.total_price.set(total);
    }

    fn sync_selected_rooms_with_rates(
        rates: Option<Vec<DomainRoomGroup>>,
        previous_rates: Option<Vec<DomainRoomGroup>>,
    ) {
        let this: Self = expect_context();
        match rates {
            Some(groups) if !groups.is_empty() => {
                let valid_ids: HashSet<_> = groups
                    .iter()
                    .flat_map(|g| g.room_types.iter())
                    .map(|v| v.rate_key.clone())
                    .collect();

                let mut changed = false;
                this.selected_rooms.update(|rooms| {
                    let mut to_remove = Vec::new();
                    // We don't implement complicated rekeying for now unless essential
                    // (previous code rekeyed if mapped_room_id matched but rate_key changed)
                    // We can attempt rekeying if we have mapped_room_id.

                    for room_id in rooms.keys().cloned().collect::<Vec<_>>() {
                        if valid_ids.contains(&room_id) {
                            continue;
                        }
                        to_remove.push(room_id);
                    }

                    for room_id in to_remove {
                        rooms.remove(&room_id);
                        changed = true;
                    }
                });
                if changed {
                    Self::update_total_price();
                }
            }
            _ => {
                // Clear selections
                if !this.selected_rooms.get_untracked().is_empty() {
                    this.selected_rooms.set(HashMap::new());
                    this.total_price.set(0.0);
                }
            }
        }
    }
}

impl GlobalStateForLeptos for HotelDetailsUIState {}
