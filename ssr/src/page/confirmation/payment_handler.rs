use crate::api::payments::ports::GetPaymentStatusResponse;
use crate::cprintln;
use crate::page::confirmation::booking_handler::read_booking_details_from_local_storage;
use crate::utils::app_reference;
use crate::{
    api::payments::{nowpayments_get_payment_status, ports::GetPaymentStatusRequest},
    canister::backend,
    page::PaymentBookingStatusUpdates,
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

    // let update_payment_details_to_backend = create_resource(
    //     move || {
    //         payment_booking_step_signals
    //             .p01_fetch_payment_details_from_api
    //             .get()
    //     },
    //     move |p01_fetch_payment_details_from_api| async move {
    //         if !p01_fetch_payment_details_from_api {
    //             // the data will not be present in context => early return
    //             log!("early return - p01_fetch_payment_details_from_api - {p01_fetch_payment_details_from_api:?}");
    //             return None;
    //         }

    //         // unwrap is safe - if the payment_status_response is not available, early return happens.
    //         if let Some(get_payment_status_response) =
    //             block_room_results.payment_status_response.get()
    //         {
    //             let get_payment_status_response_clone = get_payment_status_response.clone();
    //             log!("get_payment_status_response_clone - {get_payment_status_response_clone:#?}");

    //             let get_payment_status_response_for_backend = backend::BePaymentApiResponse::from(
    //                 (get_payment_status_response, "NOWPayments".to_string()),
    //             );

    //             // unwrap is safe because these details must be present in the first step aka getting booking details from backend.
    //             // and first step will set the signal p01_fetch_payment_details_from_api to true
    //             let (email, app_reference) = read_booking_details_from_local_storage().unwrap();

    //             let payment_details = backend::PaymentDetails {
    //                 booking_id: (app_reference, email),
    //                 payment_status: backend::BackendPaymentStatus::Unpaid(None),
    //                 payment_api_response: get_payment_status_response_for_backend,
    //             };

    //             let payment_details_str = serde_json::to_string(&payment_details)
    //                 .expect("payment details is cannot be serialized using serde_json");

    //             log!(" get_payment_status_response_clone.payment_status - {}",  get_payment_status_response_clone.payment_status);

    //             match get_payment_status_response_clone.payment_status.as_str() {
    //                 "finished" => {
    //                     log!("payment status finished");
    //                     let payment_id = get_payment_status_response_clone.payment_id.clone();

    //                     // // 1. save to expect_context
    //                     // block_room_results
    //                     //     .payment_status_response
    //                     //     .set(Some(get_payment_status_response_clone));

    //                     // 2. save to local storage
    //                     // set_payment_store(Some(payment_id));

    //                     // 3. save to backend - trigger other resource
    //                     payment_booking_step_signals
    //                         .p02_update_payment_details_to_backend
    //                         .set(true);

    //                     // 4. book_room_api_call.dispatch()
    //                     // from step 3.

    //                     // return
    //                     Some("finished".to_string())
    //                 }
    //                 any_other => {
    //                     log!("payment status is {any_other:?}");
    //                     Some(any_other.to_string())
    //                 }
    //             }
    //         } else {
    //             None
    //         }
    //     },
    // );
    //  ==================================================================

    //  ================== save payment details to backend ======================================

    //  ==================================================================

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
                                        <div class="payment-success">
                                            "Payment completed successfully!"
                                        </div>
                                    }
                                } else {
                                    view! {
                                        <div class="payment-pending">
                                            "Waiting for payment confirmation..."
                                        </div>
                                    }
                                }}
                            </div>
                        }
                    }
                    None => {
                        view! {
                            <div class="payment-error">
                                "No payment information found. Please contact support."
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
