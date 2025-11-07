use leptos::*;
use leptos_router::use_navigate;
use std::collections::HashMap;
use web_sys::MouseEvent;

use crate::api::auth::auth_state::AuthStateSignal;
// use crate::api::get_room;
use crate::api::client_side_api::{ClientSideApiClient, Place, PlaceData};
use crate::application_services::filter_types::UISearchFilters;
use crate::component::{
    format_price_range_value, CheckboxFilter, Destination, Footer, GuestSelection, Navbar,
    PaginationControls, PaginationInfo, PriceRangeFilter, SkeletonCards, SortBy, StarRatingFilter,
    MAX_PRICE, MIN_PRICE,
};
use crate::log;
use crate::page::{HotelDetailsParams, HotelListNavbar, HotelListParams, InputGroupContainer};
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

    let search_ctx3: UISearchCtx = expect_context();

    let search_ctx4: UISearchCtx = expect_context();

    // Initialize pagination state
    let pagination_state: UIPaginationState = expect_context();

    // Sync query params with state on page load (URL → State)
    // Parse URL params and sync to app state (URL → State)
    // This leverages use_query_map's built-in reactivity for browser navigation
    create_effect(move |_| {
        let params = query_map.get();
        if !params.0.is_empty() {
            let params_map: HashMap<String, String> = params
                .0
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();

            // Try individual query params first (NEW format), then fall back to base64 state (LEGACY)
            if let Some(hotel_params) = HotelListParams::from_query_params(&params_map) {
                // Check if we need to search for place by name (placeId missing)
                if hotel_params.place.is_none() && hotel_params.place_name_to_search.is_some() {
                    let place_name = hotel_params.place_name_to_search.clone().unwrap();
                    log!(
                        "[HotelListPage] Only placeName in URL: '{}', searching for placeId...",
                        place_name
                    );

                    // Clone params_map for async closure
                    let params_map_clone = params_map.clone();
                    let place_name_clone = place_name.clone();

                    // Spawn async task to search for place
                    spawn_local(async move {
                        let api_client = ClientSideApiClient::new();
                        match api_client.search_places(place_name_clone.clone()).await {
                            Ok(results) => {
                                if let Some(first_result) = results.first() {
                                    log!(
                                        "[HotelListPage] Found place: {} (ID: {})",
                                        first_result.display_name,
                                        first_result.place_id
                                    );

                                    // Update URL with fetched placeId
                                    let mut new_params = params_map_clone.clone();
                                    new_params.insert(
                                        "placeId".to_string(),
                                        first_result.place_id.clone(),
                                    );

                                    // Keep the placeName for display
                                    new_params.insert("placeName".to_string(), place_name_clone);

                                    // Navigate to updated URL (this will trigger the effect again)
                                    use crate::utils::query_params::update_url_with_params;
                                    update_url_with_params("/hotel-list", &new_params);
                                } else {
                                    log!(
                                        "[HotelListPage] No results found for place name: {}",
                                        place_name_clone
                                    );
                                    // TODO: Show error message to user
                                }
                            }
                            Err(e) => {
                                log!(
                                    "[HotelListPage] Place search failed for '{}': {}",
                                    place_name_clone,
                                    e
                                );
                                // TODO: Show error message to user
                            }
                        }
                    });

                    // Don't sync to app state yet - wait for place search to complete
                    // The URL update above will trigger this effect again with complete params
                    return;
                }

                // Normal case: we have complete params with placeId
                log!(
                    "Parsed hotel params from URL (individual params): {:?}",
                    hotel_params
                );
                hotel_params.sync_to_app_state();
                PreviousSearchContext::update_first_time_filled(search_ctx2.clone());
            } else if let Some(hotel_params) = HotelListParams::from_url_params(&params_map) {
                log!(
                    "Parsed hotel params from URL (legacy base64 state): {:?}",
                    hotel_params
                );
                hotel_params.sync_to_app_state();
                PreviousSearchContext::update_first_time_filled(search_ctx2.clone());
            }
        }
    });

    // Clone search_ctx for use in different closures
    let search_ctx_for_resource = search_ctx.clone();
    let search_ctx_for_url_update = search_ctx.clone();

    // Hotel search resource - triggers only when core search criteria or pagination changes
    // NOT when filters or sorting changes (those are applied client-side)
    let hotel_search_resource = create_resource(
        move || {
            // Depend on query_map to re-run when URL params change
            query_map.get();

            // Track search context changes reactively (but NOT filters/sorting)
            let place = search_ctx_for_resource.place.get();
            let date_range = search_ctx_for_resource.date_range.get();
            let adults = search_ctx_for_resource.guests.adults.get();
            let rooms = search_ctx_for_resource.guests.rooms.get();

            // Track pagination changes reactively (this should trigger new API calls)
            let current_page = pagination_state.current_page.get();
            let page_size = pagination_state.page_size.get();

            let has_valid_dates = date_range.start != (0, 0, 0) && date_range.end != (0, 0, 0);
            let has_valid_search_data = place.is_some() && adults > 0 && rooms > 0;

            // Return true when ready to search
            let is_ready = has_valid_dates && has_valid_search_data;

            // Return a tuple that changes when core search criteria or pagination changes
            // NOTE: Removed sort_options from here to avoid unnecessary API calls
            if is_ready {
                (place, date_range, adults, rooms, current_page, page_size)
            } else {
                // Return a default/non-ready state
                (None, Default::default(), 0, 0, 0, 0)
            }
        },
        move |(is_ready_place, _date_range, _adults, _rooms, current_page, page_size)| {
            let search_ctx_clone = search_ctx_for_resource.clone();
            let search_ctx_clone2 = search_ctx_for_resource.clone();
            async move {
                let is_ready = is_ready_place.is_some();
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
    let update_url_with_current_state = Callback::new(move |_: ()| {
        let current_params = HotelListParams::from_search_context(&search_ctx_for_url_update);
        current_params.update_url();
        log!(
            "Updated URL with current search state: {:?}",
            current_params
        );
    });

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
            items.into_iter().map(|(k, _)| k).take(10).collect()
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
            items.into_iter().map(|(k, _)| k).take(10).collect()
        })
    };

    let popular_filters_options_signal: Signal<Vec<String>> = Signal::derive(move || {
        vec![
            "Breakfast Included".to_string(),
            "Hotel".to_string(),
            "Parking".to_string(),
            "Swimming Pool".to_string(),
            "Pet Friendly".to_string(),
        ]
    });

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

    let popular_filters_selected_signal: Signal<Vec<String>> = {
        let filters_signal = filters_signal;
        Signal::derive(move || {
            filters_signal
                .get()
                .popular_filters
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
            leptos::Callable::call(&update_url_with_current_state, ());
        })
    };

    let amenities_on_clear = {
        let filters_signal = search_ctx.filters;
        Callback::new(move |_| {
            filters_signal.update(|f| f.amenities = None);
            leptos::Callable::call(&update_url_with_current_state, ());
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
            leptos::Callable::call(&update_url_with_current_state, ());
        })
    };

    let property_type_on_clear = {
        let filters_signal = search_ctx.filters;
        Callback::new(move |_| {
            filters_signal.update(|f| f.property_types = None);
            leptos::Callable::call(&update_url_with_current_state, ());
        })
    };

    let popular_filters_on_toggle = {
        let filters_signal = search_ctx.filters;
        Callback::new(move |label: String| {
            filters_signal.update(|f| {
                let list = f.popular_filters.get_or_insert_with(Vec::new);
                if let Some(pos) = list.iter().position(|v| v.eq_ignore_ascii_case(&label)) {
                    list.remove(pos);
                    if list.is_empty() {
                        f.popular_filters = None;
                    }
                } else {
                    list.push(label.clone());
                }
            });
            leptos::Callable::call(&update_url_with_current_state, ());
        })
    };

    let popular_filters_on_clear = {
        let filters_signal = search_ctx.filters;
        Callback::new(move |_| {
            filters_signal.update(|f| f.popular_filters = None);
            leptos::Callable::call(&update_url_with_current_state, ());
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
            leptos::Callable::call(&update_url_with_current_state, ());
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
            leptos::Callable::call(&update_url_with_current_state, ());
        })
    };

    let clear_filters = {
        let filters_signal = filters_signal;
        Callback::new(move |_| {
            filters_signal.set(UISearchFilters::default());
            leptos::Callable::call(&update_url_with_current_state, ());
        })
    };

    let disabled_filters = Signal::derive(move || false);
    let filters_collapsed = create_rw_signal(false);

    // Watch for pagination changes and update URL (pagination should trigger API calls)
    let search_list_page_for_pagination_effect = search_list_page.clone();
    create_effect(move |_| {
        let _ = pagination_state.current_page.get();
        let _ = pagination_state.page_size.get();

        // Only update URL if we have search results (prevents initial load URL spam)
        if search_list_page_for_pagination_effect
            .search_result
            .get()
            .is_some()
        {
            leptos::Callable::call(&update_url_with_current_state, ());
        }
    });

    // Watch for filter and sort changes and update URL (but don't trigger API calls)
    let search_list_page_for_filter_effect = search_list_page.clone();
    create_effect(move |_| {
        let _ = search_ctx.filters.get(); // Watch for filter changes
        let _ = search_ctx.sort_options.get(); // Watch for sort changes

        // Only update URL if we have search results (prevents initial load URL spam)
        if search_list_page_for_filter_effect
            .search_result
            .get()
            .is_some()
        {
            leptos::Callable::call(&update_url_with_current_state, ());
        }
    });

    view! {
        // Fixed header section at top
        // <div class="fixed top-0 left-0 right-0 z-50 bg-white shadow-sm">
            // <div class={
            //     let is_input_expanded = move || InputGroupState::is_open_show_full_input();
            //     move || format!(
            //         "bg-blue-600 relative transition-all duration-300 {}",
            //         if is_input_expanded() {
            //             // Expanded height on mobile/tablet when input group is open, normal on desktop
            //             "h-96 sm:h-96 md:h-80 lg:h-32"
            //         } else {
            //             // Normal collapsed height for all screen sizes
            //             "h-40 sm:h-40 md:h-36 lg:h-32"
            //         }
            //     )
            // }>
            //     <Navbar blue_header=true />

            //     // Mobile/tablet: position input group normally in the flow when expanded
            //     // Desktop: use absolute positioning (original behavior)
            //     <div class={
            //         let is_input_expanded = move || InputGroupState::is_open_show_full_input();
            //         move || format!(
            //             "w-full flex flex-col items-center px-4 {}",
            //             if is_input_expanded() {
            //                 // Mobile/tablet expanded: normal flow positioning
            //                 "justify-end h-full pb-4 lg:absolute lg:left-1/2 lg:bottom-0 lg:transform lg:-translate-x-1/2 lg:translate-y-1/2 lg:max-w-5xl lg:z-40 lg:h-auto lg:pb-0"
            //             } else {
            //                 // All screens collapsed: absolute positioning for desktop, hidden for mobile
            //                 "absolute left-1/2 bottom-0 transform -translate-x-1/2 translate-y-1/2 max-w-5xl z-40"
            //             }
            //         )
            //     }>
            //         <InputGroupContainer
            //             default_expanded=false
            //             given_disabled=disabled_input_group
            //             allow_outside_click_collapse=true
            //         />
            //     </div>
            // </div>
            <HotelListNavbar />
        // </div>

        // Dynamic spacer that only adjusts on mobile/tablet, stays normal on desktop
        <div class={
            let is_input_expanded = move || InputGroupState::is_open_show_full_input();
            move || format!(
                "transition-all duration-300 {}",
                if is_input_expanded() {
                    // Larger spacer when input is expanded on mobile/tablet, normal on desktop
                    "h-96 sm:h-96 md:h-80 lg:h-48"
                } else {
                    // Normal spacer when collapsed on all screens
                    "h-24"
                }
            )
        }></div>

        // Main scrollable section
        <section class="bg-slate-50 px-4 pb-2">
            // Desktop layout (lg screens and up) - centered with 85% width
            <div class="hidden lg:flex justify-center">
                <div class="w-[85%] max-w-7xl flex h-[calc(100vh-7rem)]">
                    // Fixed aside on left (desktop only)
                    <aside class="w-80 shrink-0 bg-slate-50 border-r border-slate-200">
                    <div class="h-full overflow-y-auto p-4">
                        <div class="flex flex-col rounded-2xl border border-slate-200 bg-white p-4 shadow-sm">
                            <div class="flex items-center gap-2">
                                <button
                                    type="button"
                                    class="flex flex-1 items-center justify-between rounded-md px-2 py-1 text-left transition-colors duration-150 hover:bg-slate-100 cursor-pointer"
                                    aria-expanded=move || (!filters_collapsed.get()).to_string()
                                    aria-label="Toggle filter sidebar"
                                    on:click=move |_| {
                                        filters_collapsed.update(|collapsed| *collapsed = !*collapsed);
                                    }
                                >
                                    <span class="text-sm font-semibold uppercase tracking-wide text-slate-600">
                                        "Filters"
                                    </span>
                                    {move || {
                                        if filters_collapsed.get() {
                                            view! {
                                                <svg class="w-4 h-4 text-slate-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 15l-7-7-7 7" />
                                                </svg>
                                            }.into_view()
                                        } else {
                                            view! {
                                                <svg class="w-4 h-4 text-slate-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                                                </svg>
                                            }.into_view()
                                        }
                                    }}
                                </button>
                                <button
                                    type="button"
                                    class="text-xs font-medium text-blue-600 transition-colors duration-150 hover:text-blue-700 disabled:text-slate-400"
                                    disabled=move || !has_active_filters.get()
                                    on:click=clear_filters.clone()
                                >
                                    "Clear filters"
                                </button>
                            </div>
                            <Show
                                when=move || !filters_collapsed.get()
                                fallback=move || view! { <></> }
                            >
                                <div class="mt-4 space-y-6 custom-scrollbar flex-1 overscroll-contain pr-1">
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
                                    <CheckboxFilter
                                        title="Popular Filters".to_string()
                                        options=popular_filters_options_signal
                                        selected=popular_filters_selected_signal
                                        on_toggle=popular_filters_on_toggle
                                        on_clear=popular_filters_on_clear
                                    />
                                    </div>
                                    <div class="border-t border-slate-100"></div>
                                    <CheckboxFilter
                                        title="Amenities".to_string()
                                        options=amenities_options_signal
                                        selected=amenities_selected_signal
                                        on_toggle=amenities_on_toggle
                                        on_clear=amenities_on_clear
                                    />
                                    <div class="border-t border-slate-100"></div>
                                    <CheckboxFilter
                                        title="Property Type".to_string()
                                        options=property_type_options_signal
                                        selected=property_types_selected_signal
                                        on_toggle=property_type_on_toggle
                                        on_clear=property_type_on_clear
                                    />
                            </Show>
                        </div>
                    </div>
                </aside>

                // Right content area (desktop)
                <div class="flex-1 min-w-0 overflow-y-auto">
                    <div class="p-4">
                        // Results count and sort by component for desktop
                        <div class="mb-4 flex flex-col lg:flex-row lg:justify-between lg:items-center gap-2 lg:gap-0">
                            <div class="text-gray-700">
                                {move || {
                                    let search_ctx = search_ctx3.clone();
                                    let pagination_state: UIPaginationState = expect_context();

                                    // Get place name
                                    let place_name = search_ctx.place.get()
                                        .map(|place| place.display_name)
                                        .unwrap_or_else(|| "this location".to_string());

                                    // Get total results from pagination metadata
                                    if let Some(pagination_meta) = pagination_state.pagination_meta.get() {
                                        if let Some(total_results) = pagination_meta.total_results {
                                            let formatted_count = if total_results >= 1000 {
                                                format!("{}k+", total_results / 1000)
                                            } else {
                                                total_results.to_string()
                                            };
                                            view! {
                                                <span>
                                                    <span class="font-semibold">{formatted_count}</span>
                                                    " Properties found in "
                                                    <span class="font-semibold">{place_name}</span>
                                                </span>
                                            }.into_view()
                                        } else {
                                            view! {
                                                <span>
                                                    "Properties found in "
                                                    <span class="font-semibold">{place_name}</span>
                                                </span>
                                            }.into_view()
                                        }
                                    } else {
                                        view! {
                                            <span>
                                                "Properties found in "
                                                <span class="font-semibold">{place_name}</span>
                                            </span>
                                        }.into_view()
                                    }
                                }}
                            </div>
                            <div class="flex justify-end lg:justify-start">
                                <SortBy />
                            </div>
                        </div>

                        // Use resource pattern with Suspense for automatic loading states
                        <Suspense fallback=move || view! { <div class="grid grid-cols-1">{fallback()}</div> }>
                            {move || {
                                // Trigger the resource loading but don't render anything
                                let _ = hotel_search_resource.get();
                                view! { <></> }
                            }}
                        </Suspense>

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
                                    let sort_options = search_ctx.sort_options.get();
                                    let filtered_hotels = filters.apply_filters_and_sort(&hotel_results, &sort_options);
                                    let min_rating_filter = filters.min_star_rating;
                                    let min_price_filter = filters.min_price_per_night;
                                    let max_price_filter = filters.max_price_per_night;

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
                                                <div class="col-span-full flex flex-col items-center justify-center gap-4 rounded-lg border border-dashed border-blue-300 bg-blue-50/60 px-4 py-6 text-center">
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
                                                                    leptos::Callable::call(&update_url_with_current_state, ());
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
                                                                    leptos::Callable::call(&update_url_with_current_state, ());
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
                                                let res = hotel_result.clone();
                                                let hotel_address = hotel_result.hotel_address.clone();
                                                let amenities = Memo::new(move |_| res.amenities.iter().filter(|f| !f.to_lowercase().contains("facility")).cloned().collect::<Vec<String>>());
                                                view! {
                                                    <HotelCardTile
                                                        img
                                                        guest_score=None
                                                        rating=hotel_result.star_rating
                                                        hotel_name=hotel_result.hotel_name.clone()
                                                        hotel_code=hotel_result.hotel_code.clone()
                                                        price=price
                                                        discount_percent=None
                                                        amenities=amenities.get()
                                                        property_type=hotel_result.property_type.clone()
                                                        class=format!(
                                                                "w-full mb-4 {} {}",
                                                                if is_disabled { "bg-gray-200 pointer-events-none" } else { "bg-white" },
                                                                ""
                                                            )
                                                        hotel_address
                                                        distance_from_center_km=hotel_result.distance_from_center_km
                                                        disabled=is_disabled
                                                    />
                                                    // <HotelCard
                                                    //     img
                                                    //     rating=hotel_result.star_rating
                                                    //     hotel_name=hotel_result.hotel_name.clone()
                                                    //     price
                                                    //     hotel_code=hotel_result.hotel_code.clone()
                                                    //     class=format!(
                                                    //             "w-full max-w-xs mx-auto px-2 sm:px-0 {} {}",
                                                    //             if is_disabled { "grayscale" } else { "" },
                                                    //             if is_disabled { "pointer-events-none opacity-50" } else { "" },
                                                    //         )
                                                    // />
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
                </div>
            </div>

            // Mobile layout (lg screens and below)
            <div class="lg:hidden min-h-screen mt-16">
                <div class="px-4 py-6">
                    // Filter toggle button for mobile
                    <div class="mb-4">
                        <button
                            type="button"
                            class="w-full flex items-center justify-between p-3 bg-white rounded-lg border border-gray-200 shadow-sm"
                            on:click=move |_| filters_collapsed.update(|c| *c = !*c)
                        >
                            <span class="font-medium">Filters</span>
                            <svg
                                class={move || format!("w-5 h-5 transition-transform {}", if filters_collapsed.get() { "rotate-180" } else { "" })}
                                fill="none"
                                stroke="currentColor"
                                viewBox="0 0 24 24"
                            >
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                            </svg>
                        </button>
                    </div>

                    // Collapsible filter section with max height to prevent footer overlap
                    <Show when=move || !filters_collapsed.get()>
                        <div class="mb-6 p-4 bg-white rounded-lg border border-gray-200 shadow-sm max-h-96 overflow-y-auto">
                            <div class="space-y-6">
                                <div class="flex items-center justify-between mb-4">
                                    <span class="text-sm font-semibold uppercase tracking-wide text-slate-600">
                                        "Filters"
                                    </span>
                                    <button
                                        type="button"
                                        class="text-xs font-medium text-blue-600 transition-colors duration-150 hover:text-blue-700 disabled:text-slate-400"
                                        disabled=move || !has_active_filters.get()
                                        on:click=clear_filters.clone()
                                    >
                                        "Clear filters"
                                    </button>
                                </div>

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
                                <CheckboxFilter
                                    title="Popular Filters".to_string()
                                    options=popular_filters_options_signal
                                    selected=popular_filters_selected_signal
                                    on_toggle=popular_filters_on_toggle
                                    on_clear=popular_filters_on_clear
                                />
                                <div class="border-t border-slate-100"></div>
                                <CheckboxFilter
                                    title="Amenities".to_string()
                                    options=amenities_options_signal
                                    selected=amenities_selected_signal
                                    on_toggle=amenities_on_toggle
                                    on_clear=amenities_on_clear
                                />
                                <div class="border-t border-slate-100"></div>
                                <CheckboxFilter
                                    title="Property Type".to_string()
                                    options=property_type_options_signal
                                    selected=property_types_selected_signal
                                    on_toggle=property_type_on_toggle
                                    on_clear=property_type_on_clear
                                />
                            </div>
                        </div>
                    </Show>

                    // Results count and sort by component for mobile
                    <div class="mb-4 flex flex-col gap-2">
                        <div class="text-gray-700 text-md">
                            {move || {
                                let search_ctx = search_ctx4.clone();
                                let pagination_state: UIPaginationState = expect_context();

                                // Get place name
                                let place_name = search_ctx.place.get()
                                    .map(|place| place.display_name)
                                    .unwrap_or_else(|| "this location".to_string());

                                // Get total results from pagination metadata
                                if let Some(pagination_meta) = pagination_state.pagination_meta.get() {
                                    if let Some(total_results) = pagination_meta.total_results {
                                        let formatted_count = if total_results >= 1000 {
                                            format!("{}k+", total_results / 1000)
                                        } else {
                                            total_results.to_string()
                                        };
                                        view! {
                                            <span>
                                                <span class="font-semibold">{formatted_count}</span>
                                                " Properties found in "
                                                <span class="font-semibold">{place_name}</span>
                                            </span>
                                        }.into_view()
                                    } else {
                                        view! {
                                            <span>
                                                "Properties found in "
                                                <span class="font-semibold">{place_name}</span>
                                            </span>
                                        }.into_view()
                                    }
                                } else {
                                    view! {
                                        <span>
                                            "Properties found in "
                                            <span class="font-semibold">{place_name}</span>
                                        </span>
                                    }.into_view()
                                }
                            }}
                        </div>
                        <div class="flex justify-start">
                            <SortBy />
                        </div>
                    </div>

                    // Mobile hotel listings
                    <div class="space-y-4">
                        <Suspense fallback=move || view! { <div class="space-y-4">{(0..5).map(|_| fallback()).collect_view()}</div> }>
                            {move || {
                                // Trigger the resource loading
                                let _ = hotel_search_resource.get();
                                view! { <></> }
                            }}
                        </Suspense>

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
                                let sort_options = search_ctx.sort_options.get();
                                let filtered_hotels = filters.apply_filters_and_sort(&hotel_results, &sort_options);

                                if hotel_results.is_empty() {
                                    view! {
                                        <div class="text-center py-8">
                                            <p class="text-gray-600">No hotels found</p>
                                        </div>
                                    }
                                } else if filtered_hotels.is_empty() {
                                    view! {
                                        <div class="text-center py-8">
                                            <p class="text-gray-600">No hotels match your filters</p>
                                            <button
                                                class="mt-4 px-4 py-2 bg-blue-500 text-white rounded-lg"
                                                on:click=move |e| leptos::Callable::call(&clear_filters, e)
                                            >
                                                Clear Filters
                                            </button>
                                        </div>
                                    }
                                } else {
                                    view! {
                                        <div>
                                            { filtered_hotels
                                            .into_iter()
                                            .map(|hotel_result| {
                                                let mut price = hotel_result
                                                .price
                                                .clone()
                                                .map(|p| p.room_price);
                                            let is_disabled = price.unwrap_or(0.0) <= 0.0;
                                            if is_disabled {
                                                price = None;
                                            }
                                            let img = if hotel_result.hotel_picture.is_empty() {
                                                "https://via.placeholder.com/300x200?text=No+Image".into()
                                            } else {
                                                hotel_result.hotel_picture.clone()
                                            };
                                            let res = hotel_result.clone();
                                            let hotel_address = hotel_result.hotel_address.clone();
                                            let amenities = Memo::new(move |_| res.amenities.iter().filter(|f| !f.to_lowercase().contains("facility")).cloned().collect::<Vec<String>>());
                                            view! {
                                                <HotelCardTile
                                                img
                                                guest_score=None
                                                rating=hotel_result.star_rating
                                                hotel_name=hotel_result.hotel_name.clone()
                                                hotel_code=hotel_result.hotel_code.clone()
                                                price=price
                                                discount_percent=None
                                                amenities=amenities.get()
                                                property_type=hotel_result.property_type.clone()
                                                class=format!(
                                                    "w-full mb-4 {}",
                                                    if is_disabled { "bg-gray-200 pointer-events-none" } else { "bg-white" }
                                                )
                                                hotel_address
                                                distance_from_center_km=hotel_result.distance_from_center_km
                                                disabled=is_disabled
                                                />
                                            }
                                        })
                                        .collect_view()}
                                    </div>
                                    }
                                }
                            }}
                        </Show>

                        // Mobile pagination controls
                        <Show
                            when=move || {
                                search_list_page.search_result.get()
                                    .map_or(false, |result| !result.hotel_list().is_empty())
                            }
                            fallback=move || view! { <></> }
                        >
                            <div class="mt-6">
                                <PaginationControls />
                            </div>
                        </Show>
                    </div>
                </div>
            </div>
        </section>

        // Footer at the bottom
        <Footer />
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
            let target_url = if let Some(hotel_params) = HotelDetailsParams::from_current_context()
            {
                hotel_params.to_shareable_url()
            } else {
                AppRoutes::HotelDetails.to_string().into()
            };
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

