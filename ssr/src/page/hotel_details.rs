// // use crate::api::provab::BlockRoomResponse;
// use crate::component::code_print::DebugDisplay;
// use crate::component::loading_button::LoadingButton;
// use crate::component::{
//     FullScreenSpinnerGray, GymIcon, Navbar, NavigatingErrorPopup, PriceDisplayV2, SkeletonPricing,
//     SpinnerGray,
// };
// use crate::page::InputGroupContainer;
// use crate::view_state_layer::hotel_details_state::{PricingBookNowState, RoomDetailsForPricingComponent};
// use crate::view_state_layer::input_group_state::{InputGroupState, OpenDialogComponent};
// use crate::utils::pluralize;
// use crate::{
//     // api::block_room,
//     app::AppRoutes,
//     component::{Divider, FilterAndSortBy, PriceDisplay, StarRating},
//     page::InputGroup,
//     // state::room_state::{RoomQueryParams, RoomState},
//     view_state_layer::{
//         api_error_state::{ApiErrorState, ApiErrorType},
//         // search_state::{BlockRoomResults, HotelInfoResults, SearchCtx},
//         // view_state::HotelInfoCtx,
//     },
// };
// // use leptos::logging::log;
// use crate::log;
// use crate::view_state_layer::GlobalStateForLeptos;
// use leptos::*;
// use leptos_icons::Icon;
// use leptos_router::use_navigate;
// use std::collections::{HashMap, HashSet};
// use web_sys::MouseEvent;

// #[derive(Clone)]
// struct Amenity {
//     icon: icondata::Icon,
//     text: String,
// }

// // let icon_map = HashMap::from([
// //     ("Free wifi", icondata::IoWifiSharp),
// //     ("Free parking", icondata::LuParkingCircle),
// //     ("Swimming pool", icondata::BiSwimRegular),
// //     ("Spa", icondata::BiSpaRegular),
// //     ("Private beach area", icondata::BsUmbrella),
// //     ("Bar", icondata::IoWineSharp),
// //     ("Family Rooms", icondata::RiHomeSmile2BuildingsLine),
// // ]);

// #[component]
// pub fn ShowHotelInfoValues() -> impl IntoView {
//     let hotel_info_results: HotelInfoResults = expect_context();

//     let description_signal = move || {
//         if let Some(hotel_info_api_response) = hotel_info_results.search_result.get() {
//             hotel_info_api_response.get_description()
//         } else {
//             "".to_owned()
//         }
//     };

//     view! { description_signal }
// }

// fn convert_to_amenities(amenities: Vec<String>) -> Vec<Amenity> {
//     // Define the icon mappings directly in the function
//     let icon_mappings = [
//         ("wifi", icondata::IoWifi),
//         ("parking", icondata::LuParkingCircle),
//         ("pool", icondata::BiSwimRegular),
//         ("swim", icondata::BiSwimRegular),
//         ("spa", crate::component::SpaIcon),
//         ("gym", crate::component::GymIcon),
//         ("terrace", crate::component::TerraceIcon),
//         ("roomservice", crate::component::RoomServiceIcon),
//         ("pet", crate::component::PetIcon),
//         ("laundry", crate::component::LaundryIcon),
//         ("bar", crate::component::BarIcon),
//         ("pub", crate::component::BarIcon),
//         ("beach", icondata::BsUmbrella),
//         ("umbrella", icondata::BsUmbrella),
//         ("family", icondata::RiHomeSmile2BuildingsLine),
//         ("room", icondata::RiHomeSmile2BuildingsLine),
//     ];

//     amenities
//         .into_iter()
//         .take(8)
//         .map(|text| {
//             let lower_text = text.to_lowercase();
//             // Find the first matching icon or default to wifi icon
//             let icon = icon_mappings
//                 .iter()
//                 .find(|(key, _)| lower_text.contains(*key))
//                 .map(|(_, icon)| *icon)
//                 .unwrap_or(icondata::IoWifi);

