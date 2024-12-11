use crate::{
    page::{BookRoomHandler, BookingHandler, Navbar, PaymentHandler},
    state::{search_state::SearchCtx, view_state::HotelInfoCtx},
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
            ("Processing", status_updates.p03_call_book_room_api),
        ];

        view! {
            <div class="flex items-center justify-center space-x-4 my-8">
                {steps
                    .clone()
                    .into_iter()
                    .enumerate()
                    .map(|(index, (label, signal))| {
                        let is_active = move || signal.get();
                        let circle_color = move || {
                            if is_active() {
                                "bg-black text-white"
                            } else {
                                "bg-gray-300 text-black"
                            }
                        };
                        let line_color = if index > 0 && steps[index - 1].1.get() {
                            "bg-black"
                        } else {
                            "bg-gray-300"
                        };

                        view! {
                            <div class="flex items-center">
                                <div
                                    class="w-8 h-8 rounded-full flex items-center justify-center font-bold transition-colors"
                                    class=move || circle_color()
                                >
                                    {(index + 1).to_string()}
                                </div>
                                {if index < steps.len() - 1 {
                                    view! {
                                        <div
                                            class="h-1 w-16 transition-colors"
                                            class=move || line_color
                                        />
                                    }
                                } else {
                                    view! { <div /> }
                                }}
                            </div>
                        }
                    })
                    .collect::<Vec<_>>()}<br />
                <div class="text-center mt-2">
                    {steps
                        .into_iter()
                        .map(|(label, _)| {
                            view! { <span class="text-sm text-gray-600 mx-4">{label}</span> }
                        })
                        .collect::<Vec<_>>()}
                </div>
            </div>
        }
    };

    view! {
        <section class="relative h-screen">
            <Navbar />
            <div class="flex flex-col items-center justify-center p-8">
                {render_progress_bar()}
                <h1 class="text-3xl font-bold mb-6 text-center">
                    "Your booking has been confirmed!"
                </h1>
                <div class="text-center mb-6">
                    <p class="font-semibold text-lg">
                        {move || hotel_info_ctx.selected_hotel_location.get()}
                    </p>
                    <p class="text-gray-600">
                        {move || {
                            let date_range = search_ctx.date_range.get();
                            format!(
                                "{} - {}",
                                format_date_fn(date_range.start),
                                format_date_fn(date_range.end),
                            )
                        }}
                    </p>
                </div>
                <div class="max-w-2xl text-center text-gray-600">
                    <p>
                        "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor
                        incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud
                        exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat."
                    </p>
                </div> <BookingHandler /> <PaymentHandler /> <BookRoomHandler />
            </div>
        </section>
    }
}

fn format_date_fn(date_tuple: (u32, u32, u32)) -> String {
    NaiveDate::from_ymd_opt(date_tuple.0 as i32, date_tuple.1, date_tuple.2)
        .map(|d| d.format("%a, %b %d").to_string())
        .unwrap_or_default()
}
