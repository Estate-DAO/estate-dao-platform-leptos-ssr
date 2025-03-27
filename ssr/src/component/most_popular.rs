use chrono::NaiveDate;
// use leptos::logging::log;
use crate::log;
use leptos::*;
use leptos_icons::*;
use leptos_router::use_navigate;
use leptos_use::{use_timestamp_with_controls, UseTimestampReturn};
use serde::{Deserialize, Serialize};

use crate::component::GuestSelection;
use crate::{
    api::search_hotel,
    app::AppRoutes,
    component::SelectedDateRange,
    state::search_state::{SearchCtx, SearchListResults},
    utils::date::{get_year_month_day, next_day},
};

use super::Destination;
use leptos_query::{query_persister, *};
use std::time::Duration;

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
}

impl From<City> for Destination {
    fn from(city: City) -> Self {
        Destination {
            city: city.city_name,
            country_name: city.country_name,
            country_code: city.country_code,
            city_id: city.city_code,
        }
    }
}

#[server(GetCityListForMostPopular)]
pub async fn read_cities_from_file(file_path: String) -> Result<Vec<City>, ServerFnError> {
    let file = std::fs::File::open(file_path.as_str())?;
    let reader = std::io::BufReader::new(file);
    let result: Vec<City> = serde_json::from_reader(reader)?;
    log!("{:?}", result.first());

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
    let search_ctx: SearchCtx = expect_context();

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

    let date_range = SelectedDateRange {
        start: next_date.get(),
        end: next_2_next_date.get(),
    };

    let navigate = use_navigate();
    let search_action = create_action(move |()| {
        let nav = navigate.clone();
        let search_ctx = search_ctx.clone();
        async move {
            log!("Search button clicked");

            SearchListResults::reset();

            //  move to the hotel listing page
            nav(AppRoutes::HotelList.to_string(), Default::default());

            // call server function inside action
            spawn_local(async move {
                let result = search_hotel(search_ctx.into()).await.ok();
                // log!("SEARCH_HOTEL_API: {result:?}");
                SearchListResults::set_search_results(result);
            });
        }
    });

    let QueryResult {
        data: destinations_resource,
        state,
        // is_loading,
        // is_fetching,
        // is_invalid,
        ..
    } = destinations_query().use_query(move || true);

    view! {
        <div class="bg-white rounded-[45px] p-4 w-full -mt-8">
            <div class="py-16 px-20">
                <div class="text-2xl font-semibold text-left mb-6">Most popular destinations</div>

                    <Suspense fallback=move || {
                        view! { <p>"Loading..."</p> }
                    }>
                <div class="grid grid-cols-3 gap-4">

                        {move || {
                            destinations_resource
                                .get()
                                .map(|dest_vec| {
                                    // log!("{dest_vec:?}");
                                    dest_vec
                                    .unwrap_or_default()
                                    .clone()
                                    .into_iter()
                                    .take(3)
                                    .map(|dest| {
                                        let country_name = dest.country_name.clone();
                                        let city_name = dest.city_name.clone();
                                        let img_url = dest.image_url.clone();
                                        let date_range = SelectedDateRange {
                                            start: next_date.get(),
                                            end: next_2_next_date.get(),
                                        };
                                        view! {
                                            <div
                                                class="rounded-lg overflow-hidden border border-gray-300 h-4/5 cursor-pointer hover:shadow-lg transition-shadow m-2"
                                                on:click=move |_| {
                                                    SearchCtx::set_destination(dest.clone().into());
                                                    SearchCtx::set_date_range(date_range.clone());
                                                    SearchCtx::set_guests(GuestSelection::default());
                                                    search_action.dispatch(())
                                                }
                                            >
                                                <img
                                                    src=img_url
                                                    alt=format!("{}, {}", city_name, country_name)
                                                    class="w-full object-cover h-3/4"
                                                />
                                                <div class="p-5 bg-white">
                                                    <p class="text-lg font-semibold mb-1">{city_name}</p>
                                                    <p class="text-sm text-gray-600 pb-4">{country_name}</p>
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
