use leptos::*;
use leptos_icons::Icon;
use leptos_router::use_navigate;

use crate::api::client_side_api::ClientSideApiClient;
use crate::component::{FullScreenSpinnerGray, Navbar, StarRating};
use crate::domain::{DomainHotelInfoCriteria, DomainHotelSearchCriteria, DomainRoomGuest};
use crate::log;
use crate::page::InputGroupContainer;
use crate::view_state_layer::ui_hotel_details::HotelDetailsUIState;
use crate::view_state_layer::ui_search_state::UISearchCtx;
use crate::view_state_layer::view_state::HotelInfoCtx;

#[derive(Clone)]
struct Amenity {
    icon: icondata::Icon,
    text: String,
}

fn convert_to_amenities(amenities: Vec<String>) -> Vec<Amenity> {
    let icon_mappings = [
        ("wifi", icondata::IoWifi),
        ("parking", icondata::LuParkingCircle),
        ("pool", icondata::BiSwimRegular),
        ("swim", icondata::BiSwimRegular),
        ("spa", icondata::IoWifi), // Fallback since custom icons not available
        ("gym", icondata::IoWifi),
        ("terrace", icondata::IoWifi),
        ("roomservice", icondata::IoWifi),
        ("pet", icondata::IoWifi),
        ("laundry", icondata::IoWifi),
        ("bar", icondata::IoWifi),
        ("pub", icondata::IoWifi),
        ("beach", icondata::BsUmbrella),
        ("umbrella", icondata::BsUmbrella),
        ("family", icondata::RiHomeSmile2BuildingsLine),
        ("room", icondata::RiHomeSmile2BuildingsLine),
    ];

    amenities
        .into_iter()
        .take(8)
        .map(|text| {
            let lower_text = text.to_lowercase();
            let icon = icon_mappings
                .iter()
                .find(|(key, _)| lower_text.contains(*key))
                .map(|(_, icon)| *icon)
                .unwrap_or(icondata::IoWifi);

            let display_text = if text.len() > 10 {
                let mut s = text[..10].to_string();
                s.push('…');
                s
            } else {
                text
            };

            Amenity {
                icon,
                text: display_text,
            }
        })
        .collect()
}

fn clip_to_30_words(text: &str) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.len() <= 30 {
        text.to_string()
    } else {
        let clipped = words[..30].join(" ");
        format!("{}...", clipped)
    }
}

