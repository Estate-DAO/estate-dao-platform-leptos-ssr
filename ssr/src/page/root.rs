use leptos::*;
// use leptos_router::*;

// use crate::component::FullScreenSpinner;

#[component]
pub fn RootPage() -> impl IntoView {
    view! {
        <main class="  py-4 ">
            <div class="">

                <Navbar />

                <HeroSection />

                <MostPopular />
            </div>
        </main>
    }
}

#[component]
pub fn HeroSection() -> impl IntoView {
    view! {
        <section class="bg-cover bg-center h-screen  bg-[url('/img/home.webp')]">
            <div class="flex flex-col items-center justify-center h-full text-center">
                <h1 class="text-4xl font-bold text-black mb-8">Hey! Where are you off to?</h1>
                <div class="flex space-x-4 mb-8">

                    <InputGroup />
                </div>
                <div class="flex space-x-4">
                    <button class="bg-gray-500 text-white px-4 py-2 rounded">Filter</button>
                    <button class="bg-gray-500 text-white px-4 py-2 rounded">Sort By</button>
                </div>
            </div>
        </section>
    }
}

#[component]
pub fn Navbar() -> impl IntoView {
    view! {
        <nav class="flex justify-between items-center py-4 px-8">
            <div class="flex items-center">
                <img src="/img/estate_dao_logo.webp" alt="Icon" class="h-8 w-full" />
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
        <div class="bg-white bg-opacity-80 rounded-full flex items-center p-1 shadow-lg max-w-4xl w-full">
            // <!-- Destination input -->
            <div class="relative flex-1">
                <select class="w-full py-2 pl-10 pr-4 text-gray-700 bg-transparent border-none rounded-full focus:outline-none">
                    <option value="" disabled selected>
                        Destination
                    </option>
                    <option value="paris">Paris</option>
                    <option value="london">London</option>
                    <option value="new-york">New York</option>
                </select>
                <div class="absolute inset-y-0 left-3 flex items-center">
                    <svg
                        class="w-5 h-5 text-gray-400"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                        xmlns="http://www.w3.org/2000/svg"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z"
                        ></path>
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M15 11a3 3 0 11-6 0 3 3 0 016 0z"
                        ></path>
                    </svg>
                </div>
            </div>

            // <!-- Date range picker -->
            <div class="flex-1 border-l border-r border-gray-300">
                <input
                    type="text"
                    placeholder="Check in — Check out"
                    class="w-full py-2 px-4 text-gray-700 bg-transparent border-none focus:outline-none"
                    onfocus="(this.type='date')"
                    onblur="(this.type='text')"
                />
            </div>

            // <!-- Guests dropdown -->
            <div class="relative flex-1">
                <button
                    id="guestsDropdown"
                    class="w-full py-2 pl-4 pr-10 text-left text-gray-700 bg-transparent rounded-full focus:outline-none"
                >
                    "0 adult • 0 children"
                </button>
                <div class="absolute inset-y-0 right-3 flex items-center">
                    <svg
                        class="w-5 h-5 text-gray-400"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                        xmlns="http://www.w3.org/2000/svg"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M19 9l-7 7-7-7"
                        ></path>
                    </svg>
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
            <button class="bg-blue-500 text-white p-2 rounded-full hover:bg-blue-600 focus:outline-none focus:ring-2 focus:ring-blue-400">
                <svg
                    class="w-6 h-6"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                    xmlns="http://www.w3.org/2000/svg"
                >
                    <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="2"
                        d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
                    ></path>
                </svg>
            </button>
        </div>
    }
}

#[component]
fn MostPopular() -> impl IntoView {
    view! {
        <div class="bg-white rounded-lg shadow-lg p-8">
            <div class="flex flex-col justify-center">
                <h1 class="text-2xl font-bold text-center mt-8">Most Popular Destinations</h1>
                <div class="flex space-x-2 justify-center mt-8">
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
        <div>
            <img
                src="/img/home.webp"
                alt="Card Image"
                class="w-full h-48 object-cover rounded-xl"
            />
            <div class="p-4">
                <h3 class="text-lg font-semibold">Card Title</h3>
                <p class="text-gray-600">This is a description of the card.</p>
            </div>
        </div>
    }
}
