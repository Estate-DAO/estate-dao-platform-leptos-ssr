use crate::canister::backend;
use crate::component::code_print::DebugDisplay;
use crate::log;
use crate::page::SSEBookingHandler;
use crate::state::confirmation_results_state::ConfirmationResultsState;
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
use leptos_use::utils::Pausable;
use leptos_use::{use_interval_fn_with_options, UseIntervalFnOptions};

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
    pub p03_fetch_booking_details: RwSignal<bool>,
    pub p04_load_booking_details_from_backend: RwSignal<bool>,
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
            (Some("MakeBookingFromBookingProvider"), "OnStepCompleted") => {
                this.p02_book_room_called_and_saved.set(true);
            }
            (Some("MockStep"), "OnStepCompleted") => {
                // (Some("GetBookingFromBackend"), "OnStepCompleted") => {
                this.p03_fetch_booking_details.set(true);
                this.p04_load_booking_details_from_backend.set(true);
                log!(
                    "MockStep completed - p03_fetch_booking_details -{:#?}",
                    this.p03_fetch_booking_details.get_untracked()
                );
            }
            _ => {}
        }
    }

    /// Reset all status flags to false
    pub fn reset() {
        let this = Self::get();
        this.p01_payment_confirmed_and_saved.set(false);
        this.p02_book_room_called_and_saved.set(false);
        this.p03_fetch_booking_details.set(false);
        this.p04_load_booking_details_from_backend.set(false);
    }

    /// Manually set p03_fetch_booking_details to true
    pub fn set_fetch_booking_details() {
        let this = Self::get();
        this.p03_fetch_booking_details.set(true);
    }
}

