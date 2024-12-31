use leptos::logging::log;
use leptos::*;
use leptos_icons::*;
use leptos_query::QueryResult;
use leptos_router::use_navigate;

use crate::{
    api::{canister::greet_call::greet_backend, search_hotel},
    app::AppRoutes,
    component::{
        DateTimeRangePickerCustom, Destination, DestinationPicker, EstateDaoIcon, FilterAndSortBy,
        GuestQuantity, GuestSelection, HSettingIcon, SelectedDateRange,
    },
    state::search_state::{SearchCtx, SearchListResults},
};
use chrono::{Datelike, NaiveDate};
use leptos::ev::MouseEvent;
use leptos_query::{query_persister, *};
use leptos_use::{use_timestamp_with_controls, UseTimestampReturn};
use serde::{Deserialize, Serialize};
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

#[component]
pub fn RootPage() -> impl IntoView {
    view! {
        <main>
            <div>
                <HeroSection />
                <MostPopular />
            </div>
            <Footer />
        </main>
    }
}

#[component]
pub fn HeroSection() -> impl IntoView {
    view! {
        <section class="bg-top bg-cover bg-no-repeat bg-[url('/img/home.webp')]">
            <Navbar />
            <div class="mt-40">
                <div class="flex flex-col items-center justify-center h-full">
                    <h1 class="text-5xl font-semibold text-black mb-8">
                        Hey! Where are you off to?
                    </h1>
                    <InputGroup />
                    <br />
                    // todo: uncomment in v2 when implementing filtering and sorting
                    // <FilterAndSortBy />
                    <br />
                    <br />
                    <br />
                    <br />
                    <div class="flex items-end px-6 py-3 bg-white rounded-xl max-w-fit w-full ">
                        "We're the first decentralized booking platform powered by ICP."
                        <span class="font-semibold text-blue-500 ml-4 inline">"Learn more"</span>
                        <Icon
                            class="w-6 h-6 font-semibold inline ml-2 text-blue-500"
                            icon=icondata::CgArrowRight
                        />
                    </div>
                    <br />
                    <br />
                    <br />
                </div>
            </div>
        </section>
    }
}

#[component]
pub fn Navbar() -> impl IntoView {
    view! {
        <nav class="flex justify-between items-center py-10 px-8">
            <div class="flex items-center text-xl">
                // <Icon icon=EstateDaoIcon />
                <a href="/">
                    <img
                        src="/img/estate_dao_logo_transparent.webp"
                        alt="Icon"
                        class="h-8 w-full"
                    />
                </a>
            </div>
            // <div class="flex space-x-8">
                // <a href="#" class="text-gray-700 hover:text-gray-900">
                //     Whitepaper
                // </a>
                // <a href="#" class="text-gray-700 hover:text-gray-900">
                //     About us
                // </a>

                // <button />
            // </div>
        </nav>
    }
}

#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <div class="py-16 px-20 flex items-center justify-between">
            <div class="flex items-center space-x-6">
                <div class="font-semibold text-xl">hello@estatedao.com</div>
                <div class="text-xl">
                    <Icon icon=icondata::IoLogoInstagram />
                </div>
                <div class="text-xl">
                    <Icon icon=icondata::BiLinkedin />
                </div>

            </div>
            <div class="text-gray-400 font-semibold">
                "Copyright © 2025 EstateDao. All Rights Reserved."
            </div>
        </div>
    }
}

