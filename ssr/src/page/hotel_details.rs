use crate::component::code_print::DebugDisplay;
use crate::component::loading_button::LoadingButton;
use crate::component::{
    FullScreenSpinnerGray, Navbar, PriceDisplayV2, SkeletonPricing, SpinnerGray,
};
use crate::state::hotel_details_state::{PricingBookNowState, RoomDetailsForPricingComponent};
use crate::utils::pluralize;
use crate::{
    api::block_room,
    app::AppRoutes,
    component::{Divider, FilterAndSortBy, PriceDisplay, StarRating},
    page::InputGroup,
    // state::room_state::{RoomQueryParams, RoomState},
    state::{
        api_error_state::{ApiErrorState, ApiErrorType},
        search_state::{BlockRoomResults, HotelInfoResults, SearchCtx},
        view_state::HotelInfoCtx,
    },
};
// use leptos::logging::log;
use crate::log;
use crate::state::GlobalStateForLeptos;
use leptos::*;
use leptos_icons::Icon;
use leptos_router::use_navigate;
use std::collections::{HashMap, HashSet};
use web_sys::MouseEvent;

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

    view! { description_signal }
}

fn convert_to_amenities(amenities: Vec<String>) -> Vec<Amenity> {
    amenities
        .into_iter()
        .take(8)
        .map(|text| Amenity {
            icon: icondata::IoWifiSharp,
            text: text.clone(),
        })
        .collect()
}

#[component]
pub fn HotelDetailsPage() -> impl IntoView {
    let hotel_info_results: HotelInfoResults = expect_context();

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
                                // <div class="absolute inset-0 bg-black bg-opacity-80 rounded-xl flex items-end p-4">
                                //     <span class="text-white text-lg font-semibold py-16 px-16">
                                //         See all photos
                                //     </span>
                                // </div>
                            </div>
                        </div>
                    </div>
                }
            }
        }
    }
}

// // #[derive(Debug, Clone)]
// // pub struct RoomCounterKeyValue {
// //     // number of rooms
// //     pub key: RwSignal<u32>,
// //     /// room_unique_id
// //     pub value: RwSignal<Option<String>>,
// // }

// // impl RoomCounterKeyValue {
// //     fn new() -> Self {
// //         Self::default()
// //     }
// // }

// // impl Default for RoomCounterKeyValue {
// //     fn default() -> Self {
// //         Self {
// //             key: create_rw_signal(0),
// //             value: create_rw_signal(None),
// //         }
// //     }
// // }

// // #[derive(Debug, Clone)]
// // pub struct RoomCounterKeyValueStatic {
// //     pub key: u32,
// //     pub value: Option<String>,
// // }

// impl From<RoomCounterKeyValue> for RoomCounterKeyValueStatic {
//     fn from(room_counter: RoomCounterKeyValue) -> Self {
//         let rkvs = RoomCounterKeyValueStatic {
//             key: room_counter.key.get_untracked(),
//             value: room_counter.value.get_untracked(),
//         };

//         log!("impl from {rkvs:#?}");
//         rkvs
//     }
// }

// #[derive(PartialEq, Debug, Default, Clone, serde::Serialize)]
// pub struct SortedRoom {
//     pub room_type: String,
//     pub room_unique_id: String,
//     pub room_count_for_given_type: u32,
//     pub room_price: f64,
// }