//             // Truncate text to 10 characters
//             let display_text = if text.len() > 10 {
//                 let mut s = text[..10].to_string();
//                 s.push('…'); // Add ellipsis to indicate truncation
//                 s
//             } else {
//                 text
//             };

//             Amenity {
//                 icon,
//                 text: display_text,
//             }
//         })
//         .collect()
// }

// #[component]
// pub fn HotelDetailsPage() -> impl IntoView {
//     PricingBookNowState::reset_room_counters();
//     let hotel_info_results: HotelInfoResults = expect_context();

//     let address_signal = move || {
//         if let Some(hotel_info_api_response) = hotel_info_results.search_result.get() {
//             hotel_info_api_response.get_address()
//         } else {
//             "".to_owned()
//         }
//     };

//     let description_signal = move || {
//         if let Some(hotel_info_api_response) = hotel_info_results.search_result.get() {
//             hotel_info_api_response.get_description()
//         } else {
//             "".to_owned()
//         }
//     };

//     let amenities_signal = move || {
//         let amenities_str =
//             if let Some(hotel_info_api_response) = hotel_info_results.search_result.get() {
//                 hotel_info_api_response.get_amenities()
//             } else {
//                 vec![]
//             };

//         convert_to_amenities(amenities_str)
//     };

//     let images_signal = move || {
//         if let Some(hotel_info_api_response) = hotel_info_results.search_result.get() {
//             hotel_info_api_response.get_images()
//         } else {
//             vec![]
//         }
//     };

//     let hotel_name_signal = move || {
//         if let Some(hotel_info_api_response) = hotel_info_results.search_result.get() {
//             hotel_info_api_response.get_hotel_name()
//         } else {
//             "".into()
//         }
//     };

//     let star_rating_signal = move || {
//         if let Some(hotel_info_api_response) = hotel_info_results.search_result.get() {
//             hotel_info_api_response.get_star_rating() as u8
//         } else {
//             0 as u8
//         }
//     };

//     let loaded = move || hotel_info_results.search_result.get().is_some();

//     view! {
//         <section class="relative min-h-screen bg-gray-50">
//             <Navbar />
//             <div class="flex flex-col items-center mt-6 p-4">
//                 <InputGroupContainer default_expanded=false  allow_outside_click_collapse=true />
//             // <FilterAndSortBy />
//             </div>
//             <Show when=loaded fallback=FullScreenSpinnerGray>
//                 <div class="w-full max-w-4xl mx-auto py-4 px-2 md:py-8 md:px-0">
//                     <div class="flex flex-col">
//                         {move || view! { <StarRating rating=star_rating_signal /> }}
//                         <div class="text-2xl md:text-3xl font-semibold">{hotel_name_signal}</div>
//                     </div>

//                     <br />
//                     // <div class="flex space-x-3 h-1/2 w-full">
//                     <div class="mt-4 md:mt-6">
//                         <HotelImages />
//                     </div>

//                     <div class="flex flex-col md:flex-row mt-6 md:mt-8 md:space-x-4">
//                         // {/* Left side */}
//                         <div class="w-full md:w-3/5 flex flex-col space-y-6">
//                             // {/* About Card */}
//                             <div class="bg-white rounded-xl shadow-md p-6 mb-2">
//                                 <div class="text-xl mb-2 font-semibold">About</div>
//                                 <div class="mb-2 text-gray-700">{move || clip_to_30_words(&description_signal())}</div>
//                             </div>
//                             // {/* Address Card */}
//                             <div class="bg-white rounded-xl shadow-md p-6 mb-2">
//                                 <div class="text-xl mb-2 font-semibold">Address</div>
//                                 <div class="text-gray-700">{address_signal}</div>
//                             </div>
//                             // {/* Amenities Card */}
//                             <div class="bg-white rounded-xl shadow-md p-6 mb-2">
//                                 <div class="text-xl mb-4 font-semibold">Amenities</div>
//                                 <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-4">
//                                     <For
//                                         each=amenities_signal
//                                         key=|amenity| amenity.text.clone()
//                                         let:amenity
//                                     >
//                                         <AmenitiesIconText icon=amenity.icon text=amenity.text />
//                                     </For>
//                                 </div>
//                             </div>
//                         </div>
//                         // {/* Right side */}
//                         <div class="w-full md:w-2/5 mt-8 md:mt-0 flex flex-col">
//                             // {/* Pricing Card */}
//                             <div class="bg-white rounded-xl shadow-md mb-2">
//                                 <PricingBookNow />
//                             </div>
//                         </div>
//                     </div>
//                 </div>
//             </Show>
//         </section>
//     }
// }