/// SSEBookingStatusUpdates
#[component]
pub fn SSEConfirmationPage() -> impl IntoView {
    let status_updates: SSEBookingStatusUpdates = expect_context();
    let confirmation_page_state: ConfirmationResultsState = expect_context();
    // let confirmation_ctx: ConfirmationResults = expect_context();

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
        let status = move || status_updates.p03_fetch_booking_details.get();
        if status() {
            "Booking authorized successfully"
        } else {
            "Processing please wait ..."
        }
    });

    // try to get the data from backend if no event from SSE comes through

    let render_progress_bar = move || {
        let steps = vec![
            (p01_sig_val, status_updates.p01_payment_confirmed_and_saved),
            (p02_sig_val, status_updates.p02_book_room_called_and_saved),
            (p03_sig_val, status_updates.p03_fetch_booking_details),
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
        <section class="relative h-screen">
            <Navbar />
            <div class="flex flex-col items-center justify-center p-8">

        // <div>
            // <h2>"This is SSE Booking!"</h2>
            <NotificationListenerWrapper />
            <SSEBookingHandler />


            // {move || {
            //         let notifications = NotificationState::get_notifications();
            //         log!("[SSEConfirmationPage] notifications: {:#?}", notifications.get());
            //       {format!("{:#?}", notifications.get())}
            //     }
            // }
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

        <Show when= move ||(status_updates.p04_load_booking_details_from_backend.get()) fallback=SpinnerGray>

        <div class="border shadow-lg rounded-lg w-3/4 px-4">
            <div class="text-center text-2xl font-bold pt-4">
                Your booking has been confirmed!
            </div>
            <Divider />
            <div class="flex justify-between items-center p-4">
                <div class="text-left">
                    <b class="text-lg font-bold mb-6 text-left">{move ||  confirmation_page_state.hotel_name.get()}</b>
                    <p class="text-sm font-sm text-gray-800">{move ||  confirmation_page_state.hotel_location.get()}</p>
                    // todo: show this data from backend struct. this is wrong.
                    <p class="text-sm font-sm text-gray-800">{move || {
                        // hotel_info_ctx.selected_hotel_image.get()
                        // hotel_info_ctx.selected_hotel_location.get()

                        // match hotel_info_ctx.booking_details.get() {
                        //     Some(BookRoomResponse::Success(booking)) => {
                        //         let destination = &booking.commit_booking.booking_details.user_selected_hotel_room_details.destination;
                        //         format!("{} - {}, {}",
                        //             destination.country_name,
                        //             destination.city_id,
                        //             destination.city
                        //         )
                        //     },
                        //     _ => String::new()
                        // }

                        view!{
                        <DebugDisplay label="Destination" value=|| format!("{:#?}", ConfirmationResultsState::get().booking_details.get())/>

                         {
                            ConfirmationResultsState::get_destination()
                            .map(|dest| format!("{}, {}", dest.city, dest.country_name))
                            .unwrap_or_else(|| "Country not available".to_string())
                         }
                        //     <pre>
                        //         {
                        //         let hotel_info_ctx: HotelInfoCtx = expect_context();
                        //         format!("{}", hotel_info_ctx.display())
                        //         }
                        //     </pre>
                        //     <pre>
                        //     {
                        //         let confirmation_page_state: ConfirmationResultsState = expect_context();
                        //         format!("{}", confirmation_page_state.display())
                        //     }
                        //     </pre>

                    }.into_view()
                    }}</p>
                </div>

                <div class="text-right">
                {move || {
                    match confirmation_page_state.booking_details.get() {
                        Some(details) => match details {
                            backend::Booking { book_room_status: Some(backend::BeBookRoomResponse { commit_booking: backend::BookingDetails { booking_ref_no, travelomatrix_id, .. }, .. }), .. } => {
                                view!{
                                    <div class="text-lg font-semibold">
                                            <p>Reference ID: {booking_ref_no}</p>
                                            <p>Booking ID: {travelomatrix_id}</p>
                                        </div>
                                    }
                                }
                            backend::Booking { book_room_status: None, .. } => {
                                view!{
                                    <div class="text-red-500">
                                        Booking failed!
                                    </div>
                                }
                              }
                            }
                            None => {
                                view!{
                                    <div class="text-gray-500">
                                        Fetching Booking, Please wait...
                                    </div>
                                }
                            }
                        }
                    }
                }
                </div>
            </div>
            <Divider />
            <b class="text-lg font-bold mb-6 text-left">Booking Details</b>
            <div class="flex justify-between items-center p-4">
                <div class="text-left">
                    <div class="flex-col">
                        {move || {
                            // let date_range = search_ctx.date_range.get();
                            // let adults = block_room_ctx.adults.get();
                            // let children = block_room_ctx.children.get();
                            let date_range = confirmation_page_state.date_range.get();
                            let adults = confirmation_page_state.adults.get();
                            let children = confirmation_page_state.children.get();

                            let no_of_adults = adults.len();
                            let no_of_child = children.len();
                            let primary_adult = adults.first().cloned().unwrap_or_default();
                            let primary_adult_clone = primary_adult.clone();
                            let primary_adult_clone2 = primary_adult.clone();
                            let primary_adult_clone3 = primary_adult.clone();

                            view! {
                                <div class="flex text-sm font-sm">
                                    <p class="text-gray-800">Check In date:</p>
                                    <b>{format_date_fn(date_range.start)}</b>
                                </div>
                                <div class="flex text-sm font-sm">
                                    <p class="text-gray-800">Check Out date:</p>
                                    <b>{format_date_fn(date_range.end)}</b>
                                </div>
                                <b>Guest Information</b>
                                <br />
                                <b>{format!("{} Adults, {} Children", no_of_adults, no_of_child)}</b>
                                <p>{format!("{} {}", primary_adult.first_name, primary_adult_clone.last_name.unwrap_or_default())}</p>
                                <p>{format!("{}", primary_adult_clone2.email.unwrap_or_default())}</p>
                                <p>{format!("{}", primary_adult_clone3.phone.unwrap_or_default())}</p>
                            }
                        }}
                    </div>
                </div>

                <div class="text-right text-xs font-semibold">
                    {move || {
                        let sorted_rooms = confirmation_page_state.sorted_rooms.get();
                        view! {
                            <>
                                {sorted_rooms.iter().map(|room| view! {
                                        <p class="text-sm text-gray-800">{format!("{}", room.room_type.to_string())}</p>
                                }).collect::<Vec<_>>()}
                            </>
                        }
                    }}
                </div>
            </div>
            <div class="text-center font-sm font-bold pb-4">
                Please take a screenshot for your reference
            </div>
        </div>
    </Show>
    </section>
    }
}

fn format_date_fn(date_tuple: (u32, u32, u32)) -> String {
    NaiveDate::from_ymd_opt(date_tuple.0 as i32, date_tuple.1, date_tuple.2)
        .map(|d| d.format("%a, %b %d").to_string())
        .unwrap_or_default()
}
