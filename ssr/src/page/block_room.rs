#![allow(unused)]
#![allow(dead_code)]

use leptos::*;
use leptos_icons::*;
use leptos_router::use_navigate;

use crate::api::get_room;
use crate::utils::pluralize;

use crate::component::SkeletonCards;
use crate::state::search_state::HotelInfoResults;
use crate::state::view_state::AdultDetail;
use crate::state::view_state::BlockRoomCtx;
use crate::state::view_state::ChildDetail;

use crate::{
    api::{
        book_room, BookRoomRequest, BookRoomResponse, BookingStatus, PassengerDetail, PaxType,
        RoomDetail,
    },
    app::AppRoutes,
    component::{Divider, FilterAndSortBy, PriceDisplay, StarRating},
    page::{InputGroup, Navbar},
    state::{
        search_state::{ConfirmationResults, SearchCtx, SearchListResults},
        view_state::HotelInfoCtx,
    },
};
use chrono::NaiveDate;
use leptos::logging::log;

#[component]
pub fn BlockRoomPage() -> impl IntoView {
    let search_ctx: SearchCtx = expect_context();
    let hotel_info_results: HotelInfoResults = expect_context();
    let hotel_info_ctx: HotelInfoCtx = expect_context();
    let search_list_results: SearchListResults = expect_context();
    let search_list_results = store_value(search_list_results); // Store it so we can clone inside closure

    let confirmation_results: ConfirmationResults = expect_context();

    let navigate = use_navigate();
    let block_room_ctx = expect_context::<BlockRoomCtx>();

    let hotel_info_results_clone = hotel_info_results.clone();
    // let room_price = create_memo(move |_| {
    //     hotel_info_results_clone
    //         .room_result
    //         .get()
    //         .and_then(|response| response.room_list.clone())
    //         .and_then(|room_list| {
    //             room_list
    //                 .get_hotel_room_result
    //                 .hotel_rooms_details
    //                 .first()
    //                 .map(|room| room.price.room_price as f64)
    //         })
    //         .unwrap_or(0.0)
    // });
    let room_price = create_memo(move |_| {
        hotel_info_results_clone
            .price_per_night
            .get()
    });

    // let total_price_per_night = create_memo(move |_| {
    //     let price = room_price.get();
    //     let num_rooms = search_ctx.guests.get().rooms.get();
    //     price * num_rooms as f64
    // });

    let num_nights = Signal::derive(move || {
        search_ctx.date_range.get().no_of_nights()
    });

    let total_price = create_memo(move |_| {
        let room_price = room_price.get();
        let nights = num_nights.get();
        room_price * nights as f64
    });

    // let final_total = create_memo(move |_| total_price.get());

    // Helper function to create passenger details
    fn create_passenger_details(
        adults: &[AdultDetail],
        children: &[ChildDetail],
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

    let adult_count = create_memo(move |_| search_ctx.guests.get().adults.get());

    let child_count = create_memo(move |_| search_ctx.guests.get().children.get());

    let children_ages = search_ctx.guests.get().children_ages.clone();

    let num_rooms = Signal::derive(move || search_ctx.guests.get().rooms.get());

    BlockRoomCtx::create_adults(adult_count.get() as usize);
    BlockRoomCtx::create_children(child_count.get() as usize);
    BlockRoomCtx::set_terms_accepted(false);

    let (trigger_validation, set_trigger_validation) = create_signal(0);

    let update_adult = move |index: usize, field: &str, value: String| {
        BlockRoomCtx::update_adult(index, field, value);
    };

    let update_child = move |index: usize, field: &str, value: String| {
        BlockRoomCtx::update_child(index, field, value);
    };

    let update_terms = move |checked: bool| {
        BlockRoomCtx::set_terms_accepted(checked);
        // Trigger validation check
    };

    let navigate = use_navigate();
    let nav = navigate.clone(); // Clone it here for the first use

    let go_back_to_details = move |ev: ev::MouseEvent| {
        ev.prevent_default();
        let _ = navigate(AppRoutes::HotelDetails.to_string(), Default::default());
    };

    let adults = block_room_ctx.adults;
    let children = block_room_ctx.children;
    let terms_accepted = block_room_ctx.terms_accepted;

    let handle_booking = create_action(move |_| {
        let nav = nav.clone(); // Use the cloned version here
        let adults_data = adults.get();
        let children_data = children.get();
        let hotel_code = hotel_info_ctx.hotel_code.get().unwrap_or_default();
        let search_list_results = search_list_results.get_value(); // Get the stored value

        async move {
            let room_detail = RoomDetail {
                passenger_details: create_passenger_details(&adults_data, &children_data),
            };

            // Get room_unique_id from HotelRoomResponse
            let block_room_id = hotel_info_results
                .room_result
                .get()
                .and_then(|room_response| room_response.room_list.clone())
                .and_then(|room_list| {
                    room_list
                        .get_hotel_room_result
                        .hotel_rooms_details
                        .first()
                        .cloned()
                })
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
                app_reference: format!("BOOKING_{}_{}", chrono::Utc::now().timestamp(), hotel_code),
                room_details: vec![room_detail],
            };

            // match book_room(book_request).await {
            //     Ok(response) => {
            //         match response.status {
            //             BookingStatus::Confirmed => {
            //                 // Set the booking details in context
            //                 ConfirmationResults::set_booking_details(Some(response));
            nav(AppRoutes::Confirmation.to_string(), Default::default());
            //             }
            //             BookingStatus::BookFailed => {
            //                 log!("Booking failed: {:?}", response.message);
            //             }
            //         }
            //     }
            //     Err(e) => {
            //         log!("Error booking room: {:?}", e);
            //     }
            // }
        }
    });
    let is_form_valid: RwSignal<bool> = create_rw_signal(false);

    // Validation logic function
    let validate_form = move || {
        let adult_list = adults.get();
        let child_list = children.get();

        // Helper function for email validation
        let is_valid_email = |email: &str| email.contains('@') && email.contains('.');

        // Helper function for phone validation
        let is_valid_phone =
            |phone: &str| phone.len() >= 10 && phone.chars().all(|c| c.is_digit(10));

        // Validate primary adult
        let primary_adult_valid = adult_list.first().map_or(false, |adult| {
            !adult.first_name.trim().is_empty()
                && adult
                    .email
                    .as_ref()
                    .map_or(false, |e| !e.trim().is_empty() && is_valid_email(e))
                && adult
                    .phone
                    .as_ref()
                    .map_or(false, |p| !p.trim().is_empty() && is_valid_phone(p))
        });

        // Validate other adults
        let other_adults_valid = adult_list
            .iter()
            .skip(1)
            .all(|adult| !adult.first_name.trim().is_empty());

        // Validate children
        let children_valid = child_list
            .iter()
            .all(|child| !child.first_name.trim().is_empty() && child.age.is_some());

        // Check if terms are accepted
        let terms_valid = terms_accepted.get();

        // Set the value of is_form_valid based on validation results
        is_form_valid
            .set(primary_adult_valid && other_adults_valid && children_valid && terms_valid);
    };

    // Call the validation function whenever inputs change
    let _ = create_effect(move |_| {
        validate_form();
    });

    // let is_form_valid = create_memo(move |_| {
    //     trigger_validation.get();

    //     let adult_list = adults.get();
    //     let child_list = children.get();

    //     // Validate primary adult (needs all fields)
    //     let primary_adult_valid = adult_list.first().map_or(false, |adult| {
    //         !adult.first_name.is_empty()
    //             && adult.email.as_ref().map_or(false, |e| !e.is_empty())
    //             && adult.phone.as_ref().map_or(false, |p| !p.is_empty())
    //     });

    //     // Validate other adults (only first name required)
    //     let other_adults_valid = adult_list
    //         .iter()
    //         .skip(1)
    //         .all(|adult| !adult.first_name.is_empty());

    //     // Validate children (first name and age required)
    //     let children_valid = child_list
    //         .iter()
    //         .all(|child| !child.first_name.is_empty() && child.age.is_some());

    //     // Terms must be accepted
    //     let terms_valid = terms_accepted.get();

    //     // All conditions must be true
    //     primary_adult_valid && other_adults_valid && children_valid && terms_valid
    // });

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

    let destination = create_memo(move |_| search_ctx.destination.get().unwrap_or_default());

    let insert_real_image_or_default = {
        move || {
            if let Some(hotel_info_api_response) = hotel_info_results.search_result.get() {
                hotel_info_api_response
                    .get_images()
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "/img/home.webp".to_string())
            } else {
                "/img/home.webp".to_string()
            }
        }
    };

    view! {

        <section class="relative h-screen">
            <Navbar />
            <div class="relative mt-24 flex h-screen  items-center place-content-center p-4 max-w-4xl mx-auto ">
                <div class="container w-4/5 justify-between gap-6">
                    <button type="text" class="text-3xl font-bold pb-4 flex" on:click=go_back_to_details>
                    // "<- You're just one step away!"
                    <span class="inline-block items-center"> <Icon icon=icondata::AiArrowLeftOutlined class="text-black font-light" /> </span>
                    <div class="ml-4">"You're just one step away!" </div>
                    </button>
                    <br />
                    <div class="p-6">
                        <h3 class="text-xl font-bold">"Your Booking Details"</h3>
                        <div class="details mb-4 flex">
                            <img
                                src=insert_real_image_or_default
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
                                date_range.format_as_human_readable_date()
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
                                let i_usize = i as usize;
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
                                            on:input=move |ev| {
                                                update_adult(i_usize, "first_name", event_target_value(&ev));
                                                validate_form();
                                            }
                                        />
                                        <input type="text" placeholder="Last Name" class="w-1/2 rounded-md border border-gray-300 p-2" />
                                        </div>
                                        {move || if i == 0 {
                                            view! {
                                                <div>
                                                    <input type="email" placeholder="Email *" class="mt-2 w-full rounded-md border border-gray-300 p-2"
                                                        required=true on:input=move |ev| update_adult(0, "email", event_target_value(&ev))
                                                    />
                                                    <input type="tel" placeholder="Phone *" class="mt-2 w-full rounded-md border border-gray-300 p-2"
                                                        required=true on:input=move |ev| update_adult(0, "phone", event_target_value(&ev))
                                                    />
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
                                let i_usize = i as usize;
                                let age_value = children_ages.get_value_at(i as u32); // Get the age for the current child

                                view! {
                                    <div class="person-details">
                                        <h3 class="font-semibold text-gray-700">{format!("Child {}", i + 1)}</h3>
                                        <div class="flex gap-4">
                                            <input
                                                type="text"
                                                placeholder="First Name *"
                                                class="w-2/5 rounded-md border border-gray-300 p-2"
                                                required=true
                                                on:input=move |ev| {
                                                    update_child(i_usize, "first_name", event_target_value(&ev));
                                                    validate_form();
                                                }
                                            />
                                            <input
                                                type="text"
                                                placeholder="Last Name"
                                                class="w-2/5 rounded-md border border-gray-300 p-2"
                                            />
                                            <select
                                                class="w-1/5 rounded-md border border-gray-300 bg-white p-2"
                                                required=true
                                                on:input=move |ev| {
                                                    update_child(i_usize, "age", event_target_value(&ev));
                                                    validate_form();
                                                }
                                            >

                                            <option disabled selected>{age_value}</option>
                                            { (1..18).map(|age| {
                                                let selected = if age == age_value { "selected" } else { "" };
                                                view! { <option value={age.to_string()} {selected}>{age}</option> }
                                            }).collect::<Vec<_>>() }
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
                            disabled=move || !is_form_valid.get()
                            on:click=move |_| handle_booking.dispatch(())
                        >
                            "Confirm and pay"
                        </button>
                    </div>
                </div>
                <div class="mb-[40rem] rounded-xl bg-white p-6 shadow-xl">
                    <h2 class="mb-4 text-xl font-bold">"₹"{room_price.get() as u64}"/" "night" </h2>
                    <Divider />
                    <div class="price-breakdown">
                        <div class="flex justify-between">
                            <span>"₹"{room_price.get() as u64}" x "{move || pluralize(num_nights.get(), "night", "nights")}</span>
                        </div>

                        <div class="price-total mt-4 flex justify-between font-bold">
                            <span>"Total"</span>
                            <span>"₹"{total_price.get() as u64}</span>
                        </div>
                    </div>
                </div>
            </div>
        </section>
    }
}
