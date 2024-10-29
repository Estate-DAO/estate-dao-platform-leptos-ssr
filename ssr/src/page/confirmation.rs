use leptos::*;
use leptos_router::use_navigate;

use crate::{
    api::hotel_info,
    app::AppRoutes,
    component::{Divider, FilterAndSortBy, PriceDisplay, StarRating},
    page::{InputGroup, Navbar},
    state::{
        search_state::{HotelInfoResults, SearchCtx, SearchListResults},
        view_state::HotelInfoCtx,
    },
};
use chrono::NaiveDate;
use leptos::logging::log;

#[component]
pub fn ConfirmationPage() -> impl IntoView {
    let hotel_info_ctx: HotelInfoCtx = expect_context();
    let search_ctx: SearchCtx = expect_context();

    let format_date = |(year, month, day): (u32, u32, u32)| {
        NaiveDate::from_ymd_opt(year as i32, month, day)
            .map(|d| d.format("%a, %b %d").to_string())
            .unwrap_or_default()
    };

    let insert_real_image_or_default = {move || {
        let val =  hotel_info_ctx.selected_hotel_image.get() ;
          if val == "" {
             "/img/home.webp".to_string()
          } else{
            val
          }
    }};

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