#[component]
pub fn PricingBookNow() -> impl IntoView {
    let search_ctx: SearchCtx = expect_context();
    let num_adults = Signal::derive(move || search_ctx.guests.get().adults.get());
    let num_rooms = Signal::derive(move || search_ctx.guests.get().rooms.get());

    let hotel_info_results: HotelInfoResults = expect_context();
    let hotel_info_results_clone = hotel_info_results.clone();
    // let pricing_book_now_state: PricingBookNowState = expect_context();

    // create_effect(move |_| {
    //     let _val = hotel_info_results_clone.room_result.track();
    //     log!("[diagnosis] hotel_info_results_clone.room_result called from create_effect");
    //     PricingBookNowState::set_rooms_available_for_booking_from_api();
    // });

    // let price = Signal::derive(move || total_room_price.get());
    let price = move || PricingBookNowState::total_room_price_for_all_user_selected_rooms();
    let num_nights = Signal::derive(move || search_ctx.date_range.get().no_of_nights());

    let is_room_available_from_api = move || PricingBookNowState::is_room_available_from_api();

    let total_rooms_selected_by_user =
        move || PricingBookNowState::total_count_of_rooms_selected_by_user();

    // room_counters
    let hotel_info_results: HotelInfoResults = expect_context();

    view! {
        <div class="flex flex-col space-y-4 shadow-lg rounded-xl border border-gray-200 p-8">
            <Show when=move || (price() > 0.0)>
                <PriceDisplayV2
                    price=move || price()
                    price_class="text-2xl font-semibold"
                />
            </Show>

            <div class="flex items-center space-x-2">
                <Icon icon=icondata::AiCalendarOutlined class="text-black text-xl" />
                <div>
                    {move || {
                        let search_ctx: SearchCtx = expect_context();
                        let date_range = search_ctx.date_range.get();
                        date_range.format_as_human_readable_date()
                    }}
                </div>
            </div>

            <div class="flex items-center space-x-2">
                <Icon icon=icondata::BsPerson class="text-black text-xl" />
                <div>{move || pluralize(num_adults.get(), "adult", "adults")}</div>
            </div>

            <div class="flex items-center space-x-2">
                <Icon icon=icondata::LuSofa class="text-black text-xl" />
                <div>{move || pluralize(num_rooms.get(), "room", "rooms")}</div>
            </div>

            <div class="flex flex-col space-y-2">
                <div class="font-semibold">Select room type:</div>

                // <DebugDisplay label="Sorted Rooms" value=move || format!("{:#?}", sorted_rooms.get()) />

                <Show when=is_room_available_from_api fallback=SkeletonPricing>
                    <For
                        each=move || PricingBookNowState::get().rooms_available_for_booking_from_api.get()
                        key=|room| room.clone()
                        let:room
                    >
                        {
                            // let (room_type, (room_count, room_unique_id, room_price)) = room;

                            let room_type = room.room_type;
                            let room_count = room.room_count;
                            let room_unique_id = room.room_unique_id;
                            let room_price = room.room_price;

                            view! {
                                <div class="flex justify-between items-center border-b border-gray-300 py-2">
                                    <span class="font-medium">
                                        {format!("{} - ${:.2}/night", room_type, room_price)}
                                    </span>
                                    <NumberCounterWrapperV2
                                        label=""
                                        // counter=counter
                                        class="mt-4"
                                        value=room_unique_id
                                        // set_value=value_signal
                                        // max_rooms=num_rooms.get_untracked()
                                        // total_selected_rooms=move || PricingBookNowState::total_count_of_rooms_selected_by_user()
                                        room_type=room_type
                                    />
                                </div>
                            }
                        }
                    </For>

                    <div>
                        {move || view!{
                            <PricingBreakdownV2
                            // room_type=room_type.clone()
                        />
                    }}
                    </div>
                </Show>
            </div>
        </div>
    }
}

