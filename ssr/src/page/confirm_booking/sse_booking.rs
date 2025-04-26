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
            <div class="flex flex-col items-center justify-center w-full py-4 sm:py-6 md:py-8">
                <div class="flex flex-row items-center justify-center w-full overflow-x-auto px-2 pb-2">
                    {move || {
                        steps
                        .clone()
                        .into_iter()
                        .enumerate()
                        .map(|(index, (label, signal))| {
                            let is_active = move || signal.get();
                            let circle_classes = move || format!("min-w-[2rem] w-6 h-6 sm:w-8 sm:h-8 rounded-full flex items-center justify-center font-medium transition-colors {}",
                                if is_active() { "bg-black text-white" } else { "bg-gray-300 text-black" });

                            let line_color = move || if is_active() {
                                    "bg-black"
                            } else {
                                    "bg-gray-300"
                            };

                            view! {
                                <div class="flex items-start shrink-0">
                                    <div class="flex flex-col items-center">
                                        <div class=circle_classes()>
                                            <span class="text-xs sm:text-sm">{(index + 1).to_string()}</span>
                                        </div>
                                        <span class="mt-2 sm:mt-3 md:mt-4 text-[10px] sm:text-xs text-gray-600 text-center break-words max-w-[80px] sm:max-w-[100px] md:max-w-[120px]">{move || label.get()}</span>
                                    </div>
                                    {if index < steps.len() - 1 {
                                        view! {
                                            <div class=format!("h-[1px] w-12 sm:w-16 md:w-24 lg:w-40 transition-colors mt-3 sm:mt-4 mx-1 sm:mx-2 {}", line_color()) />
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
            </div>
        }
    };

    view! {
        <section class="flex flex-col items-center min-h-screen w-full">
            <div class="w-full">
                <Navbar />
            </div>
            <NotificationListenerWrapper />
            <SSEBookingHandler />

            <div class="flex flex-col items-center w-full max-w-4xl mx-auto px-3 sm:px-4 md:px-6 lg:px-8 pt-4 sm:pt-6 md:pt-8">
                <div class="w-full mb-8 sm:mb-12 md:mb-16">
                    {render_progress_bar()}
                </div>
                <Show when= move ||(status_updates.p04_load_booking_details_from_backend.get()) fallback=SpinnerGray>
                <div class="w-full max-w-full sm:max-w-[450px] md:max-w-[500px] lg:max-w-[600px] border border-blue-400 rounded-lg p-3 sm:p-4 md:p-6 space-y-2 sm:space-y-3 md:space-y-4">
                    <div class="text-center text-lg sm:text-xl md:text-2xl font-semibold">
                        Your Booking has been confirmed!
                    </div>
                    <Divider />
                    <div class="space-y-1">
                        <h2 class="text-base sm:text-lg md:text-xl font-semibold">{move || confirmation_page_state.hotel_name.get()}</h2>
                        <p class="text-gray-600 text-xs md:text-sm">
                            <span>{move || confirmation_page_state.hotel_location.get()}</span>
                            <span class="ml-1 text-gray-600 text-xs md:text-sm">{move || {
                                ConfirmationResultsState::get_destination()
                                .map(|dest| format!("{}, {}", dest.city, dest.country_name))
                                .unwrap_or_else(|| "Location details not available".to_string())
                            }}</span>
                        </p>
                    </div>

                    <div class="space-y-2 sm:space-y-3 md:space-y-4">
                        <div class="space-y-1 sm:space-y-1.5 md:space-y-2">
                            <Divider />
                            <p class="text-gray-600 text-xs md:text-sm">Reference ID</p>
                            <p class="font-mono text-xs sm:text-sm md:text-base break-all">{move || {
                                match confirmation_page_state.booking_details.get() {
                                    Some(details) => match details {
                                        backend::Booking { book_room_status: Some(backend::BeBookRoomResponse { commit_booking: backend::BookingDetails { booking_ref_no, .. }, .. }), .. } =>
                                            booking_ref_no.clone(),
                                        _ => "Data missing".to_string()
                                    },
                                    None => "Data missing".to_string()
                                }
                            }}</p>
                        </div>
                        <div class="space-y-1 sm:space-y-1.5 md:space-y-2">
                            <Divider />
                            <p class="text-gray-600 text-xs md:text-sm">Booking ID</p>
                            <p class="font-mono text-xs sm:text-sm md:text-base break-all">{move || {
                                match confirmation_page_state.booking_details.get() {
                                    Some(details) => match details {
                                        backend::Booking { book_room_status: Some(backend::BeBookRoomResponse { commit_booking: backend::BookingDetails { travelomatrix_id, .. }, .. }), .. } =>
                                            travelomatrix_id.clone(),
                                        _ => "Data missing".to_string()
                                    },
                                    None => "Data missing".to_string()
                                }
                            }}</p>
                        </div>
                    </div>

                    <div class="flex justify-between items-center border-t border-b border-gray-200 py-2 sm:py-3 md:py-4">
                        <div class="ml-1 md:ml-2">
                            <span class="text-gray-600 text-[10px] sm:text-xs md:text-sm">Check-in</span>
                            <p class="font-semibold text-xs sm:text-sm md:text-base">{move || {
                                let date_range = confirmation_page_state.date_range.get();
                                format_date_fn(date_range.start)
                            }}</p>
                        </div>
                        <div class="text-center text-[8px] sm:text-[10px] md:text-xs text-gray-600 rounded-lg bg-blue-100 px-1 sm:px-1.5 md:px-2 py-1">
                            {move || {
                                let date_range = confirmation_page_state.date_range.get();
                                let nights = (date_range.end.2 - date_range.start.2) as i32;
                                format!("{} Nights", nights.max(0))
                            }}
                        </div>
                        <div class="ml-1 md:ml-2">
                            <span class="text-gray-600 text-[10px] sm:text-xs md:text-sm">Check-out</span>
                            <p class="font-semibold text-xs sm:text-sm md:text-base">{move || {
                                let date_range = confirmation_page_state.date_range.get();
                                format_date_fn(date_range.end)
                            }}</p>
                        </div>
                    </div>

                    <div class="space-y-1 sm:space-y-1.5 md:space-y-2">
                        <h3 class="text-[8px] sm:text-[10px] md:text-xs text-gray-600">Guests & Rooms</h3>
                        <p class="text-[10px] sm:text-xs md:text-sm">{move || {
                            let adults = confirmation_page_state.adults.get();
                            let children = confirmation_page_state.children.get();
                            format!("{} Room, {} Adult{} • {} children",
                                1,
                                adults.len(),
                                if adults.len() > 1 { "s" } else { "" },
                                children.len())
                        }}</p>
                    </div>

                    <div class="space-y-1 sm:space-y-1.5 md:space-y-2">
                        <h3 class="font-semibold mb-1 sm:mb-2 md:mb-3 text-xs sm:text-sm md:text-base">Guest Information</h3>
                        {move || {
                            let adults = confirmation_page_state.adults.get();
                            let primary_adult = adults.first().cloned().unwrap_or_default();
                            view! {
                                <div class="space-y-0.5 sm:space-y-1">
                                    <p class="text-[10px] sm:text-xs md:text-sm">{format!("{} {}",
                                        primary_adult.first_name,
                                        primary_adult.last_name.unwrap_or_default())
                                    }</p>
                                    <p class="text-[10px] sm:text-xs md:text-sm text-gray-600">{
                                        primary_adult.email.unwrap_or("Email not provided".to_string())
                                    }</p>
                                    <p class="text-[10px] sm:text-xs md:text-sm text-gray-600">{
                                        primary_adult.phone.unwrap_or("Phone not provided".to_string())
                                    }</p>
                                </div>
                            }
                        }}
                    </div>

                    <div class="text-center text-[10px] sm:text-xs md:text-sm font-medium text-gray-600 pt-2">
                        Please take a screenshot for your reference
                    </div>
                </div>
            </Show>
            </div>

        </section>
    }
}

fn format_date_fn(date_tuple: (u32, u32, u32)) -> String {
    NaiveDate::from_ymd_opt(date_tuple.0 as i32, date_tuple.1, date_tuple.2)
        .map(|d| d.format("%a, %b %d").to_string())
        .unwrap_or_default()
}
