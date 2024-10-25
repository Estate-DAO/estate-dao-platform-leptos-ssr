use crate::component::FullScreenSpinnerGray;
use crate::utils::pluralize;
use crate::{
    component::{Divider, FilterAndSortBy, PriceDisplay, StarRating},
    page::{InputGroup, Navbar},
    state::search_state::{HotelInfoResults, SearchCtx},
    app::AppRoutes,
};
use leptos::logging::log;
use leptos::*;
use leptos_icons::Icon;
use svg::Image;

#[derive(Clone)]
struct Amenity {
    icon: icondata::Icon,
    text: String,
    // text: &'static str,
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
pub fn ShowHotelInfoValues() -> impl IntoView {
    let hotel_info_results: HotelInfoResults = expect_context();

    let description_signal = move || {
        if let Some(hotel_info_api_response) = hotel_info_results.search_result.get() {
            hotel_info_api_response.get_description()
        } else {
            "".to_owned()
        }
    };

    view! { {description_signal} }
}

// macro_rules! create_reactive_value {
//     ($name:ident, $hotel_info_results:ident, $getter:ident) => {
//         let $name = move || {
//             if let Some(hotel_info_api_response) = $hotel_info_results.search_result.get() {
//                 hotel_info_api_response.$getter()
//             } else {
//                 "".to_owned()
//             }
//         };
//     };
// }

fn convert_to_amenities(amenities: Vec<String>) -> Vec<Amenity> {
    amenities
        .into_iter()
        .map(|text| Amenity {
            icon: icondata::IoWifiSharp,
            text: text.clone(),
        })
        .collect()
}

#[component]
pub fn HotelDetailsPage() -> impl IntoView {
    let rating = 4;

    let hotel_info_results: HotelInfoResults = expect_context();

    create_effect(move |_| {
        log!(
            "hotel_info_results: {:?}",
            hotel_info_results.search_result.get()
        );
    });

    let address_signal = move || {
        if let Some(hotel_info_api_response) = hotel_info_results.search_result.get() {
            hotel_info_api_response.get_address()
        } else {
            "".to_owned()
        }
    };

    let description_signal = move || {
        if let Some(hotel_info_api_response) = hotel_info_results.search_result.get() {
            hotel_info_api_response.get_description()
        } else {
            "".to_owned()
        }
    };

    let amenities_signal = move || {
        let amenities_str =
            if let Some(hotel_info_api_response) = hotel_info_results.search_result.get() {
                hotel_info_api_response.get_amenities()
            } else {
                vec![]
            };

        convert_to_amenities(amenities_str)
    };

    let images_signal = move || {
        if let Some(hotel_info_api_response) = hotel_info_results.search_result.get() {
            hotel_info_api_response.get_images()
        } else {
            vec![]
        }
    };

    let hotel_name_signal = move || {
        if let Some(hotel_info_api_response) = hotel_info_results.search_result.get() {
            hotel_info_api_response.get_hotel_name()
        } else {
            "".into()
        }
    };

    let star_rating_signal = move || {
        if let Some(hotel_info_api_response) = hotel_info_results.search_result.get() {
            hotel_info_api_response.get_star_rating() as u8
        } else {
            0 as u8
        }
    };

    create_effect(move |_| {
        log!("images_signal: {:?}", images_signal());
    });

    let loaded = move || hotel_info_results.search_result.get().is_some();
    // create_reactive_value!( address_signal, hotel_info_results, get_address );
    // create_reactive_value!( description_signal, hotel_info_results, get_description );

    view! {
        <section class="relative h-screen">
            <Navbar />
            <div class="flex flex-col items-center mt-6 p-4">
                <InputGroup />
            // <FilterAndSortBy />
            </div>
            <Show when=loaded fallback=FullScreenSpinnerGray>
                <div class="max-w-4xl mx-auto py-8">
                    <div class="flex flex-col">
                        {move || view! { <StarRating rating=star_rating_signal /> }}
                        <div class="text-3xl font-semibold">{hotel_name_signal}</div>
                    </div>

                    <br />
                    // <div class="flex space-x-3 h-1/2 w-full">
                    <div class="space-y-3">

                        <HotelImages />
                    </div>

                    // bottom half

                    <div class="flex mt-8 space-x-2">

                        // left side div
                        <div class="basis-3/5">
                            // About component
                            <div class="flex flex-col space-y-4">
                                <div class="text-xl">About</div>
                                <div class="mb-8">{description_signal}</div>
                            </div>
                            <hr class="mt-14 mb-5 border-t border-gray-300" />
                            // Address bar component
                            <div class=" flex flex-col space-y-8 mt-8">
                                <div class="text-xl">Address</div>
                                <div>{address_signal}</div>
                            </div>
                            <hr class="mt-14 mb-5 border-t border-gray-300" />
                            // amenities component
                            <div class=" flex flex-col space-y-8 mt-8">
                                <div class="text-xl">Amenities</div>
                                <div class="grid grid-cols-3 gap-4">
                                    <For
                                        each=amenities_signal
                                        key=|amenity| amenity.text.clone()
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
            </Show>
        </section>
    }
}

#[component]
pub fn HotelImages() -> impl IntoView {
    let hotel_info_results: HotelInfoResults = expect_context();

    let images_signal = move || {
        if let Some(hotel_info_api_response) = hotel_info_results.search_result.get() {
            let mut images = hotel_info_api_response.get_images();
            if images.len() < 6 {
                let repeat_count = 6 - images.len();
                let repeated_images = images.clone();
                images.extend(repeated_images.into_iter().take(repeat_count));
            }
            images
        } else {
            vec![]
        }
    };

    {
        move || {
            if images_signal().is_empty() {
                view! { <div>No images</div> }
            } else {
                view! {
                    <div class="flex flex-col space-y-3">
                        <div class="flex space-x-3  space-y-2 h-1/2 w-full">
                            <img
                                src=move || images_signal()[0].clone()
                                alt="Destination"
                                class="w-3/5 h-96 rounded-xl"
                            />
                            <div class=" flex flex-col space-y-3 w-2/5">
                                <img
                                    src=move || images_signal()[1].clone()
                                    alt="Destination"
                                    class="object-fill h-[186px] w-full rounded-xl"
                                />
                                <img
                                    src=move || images_signal()[2].clone()
                                    alt="Destination"
                                    class="object-fill h-[186px] w-full rounded-xl"
                                />
                            </div>
                        </div>
                        <div class="flex justify-between space-x-3">
                            <img
                                src=move || images_signal()[3].clone()
                                alt="Destination"
                                class="w-72 h-48 rounded-xl"
                            />
                            <img
                                src=move || images_signal()[4].clone()
                                alt="Destination"
                                class="w-72 h-48 rounded-xl"
                            />
                            <div class="relative w-72 h-48 rounded-xl">
                                <img
                                    src=move || images_signal()[5].clone()
                                    alt="Destination"
                                    class="object-cover h-full w-full rounded-xl"
                                />
                                <div class="absolute inset-0 bg-black bg-opacity-80 rounded-xl flex items-end p-4">
                                    <span class="text-white text-lg font-semibold p-16 h-24">
                                        See all photos
                                    </span>
                                </div>
                            </div>
                        </div>
                    </div>
                }
            }
        }
    }
}

#[component]
pub fn PricingBookNow() -> impl IntoView {
    let hotel_info_results: HotelInfoResults = expect_context();
    let search_ctx: SearchCtx = expect_context();
    let num_rooms = Signal::derive(move || search_ctx.guests.get().rooms.get());

    let price = Signal::derive(move || {
        if let Some(hotel_info_api_response) = hotel_info_results.search_result.get() {
            hotel_info_api_response.get_room_price()
        } else {
            0.0
        }
    });

    // let price = create_rw_signal(40500.9);
    let deluxe_counter = create_rw_signal(3_u32);
    let luxury_counter = create_rw_signal(0_u32);

    let num_nights = create_rw_signal(3_u32);
    let taxes_fees = create_rw_signal(1000_f64);

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
                <div>{move || pluralize(num_rooms.get(), "room", "rooms")}</div>
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
    #[prop(into)] price_per_night: Signal<f64>,
    #[prop(into)] number_of_nights: Signal<u32>,
    #[prop(into)] taxes_fees: Signal<f64>,
) -> impl IntoView {
    let per_night_calc =
        create_memo(move |_| price_per_night.get() * number_of_nights.get() as f64);
    let total_calc = create_memo(move |_| per_night_calc.get() + taxes_fees.get() as f64);
    let row_format_class = "flex justify-between";
    
    
    // let hotel_info_page: HotelDetails = expect_context();
    // // let search_list_page_clone = search_list_page.clone();

    // let navigate = use_navigate();

    // // let hotel_code_cloned = hotel_code.clone();

    // let search_hotel_info_action = create_action(move |_| {
    //     let nav = navigate.clone();
    //     let search_list_page = search_list_page.clone();
    //     let hotel_code = hotel_code.clone();
    //     log!("from action -- {search_list_page:?}");
    //     log!("from action -- {hotel_code:?}");
    //     async move {
    //         //  move to the hotel info page
    //         nav(AppRoutes::BlockRoom.to_string(), Default::default());

    //         HotelInfoResults::reset();

    //         let hotel_info_request = search_list_page.hotel_info_request(&hotel_code);
    //         log!("{hotel_info_request:?}");

    //         // call server function inside action
    //         spawn_local(async move {
    //             let result = hotel_info(hotel_info_request).await.ok();
    //             log!("SEARCH_HOTEL_API: {result:?}");
    //             HotelInfoResults::set_info_results(result);
    //         });
    //     }
    // });
    
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
                <button 
                    class="w-full bg-blue-600 text-white py-3 rounded-full hover:bg-blue-800"
                    // on:click=move |ev| {
                    //     ev.prevent_default();
                    //     let hotel_view_info_ctx: HotelInfoCtx = expect_context();
                    //     hotel_view_info_ctx.hotel_code.set(Some(hotel_code_cloned.clone()));
                    //     log!("hotel_code: {}", hotel_code_cloned);
                    //     search_hotel_room_action.dispatch(());
                    //     search_hotel_info_action.dispatch(())
                    // }
                >
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
    let merged_class = format!("flex items-center justify-between {}", class);

    view! {
        <div class=merged_class>
            <p>{label}</p>
            <div class="flex items-center space-x-1">
                <button
                    class="ps-2 py-1 text-2xl"
                    on:click=move |_| counter.update(|n| *n = if *n > 0 { *n - 1 } else { 0 })
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