// fn clip_to_30_words(text: &str) -> String {
//     let words: Vec<&str> = text.split_whitespace().collect();
//     if words.len() <= 30 {
//         text.to_string()
//     } else {
//         let clipped = words[..30].join(" ");
//         format!("{}...", clipped)
//     }
// }

// #[component]
// pub fn HotelImages() -> impl IntoView {
//     let hotel_info_results: HotelInfoResults = expect_context();

//     let images_signal = move || {
//         if let Some(hotel_info_api_response) = hotel_info_results.search_result.get() {
//             let mut images = hotel_info_api_response.get_images();
//             if images.len() < 6 {
//                 let repeat_count = 6 - images.len();
//                 let repeated_images = images.clone();
//                 images.extend(repeated_images.into_iter().take(repeat_count));
//             }
//             images
//         } else {
//             vec![]
//         }
//     };

//     {
//         move || {
//             if images_signal().is_empty() {
//                 view! { <div>No images</div> }
//             } else {
//                 view! {
//                     <div>
//                         <div class="block sm:hidden">
//                             <img
//                                 src=move || images_signal()[0].clone()
//                                 alt="Destination"
//                                 class="w-full h-64 rounded-xl object-cover"
//                             />
//                         </div>
//                         <div class="hidden sm:flex flex-col space-y-3">
//                             <div class="flex flex-col sm:flex-row space-y-3 sm:space-y-0 sm:space-x-3">
//                                 <img
//                                     src=move || images_signal()[0].clone()
//                                     alt="Destination"
//                                     class="w-full sm:w-3/5 h-64 sm:h-96 rounded-xl object-cover"
//                                 />
//                                 <div class="flex flex-row sm:flex-col space-x-3 sm:space-x-0 sm:space-y-3 w-full sm:w-2/5">
//                                     <img
//                                         src=move || images_signal()[1].clone()
//                                         alt="Destination"
//                                         class="w-1/2 sm:w-full h-32 sm:h-[186px] rounded-xl object-cover sm:object-fill"
//                                     />
//                                     <img
//                                         src=move || images_signal()[2].clone()
//                                         alt="Destination"
//                                         class="w-1/2 sm:w-full h-32 sm:h-[186px] rounded-xl object-cover sm:object-fill"
//                                     />
//                                 </div>
//                             </div>
//                             <div class="flex justify-between space-x-3">
//                                 <img
//                                     src=move || images_signal()[3].clone()
//                                     alt="Destination"
//                                     class="w-72 h-48 rounded-xl"
//                                 />
//                                 <img
//                                     src=move || images_signal()[4].clone()
//                                     alt="Destination"
//                                     class="w-72 h-48 rounded-xl"
//                                 />
//                                 <div class="relative w-72 h-48 rounded-xl">
//                                     <img
//                                         src=move || images_signal()[5].clone()
//                                         alt="Destination"
//                                         class="object-cover h-full w-full rounded-xl"
//                                     />
//                                     // <div class="absolute inset-0 bg-black bg-opacity-80 rounded-xl flex items-end p-4">
//                                     //     <span class="text-white text-lg font-semibold py-16 px-16">
//                                     //         See all photos
//                                     //     </span>
//                                     // </div>
//                                 </div>
//                             </div>
//                         </div>
//                     </div>
//                 }
//             }
//         }
//     }
// }

// // // #[derive(Debug, Clone)]
// // // pub struct RoomCounterKeyValue {
// // //     // number of rooms
// // //     pub key: RwSignal<u32>,
// // //     /// room_unique_id
// // //     pub value: RwSignal<Option<String>>,
// // // }

