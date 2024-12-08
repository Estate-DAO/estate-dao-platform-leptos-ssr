use crate::{
    page::{BookingHandler, Navbar},
    state::{search_state::SearchCtx, view_state::HotelInfoCtx},
};
use chrono::NaiveDate;
use leptos::*;

/// prefix p01 is given to variable names to indicate the order
/// they should be set true in.
/// since rust variables cannot have names starting from numbers, p01 is used instead of 01
#[derive(Debug, Clone, Default)]
pub struct PaymentBookingStatusUpdates {
    pub p01_fetch_payment_details_from_api: RwSignal<bool>,
    pub p02_update_payment_details_to_backend: RwSignal<bool>,
    pub p03_update_booking_details_to_backend: RwSignal<bool>,
    pub p04_call_book_room_api: RwSignal<bool>,
}

#[component]
pub fn ConfirmationPage() -> impl IntoView {
    let hotel_info_ctx: HotelInfoCtx = expect_context();
    let search_ctx: SearchCtx = expect_context();
    // to keep track of steps and reactive udpates
    provide_context(PaymentBookingStatusUpdates::default());

    let insert_real_image_or_default = {
        move || {
            let val = hotel_info_ctx.selected_hotel_image.get();
            if val == "" {
                "/img/home.webp".to_string()
            } else {
                val
            }
        }
    };

    view! {
        <section class="relative h-screen">
            <Navbar />
            <div class="flex flex-col items-center justify-center p-8">
                <img
                    src=insert_real_image_or_default
                    alt={move || hotel_info_ctx.selected_hotel_name.get()}
                    class="w-96 h-64 rounded-xl object-cover mb-8"
                />

                <h1 class="text-3xl font-bold mb-6 text-center">
                    "Awesome, your booking is confirmed!"
                </h1>

                <div class="text-center mb-6">
                    <p class="font-semibold">{move || hotel_info_ctx.selected_hotel_location.get()}</p>
                    <p class="text-gray-600">
                        {move || {
                            let date_range = search_ctx.date_range.get();
                            format!("{} - {}",
                                format_date_fn(date_range.start),
                                format_date_fn(date_range.end)
                            )
                        }}
                    </p>
                </div>

                <p class="max-w-2xl text-center text-gray-600">
                    "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor
                    incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud 
                    exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat."
                </p>
                <BookingHandler />
            </div>
        </section>
    }
}

fn format_date_fn(date_tuple: (u32, u32, u32)) -> String {
    NaiveDate::from_ymd_opt(date_tuple.0 as i32, date_tuple.1, date_tuple.2)
        .map(|d| d.format("%a, %b %d").to_string())
        .unwrap_or_default()
}
