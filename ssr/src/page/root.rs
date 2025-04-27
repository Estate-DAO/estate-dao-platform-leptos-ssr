// use crate::component::outside_click_detector::OutsideClickDetector;
// use leptos::logging::log;
use crate::log;
use crate::state::input_group_state::{InputGroupState, OpenDialogComponent};
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
        FullScreenBannerForMobileModeNotReady, GuestQuantity, GuestSelection, HSettingIcon,
        SelectedDateRange,
    },
    state::search_state::{SearchCtx, SearchListResults},
};
// use chrono::{Datelike, NaiveDate};
use crate::page::InputGroupContainer;
use crate::utils::date::*;
use leptos::ev::MouseEvent;
use leptos_query::{query_persister, *};
use leptos_use::{on_click_outside, use_timestamp_with_controls, UseTimestampReturn};
use serde::{Deserialize, Serialize};
use std::time::Duration;

// use leptos::ev;
// use leptos::html::*;
// use leptos::{event_target, window_event_listener};
// use wasm_bindgen::JsCast;

#[component]
pub fn RootPage() -> impl IntoView {
    view! {
        <main>
            // <FullScreenBannerForMobileModeNotReady>
            <div>
                <HeroSection />
                <MostPopular />
            </div>
            // </FullScreenBannerForMobileModeNotReady>
            // <Footer />
            // <LiveSelectExample />
        </main>
    }
}

