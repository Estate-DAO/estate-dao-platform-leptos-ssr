use super::GlobalStateForLeptos;
// use crate::api::provab::HotelRoomDetail;
// use crate::api::{HotelRoomDetail, RoomDetail};
use crate::canister::backend::RoomDetails;
use crate::{log, warn};
use leptos::prelude::*;
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Default)]
pub struct RoomDetailsForPricingComponent {
    pub room_type: String,
    pub room_count: u32,
    pub room_unique_id: String,
    pub room_price: f64,
}

impl PartialEq for RoomDetailsForPricingComponent {
    fn eq(&self, other: &Self) -> bool {
        self.room_type == other.room_type
            && self.room_count == other.room_count
            && self.room_unique_id == other.room_unique_id
            && (self.room_price - other.room_price).abs() < 0.01
    }
}

impl Eq for RoomDetailsForPricingComponent {}

impl Hash for RoomDetailsForPricingComponent {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.room_type.hash(state);
        self.room_count.hash(state);
        self.room_unique_id.hash(state);
        self.room_price.to_bits().hash(state);
    }
}

pub struct RoomForPricingIter<'a> {
    inner: std::collections::btree_map::Iter<'a, String, RoomDetailsForPricingComponent>,
}

impl<'a> Iterator for RoomForPricingIter<'a> {
    type Item = RoomDetailsForPricingComponent;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|(room_type, room_details)| RoomDetailsForPricingComponent {
                room_type: room_type.clone(),
                room_count: room_details.room_count,
                room_unique_id: room_details.room_unique_id.clone(),
                room_price: room_details.room_price,
            })
    }
}

pub struct RoomForPricingIntoIter {
    inner: std::collections::btree_map::IntoIter<String, RoomDetailsForPricingComponent>,
}

impl Iterator for RoomForPricingIntoIter {
    type Item = RoomDetailsForPricingComponent;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|(room_type, room_details)| RoomDetailsForPricingComponent {
                room_type: room_type.clone(),
                room_count: room_details.room_count,
                room_unique_id: room_details.room_unique_id.clone(),
                room_price: room_details.room_price,
            })
    }
}

/// key is the room_type, value is (room_count, room_unique_id, room_price)
#[derive(Default, PartialEq, Eq, Debug, Clone)]
pub struct RoomForPricing(BTreeMap<String, RoomDetailsForPricingComponent>);
// pub struct RoomForPricing(BTreeMap<String, (u32, String, f64)>);

impl RoomForPricing {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    pub fn upsert(
        &mut self,
        room_type: String,
        room_count: u32,
        room_unique_id: String,
        room_price: f64,
    ) {
        self.0.insert(
            room_type.clone(),
            RoomDetailsForPricingComponent {
                room_type: room_type.clone(),
                room_count,
                room_unique_id,
                room_price,
            },
        );
    }

    pub fn increment(&mut self, room_type: String) {
        log!("[room_for_pricing] increment - room_type: {:?}", room_type);
        if let Some(room_details) = self.get(&room_type) {
            log!(
                "[room_for_pricing] increment - room_details: {:?}",
                room_details
            );
            self.upsert(
                room_type,
                room_details.room_count + 1,
                room_details.room_unique_id.clone(),
                room_details.room_price,
            );
        } else {
            log!("[room_for_pricing] increment - RoomForPricing increment failed");
            // self.upsert(room_type, 1, room_unique_id, room_price);
        }
    }

    pub fn accumulate(&mut self, room_type: String, room_unique_id: String, room_price: f64) {
        if let Some(room_details) = self.get(&room_type) {
            self.upsert(
                room_type.clone(),
                room_details.room_count + 1,
                room_unique_id.clone(),
                room_price,
            );
        } else {
            // log!(" RoomForPricing increment failed")
            self.upsert(room_type, 1, room_unique_id, room_price);
        }
    }

