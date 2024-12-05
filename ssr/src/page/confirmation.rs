use codee::string::JsonSerdeCodec;
use leptos::*;
use leptos_router::use_navigate;
use leptos_use::storage::use_local_storage;

use crate::{
    api::{canister::get_user_booking::get_user_booking_backend, hotel_info},
    app::AppRoutes,
    component::{Divider, FilterAndSortBy, PriceDisplay, SelectedDateRange, StarRating},
    page::{InputGroup, Navbar},
    state::{
        search_state::{HotelInfoResults, SearchCtx, SearchListResults},
        view_state::HotelInfoCtx,
    },
    utils::app_reference::BookingId,
};
use chrono::NaiveDate;
use leptos::logging::log;

#[component]
pub fn ConfirmationPage() -> impl IntoView {
    create_effect(move |_| {
        let (state, set_state, _) = use_local_storage::<BookingId, JsonSerdeCodec>("booking_id");
        let app_reference_string = state.get().get_app_reference();
        let email = state.get().get_email();

        log!(
            "EMAIL >>>{:?}\nAPP_REF >>>{:?}",
            email,
            app_reference_string
        );
        let email_cloned_twice = state.get().get_email();

        spawn_local(async move {
            match get_user_booking_backend(email_cloned_twice).await {
                Ok(response) => match response {
                    Some(booking) => {
                        let app_reference_string_cloned = app_reference_string.clone();
                        let email_cloned = email.clone();
                        let found_booking = booking
                            .into_iter()
                            .find(|b| {
                                b.booking_id
                                    == (app_reference_string_cloned.clone(), email_cloned.clone())
                            })
                            .unwrap();
                        let date_range = SelectedDateRange {
                            start: found_booking
                                .user_selected_hotel_room_details
                                .date_range
                                .start,
                            end: found_booking
                                .user_selected_hotel_room_details
                                .date_range
                                .end,
                        };
                        SearchCtx::set_date_range(date_range);
                        HotelInfoCtx::set_selected_hotel_details(
                            found_booking
                                .user_selected_hotel_room_details
                                .hotel_details
                                .hotel_name,
                            found_booking
                                .user_selected_hotel_room_details
                                .hotel_details
                                .hotel_image,
                            found_booking
                                .user_selected_hotel_room_details
                                .hotel_details
                                .hotel_location,
                        );
                    }
                    None => {
                        log!("No booking available")
                    }
                },
                Err(e) => {
                    log!("Error greeting knull {:?}", e);
                }
            }
        });
    });

    let hotel_info_ctx: HotelInfoCtx = expect_context();
    let search_ctx: SearchCtx = expect_context();

    let format_date = |(year, month, day): (u32, u32, u32)| {
        NaiveDate::from_ymd_opt(year as i32, month, day)
            .map(|d| d.format("%a, %b %d").to_string())
            .unwrap_or_default()
    };

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
                                format_date(date_range.start),
                                format_date(date_range.end)
                            )
                        }}
                    </p>
                </div>

                <p class="max-w-2xl text-center text-gray-600">
                    "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor
                    incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud 
                    exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat."
                </p>
            </div>
        </section>
    }
}
