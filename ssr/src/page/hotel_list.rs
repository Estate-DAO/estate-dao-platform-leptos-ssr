use leptos::*;
use leptos_router::use_navigate;

// use crate::api::get_room;
use crate::api::client_side_api::ClientSideApiClient;
use crate::component::{Destination, GuestSelection, Navbar, SkeletonCards};
use crate::log;
use crate::page::{HotelDetailsParams, HotelListParams, InputGroupContainer};
use crate::utils::query_params::QueryParamsSync;
use crate::view_state_layer::input_group_state::{InputGroupState, OpenDialogComponent};
use crate::view_state_layer::ui_hotel_details::HotelDetailsUIState;
use crate::view_state_layer::ui_search_state::{SearchListResults, UISearchCtx};
use crate::view_state_layer::view_state::HotelInfoCtx;
use crate::view_state_layer::GlobalStateForLeptos;
// use crate::state::input_group_state::{InputGroupState, OpenDialogComponent};
// use crate::state::search_state::HotelInfoResults;
use crate::{
    // api::hotel_info,
    app::AppRoutes,
    component::SelectedDateRange,
    component::{FilterAndSortBy, PriceDisplay, StarRating},
    page::InputGroup,
    // state::{search_state::SearchListResults, view_state::HotelInfoCtx},
};

//  this is only for this page to track if the bar changes.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PreviousSearchContext {
    pub destination: Option<Destination>,
    pub date_range: Option<SelectedDateRange>,
    pub adults: u32,
    pub children: u32,
    pub rooms: u32,
    /// false by default
    pub first_time_filled: bool,
}

impl GlobalStateForLeptos for PreviousSearchContext {}

impl PreviousSearchContext {
    pub fn update(new_ctx: UISearchCtx) {
        let mut this: Self = expect_context();
        // let mut this = Self::get();
        this.destination = new_ctx.destination.get_untracked();
        this.rooms = new_ctx.guests.rooms.get_untracked();
        this.children = new_ctx.guests.children.get_untracked();
        this.adults = new_ctx.guests.adults.get_untracked();
        log!("[PreviousSearchContext] updated: {:?}", this);

        provide_context(this);
    }

    pub fn update_first_time_filled(new_ctx: UISearchCtx) {
        let mut this: Self = expect_context();
        Self::update(new_ctx);
        this.first_time_filled = true;
        provide_context(this);
    }

    pub fn reset_first_time_filled() {
        let mut this: Self = expect_context();
        this.first_time_filled = false;
        provide_context(this);
    }
}

//

