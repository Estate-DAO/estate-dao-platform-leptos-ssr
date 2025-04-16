use crate::log;
use crate::state::GlobalStateForLeptos;
use crate::{
    api::BookRoomResponse,
    component::{
        Divider, Navbar, NotificationData, NotificationListener, NotificationState, SpinnerFit,
        SpinnerGray,
    },
    page::{
        block_room, confirm_booking::booking_handler::read_booking_details_from_local_storage,
        BookRoomHandler, BookingHandler, PaymentHandler,
    },
    state::{
        local_storage::use_booking_id_store,
        search_state::{ConfirmationResults, HotelInfoResults, SearchCtx},
        view_state::{BlockRoomCtx, HotelInfoCtx},
    },
    utils::app_reference::BookingId,
};
use chrono::NaiveDate;
use leptos::*;

fn read_from_local_storage_booking_id_store() -> Signal<Option<BookingId>> {
    // log!("read_from_local_storage_booking_id_store");
    let (state, _, _) = use_booking_id_store();
    // log!(
    //     "read_from_local_storage_booking_id_store - state: {:#?}",
    //     state.get()
    // );
    state
}

// Wrapper component to read order_id and email from local storage and pass to NotificationListener
#[component]
pub fn NotificationListenerWrapper() -> impl IntoView {
    let state = read_from_local_storage_booking_id_store();

    view! {
        {
            move || {
                let order_id_local = state.get().map(|b| b.to_order_id()).unwrap_or_default();
                let email_local = state.get().map(|b| b.get_email()).unwrap_or_default();
                view! {
                    // <p> Values from local storage </p>
                    // <p class="bg-gray-200"> {order_id_local.clone()} </p>
                    // <p class="bg-gray-200"> {email_local.clone()} </p>
                    {
                        if !email_local.is_empty() {
                            view! {
                                <NotificationListener
                                order_id={order_id_local}
                                email={email_local}
                                event_type={"nowpayments".to_string()}
                                on_notification={Box::new(move |notification: NotificationData| {
                                    SSEBookingStatusUpdates::update_from_notification(&notification);
                                })} />
                            }.into_view()
                        } else {
                            view! { <p class="text-red-500">No email found, not subscribing to notifications.</p> }.into_view()
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SSEBookingStatusUpdates {
    pub p01_payment_confirmed_and_saved: RwSignal<bool>,
    pub p02_book_room_called_and_saved: RwSignal<bool>,
    pub p03_fetching_booking_details: RwSignal<bool>,
}

impl GlobalStateForLeptos for SSEBookingStatusUpdates {}

impl SSEBookingStatusUpdates {
    /// Call this with a notification to update booking status flags accordingly.
    pub fn update_from_notification(notification: &NotificationData) {
        let this = Self::get();
        match (
            notification.step.as_deref(),
            notification.event_type.as_str(),
        ) {
            (Some("GetPaymentStatusFromPaymentProvider"), "OnStepCompleted") => {
                this.p01_payment_confirmed_and_saved.set(true);
            }
            // todo change this later.
            (Some("MockStep"), "OnStepCompleted") => {
                this.p02_book_room_called_and_saved.set(true);
            }
            _ => {}
        }
    }

    /// Reset all status flags to false
    pub fn reset() {
        let this = Self::get();
        this.p01_payment_confirmed_and_saved.set(false);
        this.p02_book_room_called_and_saved.set(false);
        this.p03_fetching_booking_details.set(false);
    }

    /// Manually set p03_fetching_booking_details to true
    pub fn set_fetching_booking_details() {
        let this = Self::get();
        this.p03_fetching_booking_details.set(true);
    }
}

/// SSEBookingStatusUpdates
#[component]
pub fn SSEConfirmationPage() -> impl IntoView {
    let status_updates: SSEBookingStatusUpdates = expect_context();

    let p01_sig_val = Signal::derive(move || {
        let status = move || status_updates.p01_payment_confirmed_and_saved.get();
        if status() {
            "Payment confirmation successful"
        } else {
            "Confirming your payment"
        }
    });

    let p02_sig_val = Signal::derive(move || {
        let status = move || status_updates.p02_book_room_called_and_saved.get();
        if status() {
            "Booking confirmation successful"
        } else {
            "Making your booking"
        }
    });

    let p03_sig_val = Signal::derive(move || {
        let status = move || status_updates.p03_fetching_booking_details.get();
        if status() {
            "Booking authorized successfully"
        } else {
            "Processing please wait ..."
        }
    });

    let render_progress_bar = move || {
        let steps = vec![
            (p01_sig_val, status_updates.p01_payment_confirmed_and_saved),
            (p02_sig_val, status_updates.p02_book_room_called_and_saved),
            (p03_sig_val, status_updates.p03_fetching_booking_details),
        ];

        view! {
            <div class="flex items-center justify-center my-8">
                {move || {
                    steps
                    .clone()
                    .into_iter()
                    .enumerate()
                    .map(|(index, (label, signal))| {
                        let is_active = move || signal.get();
                        let circle_classes = move || format!("w-8 h-8 rounded-full flex flex-col items-center justify-center font-bold transition-colors {}", if is_active() { "bg-black text-white" } else { "bg-gray-300 text-black" });

                        let line_color = move || if is_active() {
                                "bg-black"
                        } else {
                                "bg-gray-300"
                        };

                        view! {
                            <div class="flex items-center">
                                <div class=circle_classes()>
                                    <span class="mt-[123px]">{(index + 1).to_string()}</span>
                                    <span class="p-8 text-sm text-gray-600">{move || label.get()}</span>
                                </div>
                                {if index < steps.len() - 1 {
                                    view! {
                                        <div class=format!("h-1 w-96 transition-colors {}", line_color()) />
                                    }
                                } else {
                                    view! { <div /> }
                                }}
                            </div>
                        }
                    })
                    .collect::<Vec<_>>()}
                }
            </div>
        }
    };

    view! {
        <div>
            <h2>"This is SSE Booking!"</h2>
            <NotificationListenerWrapper />

            {move || {
                    let notifications = NotificationState::get_notifications();
                    log!("[SSEConfirmationPage] notifications: {:#?}", notifications.get());
                  {format!("{:#?}", notifications.get())}
                }
            }
            <div class="flex flex-col items-center justify-center p-8">
                {render_progress_bar()}
                <br />
                <br />
                <br />
                <br />
                <br />
                <br />
            </div>
        </div>
    }
}