#[component]
pub fn HotelCardTile(
    img: String,
    rating: u8,
    guest_score: Option<f32>,
    hotel_name: String,
    hotel_code: String,
    price: Option<f64>,
    discount_percent: Option<u8>,
    amenities: Vec<String>,
    property_type: Option<String>,
    hotel_address: Option<String>,
    #[prop(default = None)] distance_from_center_km: Option<f64>,
    #[prop(into)] class: String,
    disabled: bool,
) -> impl IntoView {
    let price_copy = price.clone();

    let price = create_rw_signal(price);

    let wishlist_hotel_code = hotel_code.clone();

    let search_list_page: SearchListResults = expect_context();
    let hotel_view_info_ctx: HotelInfoCtx = expect_context();

    let navigate = use_navigate();

    let displayed_score = guest_score.or_else(|| {
        if rating > 0 {
            Some((rating as f32) * 2.0)
        } else {
            None
        }
    });

    let review_text = match displayed_score {
        Some(s) if s >= 9.0 => "Excellent",
        Some(s) if s >= 8.0 => "Very Good",
        Some(s) if s >= 7.0 => "Good",
        Some(s) if s >= 5.0 => "Fair",
        Some(_) => "Very Bad",
        None => "Unrated",
    };

    let rating_badge_class = match displayed_score {
        Some(s) if s >= 9.0 => "bg-emerald-700 text-white",
        Some(s) if s >= 8.0 => "bg-emerald-500 text-white",
        Some(s) if s >= 7.0 => "bg-amber-400 text-white",
        Some(s) if s >= 5.0 => "bg-amber-300 text-white",
        Some(_) => "bg-rose-500 text-white",
        None => "bg-gray-200 text-gray-700",
    };

    let on_navigate = {
        let hotel_code_cloned = hotel_code.clone();
        let navigate = navigate.clone();
        let price = price.clone();

        move || {
            // ✅ 1. Block navigation if no price or price is zero
            // if !price.get_untracked().map(|f| f > 0.0).unwrap_or(false) {
            //     log!(
            //         "Navigation blocked: no valid price for {}",
            //         hotel_code_cloned
            //     );
            //     return;
            // }

            // ✅ 2. Set context for Hotel Info
            hotel_view_info_ctx
                .hotel_code
                .set(hotel_code_cloned.clone());
            HotelDetailsUIState::reset();
            log!("Hotel code set: {}", hotel_code_cloned);

            // ✅ 3. Try to build query params
            let target_url = if let Some(hotel_params) = HotelDetailsParams::from_current_context()
            {
                hotel_params.to_shareable_url()
            } else {
                return;
            };
            log!("Opening in new tab: {}", target_url);

            // ✅ 4. Open in new tab
            if let Some(window) = web_sys::window() {
                let _ = window.open_with_url_and_target(&target_url, "_blank");
            }

            // ✅ 4. Close dialogs after navigation
            InputGroupState::toggle_dialog(OpenDialogComponent::None);
        }
    };

    view! {
        // Smaller, more compact card design
        <div on:click=move |ev| {
                ev.prevent_default();
                ev.stop_propagation();
                on_navigate();
            }
            class=format!("flex flex-col md:flex-row rounded-lg shadow-md border border-gray-200 hover:shadow-lg transition w-full min-h-[180px] md:h-64 {}", class)>
            // IMAGE: smaller dimensions for more compact design
            <div clone:hotel_code class="relative w-full h-40 md:h-full md:basis-[28%] md:flex-shrink-0">
                <img class="w-full h-full object-cover md:rounded-l-lg rounded-t-lg md:rounded-t-none" src=img alt=hotel_name.clone() />
                <Wishlist hotel_code />
            </div>

            // RIGHT CONTENT
            // Smaller padding and content area for more compact design
            <div class="flex-1 min-w-0 flex flex-col justify-between p-3 md:p-4 min-h-[140px] md:min-h-0">
                // title + reviews row
                <div class="flex flex-col md:flex-row md:items-start md:justify-between gap-3">
                    <div class="min-w-0 flex-1">
                        // Smaller text and tighter spacing
                        <h3 class="text-base font-semibold leading-tight overflow-hidden whitespace-normal break-words" style="display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical;">{hotel_name.clone()}</h3>
                        <p class="text-xs text-gray-600 mt-1 leading-snug overflow-hidden whitespace-nowrap text-ellipsis">{hotel_address.clone().unwrap_or_default()}</p>

                        // Distance from center if available
                        {distance_from_center_km.map(|distance| {
                            let formatted_distance = if distance < 1.0 {
                                format!("{:.0} m from centre", distance * 1000.0)
                            } else {
                                format!("{:.1} km from centre", distance)
                            };
                            view! {
                                <p class="text-xs text-blue-600 mt-1 leading-snug">
                                    {formatted_distance}
                                </p>
                            }
                        })}

                        // Fewer amenities with smaller spacing
                        <div class="flex flex-wrap gap-1 mt-2">{amenities.iter().take(4).map(|a| view! {
                                <span class="bg-gray-100 text-gray-700 text-xs px-2 py-0.5 rounded whitespace-nowrap">{a}</span>
                            }).collect_view()}
                        </div>
                    </div>

                    // review block
                    // on small screens it becomes full width (so it won't force overflow); on md it becomes a small right column
                    // <div class="w-full md:w-28 flex md:flex-col flex-row items-center gap-2">
                    //     <div class="flex-1 space-x-1 flex items-center">
                    //         <p class="text-sm font-medium text-gray-700">{review_text}</p>
                    //         <div class=format!("mt-1 inline-flex items-center justify-center rounded-md px-2 py-1 text-sm font-semibold {}", rating_badge_class)>
                    //             {move || displayed_score.map(|s| format!("{:.1}", s)).unwrap_or_else(|| "-".to_string())}
                    //         </div>
                    //         // <p class="text-xs text-gray-500 mt-1">(100 Reviews)</p>
                    //     </div>
                    // </div>
                </div>

                // price + CTA
                // Compact spacing and smaller button
                <div class="flex flex-col sm:flex-row sm:items-end sm:justify-between gap-2 mt-auto pt-2">
                    <div class="flex-1">
                        // {property_type.clone().map(|pt| view! {
                        //     <p class="text-sm text-gray-500">{pt}</p>
                        // })}
                    </div>
                    <Show when=move || !disabled fallback=|| view!{
                        <div class="flex items-center justify-end text-red-600 gap-2">
                            // Info Icon
                            <svg xmlns="http://www.w3.org/2000/svg"
                                class="h-4 w-4 flex-shrink-0"
                                fill="none"
                                viewBox="0 0 24 24"
                                stroke="currentColor"
                                stroke-width="2">
                                <path stroke-linecap="round" stroke-linejoin="round"
                                    d="M12 9v2m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                            </svg>

                            <p class="text-xs">
                                This property is not available.
                            </p>
                        </div>
                    }>
                        <div class="text-right flex-shrink-0">
                            {move || {
                                if let Some(p) = price_copy {
                                    view! {
                                        <p class="text-lg font-bold">
                                            ${format!("{:.0}", p)} <span class="text-xs font-normal text-gray-500">"/ night"</span>
                                        </p>
                                    }
                                } else {
                                    view! { <p class="text-sm font-bold"></p> }
                                }
                            }}
                            // <p class="text-xs text-gray-500 mt-1">"4 Nights, 1 room including taxes"</p>

                            {discount_percent.map(|d| view! {
                                <p class="text-xs font-semibold text-green-600 mt-0.5">{format!("{d}% OFF")}</p>
                            })}

                            <button class="mt-1.5 inline-block bg-blue-600 text-white px-3 py-1.5 rounded font-medium hover:bg-blue-700 text-xs w-full sm:w-auto">
                                "See Availability"
                            </button>
                        </div>
                    </Show>
                </div>
            </div>
        </div>
    }
}

