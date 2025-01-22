use crate::{
    api::{
        canister::get_user_booking::get_user_booking_backend, BookRoomResponse, BookingDetails,
        BookingDetailsContainer, BookingStatus, FailureBookRoomResponse, SuccessBookRoomResponse,
    },
    component::{Destination, Divider, Navbar, SelectedDateRange, SpinnerFit, SpinnerGray},
    page::{
        block_room,
        confirm_booking::booking_handler::{
            read_booking_details_from_local_storage, set_to_context,
        },
        BookRoomHandler, BookingHandler, PaymentHandler,
    },
    state::{
        search_state::{ConfirmationResults, HotelInfoResults, SearchCtx},
        view_state::{BlockRoomCtx, HotelInfoCtx},
    },
};
use chrono::NaiveDate;
use colored::Colorize;
use leptos::*;
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, Default)]
pub struct PaymentBookingStatusUpdates {
    pub p01_fetch_payment_details_from_api: RwSignal<bool>,
    pub p02_update_payment_details_to_backend: RwSignal<bool>,
    pub p03_call_book_room_api: RwSignal<bool>,
    pub p04_update_booking_details_to_backend: RwSignal<bool>,
}

#[component]
pub fn ConfirmationPage() -> impl IntoView {
    let hotel_info_ctx: HotelInfoCtx = expect_context();
    let search_ctx: SearchCtx = expect_context();
    let status_updates: PaymentBookingStatusUpdates = expect_context();
    let block_room_ctx: BlockRoomCtx = expect_context();
    let hotel_info_results: HotelInfoResults = expect_context();
    let confirmation_ctx: ConfirmationResults = expect_context();
    let payment_booking_step_signals: PaymentBookingStatusUpdates = expect_context();

    let p01_sig = create_rw_signal("Confirming your payment");
    let p02_sig = create_rw_signal("Making your booking");
    let p03_sig = create_rw_signal("Payment processing... please wait");

    #[cfg(not(feature = "ssr"))]
    let events_sig = {
        use futures::StreamExt;

        let mut source = gloo_net::eventsource::futures::EventSource::new("/sse/events")
            .expect("couldn't connect to SSE stream");
        let s = create_signal_from_stream(source.subscribe("message").unwrap().map(|value| {
            match value {
                Ok(value) => value
                    .1
                    .data()
                    .as_string()
                    .expect("expected string value")
                    .parse()
                    .unwrap_or_default(),
                Err(_) => 0,
            }
        }));

        on_cleanup(move || source.close());
        s
    };

    #[cfg(feature = "ssr")]
    let (events_sig, _) = create_signal(None::<0>);

    let p01_sig_val = Signal::derive(move || {
        let status = move || events_sig.get().unwrap_or_default();
        if status() > 0 {
            "Payment confirmation successfull"
        } else {
            "Confirming your payment"
        }
    });

    let p02_sig_val = Signal::derive(move || {
        let status = move || events_sig.get().unwrap_or_default();
        if status() > 1 {
            "Booking confirmation successfull"
        } else {
            "Making your booking"
        }
    });

    let p03_sig_val = Signal::derive(move || {
        let status = move || events_sig.get().unwrap_or_default();
        if status() > 2 {
            "Booking authorized successfully"
        } else {
            "Processing please wait ..."
        }
    });

    let render_progress_bar = move || {
        let steps = vec![
            (
                p01_sig_val,
                // status_updates.p01_fetch_payment_details_from_api,
                events_sig,
            ),
            (
                p02_sig_val,
                // status_updates.p02_update_payment_details_to_backend,
                events_sig,
            ),
            // (p03_sig_val, status_updates.p03_call_book_room_api),
            (p03_sig_val, events_sig),
        ];

        view! {
            <div class="flex items-center justify-center my-8">
            <span>
                "Step count: " {move || events_sig.get().unwrap_or_default()}
            </span>
                {move || {
                    steps
                    .clone()
                    .into_iter()
                    .enumerate()
                    .map(|(index, (label, signal))| {
                        let is_active = move || index + 1 == events_sig.get().unwrap() as usize;
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

    let details_from_local_storage = match read_booking_details_from_local_storage() {
        Ok(details) => Some(details),
        Err(err) => {
            println!(
                "{}",
                format!("should_fetch_from_canister - Err - {} ", err).red()
            );
            None
        }
    };

    _ = create_resource(
        move || events_sig.get(),
        move |events_sig| async move {
            if events_sig.unwrap_or_default() > 2 {
                println!("booking_id_signal_read in create_resource - {booking_id_signal_read:?}");

                let details_from_local_storage = match read_booking_details_from_local_storage() {
                    Ok(details) => Some(details),
                    Err(err) => {
                        println!(
                            "{}",
                            format!("should_fetch_from_canister - Err - {} ", err).red()
                        );
                        None
                    }
                };

                if details_from_local_storage.is_some() {
                    let (email, app_reference) = details_from_local_storage.unwrap();

                    // ================================ validate bookings ================================
                    let bookings_from_backend = get_user_booking_backend(email.clone())
                        .await
                        .map_err(|e| format!("Failed to fetch booking: ServerFnError =  {}", e))?;

                    if bookings_from_backend.is_none() {
                        return Err("No bookings present in backend.".to_string());
                    }
                    let bookings = bookings_from_backend.unwrap();

                    let found_booking_opt = bookings
                        .into_iter()
                        // .find(|b| b.booking_id == (app_reference.clone(), email.clone()));
                        .find(|b| {
                            b.booking_id.app_reference == app_reference
                                && b.booking_id.email == email
                        });

                    if found_booking_opt.is_none() {
                        return Err("Booking with given ID not in backend.".to_string());
                    }

                    let found_booking = found_booking_opt.unwrap();
                    set_to_context(found_booking);

                    Ok(Some(found_booking))
                } else {
                    log::info!("not fetch_from_canister");
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        },
    );

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
                <BookingHandler />
                <PaymentHandler />
                <BookRoomHandler />
                <Show when= move ||(events_sig.get().unwrap_or_default() > 2) fallback=SpinnerGray>

                    <div class="border shadow-lg rounded-lg w-3/4 px-4">
                        <div class="text-center text-2xl font-bold pt-4">
                            Your booking has been confirmed!
                        </div>
                        <Divider />
                        <div class="flex justify-between items-center p-4">
                            <div class="text-left">
                                <b class="text-lg font-bold mb-6 text-left">{move || hotel_info_ctx.selected_hotel_name.get()}</b>
                                <p class="text-sm font-sm text-gray-800">{move || hotel_info_ctx.selected_hotel_location.get()}</p>
                                <p class="text-sm font-sm text-gray-800">{move || {
                                    let destination = search_ctx.destination.get().unwrap_or_default();
                                    format!("{} - {}, {}",
                                        destination.country_name,
                                        destination.city_id,
                                        destination.city
                                    )}
                                }
                                </p>
                            </div>

                            <div class="text-right">
                            {move || {
                                    match confirmation_ctx.booking_details.get() {
                                        Some(details) => match details {
                                            BookRoomResponse::Success(booking) => {
                                                view!{
                                                    <div class="text-lg font-medium font-semibold">
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
                        <b class="text-lg font-bold mb-6 text-left">Bookind Details</b>
                        <div class="flex justify-between items-center p-4">
                            <div class="text-left">
                                <div class="flex-col">
                                    {move || {
                                        let date_range = search_ctx.date_range.get();
                                        let adults = block_room_ctx.adults.get();
                                        let children = block_room_ctx.children.get();
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
                                    let sorted_rooms = hotel_info_results.sorted_rooms.get();
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
