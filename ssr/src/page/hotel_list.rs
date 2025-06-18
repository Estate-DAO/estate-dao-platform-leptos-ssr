use leptos::*;
use leptos_router::use_navigate;

// use crate::api::get_room;
use crate::component::{Navbar, SkeletonCards};
use crate::log;
use crate::page::{HotelListParams, InputGroupContainer};
use crate::utils::query_params::QueryParamsSync;
use crate::view_state_layer::input_group_state::{InputGroupState, OpenDialogComponent};
use crate::view_state_layer::ui_hotel_details::HotelDetailsUIState;
use crate::view_state_layer::ui_search_state::{SearchListResults, UISearchCtx};
use crate::view_state_layer::view_state::HotelInfoCtx;
// use crate::state::input_group_state::{InputGroupState, OpenDialogComponent};
// use crate::state::search_state::HotelInfoResults;
use crate::{
    // api::hotel_info,
    app::AppRoutes,
    component::{FilterAndSortBy, PriceDisplay, StarRating},
    page::InputGroup,
    // state::{search_state::SearchListResults, view_state::HotelInfoCtx},
};

#[component]
pub fn HotelListPage() -> impl IntoView {
    let search_ctx: UISearchCtx = expect_context();
    let navigate = use_navigate();
    let query_map = leptos_router::use_query_map();

    // Sync query params with state on page load (URL → State)
    // This leverages use_query_map's built-in reactivity for browser navigation
    // create_effect(move |_| {
    //     let params = query_map.get();
    //     if !params.0.is_empty() {
    //         log!("Found query params in URL: {:?}", params);

    //         if let Some(hotel_params) = HotelListParams::from_url_params(&params.0.into_iter().collect()) {
    //             log!("Parsed hotel params from URL: {:?}", hotel_params);
    //             hotel_params.sync_to_app_state();
    //         }
    //     }
    // });

    // Example: Manual URL updates (State → URL) when user performs actions
    // This function can be called from search form submissions, filter changes, etc.
    let update_url_with_current_state = move || {
        let current_params = HotelListParams::from_search_context(&search_ctx);
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

                <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3 sm:gap-4">

                    <Show
                        when=move || search_list_page.search_result.get().is_some()
                        fallback=fallback
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

    let navigate = use_navigate();

    let hotel_code_cloned = hotel_code.clone();

    let search_hotel_info_action = create_action(move |_| {
        let nav = navigate.clone();
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
    let search_hotel_room_action = create_action(move |_| {
        let search_list_page = search_list_page_clone.clone();
        let hotel_code = hotel_code_2_cloned.clone();
        async move {
            // let hotel_room_request = search_list_page.hotel_room_request(&hotel_code);
            // call server function inside action
            spawn_local(async move {
                // todo (uncomment)
                // let result = get_room(hotel_room_request).await.ok();
                // log!("SEARCH_ROOM_API: {result:?}");
                // HotelInfoResults::set_room_results(result);
            });
        }
    });

    let hotel_view_info_ctx: HotelInfoCtx = expect_context();

    view! {
        <div // href=AppRoutes::HotelDetails.to_string()
        on:click=move |ev| {
            ev.prevent_default();
            ev.stop_propagation();

            // Set hotel code in context for hotel details page
            hotel_view_info_ctx.hotel_code.set(hotel_code_cloned.clone());
            log!("hotel_code: {}", hotel_code_cloned);

            // Reset hotel details state
            HotelDetailsUIState::reset();

            // Dispatch actions to fetch data
            search_hotel_room_action.dispatch(());
            search_hotel_info_action.dispatch(())
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