#[component]
fn Wishlist(hotel_code: String) -> impl IntoView {
    let wishlist_hotel_code = hotel_code.clone();
    let add_to_wishlist_action = Action::new(move |_: &()| {
        let check_present =
            AuthStateSignal::check_if_added_to_wishlist_untracked(&wishlist_hotel_code);
        let toggle_action = if check_present { "remove" } else { "add" };
        AuthStateSignal::toggle_wishlish(wishlist_hotel_code.clone());
        let hotel_code = wishlist_hotel_code.clone();
        async move {
            let url = format!("/api/user-wishlist/{toggle_action}/{hotel_code}");
            match gloo_net::http::Request::post(&url).send().await {
                Ok(response) => {
                    if response.status() != 200 {
                        AuthStateSignal::toggle_wishlish(hotel_code.clone());
                    }
                }
                Err(_) => {
                    logging::log!("Failed to fetch user info");
                }
            }
        }
    });

    view! {
        <Show when=move || AuthStateSignal::auth_state().get().is_authenticated() >
            <button
                on:click=move |ev| {
                    ev.prevent_default();
                    ev.stop_propagation();
                    add_to_wishlist_action.dispatch(());
                }
                class="absolute top-3 left-3 bg-white rounded-full shadow-sm"
                aria-label="Add to wishlist"
            >
                {
                    let code = hotel_code.clone();
                    move || {
                        let is_wishlisted = AuthStateSignal::check_if_added_to_wishlist(&code);

                        let heart_d = "M19.62 27.81C19.28 27.93 18.72 27.93 18.38 27.81C15.48 26.82 9 22.69 9 15.69C9 12.6 11.49 10.1 14.56 10.1C16.38 10.1 17.99 10.98 19 12.34C20.01 10.98 21.63 10.1 23.44 10.1C26.51 10.1 29 12.6 29 15.69C29 22.69 22.52 26.82 19.62 27.81Z";

                        view! {
                            <svg width="38" height="38" viewBox="0 0 38 38" xmlns="http://www.w3.org/2000/svg">
                                <circle cx="19" cy="19" r="19" fill="white" />

                                // FILL heart (underneath)
                                <path
                                    d=heart_d
                                    fill="currentColor"
                                    stroke="none"
                                    class={
                                        if is_wishlisted {
                                            "text-red-500 group-hover:text-blue-600 transition-colors"
                                        } else {
                                            "text-transparent group-hover:text-blue-600 transition-colors"
                                        }
                                    }
                                />

                                // STROKE heart (outline, always gray)
                                <path
                                    d=heart_d
                                    fill="none"
                                    stroke="#45556C"
                                    stroke-width="2"
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    class={
                                        if is_wishlisted {
                                            "stroke-red-500 group-hover:stroke-blue-600 transition-colors"
                                        } else {
                                            "stroke-[#45556C] group-hover:stroke-blue-600 transition-colors"
                                        }
                                    }
                                />
                            </svg>
                        }
                    }
                }
            </button>
        </Show>
    }
}
