// use crate::{
//     api::{
//         payments::ports::GetPaymentStatusResponse, BlockRoomRequest, BlockRoomResponse,
//         BookRoomRequest, BookRoomResponse, HotelInfoRequest, HotelInfoResponse, HotelRoomDetail,
//         HotelRoomRequest, HotelRoomResponse, HotelSearchRequest, HotelSearchResponse, RoomDetail,
//     },
//     canister::backend,
//     component::{Destination, GuestSelection, SelectedDateRange},
//     state::{hotel_details_state::PricingBookNowState, view_state::BlockRoomCtx},
//     utils::app_reference::generate_app_reference,
// };
// // use leptos::logging::log;
// use crate::log;
// use leptos::RwSignal;
// use leptos::*;
// use std::collections::HashMap;

// use super::{view_state::HotelInfoCtx, GlobalStateForLeptos};

// //  ==================================================================

// #[derive(Clone, Default, Debug)]
// pub struct SearchCtx {
//     // invalid_cnt: RwSignal<u32>,
//     pub destination: RwSignal<Option<Destination>>,
//     pub date_range: RwSignal<SelectedDateRange>,
//     pub guests: RwSignal<GuestSelection>,
//     pub on_form_reset: Trigger,
// }

// impl SearchCtx {
//     pub fn set_destination(destination: Destination) {
//         let this: Self = expect_context();

//         this.destination.set(Some(destination));
//     }

//     pub fn set_date_range(date_range: SelectedDateRange) {
//         let this: Self = expect_context();

//         this.date_range.set(date_range);
//     }

//     pub fn set_guests(guests: GuestSelection) {
//         let this: Self = expect_context();

//         this.guests.set(guests);
//     }

//     pub fn log_state() {
//         let this: Self = expect_context();

//         log::info!(
//             "\n\nguests.adults: {:?}",
//             this.guests.get_untracked().adults.get_untracked()
//         );
//         log::info!(
//             "guests.children: {:?}",
//             this.guests.get_untracked().children.get_untracked()
//         );
//         log::info!(
//             "guests.children_ages: {:?}",
//             this.guests.get_untracked().children_ages.get_untracked()
//         );

//         log::info!(
//             "\n\ndate_range.start: {:?}",
//             this.date_range.get_untracked().start
//         );
//         log::info!("date_range.end: {:?}", this.date_range.get_untracked().end);

//         log::info!(
//             "\n\ndestination: {:?}\n\n",
//             this.destination.get_untracked()
//         );
//     }

//     pub fn get_backend_compatible_date_range_untracked() -> backend::SelectedDateRange {
//         let this: Self = expect_context();
//         this.date_range.get_untracked().into()
//     }

//     pub fn get_backend_compatible_destination_untracked() -> Option<backend::Destination> {
//         let this: Self = expect_context();
//         this.destination.get_untracked().map(|dest| dest.into())
//     }
// }

// //  ==================================================================

// #[derive(Debug, Clone, Default)]
// pub struct SearchListResults {
//     pub search_result: RwSignal<Option<HotelSearchResponse>>,
// }

// impl SearchListResults {
//     fn from_leptos_context() -> Self {
//         expect_context()
//     }
//     pub fn reset() {
//         Self::from_leptos_context().search_result.set(None);
//     }

//     pub fn set_search_results(hotel_search_response: Option<HotelSearchResponse>) {
//         Self::from_leptos_context()
//             .search_result
//             .set(hotel_search_response);
//     }
//     pub fn get_hotel_code_results_token_map(&self) -> HashMap<String, String> {
//         self.search_result
//             .get_untracked()
//             .as_ref()
//             .map_or_else(HashMap::new, |response| response.get_results_token_map())
//     }

//     pub fn get_result_token(hotel_code: String) -> String {
//         let this: Self = expect_context();
//         this.get_hotel_code_results_token_map()
//             .get(&hotel_code)
//             .cloned()
//             .unwrap_or_default()
//     }

//     pub fn hotel_info_request(&self, hotel_code: &str) -> HotelInfoRequest {
//         let token = Self::get_result_token(hotel_code.into());
//         HotelInfoRequest { token }
//     }

//     pub fn hotel_room_request(&self, hotel_code: &str) -> HotelRoomRequest {
//         let token = Self::get_result_token(hotel_code.into());
//         HotelRoomRequest { token }
//     }
// }

// //  ==================================================================

