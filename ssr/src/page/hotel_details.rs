use crate::component::{FullScreenSpinnerGray, SpinnerGray};
use crate::utils::pluralize;
use crate::{
    api::block_room,
    app::AppRoutes,
    component::{Divider, FilterAndSortBy, PriceDisplay, StarRating},
    page::{InputGroup, Navbar},
    state::search_state::{BlockRoomResults, HotelInfoResults, SearchCtx},
    state::view_state::HotelInfoCtx,
};
use leptos::logging::log;
use leptos::*;
use leptos_icons::Icon;
use leptos_router::use_navigate;
use std::collections::HashMap;

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
                                    <span class="text-white text-lg font-semibold py-16 px-16">
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

#[derive(Debug, Clone)]
pub struct RoomCounterKeyValue {
    // counter room
    key: RwSignal<u32>,
    /// room_unique_id
    pub value: RwSignal<Option<String>>,
}

impl RoomCounterKeyValue {
    fn new() -> Self {
        Self::default()
    }
}

impl Default for RoomCounterKeyValue {
    fn default() -> Self {
        Self {
            key: create_rw_signal(0),
            value: create_rw_signal(None),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RoomCounterKeyValueStatic {
    pub key: u32,
    pub value: Option<String>,
}

impl From<RoomCounterKeyValue> for RoomCounterKeyValueStatic {
    fn from(room_counter: RoomCounterKeyValue) -> Self {
        let rkvs = RoomCounterKeyValueStatic {
            key: room_counter.key.get_untracked(),
            value: room_counter.value.get_untracked(),
        };

        log!("impl from {rkvs:#?}");
        rkvs
    }
}

#[derive(PartialEq, Debug, Default, Clone)]
pub struct SortedRoom {
    pub room_type: String,
    pub room_unique_id: String,
    pub room_count_for_given_type: u32,
    pub room_price: f64,
}

#[component]
pub fn PricingBookNow() -> impl IntoView {
    let search_ctx: SearchCtx = expect_context();
    let num_adults = Signal::derive(move || search_ctx.guests.get().adults.get());
    let num_rooms = Signal::derive(move || search_ctx.guests.get().rooms.get());

    let hotel_info_results: HotelInfoResults = expect_context();
    let hotel_info_results_clone = hotel_info_results.clone();

    // Create a memo for room details
    let room_details = create_memo(move |_| {
        let details = hotel_info_results_clone
            .get_hotel_room_details()
            .unwrap_or_default();
        log!("Room details: {:?}", details);
        details
    });

    // Create RwSignal for room counters map
    let room_counters = hotel_info_results.room_counters;
    // let room_counters = create_rw_signal(HashMap::<String, RoomCounterKeyValue>::new());

    let sorted_rooms_called = create_rw_signal(false);

    // Create a memo for the processed room data
    let sorted_rooms = create_memo(move |_| {
        // (String, u32, f64) = (RoomUniqueId, Room Count , Room Price)
        let mut room_count_map: HashMap<String, (String, u32, f64)> = HashMap::new();

        for room in room_details.get() {
            let room_type = room.room_type_name.to_string(); // Convert to owned String

            let entry = room_count_map
                .entry(room_type.clone())
                .or_insert(("".to_string(), 0, 0.0));
            entry.0 = room.room_unique_id;
            entry.1 += 1;
            entry.2 = room.price.offered_price as f64;

            room_counters.update(|counters| {
                if !counters.contains_key(&room_type) {
                    counters.insert(room_type, RoomCounterKeyValue::new());
                }
            });
        }

        let mut sorted: Vec<SortedRoom> = room_count_map
            .into_iter()
            .map(|(k, v)| SortedRoom {
                room_type: k,
                room_unique_id: v.0,
                room_count_for_given_type: v.1,
                room_price: v.2,
            })
            .collect();

        sorted.sort_by(|a, b| a.room_type.cmp(&b.room_type)); // Sort by room_type
        sorted.truncate(5);
        sorted_rooms_called.set(true);
        sorted
    });

    // Create a memo for total price calculation
    let total_room_price = create_memo(move |_| {
        sorted_rooms.get().iter().fold(
            0.0,
            |acc,
             SortedRoom {
                 room_type,
                 room_unique_id,
                 room_count_for_given_type,
                 room_price,
             }| {
                let counter = room_counters
                    .get()
                    .get(room_type.as_str()) // Change: use as_str() to get a string slice
                    .map(|sig| sig.key.get())
                    .unwrap_or(0);
                acc + (room_price * counter as f64)
            },
        )
    });

    let price = Signal::derive(move || total_room_price.get());
    let num_nights = Signal::derive(move || search_ctx.date_range.get().no_of_nights());

    let total_selected_rooms = create_rw_signal(
        room_counters
            .get()
            .values()
            .fold(0, |acc, counter| acc + counter.key.get()),
    );

    let room_counters_clone = room_counters.clone();

    view! {
        <div class="flex flex-col space-y-4 shadow-lg p-4 rounded-xl border border-gray-200 p-8">
            <Show when=move || (price.get() > 0.0)>
                <PriceDisplay price=price price_class="text-2xl font-semibold" />
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
                // <Show when=move || sorted_rooms_called.get() fallback=SpinnerGray>
                    <For
                        each=move || sorted_rooms.get()
                        key=|SortedRoom { room_type, .. }| room_type.clone()
                        let:room
                    >
                        {
                            let SortedRoom {
                                room_type,
                                room_unique_id,
                                room_count_for_given_type,
                                room_price,
                            } = room;
                            let base_counter = room_counters
                                .get()
                                .get(&room_type)
                                .cloned()
                                .unwrap_or_else(|| RoomCounterKeyValue::new());

                            view! {
                                <div class="flex justify-between items-center border-b border-gray-300 py-2">
                                    <span class="font-medium">
                                        // {format!("{} - ₹{:.2}/night", room_type,  room_price)}
                                        {format!("{} - ${:.2}/night", room_type, room_price)}
                                    </span>
                                    <NumberCounterWrapper
                                        label=""
                                        counter=base_counter.key
                                        class="mt-4"
                                        value=room_unique_id
                                        set_value=base_counter.value
                                        max_rooms=num_rooms.get_untracked()
                                        total_selected_rooms=total_selected_rooms
                                    />
                                </div>
                            }
                        }
                    </For>

                    <div>
                        <PricingBreakdown
                            price_per_night=price
                            number_of_nights=num_nights
                            room_counters=room_counters_clone
                            sorted_rooms=sorted_rooms.get()
                        />
                    </div>
                // </Show>
            </div>
        </div>
    }
}

#[component]
pub fn PricingBreakdown(
    #[prop(into)] price_per_night: Signal<f64>,
    #[prop(into)] number_of_nights: Signal<u32>,
    #[prop(into)] room_counters: RwSignal<HashMap<String, RoomCounterKeyValue>>,
    #[prop(into)] sorted_rooms: Vec<SortedRoom>,
) -> impl IntoView {
    let per_night_calc =
        create_memo(move |_| price_per_night.get() * number_of_nights.get() as f64);
    let total_calc = create_memo(move |_| per_night_calc.get());
    let row_format_class = "flex justify-between";

    let navigate = use_navigate();

    // let hotel_info_results: HotelInfoResults = expect_context();
    // let hotel_info_ctx: HotelInfoCtx = expect_context();

    let block_room_action = create_action(move |_| {
        let nav = navigate.clone();
        let hotel_info_results: HotelInfoResults = expect_context();

        let uniq_room_ids: Vec<String> = room_counters
            .get_untracked()
            .values()
            .filter_map(|counter| counter.value.get_untracked())
            .collect();

        let sorted_rooms_clone = sorted_rooms.clone();

        async move {
            // Reset previous block room results
            BlockRoomResults::reset();

            // Create block room request using HotelInfoResults
            // todo use room_counters
            hotel_info_results.set_price_per_night(price_per_night.get());
            // hotel_info_results.set_room_counters(room_counters.get());
            hotel_info_results.set_block_room_counters(room_counters.get());

            hotel_info_results.set_sorted_rooms(sorted_rooms_clone);

            let block_room_request = hotel_info_results.block_room_request(uniq_room_ids);

            // Call server function inside action
            spawn_local(async move {
                let result = block_room(block_room_request).await.ok();
                log!("BLOCK_ROOM_API: {result:?}");
                BlockRoomResults::set_results(result);
            });

            // Navigate to block room page
            nav(AppRoutes::BlockRoom.to_string(), Default::default());
        }
    });

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

            // Total
            <div class=row_format_class>
                <div class="font-semibold">Total</div>
                <div class="flex-none">
                    <PriceDisplay price=total_calc appended_text=Some("".into()) />
                </div>
            </div>

            <div class="flex flex-col space-y-8">
                <div class="text-sm text-right font-semibold">
                    "Cryptocurrency payments accepted!"
                </div>
                <button
                    class="w-full bg-blue-600 text-white py-3 rounded-full hover:bg-blue-800"
                    on:click=move |_| block_room_action.dispatch(())
                >
                    "Book Now"
                </button>
            </div>
        </div>
    }
}

#[component]
pub fn NumberCounterWrapper(
    #[prop(into)] label: String,
    #[prop(default = "".into(), into)] class: String,
    counter: RwSignal<u32>,
    /// RoomUniqueId passed as String
    value: String,
    /// This signal is used to store RoomUniqueId
    set_value: RwSignal<Option<String>>,
    max_rooms: u32,
    total_selected_rooms: RwSignal<u32>,
) -> impl IntoView {
    // Sets the value of the signal if the counter is non-zero
    create_effect(move |_| {
        if counter.get() > 0 {
            // log!("base_counter.value: {value:?}");
            set_value.set(Some(value.clone()));
        } else {
            set_value.set(None); // Or handle the zero case differently
        }
    });

    let can_increment = move || total_selected_rooms.get() < max_rooms;
    let can_decrement = move || counter.get() > 0;

    view! {
        <div class=format!("flex items-center justify-between {}", class)>
            <p>{label}</p>
            <div class="flex items-center space-x-1">
                <button
                    class="ps-2 py-1 text-2xl"
                    disabled=move || !can_decrement()
                    on:click=move |_| {
                        counter.update(|n| *n = if *n > 0 { *n - 1 } else { 0 });
                        total_selected_rooms.update(|n| *n = if *n > 0 { *n - 1 } else { 0 });
                    }
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
                <button
                    class="py-1 text-2xl"
                    on:click=move |_| {
                        if can_increment() {
                            let new_count = counter.get() + 1;
                            if new_count + (total_selected_rooms.get() - counter.get()) <= max_rooms
                            {
                                counter.set(new_count);
                                total_selected_rooms.update(|n| *n += 1);
                            } else {
                                log!("Cannot add more rooms. Maximum rooms reached.");
                            }
                        } else {
                            log!("Cannot add more rooms. Maximum rooms reached.");
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
