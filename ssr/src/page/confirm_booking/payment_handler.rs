use crate::api::canister::update_payment_details::update_payment_details_backend;
use crate::api::payments::ports::GetPaymentStatusResponse;
use crate::api::{FailureGetPaymentStatusResponse, SuccessGetPaymentStatusResponse};
use crate::canister::backend::{BePaymentApiResponse, PaymentDetails};
use crate::component::SelectedDateRange;
use crate::cprintln;
use crate::page::confirm_booking::booking_handler::read_booking_details_from_local_storage;
use crate::utils::app_reference;
use crate::{
    api::payments::{nowpayments_get_payment_status, ports::GetPaymentStatusRequest},
    canister::backend,
    page::{
        confirm_booking::payment_handler::backend::BackendPaymentStatus::Paid,
        PaymentBookingStatusUpdates,
    },
    state::{
        local_storage::{use_booking_id_store, use_payment_store},
        search_state::{BlockRoomResults, ConfirmationResults},
        view_state::{BlockRoomCtx, HotelInfoCtx},
    },
};
use colored::Colorize;
use k256::elliptic_curve::rand_core::block;
use leptos::logging::log;
use leptos::*;
use leptos_query::{QueryOptions, QueryScope, ResourceOption};
use leptos_router::*;
use leptos_use::{use_interval_fn_with_options, utils::Pausable, UseIntervalFnOptions};

#[allow(non_snake_case)]
#[derive(Params, PartialEq, Clone, Debug)]
struct NowpaymentsPaymentId {
    NP_id: u64,
}

