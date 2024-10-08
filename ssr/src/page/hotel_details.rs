use leptos::*;
use leptos_icons::Icon;

use crate::{
    component::{FilterAndSortBy, PriceDisplay, StarRating, Divider},
    page::{InputGroup, Navbar},
};

#[derive(Clone)]
struct Amenity {
    icon: icondata::Icon,
    text: &'static str,
}

// let icon_map = HashMap::from([
//     ("Free wifi", icondata::IoWifiSharp),
//     ("Free parking", icondata::LuParkingCircle),
//     ("Swimming pool", icondata::BiSwimRegular),
//     ("Spa", icondata::BiSpaRegular),
//     ("Private beach area", icondata::BsUmbrella),
//     ("Bar", icondata::IoWineSharp),
//     ("Family Rooms", icondata::RiHomeSmile2BuildingsLine),
// ]);


#[component]
pub fn HotelDetailsPage() -> impl IntoView {

    let rating = create_rw_signal(4);

    let amenities = vec![
        Amenity {
            icon: icondata::IoWifiSharp,
            text: "Free wifi",
        },
        Amenity {
            icon: icondata::LuParkingCircle,
            text: "Free parking",
        },
        Amenity {
            icon: icondata::BiSwimRegular,
            text: "Swimming pool",
        },
        Amenity {
            icon: icondata::BiSpaRegular,
            text: "Spa",
        },
        Amenity {
            icon: icondata::BsUmbrella,
            text: "Private beach area",
        },
        Amenity {
            icon: icondata::IoWineSharp,
            text: "Bar",
        },
        Amenity {
            icon: icondata::RiHomeSmile2BuildingsLine,
            text: "Family rooms",
        },
    ];
    // let amenities: Vec<Amenity> = amenity_texts.iter()
    // .filter_map(|text| icon_map.get(text.to_lowercase().as_str()).map(|&icon| Amenity { icon, text }))
    // .collect();

    view! {
        <section class="relative h-screen">
            <Navbar />
            <div class="flex flex-col items-center mt-6 p-4">
                <InputGroup />
                <FilterAndSortBy />
            </div>

            <div class="max-w-4xl mx-auto py-8">
                <div class="flex flex-col">
                    <StarRating rating=rating />
                    <div class="text-3xl font-semibold">Riva Beach Resort</div>
                </div>

                <br />
                <div class="space-y-3">
                    <div class="flex space-x-3 h-1/2 w-full">
                        <img
                            src="/img/home.webp"
                            alt="Destination"
                            class="w-3/5 h-[397px] rounded-xl"
                        />
                        <div class="space-y-3 w-2/5">
                            <img
                                src="/img/home.webp"
                                alt="Destination"
                                class="object-fill h-[193px] w-full rounded-xl"
                            />
                            <img
                                src="/img/home.webp"
                                alt="Destination"
                                class="object-fill h-[193px] w-full rounded-xl"
                            />
                        </div>
                    </div>
                    <div class="flex space-x-3">
                        <img
                            src="/img/home.webp"
                            alt="Destination"
                            class="w-[290px] h-1/3 rounded-xl"
                        />
                        <img
                            src="/img/home.webp"
                            alt="Destination"
                            class="w-[290px] h-1/3 rounded-xl"
                        />
                        <div class="relative w-[290px] h-1/3 rounded-xl">
                            <img
                                src="/img/home.webp"
                                alt="Destination"
                                class="object-cover h-full w-full rounded-xl"
                            />
                            <div class="absolute inset-0 bg-black bg-opacity-80 rounded-xl flex items-end p-4">
                                <span class="text-white text-lg font-semibold p-16">
                                    See all photos
                                </span>
                            </div>
                        </div>
                    </div>
                </div>

                // bottom half

                <div class="flex mt-8 space-x-2">

                    // left side div
                    <div class="basis-3/5">
                        // About component
                        <div class="flex flex-col space-y-4">
                            <div class="text-xl">About</div>
                            <div class="mb-8">
                                "As the sun dipped below the horizon, casting a golden glow across the tranquil lake, Sarah found herself lost in thought. The gentle lapping of waves against the shore provided a soothing backdrop to her contemplation. Suddenly, a rustling in the nearby bushes caught her attention. To her amazement, a magnificent deer emerged, its antlers silhouetted against the fading light. Their eyes met for a brief moment, a connection bridging the gap between human and nature. Just as quickly as it appeared, the deer vanished into the forest, leaving Sarah with a sense of wonder and a story she'd cherish forever. The encounter reminded her of life's unexpected beauty and the importance of being present in each moment."
                            </div>
                        </div>
                        <hr class="mt-14 mb-10 border-t border-gray-300" />

                        // amenities component
                        <div class=" flex flex-col space-y-8 mt-8">
                            <div class="text-xl">Amenities</div>
                            <div class="grid grid-cols-3 gap-4">
                                <For
                                    each=move || amenities.clone()
                                    key=|amenity| amenity.text
                                    let:amenity
                                >
                                    <AmenitiesIconText icon=amenity.icon text=amenity.text />
                                </For>
                            </div>
                        </div>
                    </div>

                    // right side div
                    <div class="basis-2/5">
                        // pricing component
                        // card component
                        <PricingBookNow />

                    </div>
                </div>
            </div>
        </section>
    }
}


