use chrono::NaiveDate;
// use leptos::logging::log;
use crate::log;
use leptos::*;
use leptos_icons::*;
use leptos_use::{use_timestamp_with_controls, UseTimestampReturn};
use serde::{Deserialize, Serialize};

use crate::component::GuestSelection;
use crate::utils::search_action::create_default_search_action;
use crate::view_state_layer::input_group_state::{InputGroupState, OpenDialogComponent};
use crate::{
    // api::search_hotel,
    component::SelectedDateRange,
    utils::date::{add_days, get_year_month_day, next_day},
    view_state_layer::ui_search_state::{UIPaginationState, UISearchCtx},
};

use super::Destination;
use leptos_query::{query_persister, *};
use std::time::Duration;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::window;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct City {
    #[serde(rename = "city_name")]
    city_name: String,
    #[serde(rename = "country_name")]
    country_name: String,
    #[serde(rename = "city_code")]
    city_code: String,
    #[serde(rename = "country_code")]
    country_code: String,
    #[serde(default)] // This will use a default value if image_url is not in JSON
    image_url: String,
    #[serde(default)]
    latitude: Option<f64>,
    #[serde(default)]
    longitude: Option<f64>,
}

impl From<City> for Destination {
    fn from(city: City) -> Self {
        Destination {
            city: city.city_name,
            country_name: city.country_name,
            country_code: city.country_code,
            city_id: city.city_code,
            latitude: city.latitude,
            longitude: city.longitude,
        }
    }
}

#[server(GetCityListForMostPopular)]
pub async fn read_cities_from_file(file_path: String) -> Result<Vec<City>, ServerFnError> {
    let file = std::fs::File::open(file_path.as_str())?;
    let reader = std::io::BufReader::new(file);
    let result: Vec<City> = serde_json::from_reader(reader)?;
    log!(" read_cities_from_file - {:?}", result.first());

    Ok(result)
}

fn destinations_query() -> QueryScope<bool, Option<Vec<City>>> {
    // log!("destinations_query called");
    leptos_query::create_query(
        |_| async move {
            // log!("will call read_destinations_from_file in async move");
            read_cities_from_file("city.json".into()).await.ok()
        },
        QueryOptions {
            default_value: None,
            refetch_interval: None,
            resource_option: Some(ResourceOption::NonBlocking),
            stale_time: Some(Duration::from_secs(2 * 60)),
            gc_time: Some(Duration::from_secs(5 * 60)),
        },
    )
}

