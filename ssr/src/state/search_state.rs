use crate::{
    api::{
        HotelInfoRequest, HotelInfoResponse, HotelRoomRequest, HotelRoomResponse,
        HotelSearchRequest, HotelSearchResponse, BlockRoomResponse
    },
    component::{GuestSelection, SelectedDateRange},
};
use leptos::RwSignal;
use leptos::*;
use std::collections::HashMap;

#[derive(Clone, Default, Debug)]
pub struct SearchCtx {
    // invalid_cnt: RwSignal<u32>,
    pub destination: RwSignal<Option<String>>,
    pub date_range: RwSignal<SelectedDateRange>,
    pub guests: RwSignal<GuestSelection>,
    pub on_form_reset: Trigger,
}

impl SearchCtx {
    pub fn set_destination(destination: String) {
        let this: Self = expect_context();

        this.destination.set(Some(destination));
    }

    pub fn set_date_range(date_range: SelectedDateRange) {
        let this: Self = expect_context();

        this.date_range.set(date_range);
    }

    // pub fn set_adults(num_adult: u32) {
    //     let this: Self = expect_context();

    //     this.guests
    //         .update(|guest_selection| guest_selection.adults.set(num_adult));
    // }

    // pub fn set_children(num_children: u32) {
    //     let this: Self = expect_context();

    //     this.guests
    //         .update(|guest_selection| guest_selection.children.set(num_children));
    // }

    // /// ensure total number of children does not exceed the number of children ages
    // pub fn set_children_ages(ages: Vec<u32>) {
    //     let this: Self = expect_context();

    //     let children_count = this.guests.get_untracked().children.get();
    //     if children_count >= 1 && ages.len() <= children_count as usize {
    //         this.guests.update(|guest_selection| {
    //             guest_selection
    //                 .children_ages
    //                 .update(|existing_ages| existing_ages.extend(ages))
    //         });
    //     }
    // }

    //     pub fn set_guests(guest_selection: GuestSelection) {
    //         let this: Self = expect_context();

    //         this.guests.set(guest_selection);
    //     }

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
}

impl HotelInfoResults {
    fn from_leptos_context() -> Self {
        expect_context()
    }

    pub fn reset() {
        Self::from_leptos_context().search_result.set(None);
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
}

#[derive(Debug, Clone, Default)]
pub struct BlockRoomResults {
    pub block_room_results: RwSignal<Option<BlockRoomResponse>>,
}



