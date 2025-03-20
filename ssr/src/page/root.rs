// use leptos::logging::log;
use crate::log;
use leptos::*;
use leptos_icons::*;
use leptos_query::QueryResult;
use leptos_router::use_navigate;

use crate::component::{Footer, MostPopular, Navbar};
use crate::{
    api::{canister::greet_call::greet_backend, search_hotel},
    app::AppRoutes,
    component::{
        DateTimeRangePickerCustom, Destination, DestinationPicker, EstateDaoIcon, FilterAndSortBy,
        GuestQuantity, GuestSelection, HSettingIcon, SelectedDateRange,
    },
    state::search_state::{SearchCtx, SearchListResults},
};
// use chrono::{Datelike, NaiveDate};
use crate::utils::date::*;
use leptos::ev::MouseEvent;
use leptos_query::{query_persister, *};
use leptos_use::{use_timestamp_with_controls, UseTimestampReturn};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[component]
pub fn RootPage() -> impl IntoView {
    view! {
        <main>
            <div>
                <HeroSection />
                <MostPopular />
            </div>
            // <Footer />
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
                        <a
                            href="https://internetcomputer.org/"
                            target="_blank"
                            rel="noopener noreferrer"
                            class="font-semibold text-blue-500 ml-4 inline"
                        >
                            "Learn more about ICP "
                        </a>
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
