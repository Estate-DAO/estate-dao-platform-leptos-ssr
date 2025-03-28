use super::{search_state::HotelInfoResults, GlobalStateForLeptos};
use crate::api::HotelRoomDetail;
// use crate::api::{HotelRoomDetail, RoomDetail};
use crate::canister::backend::RoomDetails;
use crate::log;
use leptos::*;
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
        if let Some(room_details) = self.get(&room_type) {
            self.upsert(
                room_type.clone(),
                room_details.room_count + 1,
                room_details.room_unique_id.clone(),
                room_details.room_price,
            );
        } else {
            log!(" RoomForPricing increment failed")
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
        if let Some(room_details) = self.get(&room_type) {
            self.upsert(
                room_type.clone(),
                room_details.room_count - 1,
                room_details.room_unique_id.clone(),
                room_details.room_price,
            );
        }
    }
    pub fn get(&self, room_type: &str) -> Option<&RoomDetailsForPricingComponent> {
        self.0.get(room_type)
    }

    pub fn get_or_default(&mut self, room_type: &str) -> RoomDetailsForPricingComponent {
        if let Some(room_details) = self.0.get(room_type) {
            room_details.clone()
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

    pub fn increment_room_counter(room_type: String) {
        let this = Self::get();
        let mut room_counters = this.room_counters_as_chosen_by_user.get();
        room_counters.increment(room_type);
        this.room_counters_as_chosen_by_user.set(room_counters);
    }

    pub fn decrement_room_counter(room_type: String) {
        let this = Self::get();
        let mut room_counters = this.room_counters_as_chosen_by_user.get();
        room_counters.decrement(room_type);
        this.room_counters_as_chosen_by_user.set(room_counters);
    }

    pub fn get_count_of_rooms_for_room_type(room_type: String) -> u32 {
        let this = Self::get();
        let mut room_counters = this.room_counters_as_chosen_by_user.get();
        let count = room_counters.get_or_default(&room_type).room_count;
        this.room_counters_as_chosen_by_user.set(room_counters);

        count
    }

    pub fn get_room_counters() -> RoomForPricing {
        let this = Self::get();
        this.room_counters_as_chosen_by_user.get()
    }

    pub fn set_rooms_available_for_booking_from_api(room_details: Option<Vec<HotelRoomDetail>>) {
        // todo (bug145): max call stack size exceeded
        // let room_details = Self::room_details_from_hotel_details();
        // let hotel_info_results: HotelInfoResults = expect_context();
        // let room_details = hotel_info_results.get_hotel_room_details();

        if let Some(room_details) = room_details {
            let mut room_for_pricing = RoomForPricing::new();

            for room in room_details {
                let room_type = room.room_type_name.clone();
                room_for_pricing.accumulate(
                    room_type,
                    room.room_unique_id,
                    room.price.offered_price,
                );
            }
            Self::get()
                .rooms_available_for_booking_from_api
                .set(room_for_pricing);
        }
    }

    pub fn set_room_counters_as_chosen_by_user(
        room_type: String,
        room_details: RoomDetailsForPricingComponent,
    ) {
        Self::get()
            .room_counters_as_chosen_by_user
            .update(|room_for_pricing| {
                room_for_pricing.upsert(
                    room_type,
                    room_details.room_count,
                    room_details.room_unique_id,
                    room_details.room_price,
                );
            });
    }

    pub fn total_count_of_rooms_selected_by_user() -> u32 {
        let room_counters = Self::get().room_counters_as_chosen_by_user.get();
        room_counters
            .iter()
            .map(|room_details| room_details.room_count)
            .sum()
    }

    pub fn unique_room_ids() -> Vec<String> {
        let room_counters = Self::get().room_counters_as_chosen_by_user.get();
        room_counters
            .iter()
            .map(|room_details| room_details.room_unique_id.clone())
            .collect()
    }

    pub fn total_room_price_for_all_user_selected_rooms() -> f64 {
        let room_counters = Self::get().room_counters_as_chosen_by_user.get();

        room_counters
            .iter()
            .map(|room_details| (room_details.room_count as f64) * room_details.room_price)
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

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_get_count_of_rooms_for_room_type() {

//         // Add a room type with count
//         PricingBookNowState::set_room_counters_as_chosen_by_user(
//             "Deluxe".to_string(),
//             RoomDetailsForPricingComponent {
//                 room_type: "Deluxe".to_string(),
//                 room_count: 2,
//                 room_unique_id: "123".to_string(),
//                 room_price: 100.0,
//             }
//         );

//         // Test existing room type
//         let count_deluxe = PricingBookNowState::get_count_of_rooms_for_room_type("Deluxe".to_string());
//         assert_eq!(count_deluxe, 2, "Should return correct count for existing room type");

//         // Test non-existing room type
//         let count_suite = PricingBookNowState::get_count_of_rooms_for_room_type("Suite".to_string());
//         assert_eq!(count_suite, 0, "Should return 0 for non-existing room type");
//     }
// }
