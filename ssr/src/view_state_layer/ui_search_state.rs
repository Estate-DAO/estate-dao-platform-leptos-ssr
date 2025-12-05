use crate::{
    // api::{
    //     payments::ports::GetPaymentStatusResponse, BlockRoomRequest, BlockRoomResponse,
    //     BookRoomRequest, BookRoomResponse, HotelInfoRequest, HotelInfoResponse, HotelRoomDetail,
    //     HotelRoomRequest, HotelRoomResponse, HotelSearchRequest, HotelSearchResponse, RoomDetail,
    // },
    api::{
        client_side_api::{Place, PlaceData},
        consts::PAGINATION_LIMIT,
    },
    application_services::filter_types::{UISearchFilters, UISortOptions},
    canister::backend,
    component::{Destination, GuestSelection, SelectedDateRange},
    domain::{DomainHotelListAfterSearch, DomainPaginationMeta, DomainPaginationParams},
    utils::app_reference::generate_app_reference,
};
// use leptos::logging::log;
use crate::log;
use leptos::RwSignal;
use leptos::*;
use std::collections::HashMap;

use super::{view_state::HotelInfoCtx, GlobalStateForLeptos};

//  ==================================================================

#[derive(Clone, Debug)]
pub struct UISearchCtx {
    pub place: RwSignal<Option<Place>>,
    pub place_details: RwSignal<Option<PlaceData>>,
    pub destination: RwSignal<Option<Destination>>,
    pub date_range: RwSignal<SelectedDateRange>,
    pub guests: GuestSelection,
    pub filters: RwSignal<UISearchFilters>,
    pub sort_options: RwSignal<UISortOptions>,
}

impl Default for UISearchCtx {
    fn default() -> Self {
        Self {
            place: RwSignal::new(None),
            place_details: RwSignal::new(None),
            destination: RwSignal::new(None),
            date_range: RwSignal::new(SelectedDateRange::default()),
            guests: GuestSelection::default(),
            filters: RwSignal::new(UISearchFilters::default()),
            sort_options: RwSignal::new(UISortOptions::default_sort()),
        }
    }
}

impl UISearchCtx {
    pub fn set_destination(destination: Destination) {
        let this: Self = expect_context();

        this.destination.set(Some(destination));
    }

    pub fn set_place(place: Place) {
        let this: Self = expect_context();
        this.place.set(Some(place));
    }

    pub fn set_place_details(place_details: Option<PlaceData>) {
        let this: Self = expect_context();
        if this.place.get_untracked().is_none() {
            log::warn!("UISearchCtx::set_place_details called but place is None. This may indicate inconsistent state.");
            return;
        }
        this.place_details.set(place_details);
    }

    pub fn set_date_range(date_range: SelectedDateRange) {
        let this: Self = expect_context();

        this.date_range.set(date_range);
    }

    pub fn set_guests(guests: GuestSelection) {
        let this: Self = expect_context();

        batch(|| {
            this.guests.adults.set(guests.adults.get_untracked());
            this.guests.children.set(guests.children.get_untracked());
            this.guests.rooms.set(guests.rooms.get_untracked());
            this.guests
                .children_ages
                .set_vec(guests.children_ages.get_untracked());
        });
    }

    pub fn set_filters(filters: UISearchFilters) {
        let this: Self = expect_context();
        this.filters.set(filters);
    }

    pub fn update_filters(f: impl FnOnce(&mut UISearchFilters)) {
        let this: Self = expect_context();
        this.filters.update(|filters| f(filters));
    }

    pub fn set_sort_options(sort_options: UISortOptions) {
        let this: Self = expect_context();
        this.sort_options.set(sort_options);
    }