#[component]
pub fn InputGroup(#[prop(optional, into)] disabled: MaybeSignal<bool>) -> impl IntoView {
    // -------------------------------------
    // BACKGROUND CLASSES FOR DISABLED STATE
    // -------------------------------------

    let bg_class = move || {
        if disabled.get() {
            "bg-gray-300 bg-opacity-[40%]"
        } else {
            "bg-white bg-opacity-[40%]"
        }
    };

    let bg_search_class = move || {
        if disabled.get() {
            "bg-gray-300"
        } else {
            "bg-white text-white hover:bg-blue-200"
        }
    };

    let bg_search_icon_class = move || {
        if disabled.get() {
            "text-gray-400"
        } else {
            "text-blue-600 "
        }
    };

    let search_ctx: SearchCtx = expect_context();

    let destination_display = create_memo(move |_| {
        search_ctx
            .destination
            .get()
            .map(|d| format!("{}, {}", d.city, d.country_name))
            .unwrap_or_else(|| "Where to?".to_string())
    });

    let navigate = use_navigate();
    let search_action = create_action(move |_| {
        let nav = navigate.clone();
        let search_ctx = search_ctx.clone();
        async move {
            log!("Search button clicked");
            //  move to the hotel listing page
            nav(AppRoutes::HotelList.to_string(), Default::default());

            SearchListResults::reset();

            // call server function inside action
            spawn_local(async move {
                let result = search_hotel(search_ctx.into()).await.ok();
                // log!("SEARCH_HOTEL_API: {result:?}");
                SearchListResults::set_search_results(result);
            });
        }
    });

    // let greet_action = create_action(move |_| async move {
    //     match greet_backend("Knull".to_string()).await {
    //         Ok(response) => {
    //             log!("{:#}", response);
    //         }
    //         Err(e) => {
    //             log!("Error greeting knull {:?}", e);
    //         }
    //     }
    // });

    // -------------------------------------

    view! {
        <div class=move || {
            format!(
                " {} backdrop-blur rounded-full flex items-center p-2 border border-gray-300 divide-x divide-white max-w-4xl w-full z-[70]",
                bg_class(),
            )
        }>
            // <!-- Destination input -->

            <div class="relative flex-1">
                <div class="absolute inset-y-0 left-2 text-xl flex items-center">
                    <Icon icon=icondata::BsMap class="text-black" />
                </div>

                <button
                    class="w-full ml-2 py-2 pl-8 text-gray-800 bg-transparent border-none focus:outline-none text-sm text-left"
                    disabled=disabled
                >
                    {move || destination_display.get()}
                </button>

                <Show when=move || !disabled.get()>
                    <div class="absolute inset-0">
                        <DestinationPicker />
                    </div>
                </Show>
            </div>

            // <!-- Date range picker -->
            <div class="relative flex-1 border-l border-r border-white">
                <DateTimeRangePickerCustom />

            </div>

            // <!-- Guests dropdown -->
            <div class="relative flex-1 flex items-center">
                <GuestQuantity />
            </div>

            // <!-- Search button -->
            <button
                on:click=move |_| search_action.dispatch(())
                class=move || {
                    format!(" {}  text-2xl p-2 rounded-full  focus:outline-none", bg_search_class())
                }
            >
                <div>
                    // done with tricks shared by generous Prakash!
                    <Show
                        when=move || disabled.get()
                        fallback=move || {
                            view! {
                                <Icon
                                    icon=icondata::AiSearchOutlined
                                    class=format!("{} p-[1px]", bg_search_icon_class())
                                />
                            }
                        }
                    >
                        <Icon
                            icon=icondata::AiSearchOutlined
                            class=format!("{} p-[1px]", bg_search_icon_class())
                        />
                    </Show>
                </div>
            </button>
        // <button
        // on:click=move |_| greet_action.dispatch(())
        // class=move || {
        // format!(" {}  text-2xl p-2 rounded-full  focus:outline-none", bg_search_class())
        // }
        // >
        // Greet me!
        // </button>
        </div>
    }
}

#[server(GetCityList)]
pub async fn read_destinations_from_file(file_path: String) -> Result<Vec<City>, ServerFnError> {
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
            read_destinations_from_file("city.json".into()).await.ok()
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

fn is_leap_year(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_in_month(month: u32, year: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => panic!("Invalid month"),
    }
}

fn prev_day(year: u32, month: u32, day: u32) -> (u32, u32, u32) {
    if day == 1 {
        if month == 1 {
            (year - 1, 12, 31)
        } else {
            (year, month - 1, days_in_month(month - 1, year))
        }
    } else {
        (year, month, day - 1)
    }
}

fn next_day(year: u32, month: u32, day: u32) -> (u32, u32, u32) {
    let days = days_in_month(month, year);
    if day == days {
        if month == 12 {
            (year + 1, 1, 1)
        } else {
            (year, month + 1, 1)
        }
    } else {
        (year, month, day + 1)
    }
}

fn get_year_month_day(timestamp: f64) -> (u32, u32, u32) {
    let secs = (timestamp / 1000_f64).floor() as i64;
    let naive = chrono::NaiveDateTime::from_timestamp_opt(secs, 0).unwrap();
    let datetime: chrono::DateTime<chrono::Utc> = chrono::DateTime::from_utc(naive, chrono::Utc);
    (datetime.year() as u32, datetime.month(), datetime.day())
}

#[component]
fn MostPopular() -> impl IntoView {
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
            //  move to the hotel listing page
            nav(AppRoutes::HotelList.to_string(), Default::default());

            SearchListResults::reset();

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
                                                class="rounded-lg overflow-hidden border border-gray-300 h-4/5 cursor-pointer hover:shadow-lg transition-shadow"
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
                                                    class="w-full object-cover w-96 h-3/4"
                                                />
                                                <div class="p-4 bg-white">
                                                    <p class="text-lg font-semibold">{city_name}</p>
                                                    <p class="text-sm text-gray-600">{country_name}</p>
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
