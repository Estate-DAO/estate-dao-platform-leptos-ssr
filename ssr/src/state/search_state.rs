use crate::{
    api::{
        BlockRoomRequest, BlockRoomResponse, BookRoomRequest, BookRoomResponse, HotelInfoRequest, HotelInfoResponse, HotelRoomDetail, HotelRoomRequest, HotelRoomResponse, HotelSearchRequest, HotelSearchResponse, RoomDetail
    },
    component::{Destination, GuestSelection, SelectedDateRange},
    page::{RoomCounterKeyValue, RoomCounterKeyValueStatic, SortedRoom},
};
use leptos::logging::log;
use leptos::RwSignal;
use leptos::*;
use std::collections::HashMap;

use super::view_state::{BlockRoomCtx, HotelInfoCtx};

#[derive(Clone, Default, Debug)]
pub struct SearchCtx {
    // invalid_cnt: RwSignal<u32>,
    pub destination: RwSignal<Option<Destination>>,
    pub date_range: RwSignal<SelectedDateRange>,
    pub guests: RwSignal<GuestSelection>,
    pub on_form_reset: Trigger,
}

impl SearchCtx {
    pub fn set_destination(destination: Destination) {
        let this: Self = expect_context();

        this.destination.set(Some(destination));
    }

    pub fn set_date_range(date_range: SelectedDateRange) {
        let this: Self = expect_context();

        this.date_range.set(date_range);
    }

    pub fn log_state() {
        let this: Self = expect_context();

        log::info!(
            "\n\nguests.adults: {:?}",
            this.guests.get_untracked().adults.get_untracked()
        );
        log::info!(
            "guests.children: {:?}",
            this.guests.get_untracked().children.get_untracked()
        );
        log::info!(
            "guests.children_ages: {:?}",
            this.guests.get_untracked().children_ages.get_untracked()
        );

        log::info!(
            "\n\ndate_range.start: {:?}",
            this.date_range.get_untracked().start
        );
        log::info!("date_range.end: {:?}", this.date_range.get_untracked().end);

        log::info!(
            "\n\ndestination: {:?}\n\n",
            this.destination.get_untracked()
        );
    }
}

#[derive(Debug, Clone, Default)]
pub struct SearchListResults {
    pub search_result: RwSignal<Option<HotelSearchResponse>>,
}

impl SearchListResults {
    fn from_leptos_context() -> Self {
        expect_context()
    }
    pub fn reset() {
        Self::from_leptos_context().search_result.set(None);
    }

    pub fn set_search_results(hotel_search_response: Option<HotelSearchResponse>) {
        Self::from_leptos_context()
            .search_result
            .set(hotel_search_response);
    }
    pub fn get_hotel_code_results_token_map(&self) -> HashMap<String, String> {
        self.search_result
            .get_untracked()
            .as_ref()
            .map_or_else(HashMap::new, |response| response.get_results_token_map())
    }

    fn get_result_token(&self, hotel_code: String) -> String {
        self.get_hotel_code_results_token_map()
            .get(&hotel_code)
            .unwrap()
            .clone()
    }

    pub fn hotel_info_request(&self, hotel_code: &str) -> HotelInfoRequest {
        let token = self.get_result_token(hotel_code.into());
        HotelInfoRequest { token }
    }

    pub fn hotel_room_request(&self, hotel_code: &str) -> HotelRoomRequest {
        let token = self.get_result_token(hotel_code.into());
        HotelRoomRequest { token }
    }
}

#[derive(Debug, Clone, Default)]
pub struct HotelInfoResults {
    pub search_result: RwSignal<Option<HotelInfoResponse>>,
    pub room_result: RwSignal<Option<HotelRoomResponse>>,
    pub price_per_night: RwSignal<f64>,
    pub room_counters: RwSignal<HashMap<String, RoomCounterKeyValue>>,
    pub block_room_counters: RwSignal<HashMap<String, RoomCounterKeyValueStatic>>,
}

impl HotelInfoResults {
    fn from_leptos_context() -> Self {
        expect_context()
    }

    pub fn reset() {
        Self::from_leptos_context().search_result.set(None);
        Self::from_leptos_context().room_result.set(None);
    }

    pub fn set_info_results(hotel_info_response: Option<HotelInfoResponse>) {
        Self::from_leptos_context()
            .search_result
            .set(hotel_info_response);
    }

    pub fn set_room_results(hotel_room_response: Option<HotelRoomResponse>) {
        Self::from_leptos_context()
            .room_result
            .set(hotel_room_response);
    }

