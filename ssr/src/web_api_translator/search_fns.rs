cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {
        // use crate::adapters::ProvabAdapter;
    }
}
// use crate::application_services::HotelService;
use crate::component::SelectedDateRange;
use crate::component::{Destination, GuestSelection};
use crate::domain::{DomainHotelListAfterSearch, DomainRoomGuest};
use crate::{
    domain::DomainHotelSearchCriteria,
    view_state_layer::ui_search_state::{UIPaginationState, UISearchCtx},
};
use crate::{log, utils};
use leptos::prelude::*;

impl From<UISearchCtx> for DomainHotelSearchCriteria {
    fn from(ctx: UISearchCtx) -> Self {
        let check_in_date = ctx.date_range.get_untracked().start;
        let check_out_date = ctx.date_range.get_untracked().end;
        let no_of_nights = ctx.date_range.get_untracked().no_of_nights();

        // todo (better hanlding)
        let place_id = ctx
            .place
            .get_untracked()
            .map(|f| f.place_id)
            .unwrap_or_default();
        let destination = ctx.place_details.get_untracked().unwrap_or_default();

        // Get pagination parameters from UI state if available
        let pagination = UIPaginationState::get_pagination_params();

        // Debug logging for pagination parameters
        // log!(
        //     "[PAGINATION-DEBUG] 🎯 Frontend Pagination Debug: pagination_params={:?}",
        //     pagination
        // );

        let request = DomainHotelSearchCriteria {
            check_in_date,
            check_out_date,
            no_of_nights,
            no_of_rooms: ctx.guests.rooms.get_untracked(),
            room_guests: vec![ctx.guests.into()],
            // destination_city_id: destination.city_id.parse().unwrap_or_default(),
            // destination_city_name: destination.city,
            // destination_country_code: destination.country_code,
            // destination_country_name: destination.country_name,
            guest_nationality: "US".into(), // Hardcoded for now, can be dynamic based on user profile or selection
            // destination_latitude: Some(destination.location.latitude),
            // destination_longitude: Some(destination.location.longitude),
            place_id,
            pagination,
            // ..Default::default()
        };

        log!("HotelSearchRequest: {request:?}");

        request
    }
}

impl From<GuestSelection> for DomainRoomGuest {
    fn from(guest_selection: GuestSelection) -> Self {
        let ages_u32: Vec<u32> = guest_selection.children_ages.get_untracked();
        let children_ages_converted: Option<Vec<String>> = if ages_u32.is_empty() {
            None
        } else {
            Some(
                ages_u32
                    .into_iter()
                    .map(|age| age.to_string())
                    .collect::<Vec<String>>(),
            )
        };

        DomainRoomGuest {
            no_of_adults: guest_selection.adults.get_untracked(),
            no_of_children: guest_selection.children.get_untracked(),
            children_ages: children_ages_converted,
        }
    }
}

// #[server(SearchHotel)]
// pub async fn search_hotel(
//     request: DomainHotelSearchCriteria,
// ) -> Result<DomainHotelListAfterSearch, ServerFnError> {
//     let hotel_service: HotelService<ProvabAdapter> = expect_context();
//     hotel_service.search_hotels(request).await.map_err(|e| {
//         log!("server_fn_error: SEARCH_HOTEL_API - {}", e.to_string());
//         ServerFnError::ServerError(e.to_string())
//     })
// }
