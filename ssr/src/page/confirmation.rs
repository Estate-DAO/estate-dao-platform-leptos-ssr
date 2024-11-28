use leptos::*;
use leptos_router::use_navigate;

use crate::{
    api::{book_room, hotel_info},
    app::AppRoutes,
    component::{Divider, FilterAndSortBy, PriceDisplay, StarRating},
    page::Navbar,
    state::{
        search_state::{
            BlockRoomResults, ConfirmationResults, HotelInfoResults, SearchCtx, SearchListResults,
        },
        view_state::HotelInfoCtx,
    },
};
use chrono::NaiveDate;
use leptos::logging::log;

#[component]
pub fn ConfirmationPage() -> impl IntoView {
    let hotel_info_ctx: HotelInfoCtx = expect_context();
    let search_ctx: SearchCtx = expect_context();

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

    let handle_book_room = create_action(move |_| {
        let block_room = expect_context::<BlockRoomResults>();

        async move {
            let book_room_request = block_room.book_room_request();
            let cloned_book_room_req = book_room_request.clone();
            spawn_local(async move {
                let result = book_room(book_room_request).await.ok();
                ConfirmationResults::set_booking_details(result);
            });
        }
    });

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
                            date_range.format_as_human_readable_date()
                        }}
                    </p>
                </div>

                <p class="max-w-2xl text-center text-gray-600">
                    "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor
                    incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud 
                    exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat."
                </p>
            </div>

            <button
            class="mt-6 w-1/3 rounded-full bg-blue-600 py-3 text-white hover:bg-blue-700 disabled:bg-gray-300"
            on:click=move |_| {
                // Assuming handle_booking dispatches an action and calls book_room
                handle_book_room.dispatch(());
            }
        >
            "Book Room"
        </button>
        </section>
    }
}
