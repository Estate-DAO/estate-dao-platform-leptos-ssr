use leptos::*;

use crate::{
    app::AppRoutes,
    component::{FilterAndSortBy, PriceDisplay, StarRating},
    page::{InputGroup, Navbar},
};

#[component]
pub fn HotelListPage() -> impl IntoView {
    let disabled_input_group = create_rw_signal(true);

    view! {
        <section class="relative h-screen">
            <Navbar />
            <div class="flex flex-col items-center mt-6 p-4">
                <InputGroup disabled=disabled_input_group />
                <FilterAndSortBy />
            </div>

            <div class="mx-auto">
                <div class="px-20 grid justify-items-center grid-cols-1 sm:grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                    {(0..30).map(|_| view! { <HotelCard /> }).collect::<Vec<_>>()}
                </div>
            </div>
        </section>
    }
}

#[component]
pub fn HotelCard() -> impl IntoView {
    let rating = create_rw_signal(4);
    let price = create_rw_signal(40500);
    view! {
        <a href=AppRoutes::HotelDetails.to_string()>
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
