use crate::{
    // api::{
    //     payments::ports::GetPaymentStatusResponse, BlockRoomRequest, BlockRoomResponse,
    //     BookRoomRequest, BookRoomResponse, HotelInfoRequest, HotelInfoResponse, HotelRoomDetail,
    //     HotelRoomRequest, HotelRoomResponse, HotelSearchRequest, HotelSearchResponse, RoomDetail,
    // },
    canister::backend,
    component::{Destination, GuestSelection, SelectedDateRange},
    domain::DomainHotelListAfterSearch,
    utils::app_reference::generate_app_reference,
};
// use leptos::logging::log;
use crate::log;
use leptos::RwSignal;
use leptos::*;
use std::collections::HashMap;

use super::{view_state::HotelInfoCtx, GlobalStateForLeptos};

//  ==================================================================

#[derive(Clone, Default, Debug)]
pub struct UISearchCtx {
    // invalid_cnt: RwSignal<u32>,
    pub destination: RwSignal<Option<Destination>>,
    pub date_range: RwSignal<SelectedDateRange>,
    pub guests: GuestSelection,
    // pub on_form_reset: Trigger,
}

impl UISearchCtx {
    pub fn set_destination(destination: Destination) {
        let this: Self = expect_context();

        this.destination.set(Some(destination));
    }

    pub fn set_date_range(date_range: SelectedDateRange) {
        let this: Self = expect_context();

        this.date_range.set(date_range);
    }

    pub fn set_guests(guests: GuestSelection) {
        let this: Self = expect_context();

        this.guests.adults.set(guests.adults.get_untracked());
        this.guests.children.set(guests.children.get_untracked());
        this.guests.rooms.set(guests.rooms.get_untracked());
        this.guests
            .children_ages
            .set(guests.children_ages.get_untracked());
    }

    pub fn log_state() {
        let this: Self = expect_context();

        log::info!(
            "\n\nguests.adults: {:?}",
            this.guests.adults.get_untracked()
        );
        log::info!(
            "guests.children: {:?}",
            this.guests.children.get_untracked()
        );
        log::info!(
            "guests.children_ages: {:?}",
            this.guests.children_ages.get_untracked()
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

    pub fn get_backend_compatible_date_range_untracked() -> backend::SelectedDateRange {
        let this: Self = expect_context();
        this.date_range.get_untracked().into()
    }

    pub fn get_backend_compatible_destination_untracked() -> Option<backend::Destination> {
        let this: Self = expect_context();
        this.destination.get_untracked().map(|dest| dest.into())
    }
}

//  ==================================================================

#[derive(Debug, Clone, Default)]
pub struct SearchListResults {
    pub search_result: RwSignal<Option<DomainHotelListAfterSearch>>,
}

impl SearchListResults {
    fn from_leptos_context() -> Self {
        expect_context()
    }
    pub fn reset() {
        Self::from_leptos_context().search_result.set(None);
    }

    pub fn set_search_results(hotel_search_response: Option<DomainHotelListAfterSearch>) {
        Self::from_leptos_context()
            .search_result
            .set(hotel_search_response);
    }

    pub fn get_hotel_code_results_token_map() -> HashMap<String, String> {
        Self::from_leptos_context()
            .search_result
            .get_untracked()
            .as_ref()
            .map_or_else(HashMap::new, |response| response.get_results_token_map())
    }

    pub fn get_result_token(hotel_code: String) -> String {
        Self::get_hotel_code_results_token_map()
            .get(&hotel_code)
            .cloned()
            .unwrap_or_default()
    }

    // pub fn hotel_info_request(&self, hotel_code: &str) -> HotelInfoRequest {
    //     let token = Self::get_result_token(hotel_code.into());
    //     HotelInfoRequest { token }
    // }

    // pub fn hotel_room_request(&self, hotel_code: &str) -> HotelRoomRequest {
    //     let token = Self::get_result_token(hotel_code.into());
    //     HotelRoomRequest { token }
    // }
}
