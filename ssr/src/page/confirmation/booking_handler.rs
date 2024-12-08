use crate::{
    api::{
        book_room, canister::get_user_booking::get_user_booking_backend, BookRoomRequest,
        RoomDetail,
    },
    canister::backend,
    page::{confirmation::confirmation::PaymentBookingStatusUpdates, create_passenger_details},
    state::{
        local_storage::use_booking_id_store,
        search_state::{BlockRoomResults, ConfirmationResults, SearchCtx},
        view_state::{BlockRoomCtx, HotelInfoCtx},
    },
    utils::app_reference,
};
use colored::Colorize;
use leptos::*;
// use crate::cprintln;

fn read_booking_details_from_local_storage() -> Result<(String, String), String> {
    let booking_id_signal_read = use_booking_id_store().0;
    let reactive_closure_for_reading = move || booking_id_signal_read.get_untracked();

    let (email, app_reference) = reactive_closure_for_reading()
        .and_then(|booking| Some((booking.get_email(), booking.get_app_reference())))
        .ok_or("Email and App Reference not valid in local_storage")?;

    Ok((email, app_reference))
}

fn set_to_context(found_booking: backend::Booking) {
    // let found_booking_clone = found_booking.clone();
    let block_room_ctx = expect_context::<BlockRoomCtx>();
    let block_room = expect_context::<BlockRoomResults>();
    let confirmation_ctx = expect_context::<ConfirmationResults>();
    let hotel_info_ctx = expect_context::<HotelInfoCtx>();

    let payment_booking_step_signals: PaymentBookingStatusUpdates = expect_context();

    let date_range = crate::component::SelectedDateRange {
        start: found_booking
            .user_selected_hotel_room_details
            .date_range
            .start,
        end: found_booking
            .user_selected_hotel_room_details
            .date_range
            .end,
    };

    SearchCtx::set_date_range(date_range);
    HotelInfoCtx::set_selected_hotel_details(
        found_booking
            .user_selected_hotel_room_details
            .hotel_details
            .hotel_code,
        found_booking
            .user_selected_hotel_room_details
            .hotel_details
            .hotel_name,
        found_booking
            .user_selected_hotel_room_details
            .hotel_details
            .hotel_image,
        found_booking
            .user_selected_hotel_room_details
            .hotel_details
            .hotel_location,
    );
    BlockRoomResults::set_id(Some(
        found_booking
            .user_selected_hotel_room_details
            .hotel_details
            .block_room_id,
    ));
}

