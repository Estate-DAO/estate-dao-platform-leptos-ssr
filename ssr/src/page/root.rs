
use leptos::*;
use leptos_icons::*;
use leptos_router::use_navigate;
use leptos::logging::log;
 
 use crate::{app::AppRoutes, component::{Destination, EstateDaoIcon, FilterAndSortBy, GuestQuantity, HSettingIcon}};
 
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
                    <FilterAndSortBy />
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
                <img src="/img/estate_dao_logo_transparent.webp" alt="Icon" class="h-8 w-full" />
            </div>
            <div class="flex space-x-8">
                <a href="#" class="text-gray-700 hover:text-gray-900">
                    Whitepaper
                </a>
                <a href="#" class="text-gray-700 hover:text-gray-900">
                    About us
                </a>
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

    let bg_class = if disabled.get() {
        "bg-gray-300 bg-opacity-[40%]"
    } else {
        "bg-white bg-opacity-[40%]"
    };

    let bg_search_class = if disabled.get() {
        "bg-gray-300"
    } else {
        "bg-white text-white hover:bg-blue-200"
    };

    let bg_search_icon_class = if disabled.get() {
        "text-gray-400"
    } else {
        "text-blue-600 "
    };

     let navigate = use_navigate();
     let search_action = create_action( move |_| {
         let nav = navigate.clone();
         async move {
             log!("Search button clicked");
             nav(AppRoutes::HotelList.to_string(), Default::default());
         }
     });

    // -------------------------------------

    view! {
        <div class=format!(
            " {} backdrop-blur rounded-full flex items-center p-2 border border-gray-300 divide-x divide-white max-w-4xl w-full",
            bg_class,
        )>
            // <!-- Destination input -->

            <div class="relative flex-1">
                <Destination />
            </div>

            // <!-- Date range picker -->
            <div class="relative flex-1 border-l border-r border-white">
                <div class="absolute inset-y-0 left-2 flex items-center text-2xl">
                    <Icon icon=icondata::AiCalendarOutlined class="text-black font-light" />
                </div>

                <input
                    type="text"
                    placeholder="Check in — Check out"
                    class="w-full ml-2 py-2 pl-8 text-black bg-transparent border-none focus:outline-none text-sm"
                    onfocus="(this.type='date')"
                    onblur="(this.type='text')"
                />

            </div>

            // <!-- Guests dropdown -->
            <div class="relative flex-1 flex items-center">
                <GuestQuantity />
            </div>


            // <!-- Search button -->
            <button
            on:click=move |_| search_action.dispatch(())
            class=format!(
                " {}  text-2xl p-2 rounded-full  focus:outline-none",
                bg_search_class,
            )>
                <div>
                    <Icon
                        icon=icondata::AiSearchOutlined
                        class=format!("{} p-[1px]", bg_search_icon_class)
                    />
                </div>
            </button>
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
