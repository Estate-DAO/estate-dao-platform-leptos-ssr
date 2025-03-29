use crate::api::canister::update_payment_details::update_payment_details_backend;
use crate::api::payments::ports::GetPaymentStatusResponse;
use crate::api::{FailureGetPaymentStatusResponse, SuccessGetPaymentStatusResponse};
use crate::canister::backend::{BePaymentApiResponse, PaymentDetails};
use crate::component::SelectedDateRange;
use crate::cprintln;
use crate::page::confirm_booking::booking_handler::read_booking_details_from_local_storage;
use crate::state::confirmation_results_state::{self, ConfirmationResultsState};
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

    // let block_room_results: BlockRoomResults = expect_context();
    let confirmation_results: ConfirmationResults = expect_context();

    let payment_booking_step_signals: PaymentBookingStatusUpdates = expect_context();
    // let confirmation_results_state: ConfirmationResultsState = expect_context();

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
                ConfirmationResultsState::set_payment_results_from_api(resp.clone());
                // BlockRoomResults::set_payment_results(resp.clone());
                // store payment_id in local storage
                set_payment_store(Some(payment_id));

                if resp.is_some() {
                    log!("nowpayments_get_payment_status - {resp:?}");

                    // Check if payment status is finished
                    if let Some(payment_status) =
                        resp.as_ref().and_then(|r| Some(r.get_payment_status()))
                    {
                        log!(
                            "nowpayments_get_payment_status - Payment status: {}",
                            payment_status
                        );

                        if payment_status == "finished" {
                            // Only set p02 signal when payment is finished
                            payment_booking_step_signals
                                .p02_update_payment_details_to_backend
                                .set(true);
                        }
                        // For other statuses, we continue polling
                    }
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

            if !ConfirmationResultsState::payment_status_from_api_is_finished_check() {
                log!(
                    "early return - update_payment_details_to_backend - {:?}",
                    ConfirmationResultsState::get_payment_status()
                );
                return None;
            }

            // unwrap is safe - if the payment_status_response is not available, early return happens.
            if let Some(get_payment_status_response) =
                ConfirmationResultsState::payment_status_response_from_api()
            {
                let get_payment_status_response_clone = get_payment_status_response.clone();
                log!("update_payment_details_to_backend- get_payment_status_response_clone - {get_payment_status_response_clone:#?}");

                // if ConfirmationResultsState::payment_status_from_backend_is_finished_check(){
                //     return None
                // }
                // unwrap is safe because these details must be present in the first step aka getting booking details from backend.
                // and first step will set the signal p01_fetch_payment_details_from_api to true
                let (email, app_reference) = read_booking_details_from_local_storage().unwrap();

                let payment_api_response = BePaymentApiResponse::from((
                    get_payment_status_response_clone.clone(),
                    "NOWPayments".to_string(),
                ));

                log!("update_payment_details_to_backend - BePaymentApiResponse - {payment_api_response:#?}");
                let (email, app_reference_string) =
                    read_booking_details_from_local_storage().unwrap();

                let payment_api_response_cloned = payment_api_response.clone();

                let payment_details = PaymentDetails {
                    booking_id: backend::BookingId {
                        app_reference: app_reference_string.clone(),
                        email: email.clone(),
                    },
                    // Sending order_id currently with this, change as necessary
                    payment_status: Paid(payment_api_response_cloned.order_id),
                    payment_api_response,
                };

                let payment_details_str = serde_json::to_string(&payment_details)
                    .expect("payment details is not valid json");

                let booking_id_for_request = backend::BookingId {
                    app_reference: app_reference_string.clone(),
                    email,
                };

                let payment_status_from_api = get_payment_status_response.get_payment_status();

                log!("update_payment_details_to_backend - payment_status_from_api - {payment_status_from_api:#?}");

                spawn_local(async move {
                    match update_payment_details_backend(
                        booking_id_for_request,
                        payment_details_str,
                    )
                    .await
                    {
                        Ok(booking) => {
                            ConfirmationResultsState::set_booking_details(Some(booking.clone()));
                            println!("{}", format!("{booking:?}").red().bold());

                            if !np_payment_id.get_untracked().is_some() {
                                return;
                            }

                            // if status == "finished" {
                            //     println!(
                            //         "{}",
                            //         format!("setting p03 signal - status - {:?}", status)
                            //             .red()
                            //             .bold()
                            //     );
                            payment_booking_step_signals
                                .p03_call_book_room_api
                                .set(true);
                            // } else {
                            //     // retry handler
                            //     todo!();
                            // }
                        }
                        Err(e) => {
                            log!("update_payment_details_to_backend - {:?}", e);
                            return;
                        }
                    }
                });
                // do not add any UI part because of this resource
                // Some(payment_status_from_api.clone())
                return None::<String>;
            } else {
                None
            }
        },
    );

    view! { <Suspense>{move || update_payment_details_to_backend.get()}</Suspense> }
}