#[component]
pub fn BookingHandler() -> impl IntoView {
    let (booking_id_signal_read, set_booking_id_signal_read, _) = use_booking_id_store();
    let block_room_ctx = expect_context::<BlockRoomCtx>();
    let block_room = expect_context::<BlockRoomResults>();
    let confirmation_ctx = expect_context::<ConfirmationResults>();
    let hotel_info_ctx = expect_context::<HotelInfoCtx>();

    let payment_booking_step_signals: PaymentBookingStatusUpdates = expect_context();

    let backend_booking_res = create_resource(
        move || booking_id_signal_read.get(),
        move |booking_id_signal_read| async move {
            println!("booking_id_signal_read in create_resource - {booking_id_signal_read:?}");

            let details_from_local_storage = match read_booking_details_from_local_storage() {
                Ok(details) => Some(details),
                Err(err) => {
                    println!(
                        "{}",
                        format!("should_fetch_from_canister - Err - {} ", err).red()
                    );
                    None
                }
            };

            if details_from_local_storage.is_some() {
                let (email, app_reference) = details_from_local_storage.unwrap();

                // ================================ validate bookings ================================
                let bookings_from_backend = get_user_booking_backend(email.clone())
                    .await
                    .map_err(|e| format!("Failed to fetch booking: ServerFnError =  {}", e))?;

                if bookings_from_backend.is_none() {
                    return Err("No bookings present in backend.".to_string());
                }
                let bookings = bookings_from_backend.unwrap();

                let found_booking_opt = bookings
                    .into_iter()
                    .find(|b| b.booking_id == (app_reference.clone(), email.clone()));

                if found_booking_opt.is_none() {
                    return Err("Booking with given ID not in backend.".to_string());
                }

                let found_booking = found_booking_opt.unwrap();
                let found_booking_clone = found_booking.clone();
                set_to_context(found_booking);
                Ok(Some(found_booking_clone))
            } else {
                log::info!("not fetch_from_canister");
                Ok(None)
            }
        },
    );

    // Handle booking action
    // let booking_action = create_action(move |_| {
    //     let app_reference = booking_id_signal_read
    //         .get_untracked()
    //         .and_then(|booking| Some(booking.get_app_reference()))
    //         .unwrap_or_default();

    //     let block_room = expect_context::<BlockRoomResults>();
    //     let block_room_id = block_room.block_room_id.get().unwrap_or_default();
    //     let block_room_ctx = expect_context::<BlockRoomCtx>();
    //     let adults = block_room_ctx.adults.get();
    //     let children = block_room_ctx.children.get();
    //     let room_detail = RoomDetail {
    //         passenger_details: create_passenger_details(&adults, &children),
    //     };

    //     let result_token = hotel_info_ctx.hotel_token.get_untracked();

    //     async move {
    //         spawn_local(async move {
    //             let req = BookRoomRequest {
    //                 result_token,
    //                 block_room_id,
    //                 app_reference,
    //                 room_details: vec![room_detail],
    //             };
    //             let value_for_serverfn = serde_json::to_string(&req).unwrap();

    //             match book_room(value_for_serverfn).await {
    //                 Ok(booking_result) => {
    //                     confirmation_ctx.booking_details.set(Some(booking_result));
    //                     confirmation_ctx.booking_error.set(None);
    //                 }
    //                 Err(err) => {
    //                     confirmation_ctx
    //                         .booking_error
    //                         .set(Some(format!("Booking failed: {}", err)));
    //                 }
    //             }
    //         });
    //     }
    // });

    // // Watch payment confirmation to trigger booking
    // create_effect(move |_| {
    //     if confirmation_ctx.payment_confirmed.get() {
    //         booking_action.dispatch(());
    //     }
    // });

    view! {
        <div class="booking-status-container">
        <Suspense fallback={move || view!{ "Cannot read booking_id from local_storage. Did you clear your cookies?"}}>
        {move ||

            if let Some(booking) = backend_booking_res.get(){
            let booking_id_signal_read_data = booking_id_signal_read.get();

                view!{
                    {format!("booking - {booking:#?}")}
                    <p>
                    {format!("booking_id_signal_read_data - {booking_id_signal_read_data:?}")}
                    </p>
                }.into_view()
            } else {
                let any_else = backend_booking_res.get();

                view!{
                {
                    {format!("else - {any_else:#?}")}
                }
            }.into_view()}
        }
        </Suspense>
            // {move || {
            //     let booking_details = confirmation_ctx.booking_details.get();
            //     let booking_error = confirmation_ctx.booking_error.get();

            //     match (booking_details, booking_error) {
            //         (Some(details), _) => {
            //             view! {
            //                 <div class="booking-success">
            //                     <h3>"Booking Confirmed!"</h3>
            //                     <div class="booking-details">
            //                         <p>{"Booking Reference: "} {details.commit_booking.booking_details.booking_ref_no}</p>
            //                         <p>{"Confirmation Number: "} {details.commit_booking.booking_details.confirmation_no}</p>
            //                         <p>{"Status: "} {details.commit_booking.booking_details.booking_status}</p>
            //                     </div>
            //                 </div>
            //             }
            //         },
            //         (None, Some(error)) => {
            //             view! {
            //                 <div class="booking-error">
            //                     <p>{"Error: "} {error}</p>
            //                 </div>
            //             }
            //         },
            //         (None, None) => {
            //             if confirmation_ctx.payment_confirmed.get() {
            //                 view! {
            //                     <div class="booking-pending">
            //                         "Processing your booking..."
            //                     </div>
            //                 }
            //             } else {
            //                 view! {
            //                     <div class="booking-waiting">
            //                         "Waiting for payment confirmation..."
            //                     </div>
            //                 }
            //             }
            //         }
            //     }
            // }}
        </div>
    }
}
