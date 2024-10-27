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

#[derive(Default, Clone, Debug)]
struct AdultDetail {
    first_name: String,
    last_name: Option<String>,
    email: Option<String>,     // Only for first adult
    phone: Option<String>,     // Only for first adult
}

#[derive(Default, Clone, Debug)]
struct ChildDetail {
    first_name: String,
    last_name: Option<String>,
    age: Option<u8>,
}


#[component]
pub fn BlockRoomPage() -> impl IntoView {
    let search_list_page: SearchListResults = expect_context();
    
    let adult_count = 4;
    let child_count = 2;

    // Create signals for form data
    let adults = create_rw_signal(vec![AdultDetail::default(); adult_count]);
    let children = create_rw_signal(vec![ChildDetail::default(); child_count]);
    let terms_accepted = create_rw_signal(false);

    // Validation logic
    let is_form_valid = create_memo(move |_| {
        let adult_list = adults.get();
        let child_list = children.get();
        
        // Validate first adult (needs all fields)
        let first_adult_valid = if let Some(first) = adult_list.first() {
            !first.first_name.is_empty() 
            && first.email.is_some() 
            && first.phone.is_some()
        } else {
            false
        };

        // Validate other adults (only names required)
        let other_adults_valid = adult_list.iter().skip(1).all(|adult| {
            !adult.first_name.is_empty()
        });

        // Validate children (first name and age required)
        let children_valid = child_list.iter().all(|child| {
            !child.first_name.is_empty() && child.age.is_some()
        });

        first_adult_valid && other_adults_valid && children_valid && terms_accepted.get()
    });
    
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
                            <div class="pt-6 p-2">
                                <h3 class="font-semibold">"Riva Beach Resort"</h3>
                                <p class="text-gray-600">"North Goa, India"</p>
                            </div>
                        </div>
                        <div class="details">
                            <p class="mt-2"><strong>"Dates: "</strong>"Thu, Aug 22 – Mon, Aug 27"</p>
                            <p class="mt-2"><strong>"Guests: "</strong>{adult_count} adults, {child_count} children</p>
                        </div>
                        <br />
                        <Divider />
                        <br />

                        <div class="payment-methods mt-4 space-y-6">
                        { // Adults
                            (0..adult_count).map(|i| {
                                let is_first = i == 0;
                                view! {
                                    <div class="person-details">
                                        <h3 class="font-semibold text-gray-700">
                                            {if is_first {  String::from("Primary Adult")  } else { format!("Adult {}", i + 1) }}
                                        </h3>
                                        <div class="flex gap-4">
                                            <input
                                                type="text"
                                                placeholder="First Name *"
                                                class="w-1/2 rounded-md border border-gray-300 p-2"
                                                required
                                                on:input=move |ev| {
                                                    let mut list = adults.get();
                                                    list[i].first_name = event_target_value(&ev);
                                                    adults.set(list);
                                                }
                                            />
                                            <input
                                                type="text"
                                                placeholder="Last Name"
                                                class="w-1/2 rounded-md border border-gray-300 p-2"
                                                on:input=move |ev| {
                                                    let mut list = adults.get();
                                                    list[i].last_name = Some(event_target_value(&ev));
                                                    adults.set(list);
                                                }
                                            />
                                        </div>
                                        {move || if is_first {
                                            view! {
                                                <div>
                                                    <input
                                                        type="email"
                                                        placeholder="Email *"
                                                        class="mt-2 w-full rounded-md border border-gray-300 p-2"
                                                        required
                                                        on:input=move |ev| {
                                                            let mut list = adults.get();
                                                            list[0].email = Some(event_target_value(&ev));
                                                            adults.set(list);
                                                        }
                                                    />
                                                    <input
                                                        type="tel"
                                                        placeholder="Phone Number *"
                                                        class="mt-2 w-full rounded-md border border-gray-300 p-2"
                                                        required
                                                        on:input=move |ev| {
                                                            let mut list = adults.get();
                                                            list[0].phone = Some(event_target_value(&ev));
                                                            adults.set(list);
                                                        }
                                                    />
                                                </div>
                                            }.into_view()
                                        } else {
                                            view! {
                                                <div></div>
                                            }.into_view()
                                        }}
                                    </div>
                                }
                            }).collect::<Vec<_>>()
                        }

                        { // Children
                            (0..child_count).map(|i| {
                                view! {
                                    <div class="person-details">
                                        <h3 class="font-semibold text-gray-700">{format!("Child {}", i + 1)}</h3>
                                        <div class="flex gap-4">
                                            <input
                                                type="text"
                                                placeholder="First Name *"
                                                class="w-2/5 rounded-md border border-gray-300 p-2"
                                                required
                                                on:input=move |ev| {
                                                    let mut list = children.get();
                                                    list[i].first_name = event_target_value(&ev);
                                                    children.set(list);
                                                }
                                            />
                                            <input
                                                type="text"
                                                placeholder="Last Name"
                                                class="w-2/5 rounded-md border border-gray-300 p-2"
                                                on:input=move |ev| {
                                                    let mut list = children.get();
                                                    list[i].last_name = Some(event_target_value(&ev));
                                                    children.set(list);
                                                }
                                            />
                                            <select
                                                class="w-1/5 rounded-md border border-gray-300 bg-white p-2"
                                                required
                                                on:change=move |ev| {
                                                    let mut list = children.get();
                                                    list[i].age = event_target_value(&ev).parse().ok();
                                                    children.set(list);
                                                }
                                            >
                                                <option value="" disabled selected>"Age"</option>
                                                { (1..18).map(|age| view! { <option value={age.to_string()}>{age}</option> }).collect::<Vec<_>>() }
                                            </select>
                                        </div>
                                    </div>
                                }
                            }).collect::<Vec<_>>()
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
                            <input
                                type="checkbox"
                                id="agree"
                                class="mr-2"
                                on:change=move |ev| terms_accepted.set(event_target_checked(&ev))
                            />
                            <label for="agree" class="text-sm text-gray-600">
                                "I also agree to the updated "
                                <a href="#" class="text-blue-500 hover:underline">"Terms of Service"</a>
                                ", "
                                <a href="#" class="text-blue-500 hover:underline">"Payments Terms of Service"</a>
                                " and acknowledge the "
                                <a href="#" class="text-blue-500 hover:underline">"Privacy Policy"</a>
                            </label>
                        </div>
                        <button
                            class="mt-6 w-1/3 rounded-full bg-blue-600 py-3 text-white hover:bg-blue-700 disabled:bg-gray-300"
                            disabled={move || !is_form_valid.get()}
                        >
                            "Confirm and pay"
                        </button>
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
