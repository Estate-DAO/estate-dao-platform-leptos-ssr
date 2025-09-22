use leptos::*;
use leptos_router::use_navigate;

// use crate::api::get_room;
use crate::api::client_side_api::{ClientSideApiClient, Place, PlaceData};
use crate::application_services::filter_types::UISearchFilters;
use crate::component::{
    format_price_range_value, AmenitiesFilter, Destination, GuestSelection, Navbar,
    PaginationControls, PaginationInfo, PriceRangeFilter, PropertyTypeFilter, SkeletonCards,
    StarRatingFilter, MAX_PRICE, MIN_PRICE,
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

    let filters_signal = search_ctx.filters;

    let price_filter_value = {
        let filters_signal = filters_signal;
        Signal::derive(move || {
            let f = filters_signal.get();
            match (f.min_price_per_night, f.max_price_per_night) {
                (None, None) => None,
                (min, max) => Some((min.unwrap_or(MIN_PRICE), max.unwrap_or(MAX_PRICE))),
            }
        })
    };
    let star_filter_value = {
        let filters_signal = filters_signal;
        Signal::derive(move || filters_signal.get().min_star_rating)
    };
    let has_active_filters = {
        let filters_signal = filters_signal;
        Signal::derive(move || filters_signal.get().has_filters())
    };

    // Derived filter options and selections
    let amenities_options_signal: Signal<Vec<String>> = {
        let search_list_page = search_list_page.clone();
        Signal::derive(move || {
            let mut freq: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
            if let Some(res) = search_list_page.search_result.get() {
                for h in res.hotel_list() {
                    for a in h.amenities.iter() {
                        let label = a.trim();
                        if label.is_empty() {
                            continue;
                        }
                        let lower = label.to_lowercase();
                        if lower.starts_with("facility ") || lower == "facility" {
                            continue;
                        }
                        *freq.entry(label.to_string()).or_insert(0) += 1;
                    }
                }
            }
            let mut items: Vec<(String, u32)> = freq.into_iter().collect();
            items.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
            items.into_iter().map(|(k, _)| k).take(5).collect()
        })
    };

    let property_type_options_signal: Signal<Vec<String>> = {
        let search_list_page = search_list_page.clone();
        Signal::derive(move || {
            let mut freq: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
            if let Some(res) = search_list_page.search_result.get() {
                for h in res.hotel_list() {
                    if let Some(pt) = &h.property_type {
                        let label = pt.trim();
                        if label.is_empty() {
                            continue;
                        }
                        *freq.entry(label.to_string()).or_insert(0) += 1;
                    }
                }
            }
            let mut items: Vec<(String, u32)> = freq.into_iter().collect();
            items.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
            items.into_iter().map(|(k, _)| k).take(5).collect()
        })
    };

    let amenities_selected_signal: Signal<Vec<String>> = {
        let filters_signal = filters_signal;
        Signal::derive(move || filters_signal.get().amenities.clone().unwrap_or_default())
    };

    let property_types_selected_signal: Signal<Vec<String>> = {
        let filters_signal = filters_signal;
        Signal::derive(move || {
            filters_signal
                .get()
                .property_types
                .clone()
                .unwrap_or_default()
        })
    };

    let amenities_on_toggle = {
        let filters_signal = search_ctx.filters;
        Callback::new(move |label: String| {
            filters_signal.update(|f| {
                let list = f.amenities.get_or_insert_with(Vec::new);
                if let Some(pos) = list.iter().position(|v| v.eq_ignore_ascii_case(&label)) {
                    list.remove(pos);
                    if list.is_empty() {
                        f.amenities = None;
                    }
                } else {
                    list.push(label.clone());
                }
            });
        })
    };

    let amenities_on_clear = {
        let filters_signal = search_ctx.filters;
        Callback::new(move |_| {
            filters_signal.update(|f| f.amenities = None);
        })
    };

    let property_type_on_toggle = {
        let filters_signal = search_ctx.filters;
        Callback::new(move |label: String| {
            filters_signal.update(|f| {
                let list = f.property_types.get_or_insert_with(Vec::new);
                if let Some(pos) = list.iter().position(|v| v.eq_ignore_ascii_case(&label)) {
                    list.remove(pos);
                    if list.is_empty() {
                        f.property_types = None;
                    }
                } else {
                    list.push(label.clone());
                }
            });
        })
    };

    let property_type_on_clear = {
        let filters_signal = search_ctx.filters;
        Callback::new(move |_| {
            filters_signal.update(|f| f.property_types = None);
        })
    };

    let star_filter_on_select = {
        let filters_signal = filters_signal;
        Callback::new(move |next: Option<u8>| {
            filters_signal.update(|filters| {
                if filters.min_star_rating != next {
                    filters.min_star_rating = next;
                }
            });
        })
    };

    let price_filter_on_select = {
        let filters_signal = filters_signal;
        Callback::new(move |next: Option<(f64, f64)>| {
            filters_signal.update(|filters| {
                match next {
                    None => {
                        filters.min_price_per_night = None;
                        filters.max_price_per_night = None;
                    }
                    Some((lo, hi)) => {
                        // Store None when at default bounds to avoid noisy filters
                        filters.min_price_per_night = if (lo - MIN_PRICE).abs() < f64::EPSILON {
                            None
                        } else {
                            Some(lo)
                        };
                        filters.max_price_per_night = if (hi - MAX_PRICE).abs() < f64::EPSILON {
                            None
                        } else {
                            Some(hi)
                        };
                    }
                }
            });
        })
    };

    let clear_filters = {
        let filters_signal = filters_signal;
        Callback::new(move |_| {
            filters_signal.set(UISearchFilters::default());
        })
    };

    view! {
        <section class="relative min-h-screen bg-slate-50">
            <Navbar />
            <div class="w-full mx-auto px-2 sm:px-4 pb-10">
                <div class="flex flex-col items-center mt-2 sm:mt-6">
                    <div class="w-full rounded-2xl shadow-sm">
                        <div class="p-2 sm:p-4">
                            <InputGroupContainer default_expanded=false given_disabled=disabled_input_group allow_outside_click_collapse=true />
                        </div>
                    </div>
                </div>

                <div class="mt-6 flex flex-col gap-6 lg:flex-row">
                    <aside class="w-full lg:w-80 shrink-0 lg:sticky lg:top-24">
                        <div class="lg:max-h-[calc(100vh-6rem)] lg:overflow-y-auto lg:pr-2">
                            <div class="space-y-6 rounded-2xl border border-slate-200 bg-white p-4 shadow-sm">
                                <div class="flex items-center justify-between">
                                    <h2 class="text-sm font-semibold uppercase tracking-wide text-slate-600">
                                        "Filters"
                                    </h2>
                                    <button
                                        type="button"
                                        class="text-xs font-medium text-blue-600 transition-colors duration-150 hover:text-blue-700 disabled:text-slate-400"
                                        disabled=move || !has_active_filters.get()
                                        on:click=clear_filters.clone()
                                    >
                                        "Clear filters"
                                    </button>
                                </div>
                                <div class="border-t border-slate-100"></div>
                                <PriceRangeFilter
                                    value=price_filter_value
                                    on_select=price_filter_on_select.clone()
                                />
                                <div class="border-t border-slate-100"></div>
                                <StarRatingFilter
                                    value=star_filter_value
                                    on_select=star_filter_on_select.clone()
                                />
                                <div class="border-t border-slate-100"></div>
                                <AmenitiesFilter
                                    options=amenities_options_signal
                                    selected=amenities_selected_signal
                                    on_toggle=amenities_on_toggle
                                    on_clear=amenities_on_clear
                                />
                                <div class="border-t border-slate-100"></div>
                                <PropertyTypeFilter
                                    options=property_type_options_signal
                                    selected=property_types_selected_signal
                                    on_toggle=property_type_on_toggle
                                    on_clear=property_type_on_clear
                                />
                            </div>
                        </div>
                    </aside>

                    <div class="flex-1 min-w-0">
                        // Use resource pattern with Suspense for automatic loading states
                        <Suspense fallback=move || view! { <div class="flex flex-wrap gap-4 sm:gap-5 justify-start">{fallback()}</div> }>
                            {move || {
                                // Trigger the resource loading but don't render anything
                                let _ = hotel_search_resource.get();
                                view! { <></> }
                            }}
                        </Suspense>

                        <div class="mt-4 flex flex-wrap gap-4 sm:gap-5 justify-start">

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
                                    let filters = filters_signal.get();
                                    let filtered_hotels = filters.apply_filters(&hotel_results);
                                    let min_rating_filter = filters.min_star_rating;
                                    let min_price_filter = filters.min_price_per_night;
                                    let max_price_filter = filters.max_price_per_night;

                                    if hotel_results.is_empty() {
                                        let current_page = pagination_state.current_page.get();

                                        if current_page > 1 {
                                            // Show "Go to first page" button when on page > 1 with no results
                                            view! {
                                                <div class="flex flex-col items-center justify-center mt-4 sm:mt-6 p-2 sm:p-4 w-full min-h-[200px]">
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
                                                <div class="flex flex-col items-center justify-center mt-4 sm:mt-6 p-2 sm:p-4 w-full min-h-[200px]">
                                                    <p class="text-center">
                                                        No hotels found for your search criteria.
                                                    </p>
                                                </div>
                                            }
                                        }
                                            .into_view()
                                    } else if filtered_hotels.is_empty() {
                                        if min_rating_filter.is_some()
                                            || min_price_filter.is_some()
                                            || max_price_filter.is_some()
                                        {
                                            let filter_message = match (
                                                min_rating_filter,
                                                min_price_filter,
                                                max_price_filter,
                                            ) {
                                                (Some(min_rating), Some(min_price), Some(max_price)) => format!(
                                                    "No hotels match a {min_rating}+ star rating priced between {} and {} per night on this page.",
                                                    format_price_range_value(min_price),
                                                    format_price_range_value(max_price)
                                                ),
                                                (Some(min_rating), None, Some(max_price)) => format!(
                                                    "No hotels match a {min_rating}+ star rating at or below {} per night on this page.",
                                                    format_price_range_value(max_price)
                                                ),
                                                (Some(min_rating), Some(min_price), None) => format!(
                                                    "No hotels match a {min_rating}+ star rating at or above {} per night on this page.",
                                                    format_price_range_value(min_price)
                                                ),
                                                (Some(min_rating), None, None) => format!(
                                                    "No hotels match the {min_rating}+ star filter on this page."
                                                ),
                                                (None, Some(min_price), Some(max_price)) => format!(
                                                    "No hotels priced between {} and {} per night on this page.",
                                                    format_price_range_value(min_price),
                                                    format_price_range_value(max_price)
                                                ),
                                                (None, None, Some(max_price)) => format!(
                                                    "No hotels priced at or below {} per night on this page.",
                                                    format_price_range_value(max_price)
                                                ),
                                                (None, Some(min_price), None) => format!(
                                                    "No hotels priced at or above {} per night on this page.",
                                                    format_price_range_value(min_price)
                                                ),
                                                (None, None, None) => String::new(),
                                            };

                                            view! {
                                                <div class="w-full flex flex-col items-center justify-center gap-4 rounded-lg border border-dashed border-blue-300 bg-blue-50/60 px-4 py-6 text-center">
                                                    <p class="text-sm text-slate-600">
                                                        {filter_message}
                                                    </p>
                                                    <div class="flex flex-wrap items-center justify-center gap-2">
                                                        <Show
                                                            when=move || min_rating_filter.is_some()
                                                            fallback=move || view! { <></> }
                                                        >
                                                            <button
                                                                type="button"
                                                                class="rounded-full border border-blue-500 px-4 py-2 text-sm font-medium text-blue-600 transition-colors duration-150 hover:bg-blue-500 hover:text-white focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 focus-visible:ring-offset-2"
                                                                on:click=move |_| {
                                                                    filters_signal.update(|filters| {
                                                                        filters.min_star_rating = None;
                                                                    });
                                                                }
                                                            >
                                                                "Clear star filter"
                                                            </button>
                                                        </Show>
                                                        <Show
                                                            when=move || min_price_filter.is_some() || max_price_filter.is_some()
                                                            fallback=move || view! { <></> }
                                                        >
                                                            <button
                                                                type="button"
                                                                class="rounded-full border border-blue-500 px-4 py-2 text-sm font-medium text-blue-600 transition-colors duration-150 hover:bg-blue-500 hover:text-white focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 focus-visible:ring-offset-2"
                                                                on:click=move |_| {
                                                                    filters_signal.update(|filters| {
                                                                        filters.min_price_per_night = None;
                                                                        filters.max_price_per_night = None;
                                                                    });
                                                                }
                                                            >
                                                                "Clear price filter"
                                                            </button>
                                                        </Show>
                                                    </div>
                                                </div>
                                            }
                                                .into_view()
                                        } else {
                                            view! { <></> }.into_view()
                                        }
                                    } else {
                                        filtered_hotels
                                            .iter()
                                            .map(|hotel_result| {
                                                let mut price = hotel_result
                                                    .price
                                                    .clone()
                                                    .map(|p| p.room_price);
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
                                                                "px-2 sm:px-0 basis-[16.5rem] flex-shrink-0 flex-grow-0 {} {}",
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
                                <div class="w-full">
                                    // <PaginationInfo />
                                    <PaginationControls />
                                </div>
                            </Show>
                        </div>
                    </div>
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
    let container_class = class;

    view! {
        <div
            class=container_class
            on:click=move |ev| {
                ev.prevent_default();
                ev.stop_propagation();
                on_navigate();
            }
        >
            <div class="mx-auto w-full max-w-[16.5rem] rounded-lg overflow-hidden shadow-sm border border-gray-300 bg-white">
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
                        <button class="font-semibold underline underline-offset-2 decoration-solid text-xs sm:text-sm">
                            "View details"
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}