// #[derive(Debug, Clone, Default)]
// pub struct HotelInfoResults {
//     pub search_result: RwSignal<Option<HotelInfoResponse>>,
//     pub room_result: RwSignal<Option<HotelRoomResponse>>,
//     pub price_per_night: RwSignal<f64>,
//     // pub room_counters: RwSignal<HashMap<String, RoomCounterKeyValue>>,
//     // pub block_room_counters: RwSignal<HashMap<String, RoomCounterKeyValueStatic>>,
//     // pub sorted_rooms: RwSignal<Vec<SortedRoom>>,
// }

// impl GlobalStateForLeptos for HotelInfoResults {}

// impl HotelInfoResults {
//     pub fn display(&self) -> String {
//         let json_repr = serde_json::json!({
//             "search_result": self.search_result.get_untracked(),
//             "room_result": self.room_result.get_untracked(),
//             "price_per_night": self.price_per_night.get_untracked(),
//             // "block_room_counters": self.block_room_counters.get(),
//             // "sorted_rooms": self.sorted_rooms.get_untracked()
//         });

//         serde_json::to_string_pretty(&json_repr)
//             .unwrap_or_else(|_| "Failed to serialize".to_string())
//     }

//     fn from_leptos_context() -> Self {
//         Self::get()
//     }

//     pub fn reset() {
//         Self::from_leptos_context().search_result.set(None);
//         Self::from_leptos_context().room_result.set(None);
//         // Self::from_leptos_context()
//         //     .room_counters
//         //     .set(HashMap::new());
//         // Self::from_leptos_context()
//         //     .block_room_counters
//         //     .set(HashMap::new());
//         // Self::from_leptos_context().sorted_rooms.set(Vec::new());
//     }

//     pub fn set_info_results(hotel_info_response: Option<HotelInfoResponse>) {
//         Self::from_leptos_context()
//             .search_result
//             .set(hotel_info_response);
//     }

//     pub fn set_room_results(hotel_room_response: Option<HotelRoomResponse>) {
//         let self_val = Self::from_leptos_context();
//         log!(
//             "[pricing_component_not_loading] Setting room results: {:?}",
//             hotel_room_response.is_some()
//         );
//         if let Some(response) = &hotel_room_response {
//             log!(
//                 "[pricing_component_not_loading] Room response details: {:?}",
//                 response.get_hotel_room_details()
//             );
//         }
//         self_val.room_result.set(hotel_room_response);

//         // as soon as room results are available, set rooms available for booking
//         let room_details: Option<Vec<HotelRoomDetail>> = self_val.get_hotel_room_details();
//         let room_details_prices: Vec<(String, f64)> = room_details
//             .iter()
//             .flat_map(|details| details.iter())
//             .map(|r| (r.room_type_name.clone(), r.price.offered_price))
//             .collect();
//         log!(
//             "[pricing_component_not_loading] Room details: {:?}",
//             room_details_prices
//         );
//         PricingBookNowState::set_rooms_available_for_booking_from_api(room_details);
//     }

//     pub fn get_hotel_room_details(&self) -> Option<Vec<HotelRoomDetail>> {
//         log!(
//             "[pricing_component_not_loading] get_hotel_room_details -> room_result exists: {:?}",
//             self.room_result.get().is_some()
//         );
//         let result = self.room_result.get().and_then(|response| {
//             log!("[pricing_component_not_loading] get_hotel_room_details -> mapping response");
//             let details = response.get_hotel_room_details();
//             log!(
//                 "[pricing_component_not_loading] get_hotel_room_details -> details: {:?}",
//                 details
//             );
//             details
//         });
//         log!(
//             "[pricing_component_not_loading] get_hotel_room_details -> returned: {:?}",
//             result.is_some()
//         );
//         result
//     }

//     pub fn set_price_per_night(&self, per_night_calc: f64) {
//         Self::from_leptos_context()
//             .price_per_night
//             .set(per_night_calc);
//     }

//     pub fn get_hotel_name_untracked() -> String {
//         Self::from_leptos_context()
//             .search_result
//             .get_untracked()
//             .map(|response| response.get_hotel_name())
//             .unwrap_or_default()
//     }

//     pub fn get_hotel_location_untracked() -> String {
//         Self::from_leptos_context()
//             .search_result
//             .get_untracked()
//             .map(|response| response.get_location())
//             .unwrap_or_default()
//     }

//     pub fn get_hotel_images_untracked() -> Vec<String> {
//         Self::from_leptos_context()
//             .search_result
//             .get_untracked()
//             .map(|response| response.get_images())
//             .unwrap_or_default()
//     }

//     pub fn get_at_least_one_hotel_image_untracked() -> String {
//         Self::get_hotel_images_untracked()
//             .get(0)
//             .cloned()
//             .unwrap_or_default()
//     }

//     // pub fn get_images(&self) -> Vec<String> {
//     //     self.search_result
//     //         .get()
//     //         .map(|response| response.get_images())
//     //         .unwrap_or_default()
//     // }

