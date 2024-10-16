use leptos::*;

use crate::component::SkeletonCards;
use crate::{
    app::AppRoutes,
    component::{FilterAndSortBy, PriceDisplay, StarRating},
    page::{InputGroup, Navbar},
    state::{search_state::SearchListPage, view_state::HotelViewInfoCtx},
};
use leptos::logging::log;

#[component]
pub fn HotelListPage() -> impl IntoView {
    let disabled_input_group = create_rw_signal(true);
    let search_list_page: SearchListPage = expect_context();

    view! {
        <section class="relative h-screen">
            <Navbar />
            <div class="flex flex-col items-center mt-6 p-4">
                <InputGroup disabled=disabled_input_group />
                <FilterAndSortBy />
            </div>

            <div class="mx-auto">
                <div class="px-20 grid justify-items-center grid-cols-1 sm:grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                    <Show when={move|| search_list_page.search_result.get().is_some()} fallback={move || (1..10).map(|_| view!{<SkeletonCards />}).collect_view()}>
                     {move || {
                         search_list_page.search_result
                         .get()
                         .unwrap()
                         .hotel_results()
                         .iter()
                         .map(|hotel_result|
                                view!
                                { <HotelCard rating=hotel_result.star_rating price=hotel_result.price.room_price hotel_code=hotel_result.hotel_code.clone() /> }
                        ).collect::<Vec<_>>()
                    }}

                     </Show>
                </div>
            </div>
        </section>
    }
}

#[component]
pub fn HotelCard(rating: u8, price: f64, hotel_code: String) -> impl IntoView {
    // let rating = create_rw_signal(rating);
    let price = create_rw_signal(price);
    view! {
        <a href=AppRoutes::HotelDetails.to_string() on:click=move |ev| {
            ev.prevent_default();
            let hotel_view_info_ctx: HotelViewInfoCtx = expect_context();
            hotel_view_info_ctx.hotel_code.set(Some(hotel_code.clone()));
            log!("hotel_code: {}", hotel_code);
        }>
            <div class="max-w-sm rounded-lg overflow-hidden shadow-sm border border-gray-300 bg-white">
                <img
                    class="w-full h-64 object-cover"
                    src="/img/home.webp"
                    alt="Hotel Casa De Patio"
                />

                <div class="h-24">
                    <div class="flex items-center justify-between px-6 pt-4">
                        <p>Hotel Casa De Papel</p>
                        <StarRating rating=rating />
                    </div>

                    <div class="flex items-center justify-between px-6 pt-2">
                        <PriceDisplay price=price />
                        <ViewDetails />
                    </div>
                </div>
            </div>
        </a>
    }
}

#[component]
pub fn ViewDetails() -> impl IntoView {
    view! {
        <span class="font-semibold underline underline-offset-2	 decoration-solid text-xs">
            "View details"
        </span>
    }
}