// // // impl RoomCounterKeyValue {
// // //     fn new() -> Self {
// // //         Self::default()
// // //     }
// // // }

// // // impl Default for RoomCounterKeyValue {
// // //     fn default() -> Self {
// // //         Self {
// // //             key: create_rw_signal(0),
// // //             value: create_rw_signal(None),
// // //         }
// // //     }
// // // }

// // // #[derive(Debug, Clone)]
// // // pub struct RoomCounterKeyValueStatic {
// // //     pub key: u32,
// // //     pub value: Option<String>,
// // // }

// // impl From<RoomCounterKeyValue> for RoomCounterKeyValueStatic {
// //     fn from(room_counter: RoomCounterKeyValue) -> Self {
// //         let rkvs = RoomCounterKeyValueStatic {
// //             key: room_counter.key.get_untracked(),
// //             value: room_counter.value.get_untracked(),
// //         };

// //         log!("impl from {rkvs:#?}");
// //         rkvs
// //     }
// // }

// // #[derive(PartialEq, Debug, Default, Clone, serde::Serialize)]
// // pub struct SortedRoom {
// //     pub room_type: String,
// //     pub room_unique_id: String,
// //     pub room_count_for_given_type: u32,
// //     pub room_price: f64,
// // }

// #[component]
// pub fn PricingBookNow() -> impl IntoView {
//     let search_ctx: SearchCtx = expect_context();
//     let num_adults = Signal::derive(move || search_ctx.guests.get().adults.get());
//     let num_rooms = Signal::derive(move || search_ctx.guests.get().rooms.get());

//     let hotel_info_results: HotelInfoResults = expect_context();
//     let hotel_info_results_clone = hotel_info_results.clone();
//     // let pricing_book_now_state: PricingBookNowState = expect_context();

//     // create_effect(move |_| {
//     //     let _val = hotel_info_results_clone.room_result.track();
//     //     log!("[diagnosis] hotel_info_results_clone.room_result called from create_effect");
//     //     let room_details = hotel_info_results_clone.get_hotel_room_details();
//     //     PricingBookNowState::set_rooms_available_for_booking_from_api(room_details);
//     // });

//     // let price = Signal::derive(move || total_room_price.get());
//     let price = move || PricingBookNowState::total_room_price_for_all_user_selected_rooms();
//     let num_nights = Signal::derive(move || search_ctx.date_range.get().no_of_nights());

//     // let is_room_available_from_api = move || PricingBookNowState::is_room_available_from_api();

//     let total_rooms_selected_by_user =
//         move || PricingBookNowState::total_count_of_rooms_selected_by_user();

//     // room_counters
//     // let hotel_info_results: HotelInfoResults = expect_context();

//     view! {
//         <div class="flex flex-col space-y-4 shadow-lg rounded-xl p-4 md:p-8">
//             <Show when=move || (price() > 0.0)>
//                 <PriceDisplayV2
//                     price=move || price()
//                     price_class="text-xl md:text-2xl font-semibold"
//                 />
//             </Show>

//             <div class="flex items-center space-x-2">
//                 <Icon icon=icondata::AiCalendarOutlined class="text-black text-lg md:text-xl" />
//                 <div>
//                     {move || {
//                         let search_ctx: SearchCtx = expect_context();
//                         let date_range = search_ctx.date_range.get();
//                         date_range.format_as_human_readable_date()
//                     }}
//                 </div>
//             </div>

//             <div class="flex items-center space-x-2">
//             <Icon icon=icondata::BsPerson class="text-black text-lg md:text-xl" />
//             <div>{move || pluralize(num_adults.get(), "adult", "adults")}</div>
//             </div>

//             <div class="flex items-center space-x-2">
//             <Icon icon=icondata::LuSofa class="text-black text-lg md:text-xl" />
//             <div>{move || pluralize(num_rooms.get(), "room", "rooms")}</div>
//             </div>

//             <div class="flex flex-col space-y-2">
//                 <div class="font-semibold">Select room type:</div>

