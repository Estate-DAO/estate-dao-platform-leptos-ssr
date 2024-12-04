use leptos::logging::log;
use leptos::*;
use leptos_icons::*;
use leptos_router::use_navigate;

use crate::{
    api::{canister::greet_call::greet_backend, search_hotel},
    app::AppRoutes,
    component::{
        DateTimeRangePickerCustom, DestinationPicker, EstateDaoIcon, FilterAndSortBy,
        GuestQuantity, HSettingIcon,
    },
    state::search_state::{SearchCtx, SearchListResults},
};

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
            <div class="flex space-x-8">
                <a href="#" class="text-gray-700 hover:text-gray-900">
                    Whitepaper
                </a>
                <a href="#" class="text-gray-700 hover:text-gray-900">
                    About us
                </a>

                <button>
                </button>
            </div>
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
                "Copyright © 2024 EstateDao. All Rights Reserved."
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
            //     on:click=move |_| greet_action.dispatch(())
            //     class=move || {
            //         format!(" {}  text-2xl p-2 rounded-full  focus:outline-none", bg_search_class())
            //     }
            // >
            //     Greet me!
            // </button>
        </div>
    }
}

#[component]
fn MostPopular() -> impl IntoView {
    view! {
        <div class="bg-white rounded-[45px] p-4 w-full -mt-8">
            <div class="py-16 px-20">
                <div class="text-2xl font-semibold text-left mb-6">Most popular destinations</div>
                <div class="grid grid-cols-3 gap-4">
                    <Card />
                    <Card />
                    <Card />
                </div>
            </div>
        </div>
    }
}

#[component]
fn Card() -> impl IntoView {
    view! {
        <div class="rounded-lg overflow-hidden border border-gray-300 h-4/5">
            <img src="/img/home.webp" alt="Destination" class="w-full  object-cover  w-96 h-3/4" />
            <div class="p-4 bg-white">
                <p class="text-lg font-semibold">Mehico</p>
            </div>
        </div>
    }
}
