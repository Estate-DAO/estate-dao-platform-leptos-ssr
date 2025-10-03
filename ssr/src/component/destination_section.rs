use chrono::Datelike;
use leptos::*;

use crate::{api::client_side_api::Place, component::SelectedDateRange, utils::{date::add_days, search_action::create_search_action_with_ui_state}, view_state_layer::ui_search_state::UISearchCtx};

#[component]
pub fn DestinationsSection() -> impl IntoView {
    view! {
        <section class="py-16 px-6">
            <div class="max-w-6xl mx-auto text-start mb-12">
                <h2 class="text-3xl font-bold">"Enjoy your dream vacation"</h2>
                <p class="text-gray-600 mt-2">
                    "Plan and book our perfect trip with expert advice, travel tips,
                     destination information and inspiration from us"
                </p>
            </div>

            <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 gap-6 max-w-6xl mx-auto">
                <DestinationCard
                    name="Australia"
                    image_url="/img/australia.jpg"
                    properties=57161
                    place_id="ChIJ38WHZwf9KysRUhNblaFnglM"
                />

                <DestinationCard
                    name="Japan"
                    image_url="/img/japan.jpg"
                    properties=46356
                    place_id="ChIJLxl_1w9OZzQRRFJmfNR1QvU"
                />

                <DestinationCard
                    name="New Zealand"
                    image_url="/img/new-zealand.jpg"
                    properties=16861
                    place_id="ChIJh5Z3Fw4gLG0RM0dqdeIY1rE"
                />

                <DestinationCard
                    name="Greece"
                    image_url="/img/greece.jpg"
                    properties=97019
                    place_id="ChIJY2xxEcdKWxMRHS2a3HUXOjY"
                />
            </div>
        </section>
    }
}

#[component]
pub fn DestinationCard(
    name: &'static str,
    image_url: &'static str,
    properties: u32,
    place_id: &'static str,
) -> impl IntoView {

    let date_range = create_memo(move |_| {
        let current_date = chrono::Utc::now().date_naive();
        let (current_year, current_month, current_day) = (current_date.year() as u32, current_date.month(), current_date.day());

        // Calculate next week (7 days from today)
        let next_week_start = add_days(current_year, current_month, current_day, 7);
        let next_week_end = add_days(next_week_start.0, next_week_start.1, next_week_start.2, 1);

        SelectedDateRange {
            start: next_week_start,
            end: next_week_end,
        }
    });
    
    let action = move || {
        let place = Place {
            place_id: place_id.to_string(),
            display_name: name.to_string(),
            formatted_address: String::new(),
        };
        
        UISearchCtx::set_date_range(date_range.get());
        UISearchCtx::set_place(place.clone());
        let search_action = create_search_action_with_ui_state(false.into());
        search_action.dispatch(());
    };

    view! {
        <div on:click=move |_|action() class="cursor-pointer">
            <div class="flex flex-col space-y-2 cursor-pointer hover:scale-[1.02] transition-transform">
                <img
                    src={image_url}
                    alt={name}
                    class="w-full h-48 object-cover rounded-xl shadow-sm"
                />
                <h3 class="font-semibold text-lg">{name}</h3>
                <p class="text-gray-500 text-sm">{properties} " properties"</p>
            </div>
        </div>
    }
}