//                 // <DebugDisplay label="Sorted Rooms" value=move || format!("{:#?}", sorted_rooms.get()) />

//                 <Show when=move || PricingBookNowState::is_room_available_from_api() fallback=SkeletonPricing>
//                     <For
//                         each=move || PricingBookNowState::list_rooms_in_pricing_component()
//                         // each=move || PricingBookNowState::get().rooms_available_for_booking_from_api.get()
//                         key=|room| room.clone()
//                         let:room
//                     >
//                         {
//                             // let (room_type, (room_count, room_unique_id, room_price)) = room;

//                             let room_type = room.room_type;
//                             let room_count = room.room_count;
//                             let room_unique_id = room.room_unique_id;
//                             let room_price = room.room_price;

//                             view! {
//                                 <div class="flex flex-row items-start justify-between border-b border-gray-300 py-2">
//                                     // <!-- Robust wrap: flex-1 min-w-0 for text, flex-shrink-0 for counter, items-start for top align -->
//                                     <p class="w-0 flex-1 min-w-0 font-medium text-sm md:text-base break-words whitespace-normal">
//                                         {format!("{} - ${:.2}/night", room_type, room_price)}
//                                     </p>
//                                     <div class="flex-shrink-0">
//                                         <NumberCounterWrapperV2
//                                             label=""
//                                             // counter=counter
//                                             class="mt-2 md:mt-4"
//                                             value=room_unique_id
//                                             // set_value=value_signal
//                                             // max_rooms=num_rooms.get_untracked()
//                                             // total_selected_rooms=move || PricingBookNowState::total_count_of_rooms_selected_by_user()
//                                             room_type=room_type
//                                         />
//                                     </div>
//                                 </div>
//                             }
//                         }
//                     </For>

//                     <div>
//                         {move || view!{
//                             <PricingBreakdownV2
//                             // room_type=room_type.clone()
//                         />
//                     }}
//                     </div>
//                 </Show>
//             </div>
//         </div>
//     }
// }

// #[component]
// pub fn PricingBreakdownV2(// #[prop(into)] price_per_night: Signal<f64>,
//     // #[prop(into)] number_of_nights: Signal<u32>,
//     // #[prop(into)] room_counters: RwSignal<HashMap<String, RoomCounterKeyValue>>,
//     // #[prop(into)] sorted_rooms: Vec<SortedRoom>,
// ) -> impl IntoView {
//     let row_format_class = "flex flex-row justify-between items-center w-full";

//     let search_ctx: SearchCtx = expect_context();
//     let num_nights = move || search_ctx.date_range.get().no_of_nights();

//     let total_room_price = create_memo(|_| {
//         let pricing_book_state: PricingBookNowState = expect_context();
//         pricing_book_state.room_counters_as_chosen_by_user.track();
//         let valv = PricingBookNowState::total_room_price_for_all_user_selected_rooms();
//         log!(
//             "total_room_price_for_all_user_selected_rooms - PricingBreakdownV2 - create_memo - {}",
//             valv
//         );

//         valv
//     });

//     // let total_calc = move || price_per_night.get() * number_of_nights.get() as f64;
//     let total_calc = move || {
//         let valv = total_room_price.get() * (num_nights() as f64);
//         log!("total_calc - PricingBreakdownV2 - create_memo {}", valv);
//         valv
//     };

//     let navigate = use_navigate();
//     let loading = create_rw_signal(true);

//     // let hotel_info_results: HotelInfoResults = expect_context();
//     // let hotel_info_ctx: HotelInfoCtx = expect_context();
//     let is_booking = create_rw_signal(false);

//     let block_room_action = create_action(move |_| {
//         let nav = navigate.clone();
//         let hotel_info_results: HotelInfoResults = expect_context();
//         let loading_state = loading.clone();

//         // let uniq_room_ids: Vec<String> = room_counters
//         //     .get_untracked()
//         //     .values()
//         //     .filter_map(|counter| counter.value.get_untracked())
//         //     .collect();