//     // pub fn hotel_name_signal(&self) -> String {
//     //     self.search_result
//     //         .get()
//     //         .map(|response| response.get_hotel_name())
//     //         .unwrap_or_default()
//     // }

//     // pub fn hotel_address_signal(&self) -> String {
//     //     self.search_result
//     //         .get()
//     //         .map(|response| response.get_address())
//     //         .unwrap_or_default()
//     // }

//     // let images_signal = move || {
//     //     if let Some(hotel_info_api_response) = hotel_info_results.search_result.get() {
//     //         hotel_info_api_response.get_images()
//     //     } else {
//     //         vec![]
//     //     }
//     // };

//     // pub fn set_room_counters(&self, room_counters: HashMap<String, RoomCounterKeyValue>) {
//     //     Self::from_leptos_context().room_counters.set(room_counters);
//     // }

//     // pub fn get_room_count(&self, room_type: &str) -> Option<u32> {
//     //     self.room_counters
//     //         .get()
//     //         .get(room_type)
//     //         .map(|counter| counter.key.get_untracked())
//     // }

//     // pub fn get_room_unique_id(&self, room_type: &str) -> Option<String> {
//     //     self.room_counters
//     //         .get()
//     //         .get(room_type)
//     //         .and_then(|counter| counter.value.get_untracked())
//     // }

//     // pub fn update_room_count(&self, room_type: String, count: u32) {
//     //     if let Some(counter) = self.room_counters.get().get(&room_type) {
//     //         counter.key.set(count);
//     //     } else {
//     //         let mut counters = self.room_counters.get();
//     //         let mut new_counter = RoomCounterKeyValue::default();
//     //         new_counter.key.set(count);
//     //         counters.insert(room_type, new_counter);
//     //         self.room_counters.set(counters);
//     //     }
//     // }

//     // pub fn update_room_unique_id(&self, room_type: String, unique_id: Option<String>) {
//     //     if let Some(counter) = self.room_counters.get().get(&room_type) {
//     //         counter.value.set(unique_id);
//     //     } else {
//     //         let mut counters = self.room_counters.get();
//     //         let mut new_counter = RoomCounterKeyValue::default();
//     //         new_counter.value.set(unique_id);
//     //         counters.insert(room_type, new_counter);
//     //         self.room_counters.set(counters);
//     //     }
//     // }

//     // pub fn increment_room_count(&self, room_type: String) -> bool {
//     //     if let Some(counter) = self.room_counters.get().get(&room_type) {
//     //         let current = counter.key.get();
//     //         counter.key.set(current + 1);
//     //         true
//     //     } else {
//     //         let mut counters = self.room_counters.get();
//     //         let mut new_counter = RoomCounterKeyValue::default();
//     //         new_counter.key.set(1);
//     //         counters.insert(room_type, new_counter);
//     //         self.room_counters.set(counters);
//     //         true
//     //     }
//     // }

//     // pub fn decrement_room_count(&self, room_type: String) -> bool {
//     //     if let Some(counter) = self.room_counters.get().get(&room_type) {
//     //         let current = counter.key.get();
//     //         if current > 0 {
//     //             counter.key.set(current - 1);
//     //             true
//     //         } else {
//     //             false
//     //         }
//     //     } else {
//     //         false
//     //     }
//     // }

//     // pub fn set_sorted_rooms(&self, sorted_rooms: Vec<SortedRoom>) {
//     //     Self::from_leptos_context().sorted_rooms.set(sorted_rooms);
//     // }

//     // pub fn set_block_room_counters(&self, room_counters: HashMap<String, RoomCounterKeyValue>) {
//     //     // log!("set_block_room_counters : input:  {room_counters:#?}");

//     //     let new_map: HashMap<String, RoomCounterKeyValueStatic> = room_counters
//     //         .into_iter()
//     //         .map(|(key, value)| (key, RoomCounterKeyValueStatic::from(value)))
//     //         .collect();

//     //     // log!("final new_map for block_room_counters:  {new_map:#?}");

//     //     Self::from_leptos_context().block_room_counters.set(new_map);
//     // }

//     pub fn block_room_request(&self, uniq_room_ids: Vec<String>) -> BlockRoomRequest {
//         // Get unique room IDs from the room response
//         // let room_unique_id = self
//         //     .room_result
//         //     .get()
//         //     .as_ref()
//         //     .map(|response| {
//         //         response
//         //             .room_list
//         //             .as_ref()
//         //             .map(|room_list| {
//         //                 room_list
//         //                     .get_hotel_room_result
//         //                     .hotel_rooms_details
//         //                     .iter()
//         //                     .map(|room| room.room_unique_id.clone())
//         //                     .collect()
//         //             })
//         //             .unwrap_or_default()
//         //     })
//         //     .unwrap_or_default();

