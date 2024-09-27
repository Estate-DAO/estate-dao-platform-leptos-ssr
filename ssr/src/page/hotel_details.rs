use leptos::*;
use leptos_icons::Icon;

use crate::{component::HSettingIcon, page::{InputGroup, Navbar}};

#[component]
pub fn HotelDetailsPage() -> impl IntoView {
    view! {
        <section class="relative h-screen">
            <Navbar />
            
            <div class="mt-40">
            <div class="flex flex-col items-center justify-center h-full">
                <div class="flex space-x-4 mb-8">
                    <InputGroup />
                </div>
                <div class="flex space-x-4">
                    <button class="bg-white text-black px-4 py-2 rounded-lg flex items-center ">
                        <Icon class="w-5 h-5 mr-2" icon=HSettingIcon />
                        Filter
                    </button>
                    <button class="bg-white text-black px-4 py-2 rounded-lg flex items-start">
                        Sort By <Icon icon=icondata::BiChevronDownRegular class="w-5 h-5" />
                    </button>
                </div>
            </div>
        </div>

            <div class="px-52 grid grid-cols-1 sm:grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                {(0..30).map(|_| view! { <HotelCard /> }).collect::<Vec<_>>()}
            </div>
        // <div class="px-8">
        // <HotelCard />
        // </div>
        </section>
    }
}


#[component]
pub fn HotelCard() -> impl IntoView{
    let rating = create_rw_signal(4);
    let price = create_rw_signal(40500);
    view! {
        <div class="max-w-sm rounded-lg overflow-hidden shadow-sm border border-gray-300 bg-white">
            <img class="w-full h-48 object-cover" src="/img/home.webp" alt="Hotel Casa De Patio" />
            <div class="flex items-center justify-between px-4 py-2">
                <p>Hotel Casa De Papel</p>
                <StarRating rating=rating />

            </div>

            <div class="flex items-center justify-between px-4 pb-2">
                <PriceComponent price=price />
                <ViewDetails />

            </div>
        </div>
    }
}


#[component]
pub fn PriceComponent(
    #[prop(into)]
    price: Signal<u32>,
    #[prop(default = "₹".to_string())]
    currency: String,
) -> impl IntoView {
    let formatted_price = move || {
        price().to_string()
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
            <span class="font-semibold">{currency}{formatted_price}</span>
            <span class="font-light text-sm">" /night"</span>
        </div>
    }
}

#[component]
pub fn ViewDetails() -> impl IntoView{
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
