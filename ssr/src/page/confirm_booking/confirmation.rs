use crate::{
    component::Divider, page::{block_room, confirm_booking::booking_handler::read_booking_details_from_local_storage, BookRoomHandler, BookingHandler, Navbar, PaymentHandler}, state::{search_state::{HotelInfoResults, SearchCtx}, view_state::{BlockRoomCtx, HotelInfoCtx}}
};
use chrono::NaiveDate;
use leptos::*;

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

		let (email, app_reference) = read_booking_details_from_local_storage().unwrap();

    let render_progress_bar = move || {
        let steps = vec![
            (
                "Confirming your payment",
                status_updates.p01_fetch_payment_details_from_api,
            ),
            (
                "Making your booking",
                status_updates.p02_update_payment_details_to_backend,
            ),
            ("Payment processing... please wait", status_updates.p03_call_book_room_api),
        ];

        view! {
            <div class="flex items-center justify-center space-x-4 my-8">
                {steps
                    .clone()
                    .into_iter()
                    .enumerate()
                    .map(|(index, (label, signal))| {
                        let is_active = move || signal.get();
												let circle_classes = move || format!("w-8 h-8 rounded-full flex flex-col items-center justify-center font-bold transition-colors {}", if is_active() { "bg-black text-white" } else { "bg-gray-300 text-black" });

												let previous_signal_active = index > 0 && steps.get(index - 1).map(|(_, prev_signal)| prev_signal.get()).unwrap_or(false);
												let line_color = move || if previous_signal_active {
														"bg-black"
												} else {
														"bg-gray-300"
												};

                        view! {
                            <div class="flex items-center">
															<div class=circle_classes()> 
																<span class="mt-[123px]">{(index + 1).to_string()}</span>
																<span class="p-8 text-sm text-gray-600">{label}</span>
															</div>
                                {if index < steps.len() - 1 {
                                    view! {
                                        <div class=format!("h-1 w-48 transition-colors {}", line_color()) />
                                    }
                                } else {
                                    view! { <div /> }
                                }}
                            </div>
                        }
                    })
                    .collect::<Vec<_>>()}<br />
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
								<div class="border shadow-md rounded-lg">
                <b class="text-3xl font-bold mb-6 text-center">
                    "Your booking has been confirmed!"
                </b>
								<Divider />
								<div class="flex justify-between items-center p-4">
									<div class="text-left">
										<p class="text-sm font-medium text-gray-800 font-bold">{move || hotel_info_ctx.selected_hotel_name.get()}</p>
										<p class="text-sm font-sm text-gray-800">{hotel_info_ctx.selected_hotel_location.get_untracked()}</p>
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
										<p class="text-sm font-medium text-gray-800 font-bold">Reference ID: {app_reference.clone()}</p>
										<p class="text-sm font-medium text-gray-800 font-bold">Booking ID: {app_reference.clone()}</p>
									</div>
								</div>
								<Divider />
								<b class="text-2xl font-bold mb-6 text-left">Bookind Details</b>
								<div class="flex justify-between items-center p-4">
									<div class="text-left">
										<div class="flex-col">
											{move || {
												let date_range = search_ctx.date_range.get();
												let guest_count = search_ctx.guests.get();
												let no_of_adults = guest_count.adults.get();
												let no_of_child = guest_count.children.get();
												let adult = block_room_ctx.adults.get();
												let children = block_room_ctx.children.get();
												let primary_adult = adult.first().unwrap();
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
													<b>{format!("{} Adults, {} Children", no_of_adults, no_of_child)}</b>
													<p>{format!("{} {}", primary_adult.first_name, primary_adult_clone.last_name.unwrap_or_default())}</p>
													<p>{format!("{}", primary_adult_clone2.email.unwrap_or_default())}</p>
													<p>{format!("{}", primary_adult_clone3.phone.unwrap_or_default())}</p>
												}
											}}
										</div>
									</div>
									
									<div class="text-right text-sm font-bold">
										<p class="text-sm font-medium text-gray-800 font-bold">Reference ID: {"ABCD".to_string()}</p>
										<p class="text-sm font-medium text-gray-800 font-bold">Booking ID: {app_reference}</p>
										{move || {
											let sorted_rooms = hotel_info_results.sorted_rooms.get();
											view! {
												<>
													{sorted_rooms.iter().map(|room| view! {
															<p class="text-sm text-gray-800">{room.room_type.to_string()}</p>
													}).collect::<Vec<_>>()}
												</>
											}
										}}
									</div>
								</div>

								<BookingHandler />
								<PaymentHandler /> 
								<BookRoomHandler />
							</div>
            </div>
        </section>
    }
}

fn format_date_fn(date_tuple: (u32, u32, u32)) -> String {
    NaiveDate::from_ymd_opt(date_tuple.0 as i32, date_tuple.1, date_tuple.2)
        .map(|d| d.format("%a, %b %d").to_string())
        .unwrap_or_default()
}
