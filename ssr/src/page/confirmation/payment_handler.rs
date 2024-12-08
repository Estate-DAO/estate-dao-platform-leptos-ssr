// use crate::{
//     api::payments::{nowpayments_get_payment_status, ports::GetPaymentStatusRequest},
//     canister::backend::{BePaymentApiResponse, PaymentDetails},
//     state::{
//         local_storage::{use_booking_id_store, use_payment_store},
//         search_state::{BlockRoomResults, ConfirmationResults},
//         view_state::{BlockRoomCtx, HotelInfoCtx},
//     },
// };
// use leptos::*;
// use leptos_use::{use_interval_fn_with_options, utils::Pausable, UseIntervalFnOptions};

// #[component]
// pub fn PaymentHandler() -> impl IntoView {
//     let (booking_id_signal_read, _, _) = use_booking_id_store();
//     let (payment_store, set_payment_store, _) = use_payment_store();
//     let block_room_ctx = expect_context::<BlockRoomResults>();
//     let confirmation_ctx = expect_context::<ConfirmationResults>();

//     // Payment status polling
//     let Pausable {
//         pause,
//         resume,
//         is_active,
//     } = use_interval_fn_with_options(
//         move || {
//             spawn_local(async move {
//                 if let Some(payment_id) = payment_store.get_untracked() {
//                     let resp =
//                         nowpayments_get_payment_status(GetPaymentStatusRequest { payment_id })
//                             .await
//                             .ok();
//                     if let Some(status) = resp.as_ref() {
//                         if status.payment_status == "finished" {
//                             pause();
//                         }
//                     }
//                     BlockRoomResults::set_payment_results(resp);
//                 }
//             });
//         },
//         1_00_000,
//         UseIntervalFnOptions {
//             immediate: true,
//             immediate_callback: true,
//         },
//     );

//     // Payment status effect
//     create_effect(move |_| {
//         let app_reference_string = booking_id_signal_read
//             .get_untracked()
//             .and_then(|booking| Some(booking.get_app_reference()));
//         let email = booking_id_signal_read
//             .get_untracked()
//             .and_then(|booking| Some(booking.get_email()));

//         let get_payment_status_response = block_room_ctx
//             .payment_status_response
//             .get()
//             .unwrap_or_default()
//             .into();

//         let payment_api_response =
//             BePaymentApiResponse::from((get_payment_status_response, "NOWPayments".to_string()));

//         let payment_details = PaymentDetails {
//             booking_id: (
//                 app_reference_string.unwrap_or_default(),
//                 email.unwrap_or_default(),
//             ),
//             payment_status: crate::canister::backend::BackendPaymentStatus::Unpaid(None),
//             payment_api_response,
//         };

//         // Handle payment status changes
//         match block_room_ctx.payment_status_response.get_untracked() {
//             Some(status) if status.payment_status == "finished" => {
//                 set_payment_store(Some(status.payment_id));
//                 confirmation_ctx.payment_confirmed.set(true);
//                 pause();
//             }
//             Some(_) => {
//                 confirmation_ctx.payment_confirmed.set(false);
//                 resume();
//             }
//             None => {
//                 confirmation_ctx.payment_confirmed.set(false);
//                 resume();
//             }
//         }
//     });

//     view! {
//         <div class="payment-status-container">
//             {move || {
//                 let payment_status = block_room_ctx.payment_status_response.get();
//                 match payment_status {
//                     Some(status) => {
//                         view! {
//                             <div>
//                                 <div class="payment-status">
//                                     {"Payment Status: "} {status.payment_status.clone()}
//                                 </div>
//                                 {if status.payment_status == "finished" {
//                                     view! {
//                                         <div class="payment-success">
//                                             "Payment completed successfully!"
//                                         </div>
//                                     }
//                                 } else {
//                                     view! {
//                                         <div class="payment-pending">
//                                             "Waiting for payment confirmation..."
//                                         </div>
//                                     }
//                                 }}
//                             </div>
//                         }
//                     }
//                     None => {
//                         view! {
//                             <div class="payment-error">
//                                 "No payment information found. Please contact support."
//                             </div>
//                         }
//                     }
//                 }
//             }}
//         </div>
//     }
// }
