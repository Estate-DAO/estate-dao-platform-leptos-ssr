use leptos::*;
use leptos_router::use_navigate;

// use crate::api::get_room;
use crate::api::client_side_api::{ClientSideApiClient, Place, PlaceData};
use crate::component::{
    Destination, GuestSelection, Navbar, PaginationControls, PaginationInfo, SkeletonCards,
};
use crate::log;
use crate::page::{HotelDetailsParams, HotelListParams, InputGroupContainer};
use crate::utils::query_params::QueryParamsSync;
use crate::view_state_layer::input_group_state::{InputGroupState, OpenDialogComponent};
use crate::view_state_layer::ui_hotel_details::HotelDetailsUIState;
use crate::view_state_layer::ui_search_state::{SearchListResults, UIPaginationState, UISearchCtx};
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
    pub place: Option<Place>,
    pub place_details: Option<PlaceData>,
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
        this.place = new_ctx.place.get_untracked();
        this.place_details = new_ctx.place_details.get_untracked();
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

    // Initialize pagination state
    let pagination_state: UIPaginationState = expect_context();

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

    // Hotel search resource - triggers when search context or pagination changes
    let hotel_search_resource = create_resource(
        move || {
            // Track search context changes reactively
            let place = search_ctx_for_resource.place.get();
            let date_range = search_ctx_for_resource.date_range.get();
            let adults = search_ctx_for_resource.guests.adults.get();
            let children = search_ctx_for_resource.guests.children.get();
            let rooms = search_ctx_for_resource.guests.rooms.get();

            // Track pagination changes reactively
            let current_page = pagination_state.current_page.get();
            let page_size = pagination_state.page_size.get();

            // log!("[PAGINATION-DEBUG] [hotel_search_resource] current_page: {}, page_size: {}", current_page, page_size);
            // log!("[PAGINATION-DEBUG] [hotel_search_resource] destination: {:?}", destination);
            // log!("[PAGINATION-DEBUG] [hotel_search_resource] date_range: {:?}", date_range);
            // log!("[PAGINATION-DEBUG] [hotel_search_resource] adults: {:?}", adults);
            // log!("[PAGINATION-DEBUG] [hotel_search_resource] children: {:?}", children);
            // log!("[PAGINATION-DEBUG] [hotel_search_resource] rooms: {:?}", rooms);

            // Get fresh context each time (this makes it reactive to context changes)
            let previous_search_ctx = expect_context::<PreviousSearchContext>();

            // log!("[hotel_search_resource] previous_search_ctx: {:?}", previous_search_ctx);

            let previous_place = previous_search_ctx.place.clone();
            let previous_adults = previous_search_ctx.adults;
            let previous_children = previous_search_ctx.children;
            let previous_rooms = previous_search_ctx.rooms;

            let is_same_place = place == previous_place;
            let is_same_adults = adults == previous_adults;
            let is_same_children = children == previous_children;
            let is_same_rooms = rooms == previous_rooms;
            let is_same_search_criteria =
                is_same_place && is_same_adults && is_same_children && is_same_rooms;

            let has_valid_dates = date_range.start != (0, 0, 0) && date_range.end != (0, 0, 0);
            let has_valid_search_data = place.is_some() && adults > 0 && rooms > 0;
            let is_first_load =
                previous_place.is_none() && previous_adults == 0 && previous_rooms == 0;

            // Reset pagination to first page when search criteria change
            if !is_same_search_criteria && !is_first_load {
                UIPaginationState::reset_to_first_page();
            }

            // Return true when ready to search
            let is_ready = has_valid_dates
                && has_valid_search_data
                && (is_first_load || // First load with valid data - always search
                    is_same_search_criteria); // Always search for same criteria (includes pagination changes)

            // log!(
            //     "[PAGINATION-DEBUG] [hotel_search_resource] readiness: current_page={}, is_same_destination={}, is_same_adults={}, is_same_children={}, is_same_rooms={}, is_same_search_criteria={}, has_valid_dates={}, has_valid_search_data={}, is_first_load={}, ready={}",
            //     current_page,
            //     is_same_destination,
            //     is_same_adults,
            //     is_same_children,
            //     is_same_rooms,
            //     is_same_search_criteria,
            //     has_valid_dates,
            //     has_valid_search_data,
            //     is_first_load,
            //     is_ready
            // );

            // Return a tuple that changes when pagination changes, not just a boolean
            // This ensures the resource re-runs when pagination state changes
            if is_ready {
                (true, current_page, page_size)
            } else {
                (false, 0, 0)
            }
        },
        move |(is_ready, current_page, page_size)| {
            let search_ctx_clone = search_ctx_for_resource.clone();
            let search_ctx_clone2 = search_ctx_for_resource.clone();
            async move {
                log!("[PAGINATION-DEBUG] [hotel_search_resource] Async block called with is_ready={}, current_page={}, page_size={}", is_ready, current_page, page_size);

                if !is_ready {
                    log!("[PAGINATION-DEBUG] [hotel_search_resource] Not ready yet, waiting for search criteria...");
                    return None;
                }

                log!("[PAGINATION-DEBUG] [hotel_search_resource] Search criteria ready, performing hotel search...");

                // Use the same API client as root.rs
                let api_client = ClientSideApiClient::new();
                let result = api_client.search_hotel(search_ctx_clone.into()).await;

                log!("[PAGINATION-DEBUG] [hotel_search_resource] Hotel search API completed");

                // Set results in the same way as root.rs
                SearchListResults::set_search_results(result.clone());
                PreviousSearchContext::update(search_ctx_clone2.clone());

                // Update pagination metadata from search results
                if let Some(ref response) = result {
                    log!(
                        "🔄 Setting Pagination Metadata: pagination={:?}",
                        response.pagination
                    );
                    UIPaginationState::set_pagination_meta(response.pagination.clone());
                } else {
                    log!("⚠️ No search result to extract pagination metadata from");
                }

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
                            let mut sorted_hotels = hotel_results.clone();

                            sorted_hotels.sort_by_key(|hotel_result| {
                                match hotel_result.price.as_ref().map(|p| p.room_price) {
                                    Some(price) if price > 0.0 => 0, // valid prices come first
                                    _ => 1, // None or 0.0 go to the end
                                }
                            });

                            if hotel_results.is_empty() {
                                let current_page = pagination_state.current_page.get();

                                if current_page > 1 {
                                    // Show "Go to first page" button when on page > 1 with no results
                                    view! {
                                        <div class="flex flex-col items-center justify-center mt-4 sm:mt-6 p-2 sm:p-4 col-span-full min-h-[200px]">
                                            <p class="text-center mb-4 text-gray-600">
                                                No hotels found on page {current_page}.
                                            </p>
                                            <button
                                                class="bg-blue-500 hover:bg-blue-600 text-white font-medium py-2 px-4 rounded-lg transition-colors"
                                                on:click=move |_| {
                                                    UIPaginationState::set_current_page(1);
                                                }
                                            >
                                                Go to First Page
                                            </button>
                                        </div>
                                    }
                                } else {
                                    // Show regular "No hotels found" message on page 1
                                    view! {
                                        <div class="flex flex-col items-center justify-center mt-4 sm:mt-6 p-2 sm:p-4 col-span-full min-h-[200px]">
                                            <p class="text-center">
                                                No hotels found for your search criteria.
                                            </p>
                                        </div>
                                    }
                                }
                                    .into_view()
                            } else {
                                sorted_hotels
                                    .iter()
                                    .map(|hotel_result| {
                                        let mut price = hotel_result.price.clone().map(|p| p.room_price);
                                        let is_disabled = price.unwrap_or(0.0) <= 0.0;
                                        if is_disabled {
                                            price = None; // Hide price if invalid
                                        }
                                        let img = if hotel_result.hotel_picture.is_empty() {
                                            "https://via.placeholder.com/300x200?text=No+Image".into()
                                        } else {
                                            hotel_result.hotel_picture.clone()
                                        };
                                        view! {
                                            <HotelCard
                                                img
                                                rating=hotel_result.star_rating
                                                hotel_name=hotel_result.hotel_name.clone()
                                                price
                                                hotel_code=hotel_result.hotel_code.clone()
                                                class=format!(
                                                        "w-full max-w-xs mx-auto px-2 sm:px-0 {} {}",
                                                        if is_disabled { "grayscale" } else { "" },
                                                        if is_disabled { "pointer-events-none opacity-50" } else { "" },
                                                    )
                                            />
                                        }
                                    })
                                    .collect_view()
                            }
                        }}
                    </Show>

                    // Pagination controls - only show when we have results
                    <Show
                        when=move || {
                            search_list_page.search_result.get()
                                .map_or(false, |result| !result.hotel_list().is_empty())
                        }
                        fallback=move || view! { <></> }
                    >
                        <div class="col-span-full">
                            // <PaginationInfo />
                            <PaginationControls />
                        </div>
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
    price: Option<f64>,
    hotel_code: String,
    hotel_name: String,
    class: String,
) -> impl IntoView {
    let price = create_rw_signal(price);

    let search_list_page: SearchListResults = expect_context();
    let hotel_view_info_ctx: HotelInfoCtx = expect_context();

    let navigate = use_navigate();

    // ---- Navigation Handler ----
    let on_navigate = {
        let hotel_code_cloned = hotel_code.clone();
        let navigate = navigate.clone();
        let price = price.clone();

        move || {
            // ✅ 1. Block navigation if no price or price is zero
            if !price.get_untracked().map(|f| f > 0.0).unwrap_or(false) {
                log!(
                    "Navigation blocked: no valid price for {}",
                    hotel_code_cloned
                );
                return;
            }

            // ✅ 2. Set context for Hotel Info
            hotel_view_info_ctx
                .hotel_code
                .set(hotel_code_cloned.clone());
            HotelDetailsUIState::reset();
            log!("Hotel code set: {}", hotel_code_cloned);

            // ✅ 3. Try to build query params
            let mut target_url = AppRoutes::HotelDetails.to_string().to_string();
            if let Some(hotel_params) = HotelDetailsParams::from_current_context() {
                target_url = hotel_params.to_shareable_url();
            }
            log!("Opening in new tab: {}", target_url);

            // ✅ 4. Open in new tab
            if let Some(window) = web_sys::window() {
                let _ = window.open_with_url_and_target(&target_url, "_blank");
            }

            // ✅ 4. Close dialogs after navigation
            InputGroupState::toggle_dialog(OpenDialogComponent::None);
        }
    };

    // ---- UI ----
    view! {
        <div
            on:click=move |ev| {
                ev.prevent_default();
                ev.stop_propagation();
                on_navigate();
            }
        >
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
                            {move || price.get().map(|p| view! {
                                <PriceDisplay price=p />
                            })}
                            <button
                                class="font-semibold underline underline-offset-2 decoration-solid text-xs sm:text-sm"
                            >
                                "View details"
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
