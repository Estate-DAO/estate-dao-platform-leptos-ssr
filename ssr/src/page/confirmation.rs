use codee::string::JsonSerdeCodec;
use leptos::*;
use leptos_router::use_navigate;
use leptos_use::{storage::use_local_storage, use_interval_fn, utils::Pausable};

use crate::{
    api::{
        book_room,
        canister::{
            get_user_booking::get_user_booking_backend,
            update_payment_details::update_payment_details_backend,
        },
        hotel_info,
        payments::{nowpayments_get_payment_status, ports::GetPaymentStatusRequest},
        BookRoomRequest, PassengerDetail, RoomDetail,
    },
    app::AppRoutes,
    canister::backend::{
        BackendPaymentStatus::{Paid, Unpaid},
        PaymentDetails,
    },
    component::{
        Divider, FilterAndSortBy, GuestSelection, PriceDisplay, SelectedDateRange, StarRating,
    },
    page::{create_passenger_details, InputGroup, Navbar},
    state::{
        local_storage::{use_booking_id_store, use_payment_store},
        search_state::{
            BlockRoomResults, ConfirmationResults, HotelInfoResults, SearchCtx, SearchListResults,
        },
        view_state::{BlockRoomCtx, HotelInfoCtx},
    },
    utils::app_reference::BookingId,
};
use chrono::NaiveDate;
use leptos::logging::log;
use leptos_router::*;

#[derive(Params, PartialEq, Clone)]
struct NowpaymentsPaymentId {
    np_id: u64,
}

