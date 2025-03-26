use crate::{
    api::BookRoomResponse,
    component::{Divider, Navbar, SpinnerFit, SpinnerGray},
    page::{
        block_room, confirm_booking::booking_handler::read_booking_details_from_local_storage,
        BookRoomHandler, BookingHandler, PaymentHandler,
    },
    state::{
        confirmation_results_state::ConfirmationResultsState,
        search_state::{ConfirmationResults, HotelInfoResults, SearchCtx},
        view_state::{BlockRoomCtx, HotelInfoCtx},
    },
};
use chrono::NaiveDate;
use leptos::*;
use log::log;

#[derive(Debug, Clone, Default)]
pub struct PaymentBookingStatusUpdates {
    pub p01_fetch_payment_details_from_api: RwSignal<bool>,
    pub p02_update_payment_details_to_backend: RwSignal<bool>,
    pub p03_call_book_room_api: RwSignal<bool>,
    pub p04_update_booking_details_to_backend: RwSignal<bool>,
}

#[component]
pub fn ConfirmationPage() -> impl IntoView {
    // let hotel_info_ctx: HotelInfoCtx = expect_context();
    // let search_ctx: SearchCtx = expect_context();
    let status_updates: PaymentBookingStatusUpdates = expect_context();
    // let block_room_ctx: BlockRoomCtx = expect_context();
    // let hotel_info_results: HotelInfoResults = expect_context();
    let confirmation_ctx: ConfirmationResults = expect_context();
    let payment_booking_step_signals: PaymentBookingStatusUpdates = expect_context();

    let confirmation_page_state: ConfirmationResultsState = expect_context();

    let p01_sig = create_rw_signal("Confirming your payment");
    let p02_sig = create_rw_signal("Making your booking");
    let p03_sig = create_rw_signal("Payment processing... please wait");

    let p01_sig_val = Signal::derive(move || {
        let status = move || status_updates.p01_fetch_payment_details_from_api.get();
        if status() {
            "Payment confirmation successful"
        } else {
            "Confirming your payment"
        }
    });

    let p02_sig_val = Signal::derive(move || {
        let status = move || status_updates.p02_update_payment_details_to_backend.get();
        if status() {
            "Booking confirmation successfull"
        } else {
            "Making your booking"
        }
    });

    let p03_sig_val = Signal::derive(move || {
        let status = move || status_updates.p04_update_booking_details_to_backend.get();
        if status() {
            "Booking authorized successfully"
        } else {
            "Processing please wait ..."
        }
    });

    let render_progress_bar = move || {
        let steps = vec![
            (
                p01_sig_val,
                status_updates.p01_fetch_payment_details_from_api,
            ),
            (
                p02_sig_val,
                status_updates.p02_update_payment_details_to_backend,
            ),
            (p03_sig_val, status_updates.p03_call_book_room_api),
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
                {render_progress_bar()}
                <br />
                <br />
                <br />
                <br />
                <br />
                <br />
                <div class="text-center text-red-500 text-sm mt-8 border-t pt-4">
                    <p>Do not close this tab until your payment is fully processed</p>
                    <p>to avoid issues with your booking.</p>
                </div>
                <BookingHandler />
                <PaymentHandler />
                <BookRoomHandler />
                <Show when= move ||(payment_booking_step_signals.p04_update_booking_details_to_backend.get()) fallback=SpinnerGray>

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
                                     {
                                        let confirmation_page_state: ConfirmationResultsState = expect_context();
                                        format!("{}", confirmation_page_state.hotel_location.get())

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
                                    match confirmation_ctx.booking_details.get() {
                                        Some(details) => match details {
                                            BookRoomResponse::Success(booking) => {
                                                view!{
                                                    <div class="text-lg font-semibold">
                                                        <p>Reference ID: {booking.commit_booking.booking_details.booking_ref_no}</p>
                                                        <p>Booking ID: {booking.commit_booking.booking_details.travelomatrix_id}</p>
                                                    </div>
                                                }
                                            }
                                            BookRoomResponse::Failure(booking) => {
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
            </div>
        </section>
    }
}

fn format_date_fn(date_tuple: (u32, u32, u32)) -> String {
    NaiveDate::from_ymd_opt(date_tuple.0 as i32, date_tuple.1, date_tuple.2)
        .map(|d| d.format("%a, %b %d").to_string())
        .unwrap_or_default()
}
