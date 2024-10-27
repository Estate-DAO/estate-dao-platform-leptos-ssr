#![allow(unused)]
#![allow(dead_code)]

use leptos::*;
use leptos_router::use_navigate;

use crate::api::get_room;
use crate::component::SkeletonCards;
use crate::state::search_state::HotelInfoResults;
use crate::{
    api::hotel_info,
    app::AppRoutes,
    component::{Divider, FilterAndSortBy, PriceDisplay, StarRating},
    page::{InputGroup, Navbar},
    state::{search_state::SearchListResults, view_state::HotelInfoCtx},
};
use leptos::logging::log;

#[component]
pub fn BlockRoomPage() -> impl IntoView {
    let search_list_page: SearchListResults = expect_context();

    let disabled_submit: Signal<bool> =
        Signal::derive(move || search_list_page.search_result.get().is_none());

    // Assuming the counts for adults and children are provided in the context
    let adult_count = 4; // Replace this with the actual number from the context
    let child_count = 2; // Replace this with the actual number from the context

    view! {
        <section class="relative h-screen">
            <Navbar />
            <div class="relative mt-96 flex h-screen place-content-center items-center justify-center px-[20rem] pt-48">
                <div class="container w-4/5 justify-between gap-6">
                    <button type="text" class="text-3xl font-bold pb-4">"<- You're just one step away!"</button>
                    <br />
                    <div class="p-6">
                        <h3 class="text-xl font-bold">"Your Booking Details"</h3>
                        <div class="details mb-4 flex">
                            <img src="/img/home.webp" alt="Riva Beach Resort" class="h-24 w-24 rounded-lg object-cover" />
                            <div class="pt-6">
                                <h3 class="font-semibold">"Riva Beach Resort"</h3>
                                <p class="text-gray-600">"North Goa, India"</p>
                            </div>
                        </div>
                        <div class="details">
                            <p class="mt-2"><strong>"Dates: "</strong>"Thu, Aug 22 – Mon, Aug 27"</p>
                            <p class="mt-2"><strong>"Guests: "</strong>"{adult_count} adults, {child_count} rooms"</p>
                        </div>
                        <br />
                        <Divider />
                        <br />

                        <div class="payment-methods mt-4 space-y-6">
                            { // Loop for adults
                                (0..adult_count).map(|i| {
                                    view! {
                                        <div class="person-details">
                                            <h3 class="font-semibold text-gray-700">format!("Adult {}", i + 1)</h3>
                                            <div class="flex gap-4">
                                                <input type="text" placeholder="First Name" class="w-1/2 rounded-md border border-gray-300 p-2" />
                                                <input type="text" placeholder="Last Name" class="w-1/2 rounded-md border border-gray-300 p-2" />
                                            </div>
                                            <input type="email" placeholder="Email" class="mt-2 w-full rounded-md border border-gray-300 p-2" />
                                            <input type="tel" placeholder="Phone Number" class="mt-2 w-full rounded-md border border-gray-300 p-2" />
                                        </div>
                                    }
                                }).collect::<Vec<_>>() // Collect the generated inputs into a Vec
                            }

                            { // Loop for children
                                (0..child_count).map(|i| {
                                    view! {
                                        <div class="person-details">
                                            <h3 class="font-semibold text-gray-700">format!("Child {}", i + 1)</h3>
                                            <div class="flex gap-4">
                                                <input type="text" placeholder="First Name" class="w-2/5 rounded-md border border-gray-300 p-2" />
                                                <input type="text" placeholder="Last Name" class="w-2/5 rounded-md border border-gray-300 p-2" />
                                                <select class="w-1/5 rounded-md border border-gray-300 bg-white p-2" aria-label="Age">
                                                    <option value="" disabled selected>Age</option>
                                                    { (1..11).map(|age| view! { <option>{age}</option> }).collect::<Vec<_>>() }
                                                </select>
                                            </div>
                                        </div>
                                    }
                                }).collect::<Vec<_>>() // Collect the generated inputs into a Vec
                            }
                        </div>
                        <br />
                        <Divider />
                        <br />
                        <h2 class="text-2xl font-bold">"Cancellation Policy"</h2>
                        <div class="cancellation-policy mt-6 text-sm text-gray-600">
                            "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Suspendisse felis massa, dignissim eu luctus vel, interdum facilisis orci."
                            <a href="#" class="text-blue-500 hover:underline">"Read more"</a>.
                        </div>
                        <br />
                        <Divider />
                        <br />
                        <div>
                            <input type="checkbox" id="agree" class="mr-2" />
                            <label for="agree" class="text-sm text-gray-600">
                                "I also agree to the updated "
                                <a href="#" class="text-blue-500 hover:underline">"Terms of Service"</a>
                                ", "
                                <a href="#" class="text-blue-500 hover:underline">"Payments Terms of Service"</a>
                                " and acknowledge the "
                                <a href="#" class="text-blue-500 hover:underline">"Privacy Policy"</a>
                            </label>
                        </div>
                        <button class="mt-6 w-1/3 rounded-full bg-blue-600 py-3 text-white hover:bg-blue-700">"Confirm and pay"</button>
                    </div>
                </div>
                <div class="mb-[40rem] rounded-xl bg-white p-6 shadow-xl">
                    <h2 class="mb-4 text-xl font-bold">"₹29,999/night"</h2>
                    <Divider />
                    <div class="price-breakdown">
                        <div class="flex justify-between">
                            <span>"₹29,999 x 5 nights"</span>
                            <span>"₹1,49,995"</span>
                        </div>
                        <div class="flex justify-between">
                            <span>"Taxes and fees"</span>
                            <span>"₹8,912"</span>
                        </div>
                        <div class="price-total mt-4 flex justify-between font-bold">
                            <span>"Total"</span>
                            <span>"₹1,58,907"</span>
                        </div>
                    </div>
                </div>
            </div>
        </section>
    }
}
