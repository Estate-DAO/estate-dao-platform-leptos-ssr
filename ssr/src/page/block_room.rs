#![allow(unused)]
#![allow(dead_code)]

use crate::api::_default_passenger_age;
use crate::api::block_room;
use crate::api::canister::add_booking::add_booking_backend;
use crate::api::get_room;
use crate::api::payments::nowpayments_create_invoice;
use crate::api::payments::nowpayments_get_payment_status;
use crate::api::payments::ports::CreateInvoiceRequest;
use crate::api::payments::ports::GetPaymentStatusRequest;
use crate::api::payments::NowPayments;
use crate::canister::backend::BookRoomResponse;
use crate::canister::backend::HotelDetails;
use crate::canister::backend::HotelRoomDetails;
use crate::canister::backend::RoomDetails;
use crate::canister::backend::UserDetails;
use crate::component::SkeletonCards;
// use crate::page::call_server_fn::call_add_booking_server_fn;
use crate::state::search_state::BlockRoomResults;
use crate::state::search_state::HotelInfoResults;
use crate::state::view_state::AdultDetail;
use crate::state::view_state::BlockRoomCtx;
use crate::state::view_state::ChildDetail;
use crate::utils::app_reference::generate_app_reference;
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
    component::{Divider, FilterAndSortBy, PriceDisplay, StarRating},
    page::{InputGroup, Navbar},
    state::{
        search_state::{ConfirmationResults, SearchCtx, SearchListResults},
        view_state::HotelInfoCtx,
    },
};
use chrono::NaiveDate;
use leptos::logging::log;