#[component]
pub fn PricingBreakdownV2(// #[prop(into)] price_per_night: Signal<f64>,
    // #[prop(into)] number_of_nights: Signal<u32>,
    // #[prop(into)] room_counters: RwSignal<HashMap<String, RoomCounterKeyValue>>,
    // #[prop(into)] sorted_rooms: Vec<SortedRoom>,
) -> impl IntoView {
    let row_format_class = "flex justify-between";

    let search_ctx: SearchCtx = expect_context();
    let num_nights = move || search_ctx.date_range.get().no_of_nights();

    // let total_calc = move || price_per_night.get() * number_of_nights.get() as f64;
    let total_calc = move || {
        PricingBookNowState::total_room_price_for_all_user_selected_rooms() * (num_nights() as f64)
    };

    let navigate = use_navigate();
    let loading = create_rw_signal(true);

    // let hotel_info_results: HotelInfoResults = expect_context();
    // let hotel_info_ctx: HotelInfoCtx = expect_context();
    let is_booking = create_rw_signal(false);

    let block_room_action = create_action(move |_| {
        let nav = navigate.clone();
        let hotel_info_results: HotelInfoResults = expect_context();
        let loading_state = loading.clone();

        // let uniq_room_ids: Vec<String> = room_counters
        //     .get_untracked()
        //     .values()
        //     .filter_map(|counter| counter.value.get_untracked())
        //     .collect();

        // let sorted_rooms_clone = sorted_rooms.clone();

        async move {
            // Show loading state
            loading_state.set(true);

            // Reset previous block room results
            BlockRoomResults::reset();

            // Create block room request using HotelInfoResults
            let price_per_night =
                PricingBookNowState::total_room_price_for_all_user_selected_rooms();
            hotel_info_results.set_price_per_night(price_per_night);
            // hotel_info_results.set_block_room_counters(room_counters);

            // log!(
            //     "[pricing_component_not_loading] Sorted ROOMS FROM CONTEXT: {:?}",
            //     hotel_info_results.sorted_rooms.get()
            // );

            let uniq_room_ids = PricingBookNowState::unique_room_ids();
            let block_room_request = hotel_info_results.block_room_request(uniq_room_ids);

            // Call server function
            let result = block_room(block_room_request).await.ok();
            log!("[pricing_component_not_loading] BLOCK_ROOM_API: {result:?}");

            // Handle error if the API call failed
            if result.is_none() {
                // Get the API error state
                let api_error_state = ApiErrorState::from_leptos_context();

                // Set error message
                api_error_state.set_error(
                    ApiErrorType::BlockRoom,
                    "Selected room is already booked. Please choose another room.".to_string(),
                );

                // Update loading state
                loading_state.set(false);
                BlockRoomResults::reset();
                return;
            }

            // Set results and navigate
            BlockRoomResults::set_results(result);

            is_booking.set(false);
            // Navigate to block room page
            nav(AppRoutes::BlockRoom.to_string(), Default::default());
        }
    });

    view! {
        <div class="flex flex-col space-y-2 mt-4">
            <div class=row_format_class>

            // <DebugDisplay label="Total Price" value=move || format!("{:#?}", total_calc.get()) />
            // <DebugDisplay label="per_night_calc" value=move || format!("{:#?}", per_night_calc.get()) />

                <PriceDisplayV2
                    price=move || PricingBookNowState::total_room_price_for_all_user_selected_rooms()
                    appended_text=Some(format!(" x {} nights", num_nights()))
                    price_class=""
                    base_class="inline"
                    subtext_class="font-normal"
                />
                <div class="">
                    {move ||
                       view!{
                        <PriceDisplayV2
                            price=move || total_calc()
                            price_class=""
                            appended_text=Some("".into())
                        />
                       }
                    }
                </div>
            </div>

            // Total
            <div class=row_format_class>
                <div class="font-semibold">Total</div>
                <div class="flex-none">
                    {move ||
                        view!{
                            <PriceDisplayV2
                                price=move || total_calc()
                                appended_text=Some("".into())
                            />
                        }
                    }
                </div>
            </div>

            <div class="flex flex-col space-y-8">
                <div class="text-sm text-right font-semibold">
                    "Cryptocurrency payments accepted!"
                </div>
                <LoadingButton
                is_loading=is_booking.into()
                on_click=move |_| {
                    is_booking.set(true);
                    block_room_action.dispatch(())
                }
                loading_text="Booking..."
                >
                "Book Now"
                </LoadingButton>
            </div>
        </div>
    }
}

#[component]
pub fn NumberCounterWrapperV2(
    #[prop(into)] label: String,
    #[prop(default = "".into(), into)] class: String,
    // counter: RwSignal<u32>,
    /// RoomUniqueId passed as String
    value: String,
    /// This signal is used to store RoomUniqueId
    // set_value: RwSignal<Option<String>>,
    // max_rooms: u32,
    // total_selected_rooms: impl Fn() -> u32 + 'static,
    room_type: String,
) -> impl IntoView {
    let hotel_info_results: HotelInfoResults = expect_context();

    let search_ctx: SearchCtx = expect_context();
    // let num_adults = Signal::derive(move || search_ctx.guests.get().adults.get());
    let max_rooms_allowed = move || search_ctx.guests.get().rooms.get();

    let room_type_signal = store_value(room_type.clone());
    // let room_type_signal = create_rw_signal(room_type.clone());

    let counter = move || {
        PricingBookNowState::get_count_of_rooms_for_room_type(room_type_signal.get_value().clone())
    };
    let counter_clone = move || {
        PricingBookNowState::get_count_of_rooms_for_room_type(room_type_signal.get_value().clone())
    };

    let can_increment =
        move || PricingBookNowState::total_count_of_rooms_selected_by_user() < max_rooms_allowed();
    let can_decrement = move || counter() > 0;
    // let can_decrement_clone = move || counter() > 0;

    view! {
        <div class=format!("flex items-center justify-between {}", class)>
            <p>{label}</p>
            <div class="flex items-center space-x-1">
                <button
                    class="ps-2 py-1 text-2xl"
                    disabled=move || !can_decrement()
                    on:click=move |_| {
                        if PricingBookNowState::get_count_of_rooms_for_room_type(room_type_signal.get_value().clone()) > 0 {
                            PricingBookNowState::decrement_room_counter(room_type_signal.get_value().clone());
                        }
                    }
                >
                    {"\u{2003}\u{2003}\u{2003}\u{2003}-"}
                </button>
                <p class="text-center w-6">{ move || counter_clone()}</p>
                <button
                    class="py-1 text-2xl"
                    on:click=move |_: MouseEvent| {
                        if can_increment() {
                            PricingBookNowState::increment_room_counter(room_type_signal.get_value().clone());
                        }
                    }
                >
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
