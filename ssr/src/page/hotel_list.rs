use leptos::*;
use leptos_icons::Icon;

use crate::{
    component::{FilterAndSortBy, HSettingIcon},
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
        <div class="max-w-sm rounded-lg overflow-hidden shadow-sm border border-gray-300 bg-white">
            <img class="w-full h-64 object-cover" src="/img/home.webp" alt="Hotel Casa De Patio" />

            <div class="h-24">
                <div class="flex items-center justify-between px-6 pt-4">
                    <p>Hotel Casa De Papel</p>
                    <StarRating rating=rating />
                </div>

                <div class="flex items-center justify-between px-6 pt-2">
                    <PriceComponent price=price />
                    <ViewDetails />
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn PriceComponent(
    #[prop(into)] price: Signal<u32>,
    #[prop(default = "₹".to_string())] currency: String,
) -> impl IntoView {
    let formatted_price = move || {
        price()
            .to_string()
            .as_bytes()
            .rchunks(3)
            .rev()
            .map(std::str::from_utf8)
            .collect::<Result<Vec<&str>, _>>()
            .unwrap()
            .join(",")
    };

    view! {
        <div class="flex items-center space-x-1">
            <span class="font-semibold">{currency}{"\u{00A0}"}{formatted_price}</span>
            <span class="font-light text-sm">" /night"</span>
        </div>
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

#[component]
pub fn StarRating(#[prop(into)] rating: Signal<u8>) -> impl IntoView {
    view! {
        <div class="flex items-center space-x-2">
            <div class="flex items-center space-x-0.5 ">
                {(0..5)
                    .map(|i| {
                        let icon = move || {
                            if i < rating() { icondata::BiStarSolid } else { icondata::BiStarRegular }
                        };
                        view! { <Icon class="w-3 h-3 text-blue-500" icon=icon() /> }
                    })
                    .collect::<Vec<_>>()}
            </div>
            <div class="text-xs text-blue-500">{rating}.0</div>
        </div>
    }
}