//         let room_unique_id = uniq_room_ids;

//         // Get token from SearchListResults context since that has the token map
//         let search_list_results: SearchListResults = expect_context();
//         let hotel_info_ctx: HotelInfoCtx = expect_context();

//         let hotel_code = hotel_info_ctx.hotel_code.get();

//         let token = search_list_results
//             .get_hotel_code_results_token_map()
//             .get(&hotel_code)
//             .cloned()
//             .unwrap_or_default();

//         BlockRoomRequest {
//             token,
//             room_unique_id,
//         }
//     }
// }

// //  ==================================================================

// #[derive(Debug, Clone, Default)]
// pub struct BlockRoomResults {
//     pub block_room_results: RwSignal<Option<BlockRoomResponse>>,
//     pub payment_status_response: RwSignal<Option<GetPaymentStatusResponse>>,
//     pub block_room_id: RwSignal<Option<String>>,
// }

// impl BlockRoomResults {
//     fn from_leptos_context() -> Self {
//         expect_context()
//     }

//     pub fn reset() {
//         Self::from_leptos_context().block_room_results.set(None);
//     }

//     pub fn set_results(results: Option<BlockRoomResponse>) {
//         Self::from_leptos_context().block_room_results.set(results);
//     }

//     pub fn set_id(id: Option<String>) {
//         Self::from_leptos_context().block_room_id.set(id);
//     }

//     pub fn set_payment_results(results: Option<GetPaymentStatusResponse>) {
//         Self::from_leptos_context()
//             .payment_status_response
//             .set(results);
//     }

//     pub fn get_room_price() -> f64 {
//         Self::from_leptos_context()
//             .block_room_results
//             .get()
//             .unwrap_or_default()
//             .get_room_price_summed()
//     }

//     pub fn has_valid_room_price() -> bool {
//         Self::get_room_price().abs() > 0.001
//     }

//     // todo caution - do not generate app reference here please.
//     // it is already generated in block room, and saved in backend. use it from there.
//     // pub fn book_room_request(&self) -> BookRoomRequest {
//     //     let search_list_results: SearchListResults = expect_context();
//     //     let hotel_info_ctx: HotelInfoCtx = expect_context();
//     //     let block_room_ctx: BlockRoomCtx = expect_context();

//     //     let hotel_code = hotel_info_ctx.hotel_code.get();
//     //     let result_token = search_list_results
//     //         .get_hotel_code_results_token_map()
//     //         .get(&hotel_code)
//     //         .cloned()
//     //         .unwrap_or_default();

//     //     let block_room_id = self
//     //         .block_room_results
//     //         .get_untracked()
//     //         .unwrap()
//     //         .get_block_room_id()
//     //         .expect("Block Room API call failed");

//     //     // let hotel_code = hotel_info_ctx.hotel_code.get();
//     //     // let app_reference = format!("BOOKING_{}_{}", chrono::Utc::now().timestamp(), hotel_code);
//     //     let email = <std::option::Option<std::string::String> as Clone>::clone(
//     //         &block_room_ctx.adults.get().first().unwrap().email,
//     //     )
//     //     .unwrap();

//     //     let app_reference = generate_app_reference(email);
//     //     let app_ref = app_reference
//     //         .get()
//     //         .expect("app reference is not here")
//     //         .get_app_reference();
//     //     log!("app_ref - {app_ref}");

//     //     let confirmation: ConfirmationResults = expect_context();
//     //     let room_details = confirmation.room_details.get_untracked().unwrap();

//     //     BookRoomRequest {
//     //         result_token,
//     //         block_room_id,
//     //         app_reference: app_ref,
//     //         room_details: vec![room_details],
//     //     }
//     // }
// }

// //  ==================================================================

// #[derive(Debug, Clone, Default)]
// pub struct ConfirmationResults {
//     pub booking_details: RwSignal<Option<BookRoomResponse>>,
//     pub room_details: RwSignal<Option<RoomDetail>>,
// }

// impl ConfirmationResults {
//     fn from_leptos_context() -> Self {
//         expect_context()
//     }

//     pub fn reset() {
//         Self::from_leptos_context().booking_details.set(None);
//     }

//     pub fn set_booking_details(booking_response: Option<BookRoomResponse>) {
//         Self::from_leptos_context()
//             .booking_details
//             .set(booking_response);
//     }

//     pub fn set_room_details(room_detail: Option<RoomDetail>) {
//         Self::from_leptos_context().room_details.set(room_detail);
//     }
// }