//         // let sorted_rooms_clone = sorted_rooms.clone();

//         async move {
//             // Show loading state
//             loading_state.set(true);

//             // Reset previous block room results
//             BlockRoomResults::reset();

//             // Create block room request using HotelInfoResults
//             let price_per_night =
//                 PricingBookNowState::total_room_price_for_all_user_selected_rooms();
//             hotel_info_results.set_price_per_night(price_per_night);
//             // hotel_info_results.set_block_room_counters(room_counters);

//             // log!(
//             //     "[pricing_component_not_loading] Sorted ROOMS FROM CONTEXT: {:?}",
//             //     hotel_info_results.sorted_rooms.get()
//             // );

//             let uniq_room_ids = PricingBookNowState::room_unique_ids();
//             let block_room_request = hotel_info_results.block_room_request(uniq_room_ids);

//             // Call server function
//             let result = block_room(block_room_request).await.ok();
//             log!("[pricing_component_not_loading] BLOCK_ROOM_API: {result:?}");

//             // Get the API error state and handle any errors
//             let api_error_state = ApiErrorState::from_leptos_context();
//             if api_error_state.handle_block_room_response(result.clone(), None) {
//                 loading_state.set(false);
//                 BlockRoomResults::reset();
//                 return;
//             }

//             // Set results and navigate
//             BlockRoomResults::set_results(result);

//             is_booking.set(false);
//             // Navigate to block room page
//             nav(AppRoutes::BlockRoom.to_string(), Default::default());
//             // close all the dialogs on navigation
//             InputGroupState::toggle_dialog(OpenDialogComponent::None);
//         }
//     });

//     let can_book_now_memo = create_memo(move |_| total_room_price.get() > 0.001);
//     let cannot_book_now = Signal::derive(move || !can_book_now_memo.get());

//     view! {
//         <div class="flex flex-col space-y-2 mt-4 px-2 sm:px-0">
//             // <!-- Per-night breakdown row: label left, price right, always aligned -->
//             <div class="flex flex-row justify-between items-center w-full py-2">

//                 <NavigatingErrorPopup
//                     route="/"
//                     label="Go to Home"
//                     error_type=ApiErrorType::BlockRoom
//                 />
//                 // <!-- Room price and nights, always left aligned -->
//                 <PriceDisplayV2
//                     price=move || PricingBookNowState::total_room_price_for_all_user_selected_rooms()
//                     appended_text=Some(format!(" x {} nights", num_nights()))
//                     price_class="font-normal text-sm sm:text-base"
//                     base_class=""
//                     subtext_class="font-normal"
//                 />
//                 <div class="mt-0 text-right">
//                     {move ||
//                        view!{
//                         <PriceDisplayV2
//                             price=move || total_calc()
//                             price_class="font-normal text-sm sm:text-base"
//                             base_class=""
//                             subtext_class="font-normal"
//                             appended_text=Some("".into())
//                 />
//                        }
//                     }
//                 </div>
//             </div>
//             // <!-- Total row, visually separated and aligned -->
//             <div class="flex flex-row justify-between items-center border-t border-gray-200 pt-4 mt-2 w-full">
//                 <div class="font-semibold text-base sm:text-lg">Total</div>
//                 <div class="text-right font-semibold text-lg sm:text-xl">
//                     {move ||
//                         view!{
//                     <PriceDisplayV2
//                         price=move || total_calc()
//                         price_class=""
//                         appended_text=Some("".into())
//                     />
//                         }
//                     }
//                 </div>
//             </div>
//             // <DebugDisplay label="cannot_book_now" value=move || cannot_book_now.get().to_string() />
//             // <DebugDisplay label="total_calc" value=move || total_calc().to_string() />
//             // <!-- Payment info and Book Now button, centered and spaced -->
//             <div class="flex flex-col space-y-4 mt-4 items-center">
//                 <div class="text-sm sm:text-base font-semibold text-center">
//                     "Cryptocurrency payments accepted!"
//                 </div>
//                 <LoadingButton
//                 is_loading=is_booking.into()
//                 on_click=move |_| {
//                     is_booking.set(true);
//                     block_room_action.dispatch(())
//                 }
//                 loading_text="Booking..."
//                 class="w-full sm:w-full px-4 py-2 text-base sm:text-lg"
//                 disabled=cannot_book_now
//                 >
//                 "Book Now"
//                 </LoadingButton>
//             </div>
//         </div>
//     }
// }

