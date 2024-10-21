use leptos::*;
use leptos_router::use_navigate;

use crate::component::SkeletonCards;
use crate::state::search_state::HotelInfoResults;
use crate::{
    api::hotel_info,
    app::AppRoutes,
    component::{FilterAndSortBy, PriceDisplay, StarRating},
    page::{InputGroup, Navbar},
    state::{search_state::SearchListResults, view_state::HotelInfoCtx},
};
use leptos::logging::log;

#[component]
pub fn HotelListPage() -> impl IntoView {
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

    let fallback = move || (1..10).map(|_| view! { <SkeletonCards /> }).collect_view();

    view! {
        <section class="relative h-screen">
            <Navbar />
            <div class="flex flex-col items-center mt-6 p-4">
                <InputGroup disabled=disabled_input_group />
                <FilterAndSortBy />
            </div>

            <div class="mx-auto">
                <div class="px-20 grid justify-items-center grid-cols-1 sm:grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">

                    <Show
                        when=move || search_list_page.search_result.get().is_some()
                        fallback=fallback
                    >
                        // <Transition fallback=fallback>
                            {move || {
                                search_list_page
                                    .search_result
                                    .get()
                                    .unwrap()
                                    .hotel_results()
                                    .iter()
                                    .map(|hotel_result| {
                                        view! {
                                            <HotelCard
                                                img=hotel_result.hotel_picture.clone()
                                                rating=hotel_result.star_rating
                                                hotel_name=hotel_result.hotel_name.clone()
                                                price=hotel_result.price.room_price
                                                hotel_code=hotel_result.hotel_code.clone()
                                            />
                                        }
                                    })
                                    .collect::<Vec<_>>()
                            }}

                        // </Transition>
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
) -> impl IntoView {

    let price = create_rw_signal(price);

    let search_list_page: SearchListResults = expect_context();

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

            HotelInfoResults::reset();

            let hotel_info_request = search_list_page.hotel_info_request(hotel_code);
            log!("{hotel_info_request:?}"); 

            // call server function inside action
            spawn_local(async move {
                let result = hotel_info(hotel_info_request).await.ok();
                log!("SEARCH_HOTEL_API: {result:?}");
                HotelInfoResults::set_info_results(result);
            });
        }
    });

    view! {
        <div
            // href=AppRoutes::HotelDetails.to_string()
            on:click=move |ev| {
                ev.prevent_default();
                // todoing -- 
                let hotel_view_info_ctx: HotelInfoCtx = expect_context();
                hotel_view_info_ctx.hotel_code.set(Some(hotel_code_cloned.clone()));
                log!("hotel_code: {}", hotel_code_cloned);
                search_hotel_info_action.dispatch(())
            }
        >
            <div class="max-w-sm rounded-lg overflow-hidden shadow-sm border border-gray-300 bg-white">
                <img class="w-80 h-64 object-cover" src=img alt=hotel_name.clone() />

                <div class="h-24">
                    <div class="flex items-center justify-between px-6 pt-4">
                        <p>
                            {if hotel_name.len() > 10 {
                                format!("{}...", &hotel_name[..10])
                            } else {
                                hotel_name.clone()
                            }}
                        </p>
                        <StarRating rating={move || rating} />
                    </div>

                    <div class="flex items-center justify-between px-6 pt-2">
                        <PriceDisplay price=price />
                        <button 
                            class="font-semibold underline underline-offset-2 decoration-solid text-xs"
                        >
                            "View details"
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}