    // pub fn get_hotel_room_details(&self) -> Option<Vec<HotelRoomDetail>> {
    //     // Assuming room_result is of type Option<HotelRoomResponse>
    //     if let Some(hotel_room_response) = self.room_result.get() {
    //         // Assuming hotel_room_response has a method that returns Vec<HotelRoomDetail>
    //         hotel_room_response.get_hotel_room_details() // This should return Vec<HotelRoomDetail>
    //     } else {
    //         None
    //     }
    // }
    pub fn get_hotel_room_details(&self) -> Option<Vec<HotelRoomDetail>> {
        self.room_result
            .get()
            .and_then(|response| response.get_hotel_room_details())
    }

    pub fn set_price_per_night(&self, per_night_calc: f64) {
        Self::from_leptos_context()
            .price_per_night
            .set(per_night_calc);
    }

    pub fn set_room_counters(&self, room_counters: HashMap<String, RoomCounterKeyValue>) {
        Self::from_leptos_context().room_counters.set(room_counters);
    }

    pub fn set_block_room_counters(&self, room_counters: HashMap<String, RoomCounterKeyValue>) {
        // log!("set_block_room_counters : input:  {room_counters:#?}");

        let new_map: HashMap<String, RoomCounterKeyValueStatic> = room_counters
            .into_iter()
            .map(|(key, value)| (key, RoomCounterKeyValueStatic::from(value)))
            .collect();

        // log!("final new_map for block_room_counters:  {new_map:#?}");

        Self::from_leptos_context().block_room_counters.set(new_map);
    }

    pub fn block_room_request(&self, uniq_room_ids: Vec<String>) -> BlockRoomRequest {
        // Get unique room IDs from the room response
        // let room_unique_id = self
        //     .room_result
        //     .get()
        //     .as_ref()
        //     .map(|response| {
        //         response
        //             .room_list
        //             .as_ref()
        //             .map(|room_list| {
        //                 room_list
        //                     .get_hotel_room_result
        //                     .hotel_rooms_details
        //                     .iter()
        //                     .map(|room| room.room_unique_id.clone())
        //                     .collect()
        //             })
        //             .unwrap_or_default()
        //     })
        //     .unwrap_or_default();

        let room_unique_id = uniq_room_ids;

        // Get token from SearchListResults context since that has the token map
        let search_list_results: SearchListResults = expect_context();
        let hotel_info_ctx: HotelInfoCtx = expect_context();

        let token = hotel_info_ctx
            .hotel_code
            .get()
            .and_then(|hotel_code| {
                let token_map = search_list_results.get_hotel_code_results_token_map();
                token_map.get(&hotel_code).cloned()
            })
            .unwrap_or_default();

        BlockRoomRequest {
            token,
            room_unique_id,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct BlockRoomResults {
    pub block_room_results: RwSignal<Option<BlockRoomResponse>>,
}

impl BlockRoomResults {
    fn from_leptos_context() -> Self {
        expect_context()
    }

    pub fn reset() {
        Self::from_leptos_context().block_room_results.set(None);
    }

    pub fn set_results(results: Option<BlockRoomResponse>) {
        Self::from_leptos_context().block_room_results.set(results);
    }

    pub fn book_room_request(&self) -> BookRoomRequest {
        let search_list_results: SearchListResults = expect_context();
        let hotel_info_ctx: HotelInfoCtx = expect_context();

        let result_token = hotel_info_ctx
            .hotel_code
            .get()
            .and_then(|hotel_code| {
                let token_map = search_list_results.get_hotel_code_results_token_map();
                token_map.get(&hotel_code).cloned()
            })
            .unwrap_or_default();

        let block_room_id = self.block_room_results.get_untracked().unwrap().block_room.block_room_result.block_room_id;

        let hotel_code = hotel_info_ctx.hotel_code.get().unwrap_or_default();
        let app_reference = format!("BOOKING_{}_{}", chrono::Utc::now().timestamp(), hotel_code);

        let confirmation: ConfirmationResults = expect_context();
        let room_details = confirmation.room_details.get_untracked().unwrap();

        BookRoomRequest {
            result_token,
            block_room_id,
            app_reference,
            room_details: vec![room_details],
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ConfirmationResults {
    pub booking_details: RwSignal<Option<BookRoomResponse>>,
    pub room_details: RwSignal<Option<RoomDetail>>,
}

impl ConfirmationResults {
    fn from_leptos_context() -> Self {
        expect_context()
    }

    pub fn reset() {
        Self::from_leptos_context().booking_details.set(None);
        Self::from_leptos_context().room_details.set(None);
    }

    pub fn set_booking_details(booking_response: Option<BookRoomResponse>) {
        Self::from_leptos_context()
            .booking_details
            .set(booking_response);
    }

    pub fn set_room_details(room_detail: Option<RoomDetail>) {
        Self::from_leptos_context()
        .room_details
        .set(room_detail);
    }
}