#[component]
pub fn MostPopular() -> impl IntoView {
    let search_ctx: UISearchCtx = expect_context();

    let selected_range = search_ctx.date_range;

    let (initial_date, set_initial_date) = create_signal((2025_u32, 1_u32, 1_u32));

    let next_date = Signal::derive(move || {
        let (current_year, current_month, current_day) = initial_date.get();
        next_day(current_year, current_month, current_day)
    });

    // let next_2_next_date: Signal<(u32, u32, u32)> = Signal::derive(move || {
    //     let (current_year, current_month, current_day) = initial_date.get();
    //     let (next_year, next_month, next_day) = next_day(current_year, current_month, current_day);
    //     next_day(next_year, next_month, next_day)
    // });

    let next_2_next_date: Signal<(u32, u32, u32)> = Signal::derive(move || {
        let (current_year, current_month, current_day) = next_date.get();
        next_day(current_year, current_month, current_day)
    });

    let date_range_display = create_memo(move |_prev| {
        let range = selected_range.get();
        if range.start == (0, 0, 0) && range.end == (0, 0, 0) {
            "Check in - Check out".to_string()
        } else {
            range.to_string()
        }
    });

    // todo: find a way to not set signal from effect
    create_effect(move |_| {
        let UseTimestampReturn {
            timestamp,
            is_active,
            pause,
            resume,
        } = use_timestamp_with_controls();

        // pause.pause();
        log!("timestamp: {:?}", timestamp.get_untracked());

        let (year, month, day) = get_year_month_day(timestamp.get_untracked());
        set_initial_date((year, month, day));
    });

    // Use a memo for date_range so it's reactive and tracked
    // <!-- Modified: Changed to next week (7 days from today) instead of next day -->
    let date_range = create_memo(move |_| {
        let (current_year, current_month, current_day) = initial_date.get();

        // Calculate next week (7 days from today)
        let next_week_start = add_days(current_year, current_month, current_day, 7);
        let next_week_end = add_days(next_week_start.0, next_week_start.1, next_week_start.2, 1);

        SelectedDateRange {
            start: next_week_start,
            end: next_week_end,
        }
    });

    // Use shared search action with default configuration
    let search_action = create_default_search_action();

    let QueryResult {
        data: destinations_resource,
        state,
        // is_loading,
        // is_fetching,
        // is_invalid,
        ..
    } = destinations_query().use_query(move || true);

    view! {
        <div class="bg-white rounded-[45px] p-2 md:p-4 w-full -mt-8 most-popular-card">
            <div class="py-8 px-4 md:py-16 md:px-20">
                <div class="text-xl md:text-2xl font-semibold text-left mb-6">Most popular destinations</div>

                    <Suspense fallback=move || {
                        view! { <p>"Loading..."</p> }
                    }>
                <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-4">

                        {move || {
                            destinations_resource
                                .get()
                                .map(|dest_vec| {
                                    // log!("{dest_vec:?}");
                                    dest_vec
                                    .unwrap_or_default()
                                    .clone()
                                    .into_iter()
                                    .filter(|dest| !dest.image_url.is_empty())
                                    .take(3)
                                    .map(|dest| {
                                        let country_name = dest.country_name.clone();
                                        let city_name = dest.city_name.clone();
                                        let img_url = dest.image_url.clone();
                                        view! {
                                            <div
                                                class="rounded-xl overflow-hidden border border-gray-300 h-full cursor-pointer hover:shadow-lg transition-shadow m-1 md:m-2 bg-white flex flex-col"
                                                on:click=move |ev| {
                                                    // Prevent default behavior and stop propagation
                                                    log!("[most_popular.rs] Card clicked - BEFORE prevent_default");
                                                    ev.prevent_default();
                                                    log!("[most_popular.rs] Card clicked - AFTER prevent_default");
                                                    ev.stop_propagation();
                                                    log!("[most_popular.rs] Card clicked - AFTER stop_propagation");

                                                    // Reset pagination to first page when card is clicked
                                                    UIPaginationState::reset_to_first_page();
                                                    log!("[most_popular.rs] Pagination reset to first page");

                                                    // Directly update search context
                                                    log!("[most_popular.rs] Setting destination");
                                                    UISearchCtx::set_destination(dest.clone().into());
                                                    log!("[most_popular.rs] Setting date range");
                                                    UISearchCtx::set_date_range(date_range.get());
                                                    log!("[most_popular.rs] Setting guests");

                                                    UISearchCtx::set_guests(GuestSelection::default());
                                                    log!("[most_popular.rs] ABOUT TO dispatch search action");
                                                    search_action.dispatch(());
                                                    log!("[most_popular.rs] AFTER dispatching search action");
                                                    // // Get a reference to navigate
                                                    // let nav = use_navigate();

                                                    // // Reset search results
                                                    // log!("[most_popular.rs] Resetting search results");
                                                    // SearchListResults::reset();

                                                    // // Directly navigate to hotel list page
                                                    // log!("[most_popular.rs] Navigating to hotel list page");
                                                    // nav(AppRoutes::HotelList.to_string(), Default::default());

                                                    // // Directly log the search button clicked message
                                                    // log!("Search button clicked");

                                                    // // Get search context for API call
                                                    // let search_ctx_clone = search_ctx.clone();

                                                    // // Perform search in a spawn_local
                                                    // spawn_local(async move {
                                                    //     log!("[most_popular.rs] Calling search_hotel API");
                                                    //     let result = search_hotel(search_ctx_clone.into()).await.ok();
                                                    //     log!("[most_popular.rs] Setting search results");
                                                    //     SearchListResults::set_search_results(result);
                                                    // });
                                                }
                                            >
                                                <img
                                                    src=img_url
                                                    alt=format!("{}, {}", city_name, country_name)
                                                    class="w-full object-cover h-40 md:h-56"
                                                />
                                                <div class="p-4 md:p-5 bg-white flex-1 flex flex-col justify-end">
                                                    <p class="text-base md:text-lg font-semibold mb-1">{city_name}</p>
                                                    <p class="text-xs md:text-sm text-gray-600 pb-2 md:pb-4">{country_name}</p>
                                                </div>
                                            </div>
                                        }
                                    })
                                    .collect_view()
                                })
                        }}
                        </div>

                    </Suspense>
            </div>
        </div>
    }
}