#[component]
pub fn ConfirmationPage() -> impl IntoView {
    let hotel_info_ctx: HotelInfoCtx = expect_context();
    let search_ctx: SearchCtx = expect_context();
    let search_list_results: SearchListResults = expect_context();

    let confirmation_ctx: ConfirmationResults = expect_context();

    // ========= get payments id from query param and store in local storage ==========
    let np_id_query_map = use_query::<NowpaymentsPaymentId>();

    let np_payment_id = Signal::derive(move || {
        let val = np_id_query_map().ok().and_then(|id| Some(id.np_id.clone()));
        log!("np_payment_id: {val:?}");
        val
    });

    let (payment_store, set_payment_store, _) = use_payment_store();

    // whenever the payment ID changes, we update the value in local storage as well
    create_effect(move |_| {
        if payment_store.get().is_some() {
            return;
        }
        set_payment_store(np_payment_id.get())
    });

    // ===================

    // user data
    // if in context, => go ahead
    create_effect(move |_| {
        let (state, set_state, _) = use_local_storage::<BookingId, JsonSerdeCodec>("booking_id");
        let app_reference_string = state.get().get_app_reference();
        let email = state.get().get_email();

        log!(
            "EMAIL >>>{:?}\nAPP_REF >>>{:?}",
            email,
            app_reference_string
        );
        let email_cloned_twice = state.get().get_email();

        spawn_local(async move {
            match get_user_booking_backend(email_cloned_twice).await {
                Ok(response) => match response {
                    Some(booking) => {
                        let app_reference_string_cloned = app_reference_string.clone();
                        let email_cloned = email.clone();
                        let found_booking = booking
                            .into_iter()
                            .find(|b| {
                                log!("FILTERING BOOKINGS NOW");
                                b.booking_id
                                    == (app_reference_string_cloned.clone(), email_cloned.clone())
                            })
                            .unwrap();
                        let date_range = SelectedDateRange {
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

                        // let passenger_details = Vec::<PassengerDetail>::from(&found_booking.guests);
                        // let room_detail = RoomDetail { passenger_details };
                        // let hotel_code = hotel_info_ctx.hotel_code.get_untracked();

                        // let book_room_request = BookRoomRequest {
                        //     result_token: found_booking
                        //         .user_selected_hotel_room_details
                        //         .hotel_details
                        //         .hotel_token,
                        //     block_room_id: found_booking
                        //         .user_selected_hotel_room_details
                        //         .hotel_details
                        //         .block_room_id,
                        //     app_reference: app_reference_string,
                        //     room_details: vec![room_detail],
                        // };

                        // let result = book_room(book_room_request).await; // Call book_room API
                    }
                    None => {
                        log!("No booking available")
                    }
                },
                Err(e) => {
                    log!("Error greeting knull {:?}", e);
                }
            }
        });
    });

    let format_date = |(year, month, day): (u32, u32, u32)| {
        NaiveDate::from_ymd_opt(year as i32, month, day)
            .map(|d| d.format("%a, %b %d").to_string())
            .unwrap_or_default()
    };

    let insert_real_image_or_default = {
        move || {
            let val = hotel_info_ctx.selected_hotel_image.get();
            if val == "" {
                "/img/home.webp".to_string()
            } else {
                val
            }
        }
    };

    ////////////////////////
    // Booking Start
    ////////////////////////

    // PAYMENTS DATA
    // if you have data from expect_context => go ahead
    // else get it from local_storage
    // else get it from backend

    let (state, set_state, _) =
        use_local_storage::<Option<BookingId>, JsonSerdeCodec>("booking_id");

    let booking_action = create_action(move |_| {
        let (booking_id_signal_read, booking_id_signal_write, _) = use_booking_id_store();
        // let (state, set_state, _) = use_local_storage::<BookingId, JsonSerdeCodec>("booking_id");
        let app_reference = booking_id_signal_read
            .get_untracked()
            .and_then(|booking| Some(booking.get_app_reference()))
            .unwrap_or_default();
        // let nav = navigate.clone();
        let block_room = expect_context::<BlockRoomResults>();
        let conf_res = expect_context::<ConfirmationResults>();
        // let payment_status_data = block_room.payment_status_response.get().unwrap();
        // let payment_status_data = if block_room.payment_status_response.get().unwrap().payment_status == "finished" {
        //     block_room.payment_status_response.get().unwrap()
        // } else {
        //     payment_status_data
        // };
        let hotel_info_ctx: HotelInfoCtx = expect_context();
        let search_list_result = expect_context::<SearchListResults>();
        let result_token = search_list_result.get_result_token(hotel_info_ctx.hotel_code.get());
        let block_room_id = block_room
            .block_room_results
            .get()
            .unwrap()
            .get_block_room_id()
            .unwrap_or_default();
        let block_room_ctx = expect_context::<BlockRoomCtx>();
        let adults = block_room_ctx.adults.get();
        let children = block_room_ctx.children.get();
        let room_detail = RoomDetail {
            passenger_details: create_passenger_details(&adults, &children),
        };

        async move {
            // Call server function inside action
            spawn_local(async move {
                let req = BookRoomRequest {
                    result_token,
                    block_room_id,
                    app_reference,
                    room_details: vec![room_detail],
                };
                let result = book_room(req).await.ok();
                log!("BOOK_ROOM_API: {result:?}");
                conf_res.booking_details.set(result);
            });
        }
    });

    ////////////////////////
    // Booking  END
    ////////////////////////

    ////////////////////////
    // TIMER CHECK FOR PAYMENT STATUS
    //  ////////////////////////

    let Pausable {
        pause,
        resume,
        is_active,
    } = use_interval_fn(
        move || {
            spawn_local(async move {
                if let Some(payment_id) = np_payment_id.get_untracked() {
                    let resp =
                        nowpayments_get_payment_status(GetPaymentStatusRequest { payment_id })
                            .await
                            .ok();
                    BlockRoomResults::set_payment_results(resp);
                    // get_payment_status_action.dispatch(());
                }
            });
        },
        1_00_000,
    );

    pause();
    let pause_clone = pause.clone();

    create_effect(move |_| {
        let (booking_id_signal_read, booking_id_signal_write, _) = use_booking_id_store();

        let app_reference_string = booking_id_signal_read
            .get_untracked()
            .and_then(|booking| Some(booking.get_app_reference()));
        let email = state
            .get_untracked()
            .and_then(|booking| Some(booking.get_email()));
        let app_reference_string_cloned = app_reference_string.clone();
        let email_cloned = email.clone();

        let (payment_store, set_payment_store, _) = use_payment_store();

        let payment_details = PaymentDetails {
            booking_id: (
                app_reference_string.unwrap_or_default(),
                email.unwrap_or_default(),
            ),
            payment_status: Unpaid(None),
            payment_id: payment_store
                .get_untracked()
                .unwrap_or_default()
                .to_string(),
            provider: "NOWPayments".to_string(),
        };

        let block_room = expect_context::<BlockRoomResults>();
        match block_room.payment_status_response.get_untracked() {
            Some(status) => {
                log!("payment_status_response: {:?}", status);
                if status.payment_status == "finished" {
                    let status_clone = status.clone();
                    // todo
                    // 1. save to expect_context
                    block_room.payment_status_response.set(Some(status));
                    // 2. save to local storage
                    set_payment_store(Some(status_clone.payment_id));
                    // 3. save to backend
                    update_payment_details_backend(
                        (
                            app_reference_string_cloned.unwrap_or_default(),
                            email_cloned.unwrap_or_default(),
                        ),
                        payment_details,
                    );
                    // 4. booking_action.dispatch()
                    booking_action.dispatch({});
                    // Stop the interval and proceed
                    pause(); // Return Some(()) to stop the interval
                } else {
                    let p_status = block_room.payment_status_response.get_untracked();
                    log!("payment status is {p_status:?}");
                    resume(); // Return None to continue the interval
                }
            }
            None => {
                resume();
            }
        }
    });

    on_cleanup(move || {
        pause_clone();
    });

    ////////////////////////
    // Timer END
    ////////////////////////

    // let booking_status = create_rw_signal(None);

    // create_effect(move |_| {
    //     let (state, set_state, _) = use_local_storage::<BookingId, JsonSerdeCodec>("booking_id");
    //     // let app_reference_string = state.get().get_app_reference();
    //     // let email = state.get().get_email();
    //     let params = window().location().search().unwrap_or_default();
    //     let url_params = web_sys::UrlSearchParams::new_with_str(&params)
    //         .unwrap_or(web_sys::UrlSearchParams::new().unwrap());

    //     if let Some(payment_status) = url_params.get("NP_id") {
    //         match payment_status.as_str() {
    //             "success" => {
    //                 spawn_local(async move {
    //                     booking_status.set(Some(true));
    //                 });
    //             }
    //             "cancel" => {
    //                 booking_status.set(Some(false)); // Or handle cancellation differently
    //                 log!("Payment cancelled.");
    //             }
    //             // ... other cases
    //             _ => log!("Unknown payment status: {}", payment_status),
    //         }
    //     }
    // });

    view! {
        <section class="relative h-screen">
            <Navbar />
            <div class="flex flex-col items-center justify-center p-8">
                <img
                    src=insert_real_image_or_default
                    alt={move || hotel_info_ctx.selected_hotel_name.get()}
                    class="w-96 h-64 rounded-xl object-cover mb-8"
                />

                <h1 class="text-3xl font-bold mb-6 text-center">
                    "Awesome, your booking is confirmed!"
                </h1>

                <div class="text-center mb-6">
                    <p class="font-semibold">{move || hotel_info_ctx.selected_hotel_location.get()}</p>
                    <p class="text-gray-600">
                        {move || {
                            let date_range = search_ctx.date_range.get();
                            format!("{} - {}",
                                format_date(date_range.start),
                                format_date(date_range.end)
                            )
                        }}
                    </p>
                </div>

                <p class="max-w-2xl text-center text-gray-600">
                    "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor
                    incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud 
                    exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat."
                </p>

                // {move || match booking_status.get() {
                //     Some(true) => view! { <p>"Booking successful!"</p> }, // Display success message
                //     Some(false) => view! { <p>"Booking failed or cancelled."</p> }, // Display failure/cancel message
                //     None => view! { <p>"Checking booking status..."</p> }, // Display pending message
                // }}
            </div>
        </section>
    }
}
