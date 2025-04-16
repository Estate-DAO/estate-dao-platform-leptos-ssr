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
    log!("read_from_local_storage_booking_id_store");
    let (state, _, _) = use_booking_id_store();
    log!(
        "read_from_local_storage_booking_id_store - state: {:#?}",
        state.get()
    );
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
                                <NotificationListener order_id={order_id_local} email={email_local} event_type={"nowpayments".to_string()} />
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

#[component]
pub fn SSEConfirmationPage() -> impl IntoView {
    // let notifications = NotificationState::get_notifications();

    // log!("SSEConfirmationPage");
    view! {
        <div>
            <h2>"This is SSE Booking!"</h2>
            <NotificationListenerWrapper />

            {move || {
                // view!{
                    let notifications = NotificationState::get_notifications();

                    log!("[SSEConfirmationPage] notifications: {:#?}", notifications.get());
                  {format!("{:#?}", notifications.get())}
                }
            }
        </div>
    }
}
