// use crate::log;
// use crate::{
//     api::{
//         book_room, canister::get_user_booking::get_user_booking_backend, BookRoomRequest,
//         BookRoomResponse, BookingDetails, BookingDetailsContainer, BookingStatus, HotelResult,
//         HotelSearchResponse, HotelSearchResult, Price, RoomDetail, Search, SuccessBookRoomResponse,
//     },
//     canister::backend,
//     component::SelectedDateRange,
//     page::{create_passenger_details, hotel_details, PaymentBookingStatusUpdates},
//     state::{
//         confirmation_results_state::ConfirmationResultsState,
//         hotel_details_state::RoomDetailsForPricingComponent,
//         local_storage::use_booking_id_store,
//         search_state::{
//             BlockRoomResults, ConfirmationResults, HotelInfoResults, SearchCtx, SearchListResults,
//         },
//         view_state::{BlockRoomCtx, HotelInfoCtx},
//     },
//     utils::app_reference,
// };
// use colored::Colorize;
// use leptos::*;
// use std::collections::HashMap;

// use super::SSEBookingStatusUpdates;

// pub fn read_booking_details_from_local_storage() -> Result<(String, String), String> {
//     let booking_id_signal_read = use_booking_id_store().0;
//     let reactive_closure_for_reading = move || booking_id_signal_read.get_untracked();

//     let (email, app_reference) = reactive_closure_for_reading()
//         .and_then(|booking| Some((booking.get_email(), booking.get_app_reference())))
//         .ok_or("Email and App Reference not valid in local_storage")?;

//     Ok((email, app_reference))
// }

// // pub fn read_booking_signal_from_local_storage() -> Result<(String, String), String> {
// //     let booking_id_signal_read = use_booking_id_store().0;
// //     let reactive_closure_for_reading = move || booking_id_signal_read.get_untracked();

// //     let (email, app_reference) = reactive_closure_for_reading()
// //         .and_then(|booking| Some((booking.get_email(), booking.get_app_reference())))
// //         .ok_or("Email and App Reference not valid in local_storage")?;

// //     Ok((email, app_reference))
// // }

// fn set_to_context_v2(email: String, app_reference: String, booking: backend::Booking) {
//     log!(
//         "{}",
//         format!(" set_to_context_v2 - {}, {}", email, app_reference)
//             .bright_blue()
//             .bold()
//     );
//     // todo [UAT] 2 : set hotel context from backend?? verify once
//     let date_range = SelectedDateRange {
//         start: booking.user_selected_hotel_room_details.date_range.start,
//         end: booking.user_selected_hotel_room_details.date_range.end,
//     };

//     ConfirmationResultsState::set_booking_details(Some(booking.clone()));
//     ConfirmationResultsState::set_date_range(date_range);
//     ConfirmationResultsState::set_selected_hotel_details(
//         booking
//             .user_selected_hotel_room_details
//             .hotel_details
//             .hotel_code,
//         booking
//             .user_selected_hotel_room_details
//             .hotel_details
//             .hotel_name,
//         booking
//             .user_selected_hotel_room_details
//             .hotel_details
//             .hotel_image,
//         booking
//             .user_selected_hotel_room_details
//             .hotel_details
//             .hotel_location,
//         booking
//             .user_selected_hotel_room_details
//             .hotel_details
//             .hotel_token,
//         booking
//             .user_selected_hotel_room_details
//             .hotel_details
//             .block_room_id,
//     );
//     let booking_guests = booking.guests.clone();
//     let booking_guests2 = booking.guests.clone();

//     let adults: Vec<crate::state::view_state::AdultDetail> = booking_guests.into();
//     let children: Vec<crate::state::view_state::ChildDetail> = booking_guests2.into();

//     ConfirmationResultsState::set_adults(adults);
//     ConfirmationResultsState::set_children(children);

//     let room_counts: HashMap<String, u32> = booking
//         .user_selected_hotel_room_details
//         .room_details
//         .iter()
//         .fold(HashMap::new(), |mut map, room| {
//             *map.entry(room.room_type_name.clone()).or_insert(0) += 1;
//             map
//         });

//     let sorted_rooms: Vec<RoomDetailsForPricingComponent> = booking
//         .user_selected_hotel_room_details
//         .room_details
//         .into_iter()
//         .map(|room| RoomDetailsForPricingComponent {
//             room_type: room.room_type_name.clone(),
//             room_count: room_counts.get(&room.room_type_name).cloned().unwrap_or(1),
//             room_unique_id: room.room_unique_id,
//             room_price: room.room_price as f64,
//         })
//         .collect();
//     ConfirmationResultsState::set_sorted_rooms(sorted_rooms);
// }

// #[component]
// pub fn SSEBookingHandler() -> impl IntoView {
//     let (booking_id_signal_read, set_booking_id_signal_read, _) = use_booking_id_store();
//     // let block_room_ctx = expect_context::<BlockRoomCtx>();
//     // let block_room = expect_context::<BlockRoomResults>();
//     let confirmation_ctx = expect_context::<ConfirmationResults>();
//     // let hotel_info_ctx = expect_context::<HotelInfoCtx>();

//     // let payment_booking_step_signals: PaymentBookingStatusUpdates = expect_context();
//     let payment_booking_step_signals = expect_context::<SSEBookingStatusUpdates>();
//     let booking_id_signal_read = use_booking_id_store().0;

//     let backend_booking_res = create_resource(
//         move || {
//             (
//                 payment_booking_step_signals.p03_fetch_booking_details.get(),
//                 booking_id_signal_read.get(),
//             )
//         },
//         move |(fetch_booking_details, booking_id)| async move {
//             log!("fetch_booking_details in create_resource - {fetch_booking_details:?}");

//             if booking_id.is_none() {
//                 return Err("Booking ID not found in local storage".to_string());
//             }
//             let booking_id = booking_id.unwrap();
//             let (email, app_reference) = (booking_id.get_email(), booking_id.get_app_reference());
//             // ================================ validate bookings ================================
//             let bookings_from_backend = get_user_booking_backend(email.clone())
//                 .await
//                 .map_err(|e| format!("Failed to fetch booking from backend: {}", e))?;

//             if bookings_from_backend.is_none() {
//                 return Err("No bookings present in backend".to_string());
//             }
//             let bookings = bookings_from_backend.unwrap();

//             log!("SSEBookingHandler - bookings - {bookings:#?}");
//             let found_booking_opt = bookings
//                 .into_iter()
//                 // .find(|b| b.booking_id == (app_reference.clone(), email.clone()));
//                 .find(|b| {
//                     b.booking_id.app_reference == app_reference && b.booking_id.email == email
//                 });

//             log!("SSEBookingHandler - found_booking_opt - {found_booking_opt:#?}");

//             if found_booking_opt.is_none() {
//                 return Err("Booking with given ID not in backend".to_string());
//             }

//             let found_booking = found_booking_opt.unwrap();
//             let found_booking_clone = found_booking.clone();

//             log!("calling set_to_context, {}, {}", email, app_reference);
//             set_to_context_v2(email, app_reference, found_booking);
//             payment_booking_step_signals
//                 .p04_load_booking_details_from_backend
//                 .set(true);
//             Ok(Some(found_booking_clone))
//         },
//     );

//     view! {
//         <Suspense>
//             {move || {
//                 backend_booking_res.get().map(|res| view! {
//                     <div>
//                     // {res}
//                     // {payment_booking_step_signals.p04_load_booking_details_from_backend.get()}
//                     </div>
//                 })
//             }}
//         </Suspense>
//     }
// }