#[component]
pub fn PaymentHandler() -> impl IntoView {
    let (booking_id_signal_read, _, _) = use_booking_id_store();
    let (payment_store, set_payment_store, _) = use_payment_store();

    let block_room_results: BlockRoomResults = expect_context();
    let confirmation_results: ConfirmationResults = expect_context();

    let payment_booking_step_signals: PaymentBookingStatusUpdates = expect_context();

    // ========= get payments id from query param and store in local storage ==========

    let np_id_query_map = use_query::<NowpaymentsPaymentId>();

    let np_payment_id = create_memo(move |_| {
        // let np_payment_id = Signal::derive(move || {
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
        log!("create_effect - update np_payment_id = {np_payment_id:?}");
        // let (payment_store, set_payment_store, _) = use_payment_store();
        if payment_store.get().is_some() {
            return;
        }
        set_payment_store(np_payment_id.get())
    });

    //  ================================= get_payment_status_api_call -  polling =================================\
    // let payments_api_called = create_rw_signal(false);

    let Pausable {
        pause,
        resume,
        is_active,
    } = use_interval_fn_with_options(
        move || {
            spawn_local(async move {
                if !payment_booking_step_signals
                    .p01_fetch_payment_details_from_api
                    .get_untracked()
                {
                    let p01 = payment_booking_step_signals
                        .p01_fetch_payment_details_from_api
                        .get_untracked();

                    println!(
                        "{}",
                        format!("p01_fetch_payment_details_from_api - {}", p01)
                            .red()
                            .bold()
                    );
                    // cprintln!("red", "p01_fetch_payment_details_from_api - {}", p01);
                    return;
                }

                if let None = payment_store.get_untracked() {
                    println!("{}", format!("payment_store - is None").red().bold());
                    return;
                }

                let payment_id = np_payment_id.get_untracked().unwrap();

                let resp = nowpayments_get_payment_status(GetPaymentStatusRequest { payment_id })
                    .await
                    .ok();

                // payments_api_called.set(true);
                // set to context
                BlockRoomResults::set_payment_results(resp.clone());
                // store payment_id in local storage
                set_payment_store(Some(payment_id));

                if resp.is_some() {
                    log!("nowpayments_get_payment_status - {resp:?}");

                    // trigger the save_payment_details_to_backend
                    payment_booking_step_signals
                        .p02_update_payment_details_to_backend
                        .set(true);
                }
            });
        },
        4_000,
        UseIntervalFnOptions {
            immediate: true,
            immediate_callback: true,
        },
    );

    // if the updates to remote API are done, stop the API call use_interval
    create_effect(move |_| {
        let should_stop_timer = payment_booking_step_signals
            .p02_update_payment_details_to_backend
            .get();

        log!("inside create_effect should_stop_timer - {should_stop_timer:?}");

        if should_stop_timer {
            pause();
        } else {
            resume();
        };
    });

    let update_payment_details_to_backend = create_resource(
        move || {
            payment_booking_step_signals
                .p02_update_payment_details_to_backend
                .get()
        },
        move |p02_update_payment_details_to_backend| async move {
            if !p02_update_payment_details_to_backend {
                // the data will not be present in context => early return
                log!("early return - p01_fetch_payment_details_from_api - {p02_update_payment_details_to_backend:?}");
                return None;
            }

            // unwrap is safe - if the payment_status_response is not available, early return happens.
            if let Some(get_payment_status_response) =
                block_room_results.payment_status_response.get()
            {
                let get_payment_status_response_clone = get_payment_status_response.clone();
                log!("get_payment_status_response_clone - {get_payment_status_response_clone:#?}");

                let get_payment_status_response_for_backend = backend::BePaymentApiResponse::from(
                    (get_payment_status_response, "NOWPayments".to_string()),
                );

                // unwrap is safe because these details must be present in the first step aka getting booking details from backend.
                // and first step will set the signal p01_fetch_payment_details_from_api to true
                let (email, app_reference) = read_booking_details_from_local_storage().unwrap();

                let payment_details = backend::PaymentDetails {
                    booking_id: (app_reference, email),
                    payment_status: backend::BackendPaymentStatus::Unpaid(None),
                    payment_api_response: get_payment_status_response_for_backend,
                };

                let payment_details_str = serde_json::to_string(&payment_details)
                    .expect("payment details is cannot be serialized using serde_json");

                // log!(" get_payment_status_response_clone.payment_status - {}",  get_payment_status_response_clone.payment_status);

                match get_payment_status_response_clone.get_payment_status() {
                    // TODO [UAT] : logic for other statuses too (other than finished)
                    status if status == "finished" => {
                        log!("payment status finished");

                        let payment_resp = get_payment_status_response_clone;

                        let payment_api_response =
                            BePaymentApiResponse::from((payment_resp, "NOWPayments".to_string()));

                        // let (booking_id_signal_read, booking_id_signal_write, _) = use_booking_id_store();
                        // let confirmation_ctx = expect_context::<ConfirmationResults>();
                        // let block_room_ctx = expect_context::<BlockRoomResults>();

                        let app_reference_string = booking_id_signal_read
                            .get_untracked()
                            .and_then(|booking| Some(booking.get_app_reference()));
                        let email = booking_id_signal_read
                            .get_untracked()
                            .and_then(|booking| Some(booking.get_email()));

                        if !(app_reference_string.is_some() && email.is_some()) {
                            log!(
                                "MISSING FIELD >>>\nApp reference ->{:?}\n Email -> {:?}",
                                app_reference_string,
                                email
                            );
                            return None;
                        }

                        let payment_api_response_cloned = payment_api_response.clone();
                        let app_reference_string_cloned = app_reference_string.clone();
                        let app_reference_string_cloned2 = app_reference_string.clone();
                        let email_cloned = email.clone();
                        let email_cloned2 = email.clone();

                        let payment_details = PaymentDetails {
                            booking_id: (app_reference_string.unwrap(), email.unwrap()),
                            // Sending order_id currently with this, change as necessary
                            payment_status: Paid(payment_api_response_cloned.order_id),
                            payment_api_response,
                        };

                        let payment_details_str = serde_json::to_string(&payment_details)
                            .expect("payment details is not valid json");

                        let booking_id_for_request =
                            (app_reference_string_cloned.unwrap(), email_cloned.unwrap());

                        let status_cloned = status.clone();
                        spawn_local(async move {
                            match update_payment_details_backend(
                                booking_id_for_request,
                                payment_details_str,
                            )
                            .await
                            {
                                Ok(booking) => {
                                    println!("{}", format!("{booking:?}").red().bold());
                                    // let app_reference_string_cloned =
                                    //     app_reference_string_cloned.clone();
                                    // let email_cloned = email_cloned2.clone();
                                    // let hotel_det_cloned = booking
                                    //     .user_selected_hotel_room_details
                                    //     .hotel_details
                                    //     .clone();

                                    // let date_range = SelectedDateRange {
                                    //     start: booking
                                    //         .user_selected_hotel_room_details
                                    //         .date_range
                                    //         .start,
                                    //     end: booking.user_selected_hotel_room_details.date_range.end,
                                    // };

                                    // SearchCtx::set_date_range(date_range);
                                    // HotelInfoCtx::set_selected_hotel_details(
                                    //     booking
                                    //         .user_selected_hotel_room_details
                                    //         .hotel_details
                                    //         .hotel_code,
                                    //     booking
                                    //         .user_selected_hotel_room_details
                                    //         .hotel_details
                                    //         .hotel_name,
                                    //     booking
                                    //         .user_selected_hotel_room_details
                                    //         .hotel_details
                                    //         .hotel_image,
                                    //     booking
                                    //         .user_selected_hotel_room_details
                                    //         .hotel_details
                                    //         .hotel_location,
                                    // );

                                    // // Storing hotel_location is the field given for hotel_category becoz why not
                                    // let hotel_res = HotelResult {
                                    //     hotel_code: hotel_det_cloned.hotel_code,
                                    //     hotel_name: hotel_det_cloned.hotel_name,
                                    //     hotel_picture: hotel_det_cloned.hotel_image,
                                    //     hotel_category: hotel_det_cloned.hotel_location,
                                    //     result_token: hotel_det_cloned.hotel_token,
                                    //     star_rating: 0,
                                    //     price: Price::default(),
                                    // };
                                    // let hotel_search_res = HotelSearchResult {
                                    //     hotel_results: vec![hotel_res],
                                    // };
                                    // let search_res = Search {
                                    //     hotel_search_result: hotel_search_res,
                                    // };
                                    // let hotel_search_resp = HotelSearchResponse {
                                    //     status: 0,
                                    //     message: "Default Message".to_string(),
                                    //     search: Some(search_res),
                                    // };

                                    // SearchListResults::set_search_results(Some(hotel_search_resp));

                                    // let book_room_status = booking.book_room_status;
                                    // let book_room_status_cloned = book_room_status.clone();
                                    // let book_room_status_twice = book_room_status.clone();
                                    // let booking_status_cloned_again = book_room_status_cloned.clone();
                                    // let booking_status_cloned_again1 = book_room_status_cloned.clone();
                                    // let booking_status_cloned_again2 = book_room_status_cloned.clone();
                                    // let booking_status_cloned_again3 = book_room_status_cloned.clone();

                                    // let booking_details = BookRoomResponse {
                                    //     status: book_room_status
                                    //     .as_ref()
                                    //     .map_or(BookingStatus::BookFailed, |status| status.clone().commit_booking.booking_status.into()),
                                    //     message: book_room_status_cloned.unwrap_or_default().message,
                                    //     commit_booking: BookingDetailsContainer {
                                    //         booking_details: BookingDetails {
                                    //             booking_id: booking_status_cloned_again3.unwrap_or_default().commit_booking.booking_id.0,
                                    //             booking_ref_no: booking_status_cloned_again.unwrap_or_default().commit_booking.booking_ref_no,
                                    //             confirmation_no: booking_status_cloned_again1.unwrap_or_default().commit_booking.confirmation_no,
                                    //             booking_status: match booking_status_cloned_again2.unwrap_or_default().commit_booking.booking_status {
                                    //                 crate::canister::backend::BookingStatus::Confirmed => "Confirmed".to_string(),
                                    //                 crate::canister::backend::BookingStatus::BookFailed => "BookFailed".to_string(),
                                    //             },
                                    //         },
                                    //     }
                                    // };

                                    // confirmation_ctx.booking_details.set(Some(booking_details));
                                    // let booking_guests = booking.guests.clone();
                                    // let booking_guests2 = booking.guests.clone();

                                    // let adults: Vec<crate::state::view_state::AdultDetail> = booking_guests.into();
                                    // let children: Vec<crate::state::view_state::ChildDetail> = booking_guests2.into();

                                    // let adults_clon = adults.clone();
                                    // let children_clon = children.clone();
                                    // BlockRoomCtx::set_adults(adults);
                                    // BlockRoomCtx::set_children(children);

                                    // Payment Details not being stored. Can use the calculated value above if wanna populate it anywhere.

                                    let payment_details = booking.payment_details;
                                    let payment_status = payment_details.payment_status;
                                    println!(
                                        "{}",
                                        format!("payment_status - {:?}", payment_status)
                                            .red()
                                            .bold()
                                    );

                                    let payment_api_response = payment_details.payment_api_response;

                                    if !np_payment_id.get_untracked().is_some() {
                                        return;
                                    }

                                    match payment_status {
                                        Paid(paid_str) => {
                                            println!(
                                                "{}",
                                                format!(" Paid paid-_str = {paid_str:?}")
                                                    .red()
                                                    .bold()
                                            );

                                            let response_ctx = SuccessGetPaymentStatusResponse {
                                                payment_id: np_payment_id.get_untracked().unwrap(),
                                                invoice_id: payment_api_response.invoice_id,
                                                payment_status: payment_api_response.payment_status,
                                                price_amount: payment_api_response.price_amount,
                                                price_currency: payment_api_response.price_currency,
                                                pay_amount: payment_api_response.pay_amount,
                                                actually_paid: payment_api_response.actually_paid,
                                                pay_currency: payment_api_response.pay_currency,
                                                order_id: payment_api_response.order_id,
                                                order_description: payment_api_response
                                                    .order_description,
                                                purchase_id: payment_api_response.purchase_id,
                                                created_at: payment_api_response.created_at,
                                                updated_at: payment_api_response.updated_at,
                                            };
                                            let payment_api_response_for_ctx =
                                                GetPaymentStatusResponse::Success(response_ctx);
                                            BlockRoomResults::set_payment_results(Some(
                                                payment_api_response_for_ctx,
                                            ));
                                        }
                                        _ => {
                                            log!("context remains unchanged")
                                        }
                                    }

                                    if status == "finished" {
                                        println!(
                                            "{}",
                                            format!("setting p03 signal - status - {:?}", status)
                                                .red()
                                                .bold()
                                        );
                                        payment_booking_step_signals
                                            .p03_call_book_room_api
                                            .set(true);
                                    } else {
                                        // retry handler
                                        todo!();
                                    }
                                }
                                Err(e) => {
                                    log!("Error greeting knull {:?}", e);
                                }
                            }
                        });
                        Some(status_cloned.to_string())
                    }
                    any_other => {
                        log!("payment status is {any_other:?}");
                        Some(any_other.to_string())
                    }
                }
            } else {
                None
            }
        },
    );

    view! {
        <div class="bg-gray-100 p-4 border border-emerald-800">
        <Suspense fallback={move || view!{ " Getting Payment Status "}}>

            {move || {
                let payment_status = block_room_results.payment_status_response.get();
                match payment_status {
                    Some(status) => {
                        view! {
                            <div>
                                <div class="payment-status">
                                    {"Payment Status: "} {status.get_payment_status().clone()}
                                </div>
                                {if status.get_payment_status() == "finished" {
                                    view! {
                                        <div class="text-green-700 p-2">
                                            "Payment completed successfully!"
                                        </div>
                                    }
                                } else {
                                    view! {
                                        <div class="border border-red-700 p-2">
                                            "No payment information found. Please contact support."
                                        </div>
                                    }
                                }}
                            </div>
                        }
                    }
                    None => {
                        view! {
                            <div class="border border-blue-800 p-2">
                                "Waiting for payment confirmation... Please do not close this window."
                            </div>
                        }
                    }
                }
            }}
            </Suspense>

            // <div class="bg-gray-100 p-4 border border-emerald-800">
            // <Suspense  fallback={move || view!{ " Getting Payment Status "}}>
            // {move ||
            //     if let Some(payment_status) = update_payment_details_to_backend.get(){

            //         view!{
            //             {format!("Payment status = {payment_status:?}")}
            //         }.into_view()
            //     }else{
            //         view!{
            //             "Did not retrieve payments status yet."
            //         }.into_view()
            //     }
            // }
            // </Suspense>
            // </div>
        </div>
    }
}