// #[component]
// pub fn NumberCounterWrapperV2(
//     #[prop(into)] label: String,
//     #[prop(default = "".into(), into)] class: String,
//     // counter: RwSignal<u32>,
//     /// RoomUniqueId passed as String
//     value: String,
//     /// This signal is used to store RoomUniqueId
//     // set_value: RwSignal<Option<String>>,
//     // max_rooms: u32,
//     // total_selected_rooms: impl Fn() -> u32 + 'static,
//     room_type: String,
// ) -> impl IntoView {
//     let hotel_info_results: HotelInfoResults = expect_context();

//     let search_ctx: SearchCtx = expect_context();
//     // let num_adults = Signal::derive(move || search_ctx.guests.get().adults.get());
//     let max_rooms_allowed = move || search_ctx.guests.get().rooms.get();

//     let room_type_signal = store_value(room_type.clone());
//     // let room_type_signal = create_rw_signal(room_type.clone());

//     // let counter = move || {
//     //     PricingBookNowState::get_count_of_rooms_for_room_type(room_type_signal.get_value().clone())
//     // };
//     let counter = create_rw_signal(PricingBookNowState::get_count_of_rooms_for_room_type(
//         room_type_signal.get_value().clone(),
//     ));

//     // let counter_clone = move || {
//     //     PricingBookNowState::get_count_of_rooms_for_room_type(room_type_signal.get_value().clone())
//     // };
//     // let counter_clone = create_rw_signal(PricingBookNowState::get_count_of_rooms_for_room_type(room_type_signal.get_value().clone()));

//     let can_increment =
//         move || PricingBookNowState::total_count_of_rooms_selected_by_user() < max_rooms_allowed();
//     let can_decrement = move || counter.get() > 0;
//     // let can_decrement_clone = move || counter() > 0;

//     let label_clone = label.clone();

//     view! {
//         <div class=format!("flex items-center justify-between {}", class)>
//             <Show when=move || !label.is_empty()>
//                 <p>{label_clone.clone()}</p>
//             </Show>
//             <div class="flex items-center space-x-1">
//                 <button
//                     class="ps-2 py-1 text-2xl"
//                     disabled=move || !can_decrement()
//                     on:click=move |_| {
//                         if PricingBookNowState::get_count_of_rooms_for_room_type(room_type_signal.get_value().clone()) > 0 {
//                             log!("can_decrement");
//                             counter.update(|n| *n -= 1);
//                             PricingBookNowState::decrement_room_counter(room_type_signal.get_value().clone());
//                         }
//                     }
//                 >
//                     {"\u{2003}\u{2003}\u{2003}\u{2003}-"}
//                 </button>
//                 <p class="text-center w-6">{ move || counter.get()}</p>
//                 <button
//                     class="py-1 text-2xl"
//                     on:click=move |_: MouseEvent| {
//                         if can_increment() {
//                             log!("can_increment");
//                             counter.update(|n| *n += 1);
//                             PricingBookNowState::increment_room_counter(room_type_signal.get_value().clone());
//                         }
//                     }
//                 >
//                     "+"
//                 </button>
//             </div>
//         </div>
//     }
// }

// #[component]
// pub fn AmenitiesIconText(icon: icondata::Icon, #[prop(into)] text: String) -> impl IntoView {
//     view! {
//         <div class="flex items-center">
//             <Icon class="inline text-xl" icon=icon />
//             <span class="inline ml-2">{text}</span>
//         </div>
//     }
// }

use leptos::*;

#[component]
pub fn HotelDetailsPage() -> impl IntoView {
    view! {
        <div>
            <h1>Hotel Details</h1>
        </div>
    }
}