#[component]
pub fn HeroSection() -> impl IntoView {
    // reset the search bar
    InputGroupState::toggle_dialog(OpenDialogComponent::None);

    // Define whether outside click collapse is allowed
    // On root page we don't want it enabled
    let allow_outside_click = create_rw_signal(false);

    view! {
        <section class="bg-top bg-cover bg-no-repeat bg-[url('/img/home.webp')]">
            <Navbar />
            <div class="mt-20 md:mt-40 px-4 md:px-0">
                <div class="flex flex-col items-center justify-center h-full">
                    <h1 class="text-3xl md:text-5xl font-semibold text-black mb-6 md:mb-8 text-center">
                        Hey! Where are you off to?
                    </h1>

                    <InputGroupContainer default_expanded=true given_disabled=false allow_outside_click_collapse=allow_outside_click />
                    <br />
                    // todo: uncomment in v2 when implementing filtering and sorting
                    // <FilterAndSortBy />
                    <br />
                    <br />
                    <br />
                    <br />
                    <div class="flex flex-col md:flex-row items-center md:items-end px-4 md:px-6 py-3 bg-white rounded-xl max-w-fit w-full text-center md:text-left">
                        "We're the first decentralized booking platform powered by ICP."
                        <div class="flex items-center mt-2 md:mt-0">
                            <a
                                href="https://internetcomputer.org/"
                                target="_blank"
                                rel="noopener noreferrer"
                                class="font-semibold text-blue-500 md:ml-4 inline"
                            >
                                "Learn more about ICP "
                            </a>
                            <Icon
                                class="w-6 h-6 font-semibold inline ml-2 text-blue-500"
                                icon=icondata::CgArrowRight
                            />
                        </div>
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
pub fn InputGroup(#[prop(optional, into)] given_disabled: MaybeSignal<bool>) -> impl IntoView {
    // TODO (search-button): we want to disable the button for 5 seconds before user can click on it again.
    // button with counter component

    let local_disabled = create_rw_signal(false);
    let disabled = create_memo(move |_|
        // let disabled = Signal::derive(move ||
        {
        let val = given_disabled.get() || local_disabled.get();
        log!("search_bar_disabled - {}", val);
        val
        });
    // -------------------------------------
    // BACKGROUND CLASSES FOR DISABLED STATE
    // -------------------------------------

    let bg_class = move || {
        if disabled.get() {
            // <!-- Updated disabled state to be more subtle on mobile -->
            "bg-gray-100 md:bg-gray-300 md:bg-opacity-[40%]"
        } else {
            // <!-- Removed opacity for mobile to match screenshot -->
            "bg-white md:bg-white md:bg-opacity-[40%]"
        }
    };

    let bg_search_class = move || {
        if disabled.get() {
            "bg-gray-300"
        } else {
            // <!-- Updated search button to be blue on mobile to match screenshot -->
            "bg-blue-500 md:bg-white text-white hover:bg-blue-600 md:hover:bg-blue-200"
        }
    };

    let bg_search_icon_class = move || {
        if disabled.get() {
            "text-gray-400"
        } else {
            // <!-- Updated icon color to white for mobile to match screenshot -->
            "text-white md:text-blue-600"
        }
    };

    let search_ctx: SearchCtx = expect_context();

    let destination_display = create_memo(move |_| {
        search_ctx
            .destination
            .get()
            .map(|d| format!("{}, {}", d.city, d.country_name))
            .unwrap_or_else(|| "".to_string())
    });

    let navigate = use_navigate();
    let search_action = create_action(move |_| {
        log!("[root.rs] search_action create_action callback started");
        SearchListResults::reset();
        log!("[root.rs] SearchListResults reset");

        // close all the dialogs
        InputGroupState::toggle_dialog(OpenDialogComponent::None);
        log!("[root.rs] Dialogs closed");

        let nav = navigate.clone();
        let search_ctx = search_ctx.clone();
        local_disabled.set(true);
        log!("[root.rs] local_disabled set to true");

        async move {
            log!("Search button clicked");
            log!("[root.rs] About to navigate to hotel list page");
            //  move to the hotel listing page
            nav(AppRoutes::HotelList.to_string(), Default::default());
            log!("[root.rs] Navigation triggered");

            // call server function inside action
            spawn_local(async move {
                log!("[root.rs] spawn_local started for search_hotel");
                let result = search_hotel(search_ctx.into()).await.ok();
                log!("[root.rs] search_hotel completed");
                // log!("SEARCH_HOTEL_API: {result:?}");
                SearchListResults::set_search_results(result);
                log!("[root.rs] SearchListResults set");
                local_disabled.set(false);
                log!("[root.rs] local_disabled set to false");
            });
        }
    });

    let close_closure = move |_: ()| {
        log!("[root.rs] close panel");
        InputGroupState::toggle_dialog(OpenDialogComponent::None);
    };

    let parent_div_ref: NodeRef<html::Div> = create_node_ref();

    let _ = on_click_outside(parent_div_ref, move |_| close_closure(()));

    view! {
        // <OutsideClickDetector debug=true on_outside_click=Callback::new(close_closure) exclude_selectors=vec![".most-popular-card".to_string()]>
        <div
            node_ref=parent_div_ref
            class=move || {
                format!(
                    // <!-- Changed mobile styling to use solid white background instead of transparent/backdrop-blur -->
                    // <!-- Added more rounded corners for mobile and better spacing -->
                    // <!-- Improved shadow for better card-like appearance on mobile -->
                    " {} flex flex-col md:flex-row items-stretch md:items-center md:p-1.5 md:divide-x md:divide-white max-w-4xl w-full z-[70] space-y-4 md:space-y-0 bg-white md:bg-transparent rounded-xl md:rounded-full border border-gray-200 shadow-md md:shadow-sm md:backdrop-blur",
                    bg_class(),
                )
            }
        >
            // <!-- Destination input -->
            // <!-- Improved mobile styling with better rounded corners and spacing -->
            <div class="relative flex-1 md:backdrop-blur-none border-0 md:border-0 rounded-lg md:rounded-none overflow-hidden">
                <div class="flex items-center h-[56px] px-6">
                    <Show when=move || !disabled.get()>
                        <div class="absolute inset-0">
                            <DestinationPicker />
                        </div>
                    </Show>

                    <Show when=move || disabled.get()>
                        <div class="text-xl flex items-center">
                            <Icon icon=icondata::BsMap class="text-black" />
                        </div>
                        <button
                            // <!-- Improved text styling and alignment -->
                            class="flex-1 ml-3 text-gray-800 bg-transparent border-none focus:outline-none text-base text-left flex items-center font-normal"
                            disabled=disabled.get()
                        >
                            {move || destination_display.get()}
                        </button>
                    </Show>
                </div>
            </div>

            // <!-- Date range picker -->
            // <!-- Improved mobile styling with better rounded corners and spacing -->
            <div class="relative flex-1 md:backdrop-blur-none border-t border-gray-200 md:border-0 rounded-lg md:rounded-none overflow-hidden">
                <div class="flex items-center h-[56px] px-6">
                    <DateTimeRangePickerCustom />
                </div>
            </div>

            // <!-- Guests dropdown -->
            // <!-- Improved mobile styling with better rounded corners and spacing -->
            <div class="relative flex-1 md:backdrop-blur-none border-t border-gray-200 md:border-0 rounded-lg md:rounded-none overflow-hidden">
                <div class="flex h-[56px] px-6">
                    <GuestQuantity />
                </div>
            </div>

            // <!-- Search button -->
            // <!-- Completely redesigned for mobile to match screenshot with full-width button at bottom -->
            <button
                on:click=move |ev| {
                    ev.prevent_default();
                    search_action.dispatch(())
                }
                class=move || {
                    format!(" {} text-2xl rounded-full w-full focus:outline-none flex items-center justify-center h-[56px] px-4 mx-auto mb-2 md:mb-0 md:w-auto md:mx-0", bg_search_class())
                }
            >
                <div class="flex justify-center">
                    // done with tricks shared by generous Prakash!
                    <Show
                        when=move || disabled.get()
                        fallback=move || {
                            view! {
                                // <!-- Adjusted icon size and padding for better visibility -->
                                <Icon
                                    icon=icondata::AiSearchOutlined
                                    class=format!("{} p-1 text-2xl", bg_search_icon_class())
                                />
                            }
                        }
                    >
                        // <!-- Adjusted icon size and padding for better visibility -->
                        <Icon
                            icon=icondata::AiSearchOutlined
                            class=format!("{} p-1 text-2xl", bg_search_icon_class())
                        />
                    </Show>
                </div>
            </button>

            <div class="h-2 block md:hidden"></div>

        </div>
        // </OutsideClickDetector>
    }
}
