use crate::{
    api::{
        book_room,
        canister::{
            get_user_booking::get_user_booking_backend,
            update_payment_details::update_payment_details_backend,
        },
        hotel_info,
        payments::{nowpayments_get_payment_status, ports::GetPaymentStatusRequest},
        BookRoomRequest, BookRoomResponse, BookingDetails, BookingDetailsContainer, BookingStatus,
        HotelResult, HotelSearchResponse, HotelSearchResult, PassengerDetail, Price, RoomDetail,
        Search,
    },
    app::AppRoutes,
    canister::backend::{
        BackendPaymentStatus::{Paid, Unpaid},
        BePaymentApiResponse, PaymentDetails,
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
use codee::string::JsonSerdeCodec;
use leptos::logging::log;
use leptos::*;
use leptos_router::use_navigate;
use leptos_router::*;
use leptos_use::{
    storage::use_local_storage, use_interval_fn, use_interval_fn_with_options, utils::Pausable,
    UseIntervalFnOptions,
};
use serde::{ser::SerializeStruct, Serialize, Serializer};


#[allow(non_snake_case)]
#[derive(Params, PartialEq, Clone, Debug)]
struct NowpaymentsPaymentId {
    NP_id: u64,
}

#[component]
pub fn ConfirmationPageOld() -> impl IntoView {
    let hotel_info_ctx: HotelInfoCtx = expect_context();
    let search_ctx: SearchCtx = expect_context();
    let search_list_results: SearchListResults = expect_context();
    let confirmation_ctx: ConfirmationResults = expect_context();

    // ========= get payments id from query param and store in local storage ==========
    let np_id_query_map = use_query::<NowpaymentsPaymentId>();

    let np_payment_id = Signal::derive(move || {
        let print_query_map = np_id_query_map.get();

        log!("print_query_map - {print_query_map:?}");

        let val = np_id_query_map
            .get()
            .ok()
            .and_then(|id| Some(id.NP_id.clone()));
        log!("np_payment_id: {val:?}");
        val
    });

    // whenever the payment ID changes, we update the value in local storage as well
    create_effect(move |_| {
        let (payment_store, set_payment_store, _) = use_payment_store();
        if payment_store.get().is_some() {
            return;
        }
        set_payment_store(np_payment_id.get())
    });

    // ===================

    // user data
    // if in context, => go ahead
    create_effect(move |_| {
        let (state, set_state, _) = use_booking_id_store();

        let app_reference_string = state
            .get_untracked()
            .and_then(|booking| Some(booking.get_app_reference()));
        let email = state
            .get_untracked()
            .and_then(|booking| Some(booking.get_email()));

        // log!(
        //     "EMAIL >>>{:?}\nAPP_REF >>>{:?}",
        //     email,
        //     app_reference_string
        // );
        let email_cloned_twice = email.clone().unwrap();

        spawn_local(async move {
            match get_user_booking_backend(email_cloned_twice).await {
                Ok(response) => match response {
                    Some(booking) => {
                        let app_reference_string_cloned =
                            app_reference_string.clone().unwrap_or_default();
                        let email_cloned = email.clone().unwrap_or_default();
                        let found_booking = booking
                            .into_iter()
                            .find(|b| {
                                // log!("FILTERING BOOKINGS NOW");
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
                        BlockRoomResults::set_id(Some(found_booking.user_selected_hotel_room_details.hotel_details.block_room_id));

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
                        log!("get_user_booking_backend - No booking available in backend")
                    }
                },
                Err(e) => {
                    log!("get_user_booking_backend - Error  {:?}", e);
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

    let booking_action =
        create_action(move |booking_id_signal_read: &Signal<Option<BookingId>>| {
            // let (booking_id_signal_read, booking_id_signal_write, _) = use_booking_id_store();
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
            // let block_room_id = block_room
            //     .block_room_results
            //     .get()
            //     .unwrap_or_default()
            //     .get_block_room_id()
            //     .unwrap_or_default();
            let block_room_id = block_room.block_room_id.get().unwrap_or_default();
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
                    let value_for_serverfn: String = serde_json::to_string(&req).unwrap();

                    log::warn!("REQ FOR BOOK ROOM API >>>{:?}", req);

                    let result = book_room(value_for_serverfn).await.ok();
                    log::info!("BOOK_ROOM_API: {result:?}");
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
    } = use_interval_fn_with_options(
        move || {
            spawn_local(async move {
                log!(
                    "inside use_interval_fn - np_payment_id - {:?}",
                    np_payment_id.get()
                );
                if let Some(payment_id) = np_payment_id.get_untracked() {
                    log!("inside use_interval_fn - payment_id - {payment_id:?} ");

                    let resp =
                        nowpayments_get_payment_status(GetPaymentStatusRequest { payment_id })
                            .await
                            .ok();
                    BlockRoomResults::set_payment_results(resp);
                    // get_payment_status_action.dispatch(());
                }
                log!("inside use_interval_fn - no payment id  ");
            });
        },
        1_00_000,
        UseIntervalFnOptions {
            immediate: false,
            immediate_callback: true,
        },
    );

    create_effect(move |_| {
        let (booking_id_signal_read, booking_id_signal_write, _) = use_booking_id_store();
        let confirmation_ctx = expect_context::<ConfirmationResults>();
        let block_room_ctx = expect_context::<BlockRoomResults>();

        let app_reference_string = booking_id_signal_read
            .get_untracked()
            .and_then(|booking| Some(booking.get_app_reference()));
        let email = booking_id_signal_read
            .get_untracked()
            .and_then(|booking| Some(booking.get_email()));
        let app_reference_string_cloned = app_reference_string.clone();
        let email_cloned = email.clone();
        let email_cloned_twice = email.clone();

        let (payment_store, set_payment_store, _) = use_payment_store();

        let get_payment_status_response = block_room_ctx
            .payment_status_response
            .get()
            .unwrap_or_default()
            .into();
        let payment_api_response =
            BePaymentApiResponse::from((get_payment_status_response, "NOWPayments".to_string()));

        let payment_details = PaymentDetails {
            booking_id: (
                app_reference_string.unwrap_or_default(),
                email.unwrap_or_default(),
            ),
            payment_status: Unpaid(None),
            payment_api_response,
        };

        let payment_details_str =
            serde_json::to_string(&payment_details).expect("payment details is not valid json");

        let block_room = expect_context::<BlockRoomResults>();
        let search_list_result = expect_context::<SearchListResults>();
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
                    // update_payment_details_backend(
                    //     (
                    //         app_reference_string_cloned.unwrap_or_default(),
                    //         email_cloned.unwrap_or_default(),
                    //     ),
                    //     payment_details,
                    // );
                    spawn_local(async move {
                        match update_payment_details_backend(
                            (
                                app_reference_string_cloned.unwrap_or_default(),
                                email_cloned.unwrap_or_default(),
                            ),
                            payment_details_str,
                        )
                        .await
                        {
                            Ok(booking) => {
                                println!("{:?}", booking);
                                // let app_reference_string_cloned =
                                //     app_reference_string_cloned.clone();
                                let email_cloned = email_cloned_twice.clone();
                                let hotel_det_cloned = booking
                                    .user_selected_hotel_room_details
                                    .hotel_details
                                    .clone();

                                let date_range = SelectedDateRange {
                                    start: booking
                                        .user_selected_hotel_room_details
                                        .date_range
                                        .start,
                                    end: booking.user_selected_hotel_room_details.date_range.end,
                                };

                                SearchCtx::set_date_range(date_range);
                                HotelInfoCtx::set_selected_hotel_details(
                                    booking
                                        .user_selected_hotel_room_details
                                        .hotel_details
                                        .hotel_code,
                                    booking
                                        .user_selected_hotel_room_details
                                        .hotel_details
                                        .hotel_name,
                                    booking
                                        .user_selected_hotel_room_details
                                        .hotel_details
                                        .hotel_image,
                                    booking
                                        .user_selected_hotel_room_details
                                        .hotel_details
                                        .hotel_location,
                                );

                                // Storing hotel_location is the field given for hotel_category becoz why not
                                let hotel_res = HotelResult {
                                    hotel_code: hotel_det_cloned.hotel_code,
                                    hotel_name: hotel_det_cloned.hotel_name,
                                    hotel_picture: hotel_det_cloned.hotel_image,
                                    hotel_category: hotel_det_cloned.hotel_location,
                                    result_token: hotel_det_cloned.hotel_token,
                                    star_rating: 0,
                                    price: Price::default(),
                                };
                                let hotel_search_res = HotelSearchResult {
                                    hotel_results: vec![hotel_res],
                                };
                                let search_res = Search {
                                    hotel_search_result: hotel_search_res,
                                };
                                let hotel_search_resp = HotelSearchResponse {
                                    status: 0,
                                    message: "Default Message".to_string(),
                                    search: Some(search_res),
                                };

                                SearchListResults::set_search_results(Some(hotel_search_resp));

                                let book_room_status = booking.book_room_status;
                                let book_room_status_cloned = book_room_status.clone();
                                let book_room_status_twice = book_room_status.clone();
                                let booking_status_cloned_again = book_room_status_cloned.clone();
                                let booking_status_cloned_again1 = book_room_status_cloned.clone();
                                let booking_status_cloned_again2 = book_room_status_cloned.clone();
                                let booking_status_cloned_again3 = book_room_status_cloned.clone();

                                let booking_details = BookRoomResponse {
                                    status: book_room_status
                                    .as_ref()
                                    .map_or(BookingStatus::BookFailed, |status| status.clone().commit_booking.booking_status.into()),
                                    message: book_room_status_cloned.unwrap_or_default().message,
                                    commit_booking: BookingDetailsContainer {
                                        booking_details: BookingDetails {
                                            booking_id: booking_status_cloned_again3.unwrap_or_default().commit_booking.booking_id.0,
                                            booking_ref_no: booking_status_cloned_again.unwrap_or_default().commit_booking.booking_ref_no,
                                            confirmation_no: booking_status_cloned_again1.unwrap_or_default().commit_booking.confirmation_no,
                                            booking_status: match booking_status_cloned_again2.unwrap_or_default().commit_booking.booking_status {
                                                crate::canister::backend::BookingStatus::Confirmed => "Confirmed".to_string(),
                                                crate::canister::backend::BookingStatus::BookFailed => "BookFailed".to_string(),
                                            },
                                        },
                                    }
                                };

                                confirmation_ctx.booking_details.set(Some(booking_details));
                                let booking_guests = booking.guests.clone();
                                let booking_guests2 = booking.guests.clone();

                                let adults: Vec<crate::state::view_state::AdultDetail> = booking_guests.into();
                                let children: Vec<crate::state::view_state::ChildDetail> = booking_guests2.into();

                                let adults_clon = adults.clone();
                                let children_clon = children.clone();
                                BlockRoomCtx::set_adults(adults);
                                BlockRoomCtx::set_children(children);

                                // Payment Details not being stored. Can use the calculated value above if wanna populate it anywhere.
                            }
                            Err(e) => {
                                log!("Error greeting knull {:?}", e);
                            }
                        }
                    });
                    // 4. booking_action.dispatch()
                    booking_action.dispatch(booking_id_signal_read);
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

    ////////////////////////
    // Timer END
    ////////////////////////

    view! {
        <section class="relative h-screen">
            <Navbar />
            <div class="flex flex-col items-center justify-center p-8">
                <img
                    src=insert_real_image_or_default
                    alt=move || hotel_info_ctx.selected_hotel_name.get()
                    class="w-96 h-64 rounded-xl object-cover mb-8"
                />

                <h1 class="text-3xl font-bold mb-6 text-center">
                    "Awesome, your booking is confirmed!"
                </h1>

                <div class="text-center mb-6">
                    <p class="font-semibold">
                        {move || hotel_info_ctx.selected_hotel_location.get()}
                    </p>
                    <p class="text-gray-600">
                        {move || {
                            let date_range = search_ctx.date_range.get();
                            format!(
                                "{} - {}",
                                format_date(date_range.start),
                                format_date(date_range.end),
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
            // Some(true) => view! { <p>"Booking successful!"</p> }, // Display success message
            // Some(false) => view! { <p>"Booking failed or cancelled."</p> }, // Display failure/cancel message
            // None => view! { <p>"Checking booking status..."</p> }, // Display pending message
            // }}
            </div>
        </section>
    }
}
