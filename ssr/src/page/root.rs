// use crate::component::outside_click_detector::OutsideClickDetector;
// use leptos::logging::log;
use crate::api::client_side_api::ClientSideApiClient;
use crate::domain::{DomainHotelListAfterSearch, DomainHotelSearchCriteria};
use crate::utils::search_action::create_search_action_with_ui_state;
use crate::view_state_layer::input_group_state::{InputGroupState, OpenDialogComponent};
use crate::{log, utils};
use leptos::*;
use leptos_icons::*;
use leptos_query::QueryResult;

use crate::component::{
    /* DestinationPickerV5, */ CryptoCarousel, DestinationPickerV6, DestinationsSection,
    FeaturesSection, FeedbackSection, Footer, MostPopular, Navbar,
};
use crate::{
    api::canister::greet_call::greet_backend,
    app::AppRoutes,
    component::{
        DateTimeRangePickerCustom, EstateDaoIcon, FilterAndSortBy, GuestQuantity, GuestSelection,
        HSettingIcon, SelectedDateRange,
    },
    page::HotelListParams,
    utils::query_params::QueryParamsSync,
    view_state_layer::ui_search_state::{SearchListResults, UIPaginationState, UISearchCtx},
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
                <HeroSection />
                // <LiveSelectExample />
                <FeaturesSection />
                <DestinationsSection />
                <CryptoCarousel />
                // <MostPopular />
                <FeedbackSection />
            // </FullScreenBannerForMobileModeNotReady>
                <Footer />
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
        <section class="bg-top bg-cover bg-no-repeat bg-[url('/img/home.webp')] min-h-screen">
            <Navbar />
            // <!-- Improved mobile spacing and padding -->
            <div class="mt-16 md:mt-40 px-4 md:px-0">
                <div class="flex flex-col items-center justify-center h-full">
                    // <!-- Enhanced mobile typography with better line height -->
                    <h1 class="text-2xl sm:text-3xl md:text-5xl font-semibold text-black mb-4 sm:mb-6 md:mb-8 text-center leading-tight">
                        Your Next Travel, Paid in Crypto.
                    </h1>
                    <h6 class="font-semibold text-black my-2 sm:mb-6 md:mb-8 text-center leading-tight">
                        Plan your next escape and pay in BTC, ETH, or your favorite token.
                    </h6>

                    <InputGroupContainer default_expanded=true given_disabled=false allow_outside_click_collapse=allow_outside_click />
                    <br />
                    // todo: uncomment in v2 when implementing filtering and sorting
                    // <FilterAndSortBy />
                    <br />
                    <br />
                    <br />
                    <br />
                    // <!-- Improved mobile card layout with better responsive padding -->
                    // <div class="flex flex-col md:flex-row items-center md:items-end px-3 sm:px-4 md:px-6 py-3 sm:py-4 bg-white rounded-xl max-w-fit w-full text-center md:text-left mx-2 sm:mx-0">
                    //     <span class="text-sm sm:text-base">
                    //         "We're the first decentralized booking platform powered by ICP."
                    //     </span>
                    //     <div class="flex items-center mt-2 md:mt-0">
                    //         <a
                    //             href="https://internetcomputer.org/"
                    //             target="_blank"
                    //             rel="noopener noreferrer"
                    //             class="font-semibold text-blue-500 md:ml-4 inline text-sm sm:text-base"
                    //         >
                    //             "Learn more about ICP "
                    //         </a>
                    //         <Icon
                    //             class="w-5 h-5 sm:w-6 sm:h-6 font-semibold inline ml-2 text-blue-500"
                    //             icon=icondata::CgArrowRight
                    //         />
                    //     </div>
                    // </div>
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
            "bg-gray-100 md:bg-gray-300"
        } else {
            // <!-- Removed opacity for mobile to match screenshot -->
            "bg-white"
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

    let search_ctx: UISearchCtx = expect_context();

    let place_display = create_memo(move |_| {
        search_ctx
            .place
            .get()
            .map(|d| format!("{}, {}", d.display_name, d.formatted_address))
            .unwrap_or_else(|| "".to_string())
    });

    // Use shared search action with UI state management
    let search_action = create_search_action_with_ui_state(local_disabled);

    log!("[root.rs InputGroup] Search action created with UI state management");

    // let close_closure = move |_: ()| {
    //     log!("[root.rs] close panel");
    //     InputGroupState::toggle_dialog(OpenDialogComponent::None);
    // };

    let parent_div_ref: NodeRef<html::Div> = create_node_ref();

    // let _ = on_click_outside(parent_div_ref, move |_| close_closure(()));

    view! {
        <div
            node_ref=parent_div_ref
            class=move || {
                format!(
                    // <!-- Changed mobile styling to use solid white background instead of transparent/backdrop-blur -->
                    // <!-- Added more rounded corners for mobile and better spacing -->
                    // <!-- Improved shadow for better card-like appearance on mobile -->
                    " {} flex flex-col md:flex-row items-stretch md:items-center md:p-1.5 md:divide-x md:divide-white max-w-4xl w-full z-[70] space-y-4 md:space-y-0 bg-white rounded-xl md:rounded-full border border-gray-200 shadow-md md:shadow-sm md:backdrop-blur",
                    bg_class(),
                )
            }
        >
            // <!-- Destination input -->
            // <!-- Improved mobile styling with better rounded corners and spacing -->
            // <div class="relative flex-1 md:backdrop-blur-none border-0 md:border-0 rounded-lg md:rounded-none overflow-hidden">
            <div class="relative flex-1 border-0 md:border-0 rounded-lg md:rounded-none">
                <div class="flex items-center h-[56px] px-6">
                    <Show when=move || !disabled.get()>
                        <div class="absolute inset-0">
                            <DestinationPickerV6 />
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
                            {move || place_display.get()}
                        </button>
                    </Show>
                </div>
            </div>

            // <!-- Date range picker -->
            // <!-- Improved mobile styling with better rounded corners and spacing -->
            // <div class="relative flex-1 md:backdrop-blur-none border-t border-gray-200 md:border-0 rounded-lg md:rounded-none overflow-hidden">
            <div class="relative flex-1 md:backdrop-blur-none border-t border-gray-200 md:border-0 rounded-lg md:rounded-none">
                <div class="flex items-center h-[56px] px-6">
                    <DateTimeRangePickerCustom />
                </div>
            </div>

            // <!-- Guests dropdown -->
            // <!-- Improved mobile styling with better rounded corners and spacing -->
            // <div class="relative flex-1 md:backdrop-blur-none border-t border-gray-200 md:border-0 rounded-lg md:rounded-none overflow-hidden">
            <div class="relative flex-1 md:backdrop-blur-none border-t border-gray-200 md:border-0 rounded-lg md:rounded-none">
                <div class="relative flex h-[56px] px-6">
                    <GuestQuantity />
                </div>
            </div>

            // <!-- Search button -->
            // <!-- Completely redesigned for mobile to match screenshot with full-width button at bottom -->
            <div class="px-6 md:px-0">
            <button
                on:click=move |ev| {
                    ev.prevent_default();
                    log!("[root.rs InputGroup] Search button clicked, about to dispatch search action");

                    // Reset pagination to first page when search is clicked
                    UIPaginationState::reset_to_first_page();
                    log!("[root.rs InputGroup] Pagination reset to first page");

                    // Log current UISearchCtx state before dispatch
                    let current_search_ctx: UISearchCtx = expect_context();
                    log!("[root.rs InputGroup] Current UISearchCtx before dispatch - destination: {:?}", current_search_ctx.destination.get());
                    log!("[root.rs InputGroup] Current UISearchCtx before dispatch - date_range: {:?}", current_search_ctx.date_range.get());
                    log!("[root.rs InputGroup] Current UISearchCtx before dispatch - adults: {}", current_search_ctx.guests.adults.get());

                    search_action.dispatch(());
                    log!("[root.rs InputGroup] Search action dispatched");
                }
                class=move || {
                    format!(" {} rounded-full w-full focus:outline-none flex items-center justify-center h-[56px] px-4 mx-auto mb-2 md:mb-0 md:w-auto md:mx-0", bg_search_class())
                }
            >
                <div class="flex justify-center text-2xl ">
                    // done with tricks shared by generous Prakash!
                    <div class="hidden md:block">
                    <Show
                        when=move || disabled.get()
                        fallback=move || {
                            view! {
                                <Icon
                                    icon=icondata::AiSearchOutlined
                                    class=format!("{} p-1 text-2xl", bg_search_icon_class())
                                />
                            }
                        }
                    >
                        <Icon
                            icon=icondata::AiSearchOutlined
                            class=format!("{} p-1 text-2xl", bg_search_icon_class())
                        />

                    </Show>
                    </div>
                    <div class="block md:hidden text-lg">

                    <Show
                    when=move || disabled.get()
                    fallback=move || {
                        view! {
                            <div class="disabled">Search</div>
                        }
                    }
                >
                        Search
                </Show>
                </div>

                </div>
            </button>
            </div>
            <div class="h-1 block md:hidden"></div>

        </div>
    }
}
