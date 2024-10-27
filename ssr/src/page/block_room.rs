#![allow(unused)]
#![allow(dead_code)]

use leptos::*;
use leptos_router::use_navigate;

use crate::api::get_room;

use crate::component::SkeletonCards;
use crate::state::search_state::HotelInfoResults;
use crate::{
    api::{
        book_room,
        BookRoomRequest, BookRoomResponse, RoomDetail, 
        PassengerDetail, PaxType, BookingStatus
    },
    app::AppRoutes,
    component::{Divider, FilterAndSortBy, PriceDisplay, StarRating},
    page::{InputGroup, Navbar},
    state::{search_state::{SearchCtx, SearchListResults, ConfirmationResults}, view_state::HotelInfoCtx},
};
use leptos::logging::log;
use chrono::NaiveDate;



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
    let search_ctx: SearchCtx = expect_context();
    let hotel_info_results: HotelInfoResults = expect_context();
    let hotel_info_ctx: HotelInfoCtx = expect_context();
    let search_list_results: SearchListResults = expect_context();
    let search_list_results = store_value(search_list_results);  // Store it so we can clone inside closure

    
    let confirmation_results: ConfirmationResults = expect_context();

    let navigate = use_navigate();

    // Helper function to create passenger details
    fn create_passenger_details(
        adults: &[AdultDetail], 
        children: &[ChildDetail]
    ) -> Vec<PassengerDetail> {
        let mut passengers = Vec::new();

        // Add adults
        for (i, adult) in adults.iter().enumerate() {
            passengers.push(PassengerDetail {
                title: "Mr".to_string(), // Add logic for title selection
                first_name: adult.first_name.clone(),
                middle_name: None,
                last_name: adult.last_name.clone().unwrap_or_default(),
                email: if i == 0 { 
                    adult.email.clone().unwrap_or_default() 
                } else { 
                    String::new() 
                },
                pax_type: PaxType::Adult,
                lead_passenger: i == 0,
                children_ages: None,
            });
        }

        // Add children
        for child in children {
            passengers.push(PassengerDetail {
                title: "".to_string(),
                first_name: child.first_name.clone(),
                middle_name: None,
                last_name: child.last_name.clone().unwrap_or_default(),
                email: String::new(),
                pax_type: PaxType::Child,
                lead_passenger: false,
                children_ages: child.age.map(|age| age as u32), // Convert u8 to u32
            });
        }

        passengers
    }


    let adult_count = create_memo(move |_| {
        search_ctx.guests.get().adults.get()
    });

    let child_count = create_memo(move |_| {
        search_ctx.guests.get().children.get()
    });

    let num_rooms = Signal::derive(move || search_ctx.guests.get().rooms.get());

    // Create signals for form data
    let adults = create_rw_signal(vec![AdultDetail::default(); adult_count.get() as usize]);
    let children = create_rw_signal(vec![ChildDetail::default(); child_count.get() as usize]);
    let terms_accepted = create_rw_signal(false);
    
    let (trigger_validation, set_trigger_validation) = create_signal(0);
    
    let update_adult = move |index: usize, field: &str, value: String| {
        adults.update(|list| {
            if let Some(adult) = list.get_mut(index) {
                match field {
                    "first_name" => adult.first_name = value,
                    "last_name" => adult.last_name = Some(value),
                    "email" => if index == 0 { adult.email = Some(value) },
                    "phone" => if index == 0 { adult.phone = Some(value) },
                    _ => {}
                }
            }
        });
        set_trigger_validation.update(|n| *n += 1);

    };
    
    let update_child = move |index: usize, field: &str, value: String| {
        children.update(|list| {
            if let Some(child) = list.get_mut(index) {
                match field {
                    "first_name" => child.first_name = value,
                    "last_name" => child.last_name = Some(value),
                    "age" => child.age = value.parse().ok(),
                    _ => {}
                }
            }
        });
        set_trigger_validation.update(|n| *n += 1);

    };
    
    let update_terms = move |checked: bool| {
        terms_accepted.set(checked);
        // Trigger validation check
        set_trigger_validation.update(|n| *n += 1);
    };
    
    let navigate = use_navigate();
    let nav = navigate.clone(); // Clone it here for the first use

    let go_back_to_details = move |ev: ev::MouseEvent| {
        ev.prevent_default();
        let _ = navigate(AppRoutes::HotelDetails.to_string(), Default::default());
    };
    
    let handle_booking = create_action(move |_| {
        let nav = nav.clone(); // Use the cloned version here
        let adults_data = adults.get();
        let children_data = children.get();
        let hotel_code = hotel_info_ctx.hotel_code.get().unwrap_or_default();
        let search_list_results = search_list_results.get_value();  // Get the stored value


        async move {
            let room_detail = RoomDetail {
                passenger_details: create_passenger_details(&adults_data, &children_data),
            };
    
            // Get room_unique_id from HotelRoomResponse
            let block_room_id = hotel_info_results.room_result.get()
                .and_then(|room_response| room_response.room_list.clone())
                .and_then(|room_list| room_list.get_hotel_room_result.hotel_rooms_details.first().cloned())
                .map(|hotel_room_detail| hotel_room_detail.room_unique_id.clone())
                .unwrap_or_default();
    
            let token = search_list_results
                .get_hotel_code_results_token_map()
                .get(&hotel_code)
                .cloned()
                .unwrap_or_default();
    
            let book_request = BookRoomRequest {
                result_token: token,
                block_room_id,
                app_reference: format!("BOOKING_{}_{}",
                    chrono::Utc::now().timestamp(),
                    hotel_code
                ),
                room_details: vec![room_detail],
            };
            match book_room(book_request).await {
                Ok(response) => {
                    match response.status {
                        BookingStatus::Confirmed => {
                            // Set the booking details in context
                            ConfirmationResults::set_booking_details(Some(response));
                            nav(AppRoutes::Confirmation.to_string(), Default::default());
                        },
                        BookingStatus::BookFailed => {
                            log!("Booking failed: {:?}", response.message);
                        }
                    }
                },
                Err(e) => {
                    log!("Error booking room: {:?}", e);
                }
            }
        }
    });
    
    let is_form_valid = create_memo(move |_| {
        
        trigger_validation.get();

        let adult_list = adults.get();
        let child_list = children.get();

        // Validate primary adult (needs all fields)
        let primary_adult_valid = adult_list.first().map_or(false, |adult| {
            !adult.first_name.is_empty() 
            && adult.email.as_ref().map_or(false, |e| !e.is_empty())
            && adult.phone.as_ref().map_or(false, |p| !p.is_empty())
        });

        // Validate other adults (only first name required)
        let other_adults_valid = adult_list.iter().skip(1).all(|adult| {
            !adult.first_name.is_empty()
        });

        // Validate children (first name and age required)
        let children_valid = child_list.iter().all(|child| {
            !child.first_name.is_empty() 
            && child.age.is_some()
        });

        // Terms must be accepted
        let terms_valid = terms_accepted.get();

        // All conditions must be true
        primary_adult_valid && other_adults_valid && children_valid && terms_valid
    });
    
    let images_signal = move || {
        if let Some(hotel_info_api_response) = hotel_info_results.search_result.get() {
            hotel_info_api_response.get_images()
        } else {
            vec![]
        }
    };

    let hotel_name_signal = move || {
        if let Some(hotel_info_api_response) = hotel_info_results.search_result.get() {
            hotel_info_api_response.get_hotel_name()
        } else {
            "".into()
        }
    };
    
    let destination = create_memo(move |_| {
        search_ctx.destination.get().unwrap_or_default()
    });
    
    view! {
        <section class="relative h-screen">
            <Navbar />
            <div class="relative mt-48 flex h-screen place-content-center items-center justify-center px-[20rem] pt-48">
                <div class="container w-4/5 justify-between gap-6">
                    <button type="text" class="text-3xl font-bold pb-4" on:click=go_back_to_details>"<- You're just one step away!"</button>
                    <br />
                    <div class="p-6">
                        <h3 class="text-xl font-bold">"Your Booking Details"</h3>
                        <div class="details mb-4 flex">
                            <img 
                                src={move || hotel_info_ctx.selected_hotel_image.get()}
                                alt={move || hotel_info_ctx.selected_hotel_name.get()}
                                class="h-24 w-24 rounded-lg object-cover" 
                            />
                            <div class="pt-6 p-2">
                                <h3 class="font-semibold">{move || hotel_info_ctx.selected_hotel_name.get()}</h3>
                                <p class="text-gray-600">{move || hotel_info_ctx.selected_hotel_location.get()}</p>
                            </div>
                        </div>
                        <div class="details">
                            <p class="mt-2"><strong>"Dates: "</strong>
                            {move || {
                                let date_range = search_ctx.date_range.get();
                                
                                let format_date = |(year, month, day): (u32, u32, u32)| {
                                    NaiveDate::from_ymd_opt(year as i32, month, day)
                                        .map(|d| d.format("%a, %b %d").to_string())
                                        .unwrap_or_default()
                                };
                            
                                format!("{} - {}", 
                                    format_date(date_range.start),
                                    format_date(date_range.end)
                                )
                            }}
                            </p>
                            <p class="mt-2"><strong>"Guests: "</strong>
                                {move || format!("{} adults, {} children", 
                                    adult_count.get(), 
                                    child_count.get()
                                )}
                            </p>
                            <p class="mt-2"><strong>"Rooms: "</strong>
                                {move || format!("{} {}", 
                                    num_rooms.get(),
                                    if num_rooms.get() == 1 { "room" } else { "rooms" }
                                )}
                            </p>
                        </div>
                        <br />
                        <Divider />
                        <br />

                        <div class="payment-methods mt-4 space-y-6">
                        { // Loop for adults
                            (0..adult_count.get()).map(|i| {
                                view! {
                                    <div class="person-details">
                                        <h3 class="font-semibold text-gray-700">
                                            {if i == 0 { 
                                                String::from("Primary Adult") 
                                            } else { 
                                                format!("Adult {}", i + 1) 
                                            }}
                                        </h3>
                                        <div class="flex gap-4">
                                        <input 
                                            type="text" 
                                            placeholder="First Name *" 
                                            class="w-1/2 rounded-md border border-gray-300 p-2" 
                                            required=true
                                            on:input=move |ev| update_adult(0, "first_name", event_target_value(&ev))
                                        />
                                        <input type="text" placeholder="Last Name" class="w-1/2 rounded-md border border-gray-300 p-2" />
                                        </div>
                                        {move || if i == 0 {
                                            view! {
                                                <div>
                                                    <input type="email" placeholder="Email *" class="mt-2 w-full rounded-md border border-gray-300 p-2" required=true on:input=move |ev| update_adult(0, "email", event_target_value(&ev)) />
                                                    <input type="tel" placeholder="Phone Number *" class="mt-2 w-full rounded-md border border-gray-300 p-2" required=true on:input=move |ev| update_adult(0, "phone_number", event_target_value(&ev)) />
                                                </div>
                                            }.into_view()
                                        } else {
                                            view! { <div></div> }.into_view()
                                        }}
                                    </div>
                                }
                            }).collect::<Vec<_>>()
                        }

                        { // Loop for children
                            (0..child_count.get()).map(|i| {
                                view! {
                                    <div class="person-details">
                                        <h3 class="font-semibold text-gray-700">{format!("Child {}", i + 1)}</h3>
                                        <div class="flex gap-4">
                                            <input 
                                                type="text" 
                                                placeholder="First Name *" 
                                                class="w-2/5 rounded-md border border-gray-300 p-2" 
                                                required=true
                                                on:input=move |ev| update_child(0, "first_name", event_target_value(&ev))
                                            />
                                            <input 
                                                type="text" 
                                                placeholder="Last Name" 
                                                class="w-2/5 rounded-md border border-gray-300 p-2" 
                                            />
                                            <select 
                                                class="w-1/5 rounded-md border border-gray-300 bg-white p-2" 
                                                required=true
                                                on:input=move |ev| update_adult(0, "age", event_target_value(&ev))
                                            >
                                                <option value="" disabled selected>"Age *"</option>
                                                { (1..18).map(|age| view! { <option value={age.to_string()}>{age}</option> })
                                                    .collect::<Vec<_>>() }
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
                                on:change=move |ev| update_terms(event_target_checked(&ev))
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
                            on:click=move |_| handle_booking.dispatch(())
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