    pub fn decrement(&mut self, room_type: String) {
        log!("[room_for_pricing] decrement - room_type: {:?}", room_type);
        if let Some(room_details) = self.get(&room_type) {
            log!(
                "[room_for_pricing] decrement - room_details: {:?}",
                room_details
            );
            self.upsert(
                room_type.clone(),
                room_details.room_count.saturating_sub(1),
                room_details.room_unique_id.clone(),
                room_details.room_price,
            );
        } else {
            log!("[room_for_pricing] decrement - RoomForPricing decrement failed");
        }
    }
    pub fn get(&self, room_type: &str) -> Option<RoomDetailsForPricingComponent> {
        self.0.get(room_type).cloned()
    }

    pub fn get_or_default(&mut self, room_type: &str) -> RoomDetailsForPricingComponent {
        if let Some(room_details) = self.get(room_type) {
            room_details
        } else {
            let default = RoomDetailsForPricingComponent::default();
            self.0.insert(room_type.to_string(), default.clone());
            default
        }
    }

    pub fn iter(&self) -> RoomForPricingIter<'_> {
        RoomForPricingIter {
            inner: self.0.iter(),
        }
    }

    pub fn into_iter(self) -> RoomForPricingIntoIter {
        RoomForPricingIntoIter {
            inner: self.0.into_iter(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<'a> IntoIterator for &'a RoomForPricing {
    type Item = RoomDetailsForPricingComponent;
    type IntoIter = RoomForPricingIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl IntoIterator for RoomForPricing {
    type Item = RoomDetailsForPricingComponent;
    type IntoIter = RoomForPricingIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.into_iter()
    }
}

#[derive(Clone, Default, Debug)]
pub struct PricingBookNowState {
    /// room given to us by the API
    pub rooms_available_for_booking_from_api: RwSignal<RoomForPricing>,

    /// rooms chosen by the user from UI
    pub room_counters_as_chosen_by_user: RwSignal<RoomForPricing>,
}

impl GlobalStateForLeptos for PricingBookNowState {}

impl PricingBookNowState {
    pub fn new() -> Self {
        Self::default()
    }

    // pub fn room_details_from_hotel_details() -> Option<Vec<HotelRoomDetail>> {
    //     let hotel_info_results: HotelInfoResults = expect_context();
    //     hotel_info_results.get_hotel_room_details()
    // }

    fn ensure_room_initialized(room_type: &str, room_counters: &mut RoomForPricing) -> bool {
        let rooms_available = Self::get()
            .rooms_available_for_booking_from_api
            .get_untracked();

        // Return early if room already exists
        if room_counters.get(room_type).is_some() {
            let room_details = room_counters.get(room_type).unwrap();
            if room_details.room_price == 0.0 {
                // log!("[hotel_details_state] ensure_room_initialized - room_type={room_type} already exists with zero price, overwriting");
            } else {
                // log!("[hotel_details_state] ensure_room_initialized - room_type={room_type} already exists");
                return true;
            }
        }

        // Try to initialize user_selected_from from available rooms
        rooms_available
            .get(room_type)
            .map(|room| {
                // log!("[hotel_details_state] ensure_room_initialized - room_type={room_type} found in available rooms - {:?}", room);
                room_counters.upsert(
                    room_type.to_string(),
                    0,
                    room.room_unique_id.clone(),
                    room.room_price,
                );
                // log!("[hotel_details_state] ensure_room_initialized - room_type={room_type} initialized - room_counters={:?}", room_counters);
                true
            })
            .unwrap_or_else(|| {
                warn!(
                    "[hotel_details_state] ensure_room_initialized - room_type not found in available rooms (unreachable!): {:?}",
                    room_type
                );
                false
            })
    }

    pub fn increment_room_counter(room_type: String) {
        // log!(
        //     "[hotel_details_state] - increment_room_counter - room_type: {:?}",
        //     room_type
        // );
        let this = Self::get();
        let mut room_counters = this.room_counters_as_chosen_by_user.get_untracked();

        // log!("[hotel_details_state] - increment_room_counter - room_counters before early return: {:?}", room_counters);
        if !Self::ensure_room_initialized(&room_type, &mut room_counters) {
            return;
        }

        room_counters.increment(room_type);
        // log!(
        //     "[hotel_details_state] - increment_room_counter - room_counters: {:?}",
        //     room_counters
        // );
        this.room_counters_as_chosen_by_user.set(room_counters);
    }

    pub fn decrement_room_counter(room_type: String) {
        // log!(
        //     "[hotel_details_state] - decrement_room_counter - room_type: {:?}",
        //     room_type
        // );
        let this: Self = expect_context();
        let mut room_counters = this.room_counters_as_chosen_by_user.get_untracked();

        if !Self::ensure_room_initialized(&room_type, &mut room_counters) {
            return;
        }

        room_counters.decrement(room_type);
        log!(
            "[hotel_details_state] - decrement_room_counter - room_counters: {:?}",
            room_counters
        );
        this.room_counters_as_chosen_by_user.set(room_counters);
    }

    pub fn get_count_of_rooms_for_room_type(room_type: String) -> u32 {
        let this = Self::get();
        let mut room_counters = this.room_counters_as_chosen_by_user.get_untracked();
        let count = room_counters.get_or_default(&room_type).room_count;
        this.room_counters_as_chosen_by_user.set(room_counters);

        count
    }

    pub fn get_room_counters() -> RoomForPricing {
        let this = Self::get();
        this.room_counters_as_chosen_by_user.get()
    }

    // pub fn set_rooms_available_for_booking_from_api(room_details: Option<Vec<HotelRoomDetail>>) {
    //     if let Some(room_details) = room_details {
    //         let mut room_for_pricing = RoomForPricing::new();

    //         for room in room_details {
    //             // log!("[hotel_details_state] room_details_price: {:?}", room.price.offered_price);

    //             let room_type = room.room_type_name.clone();
    //             room_for_pricing.accumulate(
    //                 room_type,
    //                 room.room_unique_id,
    //                 room.price.offered_price,
    //             );
    //         }

    //         log!(
    //             "[hotel_details_state] - room_for_pricing: {:?}",
    //             room_for_pricing
    //         );

    //         Self::get()
    //             .rooms_available_for_booking_from_api
    //             .set(room_for_pricing);
    //         let local = Self::get().rooms_available_for_booking_from_api.get();
    //         log!(
    //             "[hotel_details_state] - room_for_pricing - local: {:?}",
    //             local
    //         );
    //     }
    // }

    pub fn set_room_counters_as_chosen_by_user(
        room_type: String,
        room_details: RoomDetailsForPricingComponent,
    ) {
        log!(
            "[hotel_details_state] -    set_room_counters_as_chosen_by_user - room_type: {:?}",
            room_type
        );
        let rooms_available = Self::get().rooms_available_for_booking_from_api.get();
        if let Some(room_available) = rooms_available.get(&room_type) {
            Self::get()
                .room_counters_as_chosen_by_user
                .update(|room_for_pricing| {
                    room_for_pricing.upsert(
                        room_type,
                        room_details.room_count,
                        room_available.room_unique_id.clone(),
                        room_available.room_price,
                    );
                });
        } else {
            warn!("[hotel_details_state] - set_room_counters_as_chosen_by_user - rooms_available_for_booking_from_api - room_type not found (unreachable!): {:?}", room_type);
        }
    }

    pub fn total_count_of_rooms_selected_by_user() -> u32 {
        let room_counters = Self::get().room_counters_as_chosen_by_user.get();
        room_counters
            .iter()
            .map(|room_details| room_details.room_count)
            .sum()
    }

    /// Returns a vector of room unique IDs based on user's room selection.
    ///
    /// For each selected room type, the function includes the room's unique ID
    /// in the result vector multiple times based on the room count. For example,
    /// if a user selects 4 rooms of the same type, the unique ID for that room
    /// will appear 4 times in the result.
    ///
    /// Empty room IDs are filtered out from the result.
    pub fn room_unique_ids() -> Vec<String> {
        let room_counters = Self::get().room_counters_as_chosen_by_user.get();
        let mut result = Vec::new();

        for room_details in room_counters.iter() {
            if !room_details.room_unique_id.is_empty() {
                // Add the same unique_id multiple times based on room_count
                for _ in 0..room_details.room_count {
                    result.push(room_details.room_unique_id.clone());
                }
            }
        }

        result
    }

    pub fn total_room_price_for_all_user_selected_rooms() -> f64 {
        let room_counters = Self::get().room_counters_as_chosen_by_user.get();

        room_counters
            .iter()
            .map(|room_details| {
                log!(
                    "[hotel_details_state] - total_room_price_for_all_user_selected_rooms - {:?}",
                    room_details
                );
                (room_details.room_count as f64) * room_details.room_price
            })
            .sum()
    }

    pub fn is_room_available_from_api() -> bool {
        !Self::get()
            .rooms_available_for_booking_from_api
            .get()
            .is_empty()
    }

    pub fn reset_room_counters() {
        Self::get()
            .room_counters_as_chosen_by_user
            .set(RoomForPricing::new());
    }

    /// Returns up to 5 rooms from the available rooms for booking
    pub fn list_rooms_in_pricing_component() -> RoomForPricing {
        let mut limited_rooms = RoomForPricing::new();
        Self::get()
            .rooms_available_for_booking_from_api
            .get()
            .iter()
            .take(5)
            .for_each(|room_details_for_pricing_component| {
                limited_rooms.upsert(
                    room_details_for_pricing_component.room_type.clone(),
                    room_details_for_pricing_component.room_count,
                    room_details_for_pricing_component.room_unique_id.clone(),
                    room_details_for_pricing_component.room_price,
                );
            });
        limited_rooms
    }
}

/// to enable easy conversion
impl From<RoomForPricing> for Vec<RoomDetails> {
    fn from(room_for_pricing: RoomForPricing) -> Self {
        let mut room_details = Vec::<RoomDetails>::new();
        for (room_type, room_details_for_pricing) in room_for_pricing.0 {
            room_details.push(RoomDetails {
                room_type_name: room_type,
                room_unique_id: room_details_for_pricing.room_unique_id,
                room_price: room_details_for_pricing.room_price as f32,
            });
        }
        room_details
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_room_for_pricing_increment() {
        let mut room_pricing = RoomForPricing::new();

        // First insert a room
        room_pricing.upsert("Deluxe".to_string(), 1, "room123".to_string(), 100.0);

        // Test increment on existing room
        room_pricing.increment("Deluxe".to_string());
        let room = room_pricing.get("Deluxe").unwrap();
        assert_eq!(room.room_count, 2, "Room count should be incremented to 2");
        assert_eq!(room.room_price, 100.0, "Price should remain unchanged");
        assert_eq!(
            room.room_unique_id, "room123",
            "Room ID should remain unchanged"
        );

        // Test increment on non-existent room - should fail silently
        room_pricing.increment("Suite".to_string());
        assert!(
            room_pricing.get("Suite").is_none(),
            "Non-existent room should not be created on increment"
        );
    }

    #[test]
    fn test_room_for_pricing_decrement() {
        let mut room_pricing = RoomForPricing::new();

        // First insert a room with count 2
        room_pricing.upsert("Deluxe".to_string(), 2, "room123".to_string(), 100.0);

        // Test decrement
        room_pricing.decrement("Deluxe".to_string());
        let room = room_pricing.get("Deluxe").unwrap();
        assert_eq!(room.room_count, 1, "Room count should be decremented to 1");
        assert_eq!(room.room_price, 100.0, "Price should remain unchanged");
        assert_eq!(
            room.room_unique_id, "room123",
            "Room ID should remain unchanged"
        );

        // Test decrement below zero
        room_pricing.decrement("Deluxe".to_string());
        room_pricing.decrement("Deluxe".to_string());
        let room = room_pricing.get("Deluxe").unwrap();
        assert_eq!(room.room_count, 0, "Room count can go below zero");

        // Test decrement on non-existent room - should fail silently
        room_pricing.decrement("Suite".to_string());
        assert!(
            room_pricing.get("Suite").is_none(),
            "Non-existent room should not be created on decrement"
        );
    }
}
