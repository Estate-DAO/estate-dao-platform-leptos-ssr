#![allow(unused)]
#![allow(dead_code)]

use crate::api::_default_passenger_age;
use crate::api::block_room;
use crate::api::canister::add_booking::add_booking_backend;
use crate::api::consts::{get_payments_url, get_price_amount_based_on_env, APP_URL};
use crate::api::get_room;
use crate::api::payments::nowpayments_create_invoice;
use crate::api::payments::nowpayments_get_payment_status;
use crate::api::payments::ports::CreateInvoiceRequest;
use crate::api::payments::ports::GetPaymentStatusRequest;
use crate::api::payments::NowPayments;
use crate::canister::backend;
use crate::canister::backend::BePaymentApiResponse;
use crate::canister::backend::HotelDetails;
use crate::canister::backend::HotelRoomDetails;
use crate::canister::backend::RoomDetails;
use crate::canister::backend::UserDetails;
use crate::component::code_print::DebugDisplay;
use crate::component::{Divider, FilterAndSortBy, PriceDisplay, StarRating};
use crate::component::{ErrorPopup, Navbar, SkeletonCards, SpinnerGray};
use crate::page::InputGroup;
use crate::state::api_error_state::{ApiErrorState, ApiErrorType};
use crate::state::hotel_details_state::PricingBookNowState;
use crate::state::search_state::{
    BlockRoomResults, ConfirmationResults, HotelInfoResults, SearchCtx, SearchListResults,
};
use crate::state::view_state::{AdultDetail, BlockRoomCtx, ChildDetail, HotelInfoCtx};
use crate::utils::app_reference::{generate_app_reference, BookingId};
use crate::utils::booking_id::PaymentIdentifiers;
use crate::utils::pluralize;
use leptos::*;
use leptos_icons::*;
use leptos_router::use_navigate;
// use web_sys::localStorage;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api::payments::ports::PaymentGateway;
use crate::{
    api::{book_room, BookRoomRequest, BookingStatus, PassengerDetail, PaxType, RoomDetail},
    app::AppRoutes,
};
use chrono::NaiveDate;
// use leptos::logging::log;
use crate::{log, warn};

