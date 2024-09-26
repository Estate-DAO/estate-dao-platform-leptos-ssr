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
        </main>
    }
}

#[component]
pub fn HeroSection() -> impl IntoView {
    view! {
        <section class="relative bg-cover bg-center h-screen  bg-[url('/img/home.webp')]">
            <Navbar />
            <div class="mt-40">
                <div class="flex flex-col items-center justify-center h-full">
                    <h1 class="text-4xl font-bold text-black mb-8">Hey! Where are you off to?</h1>
                    <InputGroup />
                    <br />
                    <div class="flex space-x-4">
                        <button class="bg-white text-black px-4 py-2 rounded-lg flex items-center ">
                            <Icon class="w-5 h-5 mr-2" icon=HSettingIcon />
                            Filter
                        </button>
                        <button class="bg-white text-black px-4 py-2 rounded-lg flex items-start">
                            Sort By <Icon icon=icondata::BiChevronDownRegular class="w-5 h-5" />
                        </button>
                    </div>
                </div>
                <div class="bg-white rounded-full">
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
            <div class="flex space-x-4">
                <a href="#" class="text-gray-700 hover:text-gray-900">
                    Whitepaper
                </a>
                <a href="#" class="text-gray-700 hover:text-gray-900">
                    About Us
                </a>
            </div>
        </nav>
    }
}

#[component]
pub fn InputGroup() -> impl IntoView {
    view! {
        <div class="bg-white bg-opacity-[50%] backdrop-blur rounded-full flex items-center p-2 shadow-lg max-w-4xl w-full">
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
                    class="w-full ml-2 py-2 pl-8 text-gray-800 bg-transparent border-none focus:outline-none text-sm"
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
        <div class="bg-white rounded-lg shadow-lg p-8">
            <h3 class="text-2xl font-bold text-center mb-8">Most Popular Destinations</h3>
            <div class="grid grid-cols-3 gap-4">
                <Card />
                <Card />
                <Card />
                <Card />
                <Card />
                <Card />
            </div>
        </div>
    }
}

#[component]
fn Card() -> impl IntoView {
    view! {
        <div class="rounded-lg overflow-hidden shadow-md">
            <img
                src="/img/home.webp"
                alt="Destination"
                class="w-full h-48 object-cover"
            />
            <div class="p-4 bg-white">
                <p class="text-lg font-semibold">Mehico</p>
            </div>
        </div>
    }
}
