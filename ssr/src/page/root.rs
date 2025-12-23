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
use crate::page::InputGroupContainer;
use crate::utils::date::*;
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
use chrono::Datelike;
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
        <section class="bg-top bg-cover bg-no-repeat bg-[url('/img/home.webp')] md:min-h-screen pb-8 md:pb-0">
            <Navbar />
            // <!-- Improved mobile spacing and padding -->
            <div class="mt-8 md:mt-32 px-4 md:px-0">
                <div class="flex flex-col items-center justify-center h-full">
                    // <!-- Enhanced mobile typography with better line height -->
                    <h1 class="text-2xl sm:text-3xl md:text-5xl font-semibold text-black mb-2 sm:mb-6 md:mb-8 text-center leading-tight">
                        Your Next Travel, Paid in Crypto.
                    </h1>
                    <h6 class="font-semibold text-black mb-4 sm:mb-6 md:mb-8 text-center leading-tight text-sm md:text-base">
                        Plan your next escape and pay in BTC, ETH, or your favorite token.
                    </h6>

                    <InputGroupContainer default_expanded=true given_disabled=false allow_outside_click_collapse=allow_outside_click size="large" />

                    // Extra spacing only on desktop
                    <div class="hidden md:block">
                        <br />
                        <br />
                        <br />
                        <br />
                        <br />
                        <br />
                        <br />
                    </div>
                </div>
            </div>
        </section>
    }
}

#[component]
pub fn InputGroup(
    #[prop(optional, into)] given_disabled: MaybeSignal<bool>,
    #[prop(optional, into)] h_class: MaybeSignal<String>,
    #[prop(optional, into)] size: MaybeSignal<String>,
) -> impl IntoView {
    let local_disabled = create_rw_signal(false);
    let disabled = create_memo(move |_| given_disabled.get() || local_disabled.get());

    let bg_class = move || {
        if disabled.get() {
            "bg-gray-100"
        } else {
            "bg-white"
        }
    };
    let bg_search_class = move || {
        if disabled.get() {
            "bg-gray-300 cursor-not-allowed"
        } else {
            "bg-blue-500 hover:bg-blue-600 text-white"
        }
    };
    let bg_search_icon_class = move || {
        if disabled.get() {
            "text-gray-400"
        } else {
            "text-white"
        }
    };

    let search_ctx: UISearchCtx = expect_context();

    let place_display = create_memo(move |_| {
        search_ctx
            .place
            .get()
            .map(|d| {
                if d.formatted_address.trim().is_empty() {
                    d.display_name.clone()
                } else {
                    format!("{}, {}", d.display_name, d.formatted_address)
                }
            })
            .unwrap_or_default()
    });

    let search_action = create_search_action_with_ui_state(local_disabled);
    let parent_div_ref: NodeRef<html::Div> = create_node_ref();

    let size_clone = size.clone();
    let size_clone1 = size.clone();
    let height_class = move || {
        match size_clone.get().as_str() {
            "small" => "h-12", // 48px in navbar
            _ => "h-14",       // 56px default (hero)
        }
    };

    // add this helper next to height_class / row_h
    let btn_w = move || {
        match size_clone1.get().as_str() {
            "small" => "min-w-[48px] w-12", // compact button in navbar (48px)
            _ => "min-w-[56px]",            // 56px in hero/large
        }
    };

    // Height for each row/segment (destination/date/guests)
    let row_h = move || {
        match size.get().as_str() {
            "small" => "h-12", // 48px in navbar
            _ => "h-14",       // 56px default
        }
    };

    view! {
        <div
            node_ref=parent_div_ref
            class=move || format!(
                "relative flex flex-col md:flex-row items-stretch md:items-center max-w-4xl w-full z-[70] \
                 {bg} rounded-md border border-gray-200 shadow-md overflow-hidden md:overflow-visible \
                 md:space-y-0 space-y-3",
                bg = bg_class()
            )
        >

            // Destination
            <div class=format!("flex-1 flex items-center px-2 {}", (row_h.clone())())>
                <Show when=move || !disabled.get()>
                    <DestinationPickerV6 />
                </Show>
                <Show when=move || disabled.get()>
                     <div class="relative w-full h-full flex items-center">
                        <div class="absolute inset-y-0 left-2 flex items-center text-xl pointer-events-none">
                             <Icon icon=icondata::BsMap class="text-blue-500 font-bold"/>
                        </div>
                        <div class="w-full pl-14 text-[15px] font-medium text-left truncate text-gray-500">
                            {move || place_display.get()}
                        </div>
                    </div>
                </Show>
            </div>

            <div class="hidden md:block w-px bg-gray-200 self-stretch"></div>

            // Date range
            <div class=format!("flex-1 flex items-center px-2 {} border-t md:border-t-0 relative md:z-[80]", (row_h.clone())())>
                <DateTimeRangePickerCustom />
            </div>

            <div class="hidden md:block w-px bg-gray-200 self-stretch"></div>

            // Guests
            <div class=move || format!("flex-1 flex items-center px-2 {} border-t md:border-t-0 relative md:z-[80]", row_h())>
                <GuestQuantity />
            </div>

            // Search button
            <button
                on:click=move |ev| {
                    ev.prevent_default();

                    // Auto-set dates if not selected
                    let current_dates = search_ctx.date_range.get();
                    let has_no_dates = current_dates.start == (0, 0, 0) || current_dates.end == (0, 0, 0);

                    if has_no_dates {
                        log!("[InputGroup] No dates selected, auto-setting to next week");
                        let today = chrono::Local::now().date_naive();
                        let checkin = today + chrono::Duration::days(7);
                        let checkout = today + chrono::Duration::days(8);

                        let date_range = SelectedDateRange {
                            start: (checkin.year() as u32, checkin.month(), checkin.day()),
                            end: (checkout.year() as u32, checkout.month(), checkout.day()),
                        };

                        UISearchCtx::set_date_range(date_range);
                    }

                    UIPaginationState::reset_to_first_page();
                    search_action.dispatch(());
                }
                class=move || format!(
                    "flex items-center justify-center gap-2 transition-all duration-200 font-medium \
                    {} {} flex-shrink-0 \
                    rounded-b-md md:rounded-b-none md:rounded-r-md border-l border-white leading-none {}",
                    height_class(),            // h-12 or h-14
                    btn_w(),                   // <- new width/min-width
                    bg_search_class()
                )
                title="Search"
            >
                <Icon
                    icon=icondata::AiSearchOutlined
                    class=format!(
                        "{} text-[22px] md:text-[22px] leading-none",
                        bg_search_icon_class()
                    )
                />
                <span class="block md:hidden text-sm font-medium leading-none">"Search"</span>
            </button>
        </div>
    }
}