#[component]
pub fn BlockRoomPage() -> impl IntoView {
    // ================ SETUP context  ================
    let search_ctx: SearchCtx = expect_context();
    let search_list_results: SearchListResults = expect_context();
    let search_list_results = store_value(search_list_results); // Store it so we can clone inside closure

    let hotel_info_results: HotelInfoResults = expect_context();
    let hotel_info_ctx: HotelInfoCtx = expect_context();

    let confirmation_results: ConfirmationResults = expect_context();

    let navigate = use_navigate();

    let block_room_ctx = expect_context::<BlockRoomCtx>();
    let block_room_results_context: BlockRoomResults = expect_context();

    // ================ Pricing component signals  ================

    let room_price = Signal::derive(move || {
        // let room_price = create_memo(move |_| {
        if let Some(block_room_response) = block_room_results_context.block_room_results.get() {
            block_room_response.get_room_price_summed()
        } else {
            0.0
        }
    });

    let num_nights = Signal::derive(move || search_ctx.date_range.get().no_of_nights());

    let total_price = create_memo(move |_| {
        let room_price = room_price.get();
        let nights = num_nights.get();
        room_price * nights as f64
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

        // Set the value of is_form_valid based on validation results
        is_form_valid
            .set(primary_adult_valid && other_adults_valid && children_valid && terms_valid);
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

    let confirmation_action = create_action(move |()| {
        let nav = nav.clone(); // Use the cloned version here
        let adults_data = adults.get();
        let children_data = children.get();
        let hotel_code = hotel_info_ctx.hotel_code.get().unwrap_or_default();
        let search_list_results = search_list_results.get_value(); // Get the stored value

        async move {
            let room_detail = RoomDetail {
                passenger_details: create_passenger_details(&adults_data, &children_data),
            };
            ConfirmationResults::set_room_details(Some(room_detail));

            // Get room_unique_id from HotelRoomResponse
            // let block_room_id = hotel_info_results
            //     .room_result
            //     .get()
            //     .and_then(|room_response| room_response.room_list.clone())
            //     .and_then(|room_list| {
            //         room_list
            //             .get_hotel_room_result
            //             .hotel_rooms_details
            //             .first()
            //             .cloned()
            //     })
            //     .map(|hotel_room_detail| hotel_room_detail.room_unique_id.clone())
            //     .unwrap_or_default();

            // let token = search_list_results
            //     .get_hotel_code_results_token_map()
            //     .get(&hotel_code)
            //     .cloned()
            //     .unwrap_or_default();

            // let book_request = BookRoomRequest {
            //     result_token: token,
            //     block_room_id,
            //     app_reference: format!("BOOKING_{}_{}", chrono::Utc::now().timestamp(), hotel_code),
            //     room_details: vec![room_detail],
            // };

            // match book_room(book_request).await {
            //     Ok(response) => {
            //         match response.status {
            //             BookingStatus::Confirmed => {
            //                 // Set the booking details in context
            //                 ConfirmationResults::set_booking_details(Some(response));
            nav(AppRoutes::Confirmation.to_string(), Default::default());
            //                 }
            //                 BookingStatus::BookFailed => {
            //                     log!("Booking failed: {:?}", response.message);
            //                 }
            //             }
            //         }
            //         Err(e) => {
            //             log!("Error booking room: {:?}", e);
            //         }
            //     }
        }
    });

    // ================  PAYMENT STATUS signals  ================

    let get_payment_status_action: Action<(), ()> = create_action(move |_| async move {
        let payment_id = 5991043299_u64;
        let resp = nowpayments_get_payment_status(GetPaymentStatusRequest { payment_id })
            .await
            .ok();
        BlockRoomResults::set_payment_results(resp);
    });

    // create_effect(move |_| {
    //     // Check the URL for a payment status parameter after redirect
    //     // use_query_params()
    //     let params = window().location().search().unwrap_or_default();
    //     let url_params = web_sys::UrlSearchParams::new_with_str(&params)
    //         .unwrap_or(web_sys::UrlSearchParams::new().unwrap());

    //     if let Some(payment_status) = url_params.get("payment") {
    //         match payment_status.as_str() {
    //             "success" => {
    //                 // Payment successful, trigger booking
    //                 confirmation_action.dispatch(());
    //             }
    //             "cancel" => {
    //                 // Payment cancelled, handle accordingly (e.g., show a message)
    //                 log!("Payment cancelled.");
    //             }
    //             "partial" => {
    //                 log!("Payment partially paid.");
    //             }
    //             _ => {
    //                 log!("Unknown payment status: {}", payment_status);
    //             }
    //         }
    //     }
    // });

    // ================  Confirmation Modal signals  ================

    let show_modal = create_rw_signal(false);
    let open_modal = move |_| {
        show_modal.set(true);
    };

    let _block_room_call = create_resource(show_modal, move |modal_value| {
        let hotel_info_results = expect_context::<HotelInfoResults>();
        let room_counters = hotel_info_results.block_room_counters.get_untracked();
        // log!("block_room.rs -- component room_coutners value - \n {room_counters:#?}");
        async move {
            if modal_value {
                // log!("modal open. calling block API now");

                // log!("{:?}", room_counters ); // Add this line
                // Reset previous block room results
                BlockRoomResults::reset();

                let uniq_room_ids: Vec<String> = room_counters
                    .values()
                    .filter_map(|counter| counter.value.clone())
                    .collect();
                log!("{uniq_room_ids:#?}");

                let block_room_request = hotel_info_results.block_room_request(uniq_room_ids);

                // Call server function inside action
                spawn_local(async move {
                    let result = block_room(block_room_request).await.ok();
                    // log!("BLOCK_ROOM_API: {result:?}");
                    BlockRoomResults::set_results(result);
                });
            } else {
                log!("modal closed. Nothing to do");
            }
        }
    });

    let handle_pay_click = move |payment_method: String| {
        match payment_method.as_str() {
            "binance" => {
                todo!();
            }
            "NOWPayments" => {
                let invoice_request = CreateInvoiceRequest {
                    price_amount: 1_u32,
                    // price_amount: total_price.get() as u32,
                    price_currency: "USD".to_string(),
                    order_id: "order_watever".to_string(),
                    order_description: "Hotel Room Booking".to_string(),
                    ipn_callback_url: "https://nowpayments.io".to_string(),
                    success_url: "127.0.0.1:3000/block_room?payment=success".to_string(),
                    cancel_url: "127.0.0.1:3000/block_room?payment=cancel".to_string(),
                    partially_paid_url: "127.0.0.1:3000/block_room?payment=partial".to_string(),
                    is_fixed_rate: false,
                    is_fee_paid_by_user: false,
                };

                let sorted_rooms = hotel_info_results.sorted_rooms.get();

                let room_details = sorted_rooms
                    .into_iter()
                    .map(|sorted_room| RoomDetails {
                        room_price: sorted_room.room_price as f32,
                        room_unique_id: sorted_room.room_unique_id,
                        room_type_name: sorted_room.room_type,
                    })
                    .collect();

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
                let user_selected_hotel_room_details = HotelRoomDetails {
                    destination,
                    requested_payment_amount: total_price.get(),
                    date_range,
                    room_details,
                    hotel_details: HotelDetails {
                        hotel_code: hotel_info_ctx.hotel_code.get().unwrap(),
                        hotel_name: hotel_info_ctx.selected_hotel_name.get(),
                        hotel_image: hotel_info_ctx.selected_hotel_image.get(),
                        hotel_location: hotel_info_ctx.selected_hotel_location.get(),
                    },
                };

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

                let email = <std::option::Option<std::string::String> as Clone>::clone(
                    &block_room_ctx.adults.get().first().unwrap().email,
                )
                .unwrap();
                let email_cloned = email.clone();
                let email_cloned_twice = email.clone();
                let local_booking_id = generate_app_reference(email_cloned_twice);

                let booking_id = (local_booking_id.get().get_app_reference(), email);
                let booking_id_cloned = booking_id.clone();
                let payment_details = crate::canister::backend::PaymentDetails {
                    payment_status: crate::canister::backend::BackendPaymentStatus::Unpaid(Some(
                        "Payment Not Started".into(),
                    )),
                    booking_id,
                };

                let booking = crate::canister::backend::Booking {
                    user_selected_hotel_room_details,
                    guests,
                    booking_id: booking_id_cloned,
                    book_room_status: None,
                    payment_details,
                };

                log!("block_room page - booking - {:#?}", booking);

                spawn_local(async move {
                    let create_invoice_response = nowpayments_create_invoice(invoice_request).await;
                    log!("creating invoice");
                    match create_invoice_response {
                        Ok(resp) => {
                            log!("heres CreateInvoiceResponse");
                            log!("{:?}\n{:?}", email_cloned, booking);
                            let value_for_serverfn: String =
                                serde_json::to_string(&booking).unwrap();

                            match add_booking_backend(email_cloned, value_for_serverfn).await {
                                Ok(response) => {
                                    log!("\n\n\n ____________WORKING>>>>\n\n{:#}", response);
                                    // let _ = window().location().assign(&resp.invoice_url);
                                    let _ = window()
                                        .location()
                                        .assign(&"http://localhost:3000/confirmation".to_string());
                                }
                                Err(e) => {
                                    log!("Error saving values {:?}", e);
                                }
                            }
                        }
                        Err(e) => {
                            log!("Error creating invoice: {:?}", e);
                        }
                    }

                    // match invoice_url {
                    //     Ok(url) => {
                    //         let _ = window().location().assign(&url);
                    //         // let payment_status_response = nowpayments.get_payment_status("payment_id?").await;

                    //         // match payment_status_response {
                    //         //     Ok(status) => {
                    //         //         if status == PaymentStatus::Finished {
                    //         //             handle_booking.dispatch(());
                    //         //         } else {
                    //         //             log!("Payment not successful: {:?}", status);
                    //         //             // Optionally, redirect back to the booking page or display an error message
                    //         //         }
                    //         //     }
                    //         //     Err(e) => {
                    //         //         log!("Error getting payment status: {:?}", e);
                    //         //         // Handle error, e.g., display an error message
                    //         //     }
                    //         // }
                    //     }
                    //     Err(e) => {
                    //         log!("Error creating invoice: {:?}", e);
                    //     }
                    // }
                });
            }
            _ => { /* Handle other payment methods */ }
        }
        show_modal.set(false);
    };

    // create_effect(move |_| {
    //     // Check the URL for a payment status parameter after redirect
    //     // use_query_params()
    //     let params = window().location().search().unwrap_or_default();
    //     let url_params = web_sys::UrlSearchParams::new_with_str(&params)
    //         .unwrap_or(web_sys::UrlSearchParams::new().unwrap());

    //     if let Some(payment_status) = url_params.get("payment") {
    //         match payment_status.as_str() {
    //             "success" => {
    //                 // Payment successful, trigger booking
    //                 confirmation_action.dispatch(());
    //             }
    //             "cancel" => {
    //                 // Payment cancelled, handle accordingly (e.g., show a message)
    //                 log!("Payment cancelled.");
    //             }
    //             "partial" => {
    //                 log!("Payment partially paid.");
    //             }
    //             _ => {
    //                 log!("Unknown payment status: {}", payment_status);
    //             }
    //         }
    //     }
    // });

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

        // Set the value of is_form_valid based on validation results
        is_form_valid
            .set(primary_adult_valid && other_adults_valid && children_valid && terms_valid);
    };

    // Call the validation function whenever inputs change
    let _ = create_effect(move |_| {
        validate_form();
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
        <section class="relative h-screen">
            <Navbar />
            <div class="relative mt-24 flex h-screen  items-center place-content-center p-4 max-w-4xl mx-auto ">
                <div class="container w-4/5 justify-between gap-6">
                    <button type="text" class="text-3xl font-bold pb-4 flex" on:click=go_back_to_details>
                    // "<- You're just one step away!"
                    <span class="inline-block items-center"> <Icon icon=icondata::AiArrowLeftOutlined class="text-black font-light" /> </span>
                    <div class="ml-4">"You're just one step away!" </div>
                    </button>
                    <br />
                    <div class="p-6">
                        <h3 class="text-xl font-bold">"Your Booking Details"</h3>
                        <div class="details mb-4 flex">
                            <img
                                src=insert_real_image_or_default
                                alt={move || hotel_info_ctx.selected_hotel_name.get()}
                                class="h-24 w-24 rounded-lg object-cover"
                            />
                            <div class="pt-6 p-2">
                                <h3 class="font-semibold">{move || hotel_info_ctx.selected_hotel_name.get()}</h3>
                                <p class="text-gray-600">{move || hotel_info_ctx.selected_hotel_location.get()}</p>
                            </div>
                        </div>
                        <div class="details">
                            <p class="mt-2"><strong>"Dates: "</strong>
                            {move || {
                                let date_range = search_ctx.date_range.get();
                                date_range.format_as_human_readable_date()
                            }}
                            </p>
                            <p class="mt-2"><strong>"Guests: "</strong>
                                {move || format!("{} adults, {} children",
                                    adult_count.get(),
                                    child_count.get()
                                )}
                            </p>
                            <p class="mt-2"><strong>"Rooms: "</strong>
                                {move || format!("{} {}",
                                    num_rooms.get(),
                                    if num_rooms.get() == 1 { "room" } else { "rooms" }
                                )}
                            </p>
                        </div>
                        <br />
                        <Divider />
                        <br />

                        <div class="payment-methods mt-4 space-y-6">
                        { // Loop for adults
                            (0..adult_count.get()).map(|i| {
                                let i_usize = i as usize;
                                view! {
                                    <div class="person-details">
                                        <h3 class="font-semibold text-gray-700">
                                            {if i == 0 {
                                                String::from("Primary Adult")
                                            } else {
                                                format!("Adult {}", i + 1)
                                            }}
                                        </h3>
                                        <div class="flex gap-4">
                                        <input
                                            type="text"
                                            placeholder="First Name *"
                                            class="w-1/2 rounded-md border border-gray-300 p-2"
                                            required=true
                                            on:input=move |ev| {
                                                update_adult(i_usize, "first_name", event_target_value(&ev));
                                                validate_form();
                                            }
                                        />
                                        <input type="text" placeholder="Last Name" class="w-1/2 rounded-md border border-gray-300 p-2"
                                        required=true
                                        on:input=move |ev| {
                                            update_adult(i_usize, "last_name", event_target_value(&ev));
                                            validate_form();
                                        }
                                        />
                                        </div>
                                        {move || if i == 0 {
                                            view! {
                                                <div>
                                                    <input type="email" placeholder="Email *" class="mt-2 w-full rounded-md border border-gray-300 p-2"
                                                        required=true on:input=move |ev| update_adult(0, "email", event_target_value(&ev))
                                                    />
                                                    <input type="tel" placeholder="Phone *" class="mt-2 w-full rounded-md border border-gray-300 p-2"
                                                        required=true on:input=move |ev| update_adult(0, "phone", event_target_value(&ev))
                                                    />
                                                </div>
                                            }.into_view()
                                        } else {
                                            view! { <div></div> }.into_view()
                                        }}
                                    </div>
                                }
                            }).collect::<Vec<_>>()
                        }

                        { // Loop for children
                            (0..child_count.get()).map(|i| {
                                let i_usize = i as usize;
                                let age_value = children_ages.get_value_at(i as u32); // Get the age for the current child

                                view! {
                                    <div class="person-details">
                                        <h3 class="font-semibold text-gray-700">{format!("Child {}", i + 1)}</h3>
                                        <div class="flex gap-4">
                                            <input
                                                type="text"
                                                placeholder="First Name *"
                                                class="w-2/5 rounded-md border border-gray-300 p-2"
                                                required=true
                                                on:input=move |ev| {
                                                    update_child(i_usize, "first_name", event_target_value(&ev));
                                                    validate_form();
                                                }
                                            />
                                            <input
                                                type="text"
                                                placeholder="Last Name"
                                                class="w-2/5 rounded-md border border-gray-300 p-2"
                                            />
                                            <select
                                                class="w-1/5 rounded-md border border-gray-300 bg-white p-2"
                                                required=true
                                                on:input=move |ev| {
                                                    update_child(i_usize, "age", event_target_value(&ev));
                                                    validate_form();
                                                }
                                            >

                                            <option disabled selected>{age_value}</option>
                                            { (1..18).map(|age| {
                                                let selected = if age == age_value { "selected" } else { "" };
                                                view! { <option value={age.to_string()} {selected}>{age}</option> }
                                            }).collect::<Vec<_>>() }
                                            </select>
                                        </div>
                                    </div>
                                }
                            }).collect::<Vec<_>>()
                        }
                        </div>
                        <br />
                        <Divider />
                        <br />
                        <h2 class="text-2xl font-bold">"Cancellation Policy"</h2>
                        <div class="cancellation-policy mt-6 text-sm text-gray-600">
                            "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Suspendisse felis massa, dignissim eu luctus vel, interdum facilisis orci."
                            <a href="#" class="text-blue-500 hover:underline">"Read more"</a>.
                        </div>
                        <br />
                        <Divider />
                        <br />
                        <div>
                            <input
                                type="checkbox"
                                id="agree"
                                class="mr-2"
                                on:change=move |ev| update_terms(event_target_checked(&ev))
                            />
                            <label for="agree" class="text-sm text-gray-600">
                                "I also agree to the updated "
                                <a href="#" class="text-blue-500 hover:underline">"Terms of Service"</a>
                                ", "
                                <a href="#" class="text-blue-500 hover:underline">"Payments Terms of Service"</a>
                                " and acknowledge the "
                                <a href="#" class="text-blue-500 hover:underline">"Privacy Policy"</a>
                            </label>
                        </div>
                        <button
                            class="mt-6 w-1/3 rounded-full bg-blue-600 py-3 text-white hover:bg-blue-700 disabled:bg-gray-300"
                            disabled=move || !is_form_valid.get()
                            on:click=open_modal
                        >
                            "Confirm and pay"
                        </button>
                    </div>
                </div>
                <div class="mb-[40rem] rounded-xl bg-white p-6 shadow-xl">
                    <h2 class="mb-4 text-xl font-bold">"₹"{move || room_price.get() as u64}"/" "night" </h2>
                    <Divider />
                    <div class="price-breakdown">
                        <div class="flex justify-between">
                            <span>"₹"{move || room_price.get() as u64}" x "{move || {
                                let nights = move || num_nights.get();
                                pluralize(nights(), "night", "nights")}
                            }</span>
                        </div>

                        <div class="price-total mt-4 flex justify-between font-bold">
                            <span>"Total"</span>
                            <span>"₹"{move || total_price.get() as u64}</span>
                        </div>
                    </div>
                </div>
            </div>
        </section>
        <Show when=move || show_modal.get()>
            <div class="fixed inset-0 flex items-center justify-center z-50">
                <div class="fixed inset-0 bg-black opacity-50" on:click=move |_| show_modal.set(false)></div>
                <div class="w-1/2 max-w-[60rem] bg-white rounded-lg p-8 z-50 shadow-xl">
                    <div class="flex justify-between">
                        <h2 class="mb-4 text-xl font-bold">"₹"{move || room_price.get() as u64}"/" "night" </h2>
                        <span> "x "{move || pluralize(num_nights.get(), "night", "nights")}</span>
                    </div>
                        <Divider />
                        <div class="price-breakdown price-total mt-4 flex justify-between font-bold">
                            <span>"Total"</span >
                            <span>"₹"{move || total_price.get() as u64}</span>
                        </div>
                        <Divider />
                        <div class="font-bold">
                            <label >"Pay with"</label>
                            <div class="flex flex-col w-full mt-4">
                                <label for="binance" class="payment-button border-2 rounded-full p-3 flex items-center cursor-pointer relative peer-checked:border-green-500 peer-checked:bg-white">
                                    <div class="relative mb-8">
                                        <input type="radio" id="binance" name="payment" class="peer hidden" checked/>
                                        <div class="w-8 h-8 rounded-full border-2 border-gray-400 absolute  peer-checked:border-green-500"></div>
                                        <div class="w-6 h-6 rounded-full bg-white absolute left-1 top-1 peer-checked:bg-green-500"></div>
                                    </div>
                                    <button class="ml-5" on:click=move |_| handle_pay_click("binance".to_string())>
                                                                <svg xmlns="http://www.w3.org/2000/svg" height="35" width="160" id="svg20" version="1.1" viewBox="-76.3875 -25.59 662.025 153.54">
                                <defs id="defs14"><style id="style12"/></defs><g transform="translate(-39.87 -50.56)" id="g18"><path id="path16" d="M63 101.74L51.43 113.3l-11.56-11.56 11.56-11.56zm28.05-28.07l19.81 19.82 11.56-11.56-31.37-31.37-31.37 31.37 11.56 11.56zm39.63 16.51l-11.56 11.56 11.56 11.56 11.55-11.56zm-39.63 39.63L71.24 110l-11.56 11.55 31.37 31.37 31.37-31.37L110.86 110zm0-16.51l11.56-11.56-11.56-11.56-11.56 11.56zm122 1.11v-.16c0-7.54-4-11.31-10.51-13.79 4-2.25 7.38-5.78 7.38-12.11v-.16c0-8.82-7.06-14.52-18.53-14.52h-26.04v56.14h26.7c12.67 0 21.02-5.13 21.02-15.4zm-15.4-24c0 4.17-3.45 5.94-8.9 5.94h-11.37V84.5h12.19c5.21 0 8.1 2.08 8.1 5.77zm3.13 22.46c0 4.17-3.29 6.09-8.75 6.09h-14.65v-12.33h14.27c6.34 0 9.15 2.33 9.15 6.1zM239 129.81V73.67h-12.39v56.14zm66.39 0V73.67h-12.23v34.57l-26.3-34.57h-11.39v56.14h12.19V94.12l27.18 35.69zm68.41 0l-24.1-56.54h-11.39l-24.05 56.54h12.59l5.15-12.59h23.74l5.13 12.59zm-22.45-23.5h-14.96l7.46-18.2zm81.32 23.5V73.67h-12.23v34.57l-26.31-34.57h-11.38v56.14h12.18V94.12l27.19 35.69zm63.75-9.06l-7.85-7.94c-4.41 4-8.34 6.57-14.76 6.57-9.62 0-16.28-8-16.28-17.64v-.16c0-9.62 6.82-17.48 16.28-17.48 5.61 0 10 2.4 14.36 6.33l7.83-9.06c-5.21-5.13-11.54-8.66-22.13-8.66-17.24 0-29.27 13.07-29.27 29v.16c0 16.12 12.27 28.87 28.79 28.87 10.81.03 17.22-3.82 22.99-9.99zm52.7 9.06v-11H518.6V107h26.47V96H518.6V84.66h30.08v-11h-42.35v56.14z" fill="#f0b90b"/></g>
                                </svg>
                                    </button>
                                </label>

                                <br />

                                <label for="nowpay" class="payment-button border-2 rounded-full p-3 flex items-center cursor-pointer relative peer-checked:border-green-500 peer-checked:bg-white">
                                    <div class="relative mb-8">
                                        <input type="radio" id="nowpay" name="payment" class="peer hidden" />
                                        <div class="w-8 h-8 rounded-full border-2 border-gray-400 absolute  peer-checked:border-green-500"></div>
                                        <div class="w-6 h-6 rounded-full bg-white absolute left-1 top-1 peer-checked:bg-green-500"></div>
                                    </div>
                                    <button class="ml-10" on:click=move |_| handle_pay_click("NOWPayments".to_string())>
                                                                <svg xmlns="http://www.w3.org/2000/svg" width="163" height="21" fill="none">
                                    <path d="M12.386 13.815l-.506.161V1.05h1.84V16h-1.84L2.174 3.258l.506-.161V16H.84V1.05h1.84l9.706 12.765zm12.278 2.415c-1.365 0-2.607-.314-3.726-.943-1.104-.644-1.986-1.541-2.645-2.691-.66-1.15-.99-2.507-.99-4.071 0-1.564.33-2.921.99-4.071.659-1.165 1.54-2.062 2.645-2.691 1.119-.629 2.361-.943 3.726-.943 1.364 0 2.599.314 3.703.943 1.119.629 2.008 1.526 2.668 2.691.659 1.15.989 2.507.989 4.071 0 1.564-.33 2.921-.99 4.071a7.137 7.137 0 01-2.667 2.691c-1.104.629-2.339.943-3.703.943zm0-1.725c1.042 0 1.978-.222 2.806-.667.828-.46 1.487-1.135 1.978-2.024.49-.89.736-1.986.736-3.289 0-1.303-.246-2.392-.736-3.266-.491-.89-1.15-1.564-1.978-2.024-.828-.46-1.764-.69-2.806-.69-1.028 0-1.963.23-2.806.69-.828.46-1.488 1.135-1.978 2.024-.491.874-.736 1.963-.736 3.266s.245 2.4.736 3.289c.49.89 1.15 1.564 1.978 2.024.843.445 1.778.667 2.806.667zM38.924 16l-5.38-14.95h1.978l4.623 12.972h-.552L44.03 1.05h1.978l4.439 12.972h-.552L54.519 1.05h1.932L51.115 16h-1.932L44.767 2.568h.506L40.857 16h-1.932z" fill="#64ACFF"/>

                                    <path d="M60.78 10.71V8.985h4.692c.92 0 1.663-.284 2.23-.851.584-.583.875-1.334.875-2.254 0-.95-.291-1.702-.874-2.254-.568-.567-1.311-.851-2.231-.851h-4.324V16h-1.84V1.05h6.164c.69 0 1.334.107 1.932.322.598.215 1.12.529 1.564.943.46.414.813.92 1.058 1.518.26.598.39 1.28.39 2.047 0 .767-.13 1.449-.39 2.047a4.11 4.11 0 01-1.058 1.518 4.496 4.496 0 01-1.564.943 5.676 5.676 0 01-1.932.322H60.78zm16.675 5.52a4.46 4.46 0 01-2.507-.759c-.751-.521-1.357-1.227-1.817-2.116-.445-.905-.667-1.932-.667-3.082s.23-2.177.69-3.082c.46-.905 1.08-1.618 1.863-2.139a4.77 4.77 0 012.668-.782c1.073 0 1.886.276 2.438.828.567.537.95 1.257 1.15 2.162.215.905.322 1.909.322 3.013 0 .583-.061 1.219-.184 1.909a7.501 7.501 0 01-.644 1.955 3.997 3.997 0 01-1.242 1.518c-.537.383-1.227.575-2.07.575zm.46-1.748c.782 0 1.426-.192 1.932-.575.521-.383.905-.89 1.15-1.518.245-.644.368-1.35.368-2.116 0-.843-.13-1.58-.391-2.208-.245-.644-.629-1.142-1.15-1.495-.506-.368-1.142-.552-1.91-.552-1.15 0-2.038.406-2.667 1.219-.629.797-.943 1.81-.943 3.036 0 .813.153 1.541.46 2.185a3.96 3.96 0 001.288 1.495 3.386 3.386 0 001.863.529zm3.45-9.982h1.84V16h-1.68l-.045-.414a35.32 35.32 0 00-.07-.966 11.69 11.69 0 01-.045-.92V4.5zm4.403 0h1.978l4.117 11.04-1.61.46-4.485-11.5zm10.695 0l-6.417 16.1h-2.001l2.783-6.21 3.68-9.89h1.955zm2.552 0h1.84V16h-1.84V4.5zm5.796-.23c.613 0 1.165.092 1.656.276.491.184.905.452 1.242.805.337.353.598.782.782 1.288.184.49.276 1.05.276 1.679V16h-1.84V8.778c0-.89-.207-1.556-.621-2.001-.414-.445-1.035-.667-1.863-.667-.629 0-1.211.161-1.748.483a3.964 3.964 0 00-1.334 1.334c-.337.552-.544 1.196-.621 1.932l-.023-1.334c.077-.644.23-1.227.46-1.748a4.95 4.95 0 01.897-1.334 3.817 3.817 0 011.242-.874c.475-.2.974-.299 1.495-.299zm7.728 0c.613 0 1.165.092 1.656.276.491.184.905.452 1.242.805.337.353.598.782.782 1.288.184.49.276 1.05.276 1.679V16h-1.84V8.778c0-.89-.207-1.556-.621-2.001-.414-.445-1.035-.667-1.863-.667-.629 0-1.211.161-1.748.483a3.964 3.964 0 00-1.334 1.334c-.337.552-.544 1.196-.621 1.932l-.023-1.334c.077-.644.23-1.227.46-1.748a4.95 4.95 0 01.897-1.334 3.817 3.817 0 011.242-.874c.475-.2.974-.299 1.495-.299zm15.68 8.027h1.725a4.735 4.735 0 01-.828 2.024 4.501 4.501 0 01-1.656 1.403c-.675.337-1.457.506-2.346.506-1.074 0-2.04-.253-2.898-.759a5.765 5.765 0 01-2.047-2.116c-.491-.905-.736-1.932-.736-3.082s.237-2.177.713-3.082a5.592 5.592 0 012.001-2.139c.843-.521 1.794-.782 2.852-.782 1.119 0 2.062.253 2.829.759.782.49 1.357 1.219 1.725 2.185.383.95.537 2.124.46 3.519h-8.74c.076.751.283 1.41.621 1.978a3.706 3.706 0 001.334 1.334c.552.307 1.18.46 1.886.46.782 0 1.441-.2 1.978-.598.552-.414.928-.95 1.127-1.61zm-3.151-6.302c-.951 0-1.748.276-2.392.828-.644.552-1.074 1.303-1.288 2.254h6.739c-.062-1.012-.391-1.779-.989-2.3a3.003 3.003 0 00-2.07-.782zm7.917-1.495h1.84V16h-1.84V4.5zm5.796-.23c.614 0 1.166.092 1.656.276.491.184.905.452 1.242.805.338.353.598.782.782 1.288.184.49.276 1.05.276 1.679V16h-1.84V8.778c0-.89-.207-1.556-.621-2.001-.414-.445-1.035-.667-1.863-.667-.628 0-1.211.161-1.748.483a3.973 3.973 0 00-1.334 1.334c-.337.552-.544 1.196-.621 1.932l-.023-1.334c.077-.644.23-1.227.46-1.748.246-.521.545-.966.897-1.334a3.817 3.817 0 011.242-.874c.476-.2.974-.299 1.495-.299zm6.735.23h6.762v1.725h-6.762V4.5zm2.461-2.99h1.84V16h-1.84V1.51zm6.523 10.787h1.702c.046.66.291 1.196.736 1.61.46.414 1.119.621 1.978.621.521 0 .927-.069 1.219-.207.306-.138.529-.322.667-.552.138-.245.207-.514.207-.805 0-.353-.092-.629-.276-.828a1.856 1.856 0 00-.713-.529 8.32 8.32 0 00-1.012-.391 222.72 222.72 0 01-1.38-.529c-.46-.2-.897-.43-1.311-.69a3.474 3.474 0 01-.966-.989c-.246-.399-.368-.874-.368-1.426 0-.46.092-.89.276-1.288a2.97 2.97 0 01.782-1.058 3.86 3.86 0 011.219-.713 4.643 4.643 0 011.564-.253c.766 0 1.41.153 1.932.46.536.307.943.744 1.219 1.311.291.552.444 1.196.46 1.932h-1.587c-.108-.69-.33-1.188-.667-1.495-.322-.322-.79-.483-1.403-.483-.629 0-1.112.138-1.449.414-.338.276-.506.644-.506 1.104 0 .337.122.621.368.851.245.215.559.414.943.598.398.169.82.345 1.265.529.444.184.874.368 1.288.552.414.184.782.399 1.104.644.322.245.575.552.759.92.199.353.299.797.299 1.334 0 .644-.154 1.219-.46 1.725-.292.49-.729.874-1.311 1.15-.568.276-1.265.414-2.093.414-.752 0-1.396-.092-1.932-.276a4.319 4.319 0 01-1.311-.759 4.13 4.13 0 01-.828-1.012 3.648 3.648 0 01-.368-1.035 2.77 2.77 0 01-.046-.851z" fill="#000000"/>
                                </svg>
                                    </button>
                                </label>
                            </div>
                        </div>
                </div>
            </div>
        </Show>
    }
}

// Helper function to create passenger details
fn create_passenger_details(
    adults: &[AdultDetail],
    children: &[ChildDetail],
) -> Vec<PassengerDetail> {
    let mut passengers = Vec::new();

    // Add adults
    for (i, adult) in adults.iter().enumerate() {
        passengers.push(PassengerDetail {
            title: "Mr".to_string(), // todo Add logic for title selection
            first_name: adult.first_name.clone(),
            last_name: adult.last_name.clone().unwrap_or_default(),
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
            last_name: child.last_name.clone().unwrap_or_default(),
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
