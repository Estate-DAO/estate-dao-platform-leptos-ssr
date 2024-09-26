use leptos::*;
// use leptos_router::*;

// use crate::component::FullScreenSpinner;

// use crate::page::location_search::SearchLocation;
use leptos_icons::*;

use crate::component::HSettingIcon;

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
                    <h1 class="text-5xl font-semibold text-black mb-8">Hey! Where are you off to?</h1>
                    <InputGroup />
                    <br />
                    <div class="flex space-x-4">
                        <button class="bg-white text-black px-4 py-2 rounded-lg flex items-center ">
                            <Icon class="w-5 h-5 mr-2" icon=HSettingIcon />
                            Filters
                        </button>
                        <button class="bg-white text-black px-4 py-2 rounded-lg flex items-start">
                            Sort by <Icon icon=icondata::BiChevronDownRegular class="w-6 h-6 ml-2" />
                        </button>
                    </div>
                    <br />
                    <br />
                    <br />
                    <br />
                    <div class="flex items-end px-6 py-3 bg-white rounded-xl max-w-fit w-full ">
                        "We're the first decentralized booking platform powered by ICP."
                        <span class="font-semibold text-blue-500 ml-4 inline"> "Learn more" </span>
                        <Icon class="w-6 h-6 font-semibold inline ml-2 text-blue-500" icon=icondata::CgArrowRight />
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
            <div class="flex items-center">
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
                <div class="text-xl">
                    hello@estatedao.com
                </div>
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
pub fn InputGroup() -> impl IntoView {
    view! {
        <div class="bg-white bg-opacity-[40%] backdrop-blur rounded-full flex items-center p-2 shadow-lg max-w-4xl w-full">
            // <SearchLocation />
            // <!-- Destination input -->

            <div class="relative flex-1">
                <div class="absolute inset-y-0 left-2 text-xl flex items-center">
                    <Icon icon=icondata::BsMap class="text-black" />
                </div>

                <input
                    type="text"
                    placeholder="Destination"
                    class="w-full ml-2 py-2 pl-8 text-gray-800 bg-transparent border-none focus:outline-none text-sm"
                />
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
                <div class="absolute inset-y-0 left-2 flex items-center text-2xl">
                    <Icon icon=icondata::BsPerson class="text-black font-light" />
                </div>

                <button
                    id="guestsDropdown"
                    class="w-full flex-0 py-2 pl-10 text-left text-gray-700 text-sm font-light bg-transparent rounded-full focus:outline-none"
                >
                    "0 adult • 0 children"
                </button>

                <div class="absolute inset-y-2 text-xl right-3 flex items-center">
                    <Icon icon=icondata::BiChevronDownRegular class="text-black" />
                </div>

                <div
                    id="guestsDropdownContent"
                    class="hidden absolute right-0 mt-2 w-48 bg-white rounded-md shadow-lg"
                >
                    <div class="p-4">
                        <div class="mb-4">
                            <label for="adults" class="block text-sm font-medium text-gray-700">
                                Adults
                            </label>
                            <input
                                type="number"
                                id="adults"
                                min="0"
                                value="0"
                                class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-300 focus:ring focus:ring-blue-200 focus:ring-opacity-50"
                            />
                        </div>
                        <div>
                            <label for="children" class="block text-sm font-medium text-gray-700">
                                Children
                            </label>
                            <input
                                type="number"
                                id="children"
                                min="0"
                                value="0"
                                class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-300 focus:ring focus:ring-blue-200 focus:ring-opacity-50"
                            />
                        </div>
                    </div>
                </div>
            </div>

            // <!-- Search button -->
            <button class=" text-2xl bg-white text-white p-2 rounded-full hover:bg-blue-200 focus:outline-none focus:ring-2 focus:ring-blue-400">
                <div>
                    <Icon icon=icondata::AiSearchOutlined class="text-blue-600 p-[1px]" />
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
        <div class="rounded-lg overflow-hidden border border-gray-300 max-h-96">
            <img src="/img/home.webp" alt="Destination" class="w-full aspect-[4/3] object-cover max-h-72" />
            <div class="p-4 bg-white">
                <p class="text-lg font-semibold">Mehico</p>
            </div>
        </div>
    }
}