#[component]
pub fn BlockRoomPage() -> impl IntoView {
    // ================ SETUP context  ================
    let search_ctx: SearchCtx = expect_context();
    let search_list_results: SearchListResults = expect_context();
    let search_list_results = store_value(search_list_results); // Store it so we can clone inside closure

    let hotel_info_results: HotelInfoResults = expect_context();
    let hotel_info_ctx: HotelInfoCtx = expect_context();

    let confirmation_results: ConfirmationResults = expect_context();
    let api_error_state = ApiErrorState::from_leptos_context();

    let navigate = use_navigate();

    let block_room_ctx = expect_context::<BlockRoomCtx>();
    let block_room_results_context: BlockRoomResults = expect_context();
    let block_room_results_context_cloned: BlockRoomResults = block_room_results_context.clone();

    // ================ Pricing component signals  ================
    let room_price = Signal::derive(move || BlockRoomResults::get_room_price());
    log!("[block_room_page] - room_price - {}", room_price.get());

    let num_nights = Signal::derive(move || search_ctx.date_range.get().no_of_nights());

    // let total_price = create_memo(move |_| {
    let total_price = Signal::derive(move || {
        let room_price = room_price.get();
        let nights = num_nights.get();
        let total = room_price * nights as f64;
        log!("[block_room_page] - total_price - {}", total);
        total
    });

    // ================  Form Fields signals  ================

    let is_form_valid: RwSignal<bool> = create_rw_signal(false);

    let adult_count = create_memo(move |_| search_ctx.guests.get().adults.get());

    let child_count = create_memo(move |_| search_ctx.guests.get().children.get());

    let children_ages = search_ctx.guests.get().children_ages.clone();

    let num_rooms = Signal::derive(move || search_ctx.guests.get().rooms.get());

    BlockRoomCtx::create_adults(adult_count.get() as usize);
    BlockRoomCtx::create_children(child_count.get() as usize);
    BlockRoomCtx::set_terms_accepted(false);

    let (trigger_validation, set_trigger_validation) = create_signal(0);

    let update_adult = move |index: usize, field: &str, value: String| {
        BlockRoomCtx::update_adult(index, field, value);
    };

    let update_child = move |index: usize, field: &str, value: String| {
        BlockRoomCtx::update_child(index, field, value);
    };

    let update_terms = move |checked: bool| {
        BlockRoomCtx::set_terms_accepted(checked);
        // Trigger validation check
    };

    // ================  Form Validation signals  ================

    let adults = block_room_ctx.adults;
    let children = block_room_ctx.children;
    let terms_accepted = block_room_ctx.terms_accepted;

    // Validation logic function
    let validate_form = move || {
        let adult_list = adults.get();
        let child_list = children.get();

        // Helper function for email validation
        let is_valid_email = |email: &str| email.contains('@') && email.contains('.');

        // Helper function for phone validation
        let is_valid_phone =
            |phone: &str| phone.len() >= 10 && phone.chars().all(|c| c.is_digit(10));

        // Validate primary adult
        let primary_adult_valid = adult_list.first().map_or(false, |adult| {
            !adult.first_name.trim().is_empty()
                && adult
                    .email
                    .as_ref()
                    .map_or(false, |e| !e.trim().is_empty() && is_valid_email(e))
                && adult
                    .phone
                    .as_ref()
                    .map_or(false, |p| !p.trim().is_empty() && is_valid_phone(p))
        });

        // Validate other adults
        let other_adults_valid = adult_list
            .iter()
            .skip(1)
            .all(|adult| !adult.first_name.trim().is_empty());

        // Validate children
        let children_valid = child_list
            .iter()
            .all(|child| !child.first_name.trim().is_empty() && child.age.is_some());

        // Check if terms are accepted
        let terms_valid = terms_accepted.get();

        // Generate and store app_reference in local storage if we have an email
        if let Some(email) = adults.get().first().and_then(|adult| adult.email.clone()) {
            let booking_id_signal = generate_app_reference(email.clone());
            log!(
                "form_validation - booking_id_signal - {:?}",
                booking_id_signal.get()
            );
        }

        let booking_id_is_valid = BookingId::read_from_local_storage().is_some();
        // Set the value of is_form_valid based on validation results
        is_form_valid.set(
            primary_adult_valid
                && other_adults_valid
                && children_valid
                && terms_valid
                && booking_id_is_valid,
        );

        // log!(
        //     "form_validation - hotel_info_results.sorted_rooms - {:?}",
        //     hotel_info_results.sorted_rooms.get()
        // );
    };

    // Call the validation function whenever inputs change
    let _ = create_effect(move |_| {
        validate_form();
    });

    // ================  Page Nav and Redirect signals  ================

    let nav = navigate.clone();

    let go_back_to_details = move |ev: ev::MouseEvent| {
        ev.prevent_default();
        let _ = navigate(AppRoutes::HotelDetails.to_string(), Default::default());
    };

    let hotel_code = hotel_info_ctx.hotel_code.get();
    let search_list_results = search_list_results.get_value();

    let confirmation_action = create_action(move |()| {
        let nav = nav.clone(); // Use the cloned version here

        async move {
            nav(AppRoutes::Confirmation.to_string(), Default::default());
        }
    });

    // ================  Confirmation Modal signals  ================

    let show_modal = create_rw_signal(false);
    let block_room_called = create_rw_signal(false);
    let open_modal = move |_| {
        show_modal.set(true);
    };

    let payment_button_enabled = create_rw_signal(true);

    let should_not_have_loading_spinner: Signal<bool> = Signal::derive(move || {
        log!(
            "[block_room_page] - should_not_have_loading_spinner - block_room_called: {}",
            block_room_called.get()
        );
        log!(
            "[block_room_page] - should_not_have_loading_spinner - payment_button_enabled: {}",
            payment_button_enabled.get()
        );
        block_room_called.get() && payment_button_enabled.get()
    });

    let _block_room_call = create_resource(show_modal, move |modal_value| {
        let hotel_info_results = expect_context::<HotelInfoResults>();
        // let room_counters = hotel_info_results.block_room_counters.get_untracked();
        block_room_called.set(false);
        // log!("block_room.rs -- component room_coutners value - \n {room_counters:#?}");
        async move {
            if modal_value {
                // log!("modal open. calling block API now");

                // log!("{:?}", room_counters ); // Add this line
                // Reset previous block room results
                BlockRoomResults::reset();

                let uniq_room_ids = PricingBookNowState::room_unique_ids();
                log!("[block_room_page] - {uniq_room_ids:#?}");

                let block_room_request = hotel_info_results.block_room_request(uniq_room_ids);

                log!("[block_room_page] - block_room_request - {block_room_request:?}");
                // Call server function inside action
                spawn_local(async move {
                    // TODO(temporary-fix): temporarily disable second call to block room

                    let result = block_room(block_room_request).await.ok();
                    let res = result.clone();
                    let block_room_id = res.and_then(|resp| resp.get_block_room_id());
                    let res2 = result.clone();
                    BlockRoomResults::set_results(result);
                    BlockRoomResults::set_id(block_room_id);
                    if let Some(_) = res2.clone() {
                        // if the block_room call fails, we don't want to enable the payment button
                        block_room_called.set(true);
                    }

                    let res: BlockRoomResults = expect_context();
                    log!(
                        "[block_room_page] - block_room_results - {:?}",
                        res.block_room_results.get()
                    );
                });
            } else {
                block_room_called.set(false);
                log!("[block_room_page] - modal closed. Nothing to do");
            }
        }
    });

    let handle_pay_signal = create_rw_signal(String::new());

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

    let hotel_address_signal = move || {
        if let Some(hotel_info_api_response) = hotel_info_results.search_result.get() {
            hotel_info_api_response.get_address()
        } else {
            "".into()
        }
    };

    let handle_pay_click = create_action(move |payment_method: &String| {
        // let handle_pay_click = create_resource(handle_pay_signal, move |payment_method: String| {
        let hotel_code_cloned = hotel_code.clone();
        let search_list_results_cloned = search_list_results.clone();
        let payment_method = payment_method.clone();

        async move {
            match payment_method.as_str() {
                "binance" => {
                    todo!();
                }
                "NOWPayments" => {
                    let email = adults
                        .get()
                        .first()
                        .and_then(|adult| adult.email.clone())
                        .unwrap_or_default();

                    // Get stored booking id or generate new one
                    let booking_id = BookingId::read_from_local_storage().unwrap_or_else(|| {
                        generate_app_reference(email.clone())
                            .get()
                            .expect("[block_room_page] - Failed to generate booking id")
                    });

                    let backend_booking_id = backend::BookingId {
                        app_reference: booking_id.app_reference,
                        email: email.clone(),
                    };
                    let booking_id_cloned = backend_booking_id.clone();

                    // Create invoice request with order_id
                    let invoice_request = CreateInvoiceRequest {
                        price_amount: get_price_amount_based_on_env(total_price.get() as u32),
                        price_currency: "USD".to_string(),
                        order_id: PaymentIdentifiers::order_id_from_app_reference(
                            &booking_id_cloned.app_reference,
                            &email,
                        ),
                        order_description: "Hotel Room Booking".to_string(),
                        ipn_callback_url: format!("{}/ipn/webhook", APP_URL),
                        success_url: get_payments_url("success"),
                        cancel_url: get_payments_url("cancel"),
                        partially_paid_url: get_payments_url("partial"),
                        is_fixed_rate: false,
                        is_fee_paid_by_user: false,
                    };

                    // Log payment request for debugging
                    log!(
                        "[block_room_page] - Creating payment request with order_id: {}",
                        invoice_request.order_id
                    );

                    // todo: [UAT]  feedback - select only those room which are selected by the user!!
                    // let sorted_rooms = hotel_info_results.sorted_rooms.get();
                    // log!("SORTED-----ROOM?>>>>>>>>>>>>>>>\n{:?}", sorted_rooms);

                    // let room_details: Vec<RoomDetails> = sorted_rooms
                    //     .into_iter()
                    //     .map(|sorted_room| RoomDetails {
                    //         room_price: sorted_room.room_price as f32,
                    //         room_unique_id: sorted_room.room_unique_id,
                    //         room_type_name: sorted_room.room_type,
                    //     })
                    //     .collect();

                    let room_details: Vec<RoomDetails> =
                        PricingBookNowState::get_room_counters().into();

                    let destination = search_ctx.destination.get().map(|dest| {
                        crate::canister::backend::Destination {
                            city_id: dest.city_id,
                            city: dest.city,
                            country_code: dest.country_code,
                            country_name: dest.country_name,
                        }
                    });
                    let selected_date_range = search_ctx.date_range.get();
                    let date_range = crate::canister::backend::SelectedDateRange {
                        start: selected_date_range.start,
                        end: selected_date_range.end,
                    };

                    let hotel_token =
                        search_list_results_cloned.get_result_token(hotel_code_cloned.clone());

                    let block_room_id = block_room_results_context_cloned
                        .block_room_id
                        .get()
                        .unwrap_or_default();

                    log!(
                        "[block_room_page] - handle_pay_click - block_room_id> {:?}",
                        block_room_id
                    );
                    let user_selected_hotel_room_details = HotelRoomDetails {
                        destination,
                        requested_payment_amount: total_price.get(),
                        date_range,
                        room_details,
                        hotel_details: HotelDetails {
                            hotel_code: hotel_code_cloned.clone(),
                            hotel_name: hotel_name_signal(),
                            hotel_image: images_signal()
                                .into_iter()
                                .find(|s| !s.is_empty())
                                .unwrap_or_else(|| String::new()),
                            hotel_location: hotel_address_signal(),
                            block_room_id,
                            hotel_token,
                        },
                    };
                    log!(
                        "[block_room_page] - handle_pay_click - user_selected_hotel_room_details> {:?}",
                        user_selected_hotel_room_details
                    );

                    let adults: Vec<crate::canister::backend::AdultDetail> = block_room_ctx
                        .adults
                        .get()
                        .into_iter()
                        .map(|adult| crate::canister::backend::AdultDetail {
                            email: adult.email,
                            first_name: adult.first_name,
                            last_name: adult.last_name,
                            phone: adult.phone,
                        })
                        .collect();
                    let children: Vec<crate::canister::backend::ChildDetail> = block_room_ctx
                        .children
                        .get()
                        .into_iter()
                        .map(|child| crate::canister::backend::ChildDetail {
                            age: child.age.unwrap(),
                            first_name: child.first_name,
                            last_name: child.last_name,
                        })
                        .collect();
                    let guests = UserDetails { children, adults };

                    let email_cloned = email.clone();
                    let email_cloned_twice = email.clone();

                    // Get stored booking id or generate new one
                    let booking_id = BookingId::read_from_local_storage().unwrap_or_else(|| {
                        generate_app_reference(email_cloned_twice)
                            .get()
                            .expect("Failed to generate booking id")
                    });

                    let backend_booking_id = backend::BookingId {
                        app_reference: booking_id.app_reference,
                        email: email_cloned.clone(),
                    };
                    let booking_id_cloned = backend_booking_id.clone();

                    let payment_details = crate::canister::backend::PaymentDetails {
                        booking_id: backend_booking_id,
                        ..Default::default()
                    };

                    let booking = crate::canister::backend::Booking {
                        user_selected_hotel_room_details,
                        guests,
                        booking_id: booking_id_cloned,
                        book_room_status: None,
                        payment_details,
                    };

                    log!(
                        "[block_room_page] - handle_pay_click - booking - {:#?}",
                        booking
                    );

                    spawn_local(async move {
                        let create_invoice_response =
                            nowpayments_create_invoice(invoice_request).await;
                        log!("creating invoice");
                        match create_invoice_response {
                            Ok(resp) => {
                                log!(
                                    "[block_room_page] - handle_pay_click - CreateInvoiceResponse"
                                );
                                log!("{:?}\n{:?}", email_cloned, booking);
                                let value_for_serverfn: String =
                                    serde_json::to_string(&booking).unwrap();

                                match add_booking_backend(email_cloned, value_for_serverfn).await {
                                    Ok(response) => {
                                        log!("\n\n\n ____________WORKING>>>>\n\n{:#}", response);
                                        #[cfg(feature = "mock-provab")]
                                        {
                                            let _ =
                                                window().location().assign(&get_payments_url(""));
                                        }
                                        #[cfg(not(feature = "mock-provab"))]
                                        {
                                            let _ = window().location().assign(&resp.invoice_url);
                                        }
                                    }
                                    Err(e) => {
                                        warn!("[block_room_page] - handle_pay_click - Error add_booking_backend serverFn {:?}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                warn!("[block_room_page] - handle_pay_click - Error creating invoice: {:?}", e);
                            }
                        }
                    });
                }
                _ => { /* Handle other payment methods */ }
            }
            // show_modal.set(false);
            payment_button_enabled.set(false);
        }
    });

    // let is_form_valid = create_memo(move |_| {
    //     trigger_validation.get();

    //     let adult_list = adults.get();
    //     let child_list = children.get();

    //     // Validate primary adult (needs all fields)
    //     let primary_adult_valid = adult_list.first().map_or(false, |adult| {
    //         !adult.first_name.is_empty()
    //             && adult.email.as_ref().map_or(false, |e| !e.is_empty())
    //             && adult.phone.as_ref().map_or(false, |p| !p.is_empty())
    //     });

    //     // Validate other adults (only first name required)
    //     let other_adults_valid = adult_list
    //         .iter()
    //         .skip(1)
    //         .all(|adult| !adult.first_name.is_empty());

    //     // Validate children (first name and age required)
    //     let children_valid = child_list
    //         .iter()
    //         .all(|child| !child.first_name.is_empty() && child.age.is_some());

    //     // Terms must be accepted
    //     let terms_valid = terms_accepted.get();

    //     // All conditions must be true
    //     primary_adult_valid && other_adults_valid && children_valid && terms_valid
    // });

    let destination = create_memo(move |_| search_ctx.destination.get().unwrap_or_default());

    let insert_real_image_or_default = {
        move || {
            if let Some(hotel_info_api_response) = hotel_info_results.search_result.get() {
                hotel_info_api_response
                    .get_images()
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "/img/home.webp".to_string())
            } else {
                "/img/home.webp".to_string()
            }
        }
    };

    view! {
    <section class="relative min-h-screen bg-gray-50">
        <Navbar />
        <ErrorPopup />
        <div class="max-w-5xl mx-auto px-2 sm:px-6">
            <div class="flex items-center py-8">
                <span class="inline-flex items-center cursor-pointer" on:click=go_back_to_details>
                    <Icon icon=icondata::AiArrowLeftOutlined class="text-black font-light" />
                </span>
                <h1 class="ml-2 sm:ml-4 text-2xl sm:text-3xl font-bold">"You're just one step away!"</h1>
            </div>
        </div>
        <div class="relative flex flex-col lg:flex-row min-h-[calc(100vh-5rem)] items-start justify-center p-2 sm:p-6 max-w-5xl mx-auto gap-6">
            <div class="w-full lg:w-3/5 flex flex-col gap-8 order-1">
                <div class="p-2 sm:p-6 bg-white rounded-2xl shadow w-full">
                    <div class="flex items-center gap-3 mb-2">
                        <img
                            src={move || {
                                let imgs = images_signal();
                                if !imgs.is_empty() {
                                    imgs[0].clone()
                                } else {
                                    "/img/home.webp".to_string()
                                }
                            }}
                            alt={move || hotel_info_results.search_result.get().as_ref().map(|r| r.get_hotel_name()).unwrap_or_default()}
                            class="h-10 w-10 sm:h-12 sm:w-12 rounded-lg object-cover"
                        />
                        <div class="flex flex-col justify-center min-h-[2.5rem]">
                            <div class="font-bold text-base sm:text-lg min-h-[1.25rem]">
                                {move || hotel_info_results.search_result.get().as_ref().map(|r| r.get_hotel_name()).unwrap_or_default()}
                            </div>
                            <div class="text-gray-500 text-sm min-h-[1rem]">
                                {move || hotel_info_results.search_result.get().as_ref().map(|r| r.get_address()).unwrap_or_default()}
                            </div>
                        </div>
                    </div>
                    <hr class="my-3 border-gray-200" />
                    <div class="flex items-center justify-between mb-3">
                        <div class="flex flex-col items-start">
                            <span class="text-xs text-gray-400">Check-in</span>
                            <span class="font-semibold text-base">{move || search_ctx.date_range.get().dd_month_yyyy_start()}</span>
                        </div>
                        <div class="flex flex-col items-center">
                            <span class="bg-gray-100 rounded-full px-3 py-1 text-xs font-semibold text-gray-700 mb-1">{move || search_ctx.date_range.get().formatted_nights()}</span>
                        </div>
                        <div class="flex flex-col items-end">
                            <span class="text-xs text-gray-400">Check-out</span>
                            <span class="font-semibold text-base">{move || search_ctx.date_range.get().dd_month_yyyy_end()}</span>
                        </div>
                    </div>
                    <hr class="my-3 border-gray-200" />
                    <div class="flex items-center gap-2 mt-2">
                        <Icon icon=icondata::AiUserOutlined class="text-gray-400 text-lg" />
                        <span class="text-xs text-gray-400 font-semibold">Guests & Rooms</span>
                        <span class="font-bold text-sm ml-2 text-right">{move || format!("{} Room{}{} {} Adult{}{} {} child{}", num_rooms.get(), if num_rooms.get() == 1 { "" } else { "s" }, if num_rooms.get() > 0 { "," } else { "" }, adult_count.get(), if adult_count.get() == 1 { "" } else { "s" }, if child_count.get() > 0 { "," } else { "" }, child_count.get(), if child_count.get() == 1 { "" } else { "ren" })}</span>
                    </div>
                </div>
                // <!-- Payment summary card for mobile -->
                <div class=" lg:hidden mb-6 rounded-2xl bg-white p-4 sm:p-8 shadow-xl flex flex-col items-stretch">
                    <h2 class="mb-4 text-2xl font-bold flex items-end">
                        <span class="text-3xl font-bold">{move || format!("${:.3}", room_price.get())}</span>
                        <span class="ml-1 text-base font-normal text-gray-600">/night</span>
                    </h2>
                    <Divider class="my-4".into() />
                    <div class="price-breakdown space-y-4 mt-4">
                        <div class="flex justify-between items-center text-base">
                            <span class="text-gray-700">{move || format!("${:.3} x {} nights", room_price.get(), num_nights.get())}</span>
                            <span class="font-semibold">{move || format!("${:.3}", room_price.get() * num_nights.get() as f64)}</span>
                        </div>
                        <div class="flex justify-between items-center text-base">
                            <span class="text-gray-700">Taxes and fees</span>
                            <span class="font-semibold">$0.00</span>
                        </div>
                    </div>
                    <Divider class="my-4".into() />
                    <div class="flex justify-between items-center font-bold text-lg mb-2">
                        <span>Total</span>
                        <span class="text-2xl">{move || format!("${:.3}", total_price.get())}</span>
                    </div>
                </div>
                // <!-- End payment summary card for mobile -->
                <div class="guest-form mt-4 space-y-6">
                    {(0..adult_count.get())
                        .map(|i| {
                            let i_usize = i as usize;
                            view! {
                                <div class="person-details mb-2">
                                    <h3 class="font-semibold text-gray-700 text-sm sm:text-base mb-2">
                                        {if i == 0 {
                                            String::from("Primary Adult")
                                        } else {
                                            format!("Adult {}", i + 1)
                                        }}
                                    </h3>
                                    <div class="flex flex-col sm:flex-row gap-2 sm:gap-4">
                                        <input
                                            type="text"
                                            placeholder="First Name *"
                                            class="w-full sm:w-1/2 rounded-md border border-gray-300 p-3"
                                            required=true
                                            on:input=move |ev| {
                                                update_adult(
                                                    i_usize,
                                                    "first_name",
                                                    event_target_value(&ev),
                                                );
                                                validate_form();
                                            }
                                        />
                                        <input
                                            type="text"
                                            placeholder="Last Name"
                                            class="w-full sm:w-1/2 rounded-md border border-gray-300 p-3"
                                            required=true
                                            on:input=move |ev| {
                                                update_adult(i_usize, "last_name", event_target_value(&ev));
                                                validate_form();
                                            }
                                        />
                                    </div>
                                    {move || {
                                        if i == 0 {
                                            view! {
                                                <div class="flex flex-col sm:flex-row gap-2 sm:gap-4 mt-2">
                                                    <input
                                                        type="email"
                                                        placeholder="Email *"
                                                        class="w-full sm:w-1/2 rounded-md border border-gray-300 p-3"
                                                        required=true
                                                        on:input=move |ev| update_adult(
                                                            0,
                                                            "email",
                                                            event_target_value(&ev),
                                                        )
                                                    />
                                                    <input
                                                        type="tel"
                                                        placeholder="Phone *"
                                                        class="w-full sm:w-1/2 rounded-md border border-gray-300 p-3"
                                                        required=true
                                                        on:input=move |ev| update_adult(
                                                            0,
                                                            "phone",
                                                            event_target_value(&ev),
                                                        )
                                                    />
                                                </div>
                                            }
                                                .into_view()
                                        } else {
                                            view! { <div></div> }.into_view()
                                        }
                                    }}
                                </div>
                            }
                        })
                        .collect::<Vec<_>>()}
                        // Loop for children
                    {(0..child_count.get())
                        .map(|i| {
                            let i_usize = i as usize;
                            let age_value = children_ages.get_value_at(i as u32);
                                // Get the age for the current child

                            view! {
                                <div class="person-details mb-2">
                                    <h3 class="font-semibold text-gray-700 text-sm sm:text-base mb-2">
                                        {format!("Child {}", i + 1)}
                                    </h3>
                                    <div class="flex flex-col sm:flex-row gap-2 sm:gap-4">
                                        <input
                                            type="text"
                                            placeholder="First Name *"
                                            class="w-full sm:w-2/5 rounded-md border border-gray-300 p-3"
                                            required=true
                                            on:input=move |ev| {
                                                update_child(
                                                    i_usize,
                                                    "first_name",
                                                    event_target_value(&ev),
                                                );
                                                validate_form();
                                            }
                                        />
                                        <input
                                            type="text"
                                            placeholder="Last Name"
                                            class="w-full sm:w-2/5 rounded-md border border-gray-300 p-3"
                                        />
                                        <select
                                            class="w-full sm:w-1/5 rounded-md border border-gray-300 bg-white p-3"
                                            required=true
                                            on:input=move |ev| {
                                                update_child(i_usize, "age", event_target_value(&ev));
                                                validate_form();
                                            }
                                        >
                                            <option disabled selected>{age_value}</option>
                                            {(1..18)
                                                .map(|age| {
                                                    let selected = if age == age_value {
                                                        "selected"
                                                    } else {
                                                        ""
                                                    };
                                                    view! {
                                                        <option value=age.to_string() {selected}>{age}</option>
                                                    }
                                                })
                                                .collect::<Vec<_>>()}
                                        </select>
                                    </div>
                                </div>
                            }
                        })
                        .collect::<Vec<_>>()}
                </div>
                <div class="mt-2 flex items-start">
                    <input
                        type="checkbox"
                        id="agree"
                        class="mr-2 mt-1"
                        on:change=move |ev| update_terms(event_target_checked(&ev))
                    />
                    <label for="agree" class="text-xs sm:text-sm text-gray-600">
                        "Property once booked cannot be cancelled. Confirm the details before making payment."
                    </label>
                </div>
                <button
                    class="mt-6 w-full rounded-full bg-blue-600 py-3 text-white hover:bg-blue-700 disabled:bg-gray-300 text-base sm:text-lg font-bold shadow-lg block lg:hidden"
                    disabled=move || !is_form_valid.get()
                    on:click=open_modal
                >
                    Confirm & Book
                </button>
            </div>
            <div class="hidden lg:flex w-full lg:w-2/5 mb-8 lg:mb-0 rounded-2xl bg-white p-4 sm:p-8 shadow-xl flex-col items-stretch order-2 lg:sticky lg:top-28">
                <h2 class="mb-4 text-2xl font-bold flex items-end">
                    <span class="text-3xl font-bold">{move || format!("${:.3}", room_price.get())}</span>
                    <span class="ml-1 text-base font-normal text-gray-600">/night</span>
                </h2>
                <Divider class="my-4".into() />
                <div class="price-breakdown space-y-4 mt-4">
                    <div class="flex justify-between items-center text-base">
                        <span class="text-gray-700">{move || format!("${:.3} x {} nights", room_price.get(), num_nights.get())}</span>
                        <span class="font-semibold">{move || format!("${:.3}", room_price.get() * num_nights.get() as f64)}</span>
                    </div>
                    <div class="flex justify-between items-center text-base">
                        <span class="text-gray-700">Taxes and fees</span>
                        <span class="font-semibold">$0.00</span>
                    </div>
                </div>
                <Divider class="my-4".into() />
                <div class="flex justify-between items-center font-bold text-lg mb-2">
                    <span>Total</span>
                    <span class="text-2xl">{move || format!("${:.3}", total_price.get())}</span>
                </div>
                <button
                    class="mt-6 w-full rounded-full bg-blue-600 py-3 text-white hover:bg-blue-700 disabled:bg-gray-300 text-base sm:text-lg font-bold shadow-lg hidden lg:block"
                    disabled=move || !is_form_valid.get()
                    on:click=open_modal
                >
                    Confirm & Book
                </button>
            </div>
        </div>
        </section>
        <Show when=show_modal>
            <div class="fixed inset-0 flex items-center justify-center z-50">
                <div
                    class="fixed inset-0 bg-black opacity-50"
                    on:click=move |_| show_modal.set(false)
                />
                <div class="w-full max-w-lg bg-white rounded-lg p-4 sm:p-8 z-50 shadow-xl relative mx-2">
                    <button
                        class="absolute top-2 right-2 sm:top-4 sm:right-4 text-gray-500 hover:text-gray-700"
                        on:click=move |_| show_modal.set(false)
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                        </svg>
                    </button>
                    <h2 class="text-xl font-bold text-center mb-6">Payment</h2>
                    <div class="flex flex-col gap-2 mb-6">
                        <div class="flex justify-between items-end">
                            <span class="text-lg font-bold">{move || format!("${:.3}", room_price.get())}</span>
                            <span class="ml-1 text-base font-normal text-gray-600">/night</span>
                        </div>
                        <div class="flex justify-between items-center text-base">
                            <span class="text-gray-700">{move || format!("${:.3} x {} nights", room_price.get(), num_nights.get())}</span>
                            <span class="font-semibold">{move || format!("${:.3}", room_price.get() * num_nights.get() as f64)}</span>
                        </div>
                        <div class="flex justify-between items-center text-base">
                            <span class="text-gray-700">Taxes and fees</span>
                            <span class="font-semibold">$0.00</span>
                        </div>
                        <Divider class="my-2".into() />
                        <div class="flex justify-between items-center font-bold text-lg mb-2">
                            <span>Total</span>
                            <span class="text-2xl">{move || format!("${:.3}", total_price.get())}</span>
                        </div>
                    </div>
                    <Show when=move || { should_not_have_loading_spinner.get() } fallback=SpinnerGray>
                        <div class="font-bold">
                            <label>"Pay with"</label>
                            <div class="flex flex-col w-full mt-4">
                                // <label for="binance" class="payment-button border-2 rounded-full p-3 flex items-center cursor-pointer relative peer-checked:border-green-500 peer-checked:bg-white">
                                // <div class="relative mb-8">
                                // <input type="radio" id="binance" name="payment" class="peer hidden" checked/>
                                // <div class="w-8 h-8 rounded-full border-2 border-gray-400 absolute  peer-checked:border-green-500"></div>
                                // <div class="w-6 h-6 rounded-full bg-white absolute left-1 top-1 peer-checked:bg-green-500"></div>
                                // </div>
                                // <button class="ml-5" on:click=move |_| handle_pay_signal.set("binance".to_owned())>
                                // <svg
                                //     xmlns="http://www.w3.org/2000/svg"
                                //     height="35"
                                //     width="160"
                                //     id="svg20"
                                //     version="1.1"
                                //     viewBox="-76.3875 -25.59 662.025 153.54"
                                // >
                                // <defs id="defs14"><style id="style12"/></defs><g transform="translate(-39.87 -50.56)" id="g18"><path id="path16" d="M63 101.74L51.43 113.3l-11.56-11.56 11.56-11.56zm28.05-28.07l19.81 19.82 11.56-11.56-31.37-31.37-31.37 31.37 11.56 11.56zm39.63 16.51l-11.56 11.56 11.56 11.56 11.55-11.56zm-39.63 39.63L71.24 110l-11.56 11.55 31.37 31.37 31.37-31.37L110.86 110zm0-16.51l11.56-11.56-11.56-11.56-11.56 11.56zm122 1.11v-.16c0-7.54-4-11.31-10.51-13.79 4-2.25 7.38-5.78 7.38-12.11v-.16c0-8.82-7.06-14.52-18.53-14.52h-26.04v56.14h26.7c12.67 0 21.02-5.13 21.02-15.4zm-15.4-24c0 4.17-3.45 5.94-8.9 5.94h-11.37V84.5h12.19c5.21 0 8.1 2.08 8.1 5.77zm3.13 22.46c0 4.17-3.29 6.09-8.75 6.09h-14.65v-12.33h14.27c6.34 0 9.15 2.33 9.15 6.1zM239 129.81V73.67h-12.39v56.14zm66.39 0V73.67h-12.23v34.57l-26.3-34.57h-11.39v56.14h12.19V94.12l27.18 35.69zm68.41 0l-24.1-56.54h-11.39l-24.05 56.54h12.59l5.15-12.59h23.74l5.13 12.59zm-22.45-23.5h-14.96l7.46-18.2zm81.32 23.5V73.67h-12.23v34.57l-26.31-34.57h-11.38v56.14h12.18V94.12l27.19 35.69zm63.75-9.06l-7.85-7.94c-4.41 4-8.34 6.57-14.76 6.57-9.62 0-16.28-8-16.28-17.64v-.16c0-9.62 6.82-17.48 16.28-17.48 5.61 0 10 2.4 14.36 6.33l7.83-9.06c-5.21-5.13-11.54-8.66-22.13-8.66-17.24 0-29.27 13.07-29.27 29v.16c0 16.12 12.27 28.87 28.79 28.87 10.81.03 17.22-3.82 22.99-9.99zm52.7 9.06v-11H518.6V107h26.47V96H518.6V84.66h30.08v-11h-42.35v56.14z" fill="#f0b90b"/></g>
                                // </svg>
                                // </button>
                                // </label>

                                // <br />

                                // <label
                                //     for="nowpay"
                                //     class="payment-button border-2 rounded-full p-3 flex items-center cursor-pointer relative border-gray-500"
                                // >
                                    // <div class="relative mb-8">
                                    // <input type="radio" id="nowpay" name="payment" class="peer hidden" />
                                    // <div class="w-8 h-8 rounded-full border-2 border-gray-400 absolute  peer-checked:border-green-500"></div>
                                    // <div class="w-6 h-6 rounded-full bg-white absolute left-1 top-1 peer-checked:bg-green-500"></div>
                                    // </div>
                                    // <Show when=payment_button_enabled fallback=SpinnerGray>
                                    // {move ||  if payment_button_enabled.get() {
                                    //     view!{
                                <button
                                    class="payment-button border-2 rounded-lg p-3 flex items-center cursor-pointer relative border-gray-500"
                                    on:click=move |_| {
                                        handle_pay_click.dispatch("NOWPayments".to_owned());
                                    }
                                >
                                    <span class="px-2 py-2"> Pay With Crypto </span>
                                </button>
                                <p class="text-sm mt-4 mb-6 text-red-500">
                                    Note: Full payment required. Partial payments are not supported and will not secure your reservation.
                                </p>
                                <div class="text-center text-red-500 text-xs sm:text-sm mt-4 border-t pt-2 sm:pt-4">
                                    <p>Do not close this tab until your payment is fully processed</p>
                                    <p>to avoid issues with your booking.</p>
                                </div>
                            </div>
                        </div>
                    </Show>
                </div>
            </div>
        </Show>
    }
}

// Helper function to create passenger details
pub fn create_passenger_details(
    adults: &[AdultDetail],
    children: &[ChildDetail],
) -> Vec<PassengerDetail> {
    let mut passengers = Vec::new();

    // Add adults
    for (i, adult) in adults.iter().enumerate() {
        passengers.push(PassengerDetail {
            title: "Mr".to_string(), // todo Add logic for title selection
            first_name: adult.first_name.clone(),
            last_name: adult
                .last_name
                .clone()
                .unwrap_or_else(|| "Not found".to_string()),
            email: if i == 0 {
                adult.email.clone().unwrap_or_default()
            } else {
                String::new()
            },
            pax_type: PaxType::Adult,
            lead_passenger: i == 0,
            age: _default_passenger_age(),
            ..Default::default()
        });
    }

    // Add children
    for child in children {
        passengers.push(PassengerDetail {
            title: "".to_string(),
            first_name: child.first_name.clone(),
            last_name: child
                .last_name
                .clone()
                .unwrap_or_else(|| "Not found".to_string()),
            email: String::new(),
            pax_type: PaxType::Child,
            lead_passenger: false,
            age: child
                .age
                .map(|age| age as u32)
                .expect("child age not defined"), // Convert u8 to u32
            ..Default::default()
        });
    }

    passengers
}