#[component]
pub fn HotelDetailsV1Page() -> impl IntoView {
    let hotel_details_state: HotelDetailsUIState = expect_context();
    let hotel_info_ctx: HotelInfoCtx = expect_context();
    let ui_search_ctx: UISearchCtx = expect_context();

    // Create action for fetching hotel details
    let fetch_hotel_details = create_action(move |_: &()| {
        let client = ClientSideApiClient::new();

        async move {
            // Set loading state
            HotelDetailsUIState::set_loading(true);
            HotelDetailsUIState::set_error(None);

            // Get hotel code from context
            let hotel_code = HotelInfoCtx::get_hotel_code_untracked();
            if hotel_code.is_empty() {
                HotelDetailsUIState::set_error(Some("No hotel selected".to_string()));
                HotelDetailsUIState::set_loading(false);
                return;
            }

            // Create search criteria from UI context
            let destination = ui_search_ctx.destination.get_untracked();
            let date_range = ui_search_ctx.date_range.get_untracked();
            let guests = ui_search_ctx.guests.get_untracked();

            if destination.is_none() {
                HotelDetailsUIState::set_error(Some("Search criteria not available".to_string()));
                HotelDetailsUIState::set_loading(false);
                return;
            }

            let destination = destination.unwrap();

            // Create room guests
            let room_guests = vec![DomainRoomGuest {
                no_of_adults: guests.adults.get_untracked(),
                no_of_children: guests.children.get_untracked(),
                children_ages: if guests.children.get_untracked() > 0 {
                    Some(
                        guests
                            .children_ages
                            .get_untracked()
                            .into_iter()
                            .map(|age| age.to_string())
                            .collect(),
                    )
                } else {
                    None
                },
            }];

            // Create search criteria
            let search_criteria = DomainHotelSearchCriteria {
                destination_city_id: destination.city_id.parse().unwrap_or(0),
                destination_city_name: destination.city.clone(),
                destination_country_code: destination.country_code.clone(),
                destination_country_name: destination.country_name.clone(),
                check_in_date: (date_range.start.0, date_range.start.1, date_range.start.2),
                check_out_date: (date_range.end.0, date_range.end.1, date_range.end.2),
                no_of_nights: date_range.no_of_nights(),
                no_of_rooms: guests.rooms.get_untracked(),
                room_guests,
                guest_nationality: "US".to_string(), // Default for now
            };

            // Create hotel info criteria
            let criteria = DomainHotelInfoCriteria {
                token: hotel_code.clone(),
                hotel_ids: vec![hotel_code],
                search_criteria,
            };

            // Call API
            match client.get_hotel_info(criteria).await {
                Some(details) => {
                    log!("Hotel details loaded successfully: {}", details.hotel_name);
                    HotelDetailsUIState::set_hotel_details(Some(details));
                }
                None => {
                    HotelDetailsUIState::set_error(Some(
                        "Failed to load hotel details".to_string(),
                    ));
                }
            }

            HotelDetailsUIState::set_loading(false);
        }
    });

    // Trigger API call on component mount
    create_effect(move |_| {
        let hotel_code = hotel_info_ctx.hotel_code.get();
        if !hotel_code.is_empty() {
            log!("Fetching hotel details for: {}", hotel_code);
            fetch_hotel_details.dispatch(());
        }
    });

    let loaded = move || hotel_details_state.hotel_details.get().is_some();
    let is_loading = move || hotel_details_state.loading.get();
    let error_message = move || hotel_details_state.error.get();

    let hotel_name_signal = move || {
        if let Some(hotel_details) = hotel_details_state.hotel_details.get() {
            hotel_details.hotel_name
        } else {
            "".to_string()
        }
    };

    let star_rating_signal = move || {
        if let Some(hotel_details) = hotel_details_state.hotel_details.get() {
            hotel_details.star_rating
        } else {
            0
        }
    };

    let description_signal = move || {
        if let Some(hotel_details) = hotel_details_state.hotel_details.get() {
            hotel_details.description
        } else {
            "".to_string()
        }
    };

    let address_signal = move || {
        if let Some(hotel_details) = hotel_details_state.hotel_details.get() {
            hotel_details.address
        } else {
            "".to_string()
        }
    };

    let amenities_signal = move || {
        if let Some(hotel_details) = hotel_details_state.hotel_details.get() {
            convert_to_amenities(hotel_details.amenities)
        } else {
            vec![]
        }
    };

    let images_signal = move || {
        if let Some(hotel_details) = hotel_details_state.hotel_details.get() {
            let mut images = hotel_details.images;
            if images.len() < 6 {
                let repeat_count = 6 - images.len();
                let repeated_images = images.clone();
                images.extend(repeated_images.into_iter().take(repeat_count));
            }
            images
        } else {
            vec![]
        }
    };

    view! {
        <section class="relative min-h-screen bg-gray-50">
            <Navbar />
            <div class="flex flex-col items-center mt-6 p-4">
                <InputGroupContainer default_expanded=false allow_outside_click_collapse=true />
            </div>

            <Show
                when=move || !is_loading()
                fallback=FullScreenSpinnerGray
            >
                <Show
                    when=move || error_message().is_none()
                    fallback=move || view! {
                        <div class="w-full max-w-4xl mx-auto py-4 px-2 md:py-8 md:px-0">
                            <div class="bg-white rounded-xl shadow-md p-6">
                                <div class="text-xl font-semibold text-red-600 mb-2">Error</div>
                                <div class="text-gray-700">
                                    {error_message().unwrap_or_else(|| "Unknown error occurred".to_string())}
                                </div>
                            </div>
                        </div>
                    }
                >
                    <Show when=loaded fallback=|| view! { <div>No hotel data available</div> }>
                        <div class="w-full max-w-4xl mx-auto py-4 px-2 md:py-8 md:px-0">
                            <div class="flex flex-col">
                                <StarRating rating=move || star_rating_signal() as u8 />
                                <div class="text-2xl md:text-3xl font-semibold">{hotel_name_signal}</div>
                            </div>

                            <div class="mt-4 md:mt-6">
                                <HotelImages />
                            </div>

                            <div class="flex flex-col md:flex-row mt-6 md:mt-8 md:space-x-4">
                                <div class="w-full md:w-3/5 flex flex-col space-y-6">
                                    <div class="bg-white rounded-xl shadow-md p-6 mb-2">
                                        <div class="text-xl mb-2 font-semibold">About</div>
                                        <div class="mb-2 text-gray-700">{move || clip_to_30_words(&description_signal())}</div>
                                    </div>

                                    <div class="bg-white rounded-xl shadow-md p-6 mb-2">
                                        <div class="text-xl mb-2 font-semibold">Address</div>
                                        <div class="text-gray-700">{address_signal}</div>
                                    </div>

                                    <div class="bg-white rounded-xl shadow-md p-6 mb-2">
                                        <div class="text-xl mb-4 font-semibold">Amenities</div>
                                        <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-4">
                                            <For
                                                each=amenities_signal
                                                key=|amenity| amenity.text.clone()
                                                let:amenity
                                            >
                                                <AmenitiesIconText icon=amenity.icon text=amenity.text />
                                            </For>
                                        </div>
                                    </div>
                                </div>

                                <div class="w-full md:w-2/5 mt-8 md:mt-0 flex flex-col">
                                    <div class="bg-white rounded-xl shadow-md p-6">
                                        <div class="text-xl mb-4 font-semibold">Hotel Information</div>
                                        <div class="space-y-2">
                                            <div class="text-sm text-gray-600">Check-in: {move || {
                                                if let Some(hotel_details) = hotel_details_state.hotel_details.get() {
                                                    hotel_details.checkin
                                                } else {
                                                    "N/A".to_string()
                                                }
                                            }}</div>
                                            <div class="text-sm text-gray-600">Check-out: {move || {
                                                if let Some(hotel_details) = hotel_details_state.hotel_details.get() {
                                                    hotel_details.checkout
                                                } else {
                                                    "N/A".to_string()
                                                }
                                            }}</div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </Show>
                </Show>
            </Show>
        </section>
    }
}