#[component]
pub fn PricingBookNow () -> impl IntoView{
    let price = create_rw_signal(40500);
    let deluxe_counter = create_rw_signal(3_u32);
    let luxury_counter = create_rw_signal(0_u32);

    let num_nights = create_rw_signal(3_u32);
    let taxes_fees = create_rw_signal(1000_u32);

    view! {
        <div class="flex flex-col space-y-4 shadow-lg p-4 rounded-xl border border-gray-200 p-8">
            <PriceDisplay price=price price_class="text-2xl font-semibold" />

            <div class="flex items-center  space-x-2">
                <Icon icon=icondata::AiCalendarOutlined class="text-black  text-xl  " />
                <div>"Thu, Aug 22 -- Mon, Aug 27"</div>
            </div>

            <div class="flex items-center space-x-2">
                <Icon icon=icondata::BsPerson class="text-black text-xl" />
                <div>"4 adults"</div>
            </div>

            <div class="flex items-center  space-x-2">
                <Icon icon=icondata::LuSofa class="text-black text-xl " />
                <div>"2 rooms"</div>
            </div>

            <div class="flex flex-col space-y-2">
                <div class="font-semibold">Select room type:</div>
                <NumberCounter label="Deluxe Double Suite" counter=deluxe_counter class="mt-4" />
                <Divider />
                <NumberCounter label="Luxury Double Room" counter=luxury_counter />
                <Divider />
                <div>
                    <PricingBreakdown
                        price_per_night=price
                        number_of_nights=num_nights
                        taxes_fees
                    />
                </div>
            </div>

        </div>
    }
}


#[component]
pub fn PricingBreakdown(
    #[prop(into)] price_per_night: Signal<u32>,
    #[prop(into)] number_of_nights: Signal<u32>,
    #[prop(into)] taxes_fees: Signal<u32>,
) -> impl IntoView{

    let per_night_calc = create_memo(move |_| price_per_night.get() * number_of_nights.get());
    let total_calc = create_memo(move |_| per_night_calc.get() + taxes_fees.get());
    let row_format_class = "flex justify-between";
    view! {
        <div class="flex flex-col space-y-2 mt-4">
            <div class=row_format_class>

                <PriceDisplay
                    price=price_per_night
                    appended_text=Some(format!(" x {} nights", number_of_nights.get()))
                    price_class=""
                    base_class="inline"
                    subtext_class="font-normal"
                />
                <div class="">
                    <PriceDisplay
                        price=per_night_calc
                        price_class=""
                        appended_text=Some("".into())
                    />
                </div>
            </div>

            // taxes / fees
            <div class=row_format_class>
                <div>Taxes and fees</div>
                <div class="flex-none">

                    <PriceDisplay price=taxes_fees price_class="" appended_text=Some("".into()) />
                </div>
            </div>
            // Total
            <div class=row_format_class>
                <div class="font-semibold">Total</div>
                <div class="flex-none">

                    <PriceDisplay price=total_calc appended_text=Some("".into()) />
                </div>
            </div>

            <div class="flex flex-col space-y-8">
                <div class="text-sm text-right font-semibold">
                    Cryptocurrency payments accepted!
                </div>
                <button class="w-full bg-blue-600 text-white py-3 rounded-full hover:bg-blue-800">
                    Book Now
                </button>
            </div>
        </div>
    }
}


#[component]
pub fn NumberCounter(
    #[prop(into)] label: String,
    #[prop(default = "".into() , into)] class: String,
    counter: RwSignal<u32>,
) -> impl IntoView {

    let merged_class = format!("flex items-center justify-between {}",   class);

    view! {
        <div class=merged_class>
            <p>{label}</p>
            <div class="flex items-center space-x-1">
                <button
                    class="ps-2 py-1 text-2xl"
                    on:click=move |_| counter.update(|n| *n = (*n - 1).max(0))
                >
                    {"\u{2003}\u{2003}\u{2003}\u{2003}-"}
                </button>
                <input
                    type="number"
                    prop:value=move || counter.get().to_string()
                    on:input=move |ev| {
                        let value = event_target_value(&ev).parse().unwrap_or(0);
                        counter.set(value.max(0));
                    }
                    class=format!(
                        "{} text-center w-6",
                        "[appearance:textfield] [&::-webkit-outer-spin-button]:appearance-none [&::-webkit-inner-spin-button]:appearance-none ",
                    )
                />
                <button class="py-1 text-2xl " on:click=move |_| counter.update(|n| *n += 1)>
                    "+"
                </button>
            </div>
        </div>
    }
}

#[component]
pub fn AmenitiesIconText(icon: icondata::Icon, #[prop(into)] text: String) -> impl IntoView {
    view! {
        <div class="flex items-center">
            <Icon class="inline text-xl" icon=icon />
            <span class="inline ml-2">{text}</span>
        </div>
    }
}