#[component]
pub fn HotelListPage() -> impl IntoView {
    let search_ctx: UISearchCtx = expect_context();
    let navigate = use_navigate();
    let query_map = leptos_router::use_query_map();

    let search_ctx2: UISearchCtx = expect_context();

    // Sync query params with state on page load (URL → State)
    // This leverages use_query_map's built-in reactivity for browser navigation
    create_effect(move |_| {
        let params = query_map.get();
        if !params.0.is_empty() {
            // log!("Found query params in URL: {:?}", params);

            if let Some(hotel_params) =
                HotelListParams::from_url_params(&params.0.into_iter().collect())
            {
                // log!("Parsed hotel params from URL: {:?}", hotel_params);
                hotel_params.sync_to_app_state();
                PreviousSearchContext::update_first_time_filled(search_ctx2.clone());
            }
        }
    });

    // Clone search_ctx for use in different closures
    let search_ctx_for_resource = search_ctx.clone();
    let search_ctx_for_url_update = search_ctx.clone();

    // Hotel search resource - triggers when search context is available
    let hotel_search_resource = create_resource(
        move || {
            // Track search context changes reactively
            let destination = search_ctx_for_resource.destination.get();
            let date_range = search_ctx_for_resource.date_range.get();
            let adults = search_ctx_for_resource.guests.adults.get();
            let children = search_ctx_for_resource.guests.children.get();
            let rooms = search_ctx_for_resource.guests.rooms.get();

            // log!("[hotel_search_resource] destination: {:?}", destination);
            // log!("[hotel_search_resource] date_range: {:?}", date_range);
            // log!("[hotel_search_resource] adults: {:?}", adults);
            // log!("[hotel_search_resource] children: {:?}", children);
            // log!("[hotel_search_resource] rooms: {:?}", rooms);

            // Get fresh context each time (this makes it reactive to context changes)
            let previous_search_ctx = expect_context::<PreviousSearchContext>();

            // log!("[hotel_search_resource] previous_search_ctx: {:?}", previous_search_ctx);

            let previous_destination = previous_search_ctx.destination.clone();
            let previous_adults = previous_search_ctx.adults;
            let previous_children = previous_search_ctx.children;
            let previous_rooms = previous_search_ctx.rooms;

            let is_same_destination = destination == previous_destination;
            let is_same_adults = adults == previous_adults;
            let is_same_children = children == previous_children;
            let is_same_rooms = rooms == previous_rooms;

            let has_valid_dates = date_range.start != (0, 0, 0) && date_range.end != (0, 0, 0);
            let has_valid_search_data = destination.is_some() && adults > 0 && rooms > 0;
            let is_first_load =
                previous_destination.is_none() && previous_adults == 0 && previous_rooms == 0;

            // Return true when ready to search
            let is_ready = has_valid_dates
                && has_valid_search_data
                && (is_first_load || // First load with valid data - always search
                 (is_same_destination && is_same_adults && is_same_children && is_same_rooms)); // Subsequent loads - only if same data

            log!(
                "[hotel_search_resource] readiness: is_same_destination={}, is_same_adults={}, is_same_children={}, is_same_rooms={}, has_valid_dates={}, has_valid_search_data={}, is_first_load={}, ready={}",
                is_same_destination,
                is_same_adults,
                is_same_children,
                is_same_rooms,
                has_valid_dates,
                has_valid_search_data,
                is_first_load,
                is_ready
            );

            is_ready
        },
        move |is_ready| {
            let search_ctx_clone = search_ctx_for_resource.clone();
            let search_ctx_clone2 = search_ctx_for_resource.clone();
            async move {
                if !is_ready {
                    log!("[hotel_search_resource] Not ready yet, waiting for search criteria...");
                    return None;
                }

                log!("[hotel_search_resource] Search criteria ready, performing hotel search...");

                // Use the same API client as root.rs
                let api_client = ClientSideApiClient::new();
                let result = api_client.search_hotel(search_ctx_clone.into()).await;

                log!("[hotel_search_resource] Hotel search API completed");

                // Set results in the same way as root.rs
                SearchListResults::set_search_results(result.clone());
                PreviousSearchContext::update(search_ctx_clone2.clone());

                // Reset first_time_filled flag after successful search
                PreviousSearchContext::reset_first_time_filled();

                Some(result)
            }
        },
    );

    // Example: Manual URL updates (State → URL) when user performs actions
    // This function can be called from search form submissions, filter changes, etc.
    let update_url_with_current_state = move || {
        let current_params = HotelListParams::from_search_context(&search_ctx_for_url_update);
        current_params.update_url();
        log!(
            "Updated URL with current search state: {:?}",
            current_params
        );
    };

    // Example usage - this could be called from:
    // - Search form submission: update_url_with_current_state();
    // - Filter changes: update_url_with_current_state();
    // - Sorting changes: update_url_with_current_state();

    // ensure that context is clear. no pending signals
    // todo (uncomment)
    // HotelInfoResults::reset();
    let search_list_page: SearchListResults = expect_context();

    let disabled_input_group: Signal<bool> = Signal::derive(move || {
        let val = search_list_page.search_result.get().is_none();
        // let val = search_list_page.search_result.get().is_some();
        // log!("disabled ig - {}", val);
        // log!(
        //     "search_list_page.search_result.get(): {:?}",
        //     search_list_page.search_result.get()
        // );
        val
    });

    let fallback = move || {
        (1..10)
            .map(|_| {
                view! {
                    <SkeletonCards />
                }
            })
            .collect_view()
    };

    view! {
        <section class="relative h-screen">
            <Navbar />
            <div class="w-full max-w-xl sm:max-w-4xl mx-auto">
                <div class="flex flex-col items-center mt-2 sm:mt-6 p-2 sm:p-4">
                    <InputGroupContainer default_expanded=false given_disabled=disabled_input_group allow_outside_click_collapse=true />
                    // <FilterAndSortBy />
                </div>

                // Use resource pattern with Suspense for automatic loading states
                <Suspense fallback=move || view! { <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3 sm:gap-4">{fallback()}</div> }>
                    {move || {
                        // Trigger the resource loading but don't render anything
                        let _ = hotel_search_resource.get();
                        view! { <></> }
                    }}
                </Suspense>

                <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3 sm:gap-4">

                    <Show
                        when=move || search_list_page.search_result.get().is_some()
                        fallback=move || view! { <></> }
                    >

                        {move || {
                            let hotel_results = search_list_page
                                .search_result
                                .get()
                                .unwrap()
                                .hotel_list();
                            if hotel_results.is_empty() {

                                view! {
                                    <div class="flex flex-col items-center justify-center mt-4 sm:mt-6 p-2 sm:p-4 col-span-full min-h-[200px]">
                                        <p class="text-center">
                                            No hotels found for your search criteria.
                                        </p>
                                    </div>
                                }
                                    .into_view()
                            } else {
                                hotel_results
                                    .iter()
                                    .map(|hotel_result| {
                                        view! {
                                            <HotelCard
                                                img=hotel_result.hotel_picture.clone()
                                                rating=hotel_result.star_rating
                                                hotel_name=hotel_result.hotel_name.clone()
                                                price=hotel_result.price.room_price
                                                hotel_code=hotel_result.hotel_code.clone()
                                                class="w-full max-w-xs mx-auto px-2 sm:px-0".to_string()
                                            />
                                        }
                                    })
                                    .collect_view()
                            }
                        }}
                    </Show>
                </div>
            </div>
        </section>
    }
}

#[component]
pub fn HotelCard(
    img: String,
    rating: u8,
    price: f64,
    hotel_code: String,
    hotel_name: String,
    class: String,
) -> impl IntoView {
    let price = create_rw_signal(price);

    let search_list_page: SearchListResults = expect_context();
    let search_list_page_clone = search_list_page.clone();
    let search_ctx: UISearchCtx = expect_context();

    let navigate = use_navigate();
    let navigate_clone = navigate.clone();

    let hotel_code_cloned = hotel_code.clone();

    let search_hotel_info_action = create_action(move |_| {
        let nav = navigate_clone.clone();
        let search_list_page = search_list_page.clone();
        let hotel_code = hotel_code.clone();
        log!("from action -- {search_list_page:?}");
        log!("from action -- {hotel_code:?}");
        async move {
            //  move to the hotel info page
            nav(AppRoutes::HotelDetails.to_string(), Default::default());

            // todo (uncomment)
            // HotelInfoResults::reset();

            // Get hotel info request
            // todo (uncomment)
            // let hotel_info_request = search_list_page.hotel_info_request(&hotel_code);
            // log!("{hotel_info_request:?}");

            // Call server function inside action
            spawn_local(async move {
                // todo (uncomment)
                // let result = hotel_info(hotel_info_request).await.ok();
                // log!("SEARCH_HOTEL_API: {result:?}");
                // HotelInfoResults::set_info_results(result);

                // Navigate after data is loaded to ensure clean state transition
                nav(AppRoutes::HotelDetails.to_string(), Default::default());

                // close all the dialogs
                InputGroupState::toggle_dialog(OpenDialogComponent::None);
            });
        }
    });

    let hotel_code_2_cloned = hotel_code_cloned.clone();
    // let search_hotel_room_action = create_action(move |_: &()| {
    //     let search_list_page = search_list_page_clone.clone();
    //     let hotel_code = hotel_code_2_cloned.clone();
    //     async move {
    //         // let hotel_room_request = search_list_page.hotel_room_request(&hotel_code);
    //         // call server function inside action
    //         spawn_local(async move {
    //             // todo (uncomment)
    //             // let result = get_room(hotel_room_request).await.ok();
    //             // log!("SEARCH_ROOM_API: {result:?}");
    //             // HotelInfoResults::set_room_results(result);
    //         });
    //     }
    // });

    let hotel_view_info_ctx: HotelInfoCtx = expect_context();

    view! {
        <div // href=AppRoutes::HotelDetails.to_string()
        on:click=move |ev| {
            ev.prevent_default();
            ev.stop_propagation();

            // Set hotel code in context for hotel details page
            hotel_view_info_ctx.hotel_code.set(hotel_code_cloned.clone());
            log!("hotel_code: {}", hotel_code_cloned);
            log!("hotel_code from hotel_view_info_ctx: {}", hotel_view_info_ctx.hotel_code.get());

            // Reset hotel details state
            HotelDetailsUIState::reset();

            // Generate query params for shareable URL
            if let Some(hotel_params) = HotelDetailsParams::from_current_context() {
                log!("Generated hotel details params: {:?}", hotel_params);
                let shareable_url = hotel_params.to_shareable_url();
                log!("Navigating to hotel details with query params: {}", shareable_url);
                navigate(&shareable_url, Default::default());
            } else {
                log!("Failed to generate hotel params, using fallback navigation");
                // Fallback to original navigation
                search_hotel_info_action.dispatch(());
            }
        }>
            <div class={class}>
                <div class="w-full sm:w-72 max-w-full sm:max-w-xs rounded-lg overflow-hidden shadow-sm border border-gray-300 bg-white">
                    <img class="w-full h-40 sm:h-64 object-cover" src=img alt=hotel_name.clone() />

                    <div class="h-24">
                        <div class="flex items-center justify-between px-3 sm:px-6 pt-2 sm:pt-4">
                            <p class="text-sm sm:text-base font-medium">
                                {if hotel_name.len() > 10 {
                                    format!("{}...", hotel_name.chars().take(10).collect::<String>())
                                } else {
                                    hotel_name.clone()
                                }}
                            </p>
                            <StarRating rating=move || rating />
                        </div>

                        <div class="flex items-center justify-between px-3 sm:px-6 pt-1 sm:pt-2">
                            <PriceDisplay price=price />
                            <button class="font-semibold underline underline-offset-2 decoration-solid text-xs sm:text-sm">
                                "View details"
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