    pub fn update_sort_options(f: impl FnOnce(&mut UISortOptions)) {
        let this: Self = expect_context();
        this.sort_options.update(|sort_options| f(sort_options));
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
        let search_result_signal = Self::from_leptos_context().search_result;

        if let Some(new_response) = hotel_search_response {
            search_result_signal.update(|current_result| {
                if let Some(current) = current_result {
                    current.hotel_results.extend(new_response.hotel_results);
                    current.pagination = new_response.pagination;
                } else {
                    *current_result = Some(new_response);
                }
            });
        } else {
            search_result_signal.set(None);
        }
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

    pub fn get_pagination_meta() -> Option<DomainPaginationMeta> {
        Self::from_leptos_context()
            .search_result
            .get_untracked()
            .as_ref()
            .and_then(|response| response.pagination.clone())
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

//  ==================================================================

#[derive(Debug, Clone)]
pub struct UIPaginationState {
    pub current_page: RwSignal<u32>,
    pub page_size: RwSignal<u32>,
    pub pagination_meta: RwSignal<Option<DomainPaginationMeta>>,
}

impl GlobalStateForLeptos for UIPaginationState {}

impl UIPaginationState {
    pub fn get_pagination_params() -> Option<DomainPaginationParams> {
        let this: Self = expect_context();
        let current_page = this.current_page.get_untracked();
        let page_size = this.page_size.get_untracked();

        // Always return pagination params to ensure frontend controls pagination
        Some(DomainPaginationParams {
            page: Some(current_page),
            page_size: Some(page_size),
        })
    }

    pub fn set_current_page(page: u32) {
        let this: Self = expect_context();
        this.current_page.set(page.max(1));
    }

    pub fn set_page_size(size: u32) {
        let this: Self = expect_context();
        this.page_size.set(size.clamp(1, PAGINATION_LIMIT as u32)); // Enforce reasonable limits
    }

    pub fn set_pagination_meta(meta: Option<DomainPaginationMeta>) {
        let this: Self = expect_context();

        // Debug logging for pagination metadata setting
        // crate::log!(
        //     "[PAGINATION-DEBUG] 🔧 UIPaginationState::set_pagination_meta called with: {:?}",
        //     meta
        // );

        this.pagination_meta.set(meta);
    }

    pub fn go_to_next_page() {
        let this: Self = expect_context();
        let current = this.current_page.get_untracked();
        let meta = this.pagination_meta.get_untracked();

        // crate::log!("[PAGINATION-DEBUG] 🔄 go_to_next_page: current={}, meta={:?}", current, meta);

        if let Some(pagination_meta) = meta {
            if pagination_meta.has_next_page {
                // crate::log!("[PAGINATION-DEBUG] 🔄 Setting current_page from {} to {}", current, current + 1);
                this.current_page.set(current + 1);
                // crate::log!("[PAGINATION-DEBUG] 🔄 Current page updated to: {}", this.current_page.get_untracked());
            } else {
                // crate::log!("[PAGINATION-DEBUG] 🔄 No next page available (has_next_page=false)");
            }
        } else {
            // crate::log!("[PAGINATION-DEBUG] 🔄 No pagination meta available");
        }
    }

    pub fn go_to_previous_page() {
        let this: Self = expect_context();
        let current = this.current_page.get_untracked();
        let meta = this.pagination_meta.get_untracked();

        if let Some(pagination_meta) = meta {
            if pagination_meta.has_previous_page && current > 1 {
                this.current_page.set(current - 1);
            }
        }
    }

    pub fn reset_to_first_page() {
        let this: Self = expect_context();
        this.current_page.set(1);
        this.pagination_meta.set(None);
    }

    // Button state methods following the established pattern
    pub fn is_previous_button_disabled() -> bool {
        let this: Self = expect_context();
        let meta_option = this.pagination_meta.get(); // Make reactive!
        let is_disabled = meta_option
            .as_ref()
            .is_none_or(|meta| !meta.has_previous_page);

        // Debug logging for button states
        // crate::log!(
        //     "[PAGINATION-DEBUG] 🔘 Previous Button Debug: pagination_meta={:?}, disabled={}",
        //     meta_option, is_disabled
        // );

        is_disabled
    }

    pub fn is_next_button_disabled() -> bool {
        let this: Self = expect_context();
        let meta_option = this.pagination_meta.get(); // Make reactive!
        let is_disabled = meta_option.as_ref().is_none_or(|meta| !meta.has_next_page);

        // Debug logging for button states
        // crate::log!(
        //     "[PAGINATION-DEBUG] 🔘 Next Button Debug: pagination_meta={:?}, disabled={}",
        //     meta_option, is_disabled
        // );

        is_disabled
    }
}

impl Default for UIPaginationState {
    fn default() -> Self {
        Self {
            current_page: create_rw_signal(1),
            page_size: create_rw_signal(PAGINATION_LIMIT as u32), // Default page size
            pagination_meta: create_rw_signal(None),
        }
    }
}