#[component]
pub fn HotelImages() -> impl IntoView {
    let hotel_details_state: HotelDetailsUIState = expect_context();

    let images_signal = move || {
        if let Some(hotel_details) = hotel_details_state.hotel_details.get() {
            let mut images = hotel_details.images;
            if images.len() < 6 {
                let repeat_count = 6 - images.len();
                let repeated_images = images.clone();
                images.extend(repeated_images.into_iter().take(repeat_count));
            }
            images
        } else {
            vec![]
        }
    };

    move || {
        if images_signal().is_empty() {
            view! { <div class="text-gray-500 text-center py-8">No images available</div> }
        } else {
            view! {
                <div>
                    <div class="block sm:hidden">
                        <img
                            src=move || images_signal()[0].clone()
                            alt="Hotel"
                            class="w-full h-64 rounded-xl object-cover"
                        />
                    </div>
                    <div class="hidden sm:flex flex-col space-y-3">
                        <div class="flex flex-col sm:flex-row space-y-3 sm:space-y-0 sm:space-x-3">
                            <img
                                src=move || images_signal()[0].clone()
                                alt="Hotel"
                                class="w-full sm:w-3/5 h-64 sm:h-96 rounded-xl object-cover"
                            />
                            <div class="flex flex-row sm:flex-col space-x-3 sm:space-x-0 sm:space-y-3 w-full sm:w-2/5">
                                <img
                                    src=move || images_signal().get(1).cloned().unwrap_or_else(|| images_signal()[0].clone())
                                    alt="Hotel"
                                    class="w-1/2 sm:w-full h-32 sm:h-[186px] rounded-xl object-cover sm:object-fill"
                                />
                                <img
                                    src=move || images_signal().get(2).cloned().unwrap_or_else(|| images_signal()[0].clone())
                                    alt="Hotel"
                                    class="w-1/2 sm:w-full h-32 sm:h-[186px] rounded-xl object-cover sm:object-fill"
                                />
                            </div>
                        </div>
                        <div class="flex justify-between space-x-3">
                            <img
                                src=move || images_signal().get(3).cloned().unwrap_or_else(|| images_signal()[0].clone())
                                alt="Hotel"
                                class="w-72 h-48 rounded-xl object-cover"
                            />
                            <img
                                src=move || images_signal().get(4).cloned().unwrap_or_else(|| images_signal()[0].clone())
                                alt="Hotel"
                                class="w-72 h-48 rounded-xl object-cover"
                            />
                            <div class="relative w-72 h-48 rounded-xl">
                                <img
                                    src=move || images_signal().get(5).cloned().unwrap_or_else(|| images_signal()[0].clone())
                                    alt="Hotel"
                                    class="object-cover h-full w-full rounded-xl"
                                />
                            </div>
                        </div>
                    </div>
                </div>
            }
        }
    }
}

#[component]
pub fn AmenitiesIconText(icon: icondata::Icon, #[prop(into)] text: String) -> impl IntoView {
    view! {
        <div class="flex items-center">
            <Icon class="inline text-xl text-gray-600" icon=icon />
            <span class="inline ml-2 text-sm text-gray-700">{text}</span>
        </div>
    }
}
