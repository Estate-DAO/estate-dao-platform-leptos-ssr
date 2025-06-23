use leptos::*;
use leptos_icons::Icon;
use leptos_router::use_navigate;

use crate::api::client_side_api::ClientSideApiClient;
use crate::app::AppRoutes;
use crate::component::{loading_button::LoadingButton, FullScreenSpinnerGray, Navbar, StarRating};
use crate::domain::{DomainHotelInfoCriteria, DomainHotelSearchCriteria, DomainRoomGuest};
use crate::log;
use crate::page::InputGroupContainer;
use crate::view_state_layer::ui_block_room::{BlockRoomUIState, RoomSelectionSummary};
use crate::view_state_layer::ui_hotel_details::HotelDetailsUIState;
use crate::view_state_layer::ui_search_state::UISearchCtx;
use crate::view_state_layer::view_state::HotelInfoCtx;

// <!-- Configuration constant for number of skeleton rooms to display during loading -->
const NUMBER_OF_ROOMS: usize = 5;

#[derive(Clone)]
struct Amenity {
    icon: icondata::Icon,
    text: String,
}

/// **Phase 4 UI Enhancement: Skeleton Loading Component**
///
/// **Purpose**: Provides elegant loading states while hotel rates are being fetched
/// **Design**: Animated skeleton mimicking the actual room selection layout
/// **UX Benefit**: Users see structured loading instead of blank space
/// **Original Reference**: Enhanced based on patterns from hotel_details.rs
#[component]
pub fn RoomSelectionSkeleton() -> impl IntoView {
    view! {
        <div class="space-y-3 animate-pulse">
            // <!-- Skeleton for room type title -->
            <div class="h-6 bg-gray-200 rounded w-32"></div>

            // <!-- Skeleton for rooms - uses NUMBER_OF_ROOMS constant -->
            <For
                each=|| (0..NUMBER_OF_ROOMS)
                key=|i| *i
                let:_
            >
                <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between border border-gray-100 rounded-lg p-4 space-y-3 sm:space-y-0">
                    // <!-- Room name and price skeleton -->
                    <div class="flex-1 space-y-2">
                        <div class="h-4 bg-gray-200 rounded w-3/4"></div>
                        <div class="h-3 bg-gray-100 rounded w-1/2"></div>
                    </div>

                    // <!-- Counter skeleton -->
                    <div class="flex items-center space-x-3 sm:flex-shrink-0">
                        <div class="w-8 h-8 bg-gray-200 rounded"></div>
                        <div class="w-6 h-4 bg-gray-200 rounded"></div>
                        <div class="w-8 h-8 bg-gray-200 rounded"></div>
                    </div>
                </div>
            </For>

            // <!-- Pricing breakdown skeleton -->
            <div class="mt-6 space-y-3 pt-4 border-t border-gray-200">
                <div class="flex justify-between">
                    <div class="h-4 bg-gray-200 rounded w-24"></div>
                    <div class="h-4 bg-gray-200 rounded w-16"></div>
                </div>
                <div class="flex justify-between font-semibold border-t pt-2">
                    <div class="h-5 bg-gray-200 rounded w-20"></div>
                    <div class="h-5 bg-gray-200 rounded w-20"></div>
                </div>
            </div>

            // <!-- Button skeleton -->
            <div class="mt-6">
                <div class="w-full h-12 bg-gray-200 rounded-xl"></div>
            </div>
        </div>
    }
}

#[allow(clippy::type_complexity)]
fn get_consolidated_hotel_facility_icon_mappings() -> Vec<(&'static str, icondata::Icon)> {
    vec![
        // Fitness & Wellbeing
        ("Fitness facilities", icondata::FaDumbbellSolid),
        ("Hand sanitizer", icondata::TbHandSanitizer),
        ("Guest health check", icondata::FaUserDoctorSolid),
        ("Face masks available", icondata::FaMaskFaceSolid),
        // Room & Comfort
        ("Non-smoking", icondata::TbSmokingNo),
        ("Air conditioning", icondata::TbAirConditioning),
        ("Family rooms", icondata::LuBaby),
        ("Pets allowed", icondata::FaPawSolid),
        // Accessibility
        ("Elevator", icondata::FaElevatorSolid),
        ("Accessible facilities", icondata::LuAccessibility),
        // Services & Operations
        ("Luggage storage", icondata::TbLuggage),
        ("Fax and photocopying", icondata::FaFaxSolid),
        ("Private check-in/out", icondata::FaUserSecretSolid),
        ("Invoice provided", icondata::FaFileInvoiceSolid),
        // Security & Safety
        ("CCTV", icondata::BiCctvSolid),
        // Connectivity & Digital
        ("Contactless check-in/out", icondata::SiContactlesspayment),
        ("Cashless payment", icondata::FaCreditCardSolid),
        ("Shared item removal", icondata::BsEraser),
        ("Common area TV", icondata::FaTvSolid),
        // Food & Beverage
        ("Sanitized tableware", icondata::BsTabletFill),
        ("Dining area distancing", icondata::FaUtensilsSolid),
        // Parking
        ("Parking", icondata::LuParkingCircle),
    ]
}

fn convert_to_amenities(amenities: Vec<String>) -> Vec<Amenity> {
    let icon_mappings = get_consolidated_hotel_facility_icon_mappings();

    amenities
        .into_iter()
        // <!-- Filter out generic facility names like "Facility 256" -->
        .filter(|text| {
            let lower_text = text.to_lowercase();
            // Skip if it matches pattern "facility" followed by a number or is just "facility" alone
            !lower_text.starts_with("facility ") && lower_text != "facility"
        })
        .take(8)
        .map(|text| {
            let lower_text = text.to_lowercase();
            let icon = icon_mappings
                .iter()
                .find(|(key, _)| lower_text.contains(*key))
                .map(|(_, icon)| *icon)
                .unwrap_or(icondata::IoWifi);

            // todo: truncate if needed later.
            // let display_text = if text.len() > 10 {
            //     let mut s = text[..10].to_string();
            //     s.push('…');
            //     s
            // } else {
            //     text
            // };

            Amenity { icon, text }
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

    // Create resource to fetch hotel details when page loads
    // Following the pattern from block_room_v1.rs prebook_resource
    let hotel_details_resource = create_resource(
        move || {
            // Wait for essential data to be ready before calling API
            let hotel_code = hotel_info_ctx.hotel_code.get();
            let destination = ui_search_ctx.destination.get();
            let date_range = ui_search_ctx.date_range.get();
            let has_hotel_code = !hotel_code.is_empty();
            let has_destination = destination.is_some();
            let has_valid_dates = date_range.start != (0, 0, 0) && date_range.end != (0, 0, 0);

            // Return true when ready to call API
            has_hotel_code && has_destination && has_valid_dates
        },
        move |is_ready| {
            let guests_for_details = ui_search_ctx.guests.clone();
            async move {
                if !is_ready {
                    log!("Hotel details resource: Not ready yet, waiting for data...");
                    return None;
                }

                log!("Hotel details resource: Page data ready, fetching hotel details...");
                HotelDetailsUIState::set_loading(true);
                HotelDetailsUIState::set_error(None);

                let client = ClientSideApiClient::new();
                let guests_clone = guests_for_details.clone();

                // Get hotel code from context
                let hotel_code = HotelInfoCtx::get_hotel_code_untracked();
                if hotel_code.is_empty() {
                    HotelDetailsUIState::set_error(Some("No hotel selected".to_string()));
                    HotelDetailsUIState::set_loading(false);
                    return None;
                }

                // Create search criteria from UI context
                let destination = ui_search_ctx.destination.get_untracked();
                let date_range = ui_search_ctx.date_range.get_untracked();
                let guests = &guests_clone;

                if destination.is_none() {
                    HotelDetailsUIState::set_error(Some(
                        "Search criteria not available".to_string(),
                    ));
                    HotelDetailsUIState::set_loading(false);
                    return None;
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
                        log!("Hotel details resource: Success - {}", details.hotel_name);
                        HotelDetailsUIState::set_hotel_details(Some(details.clone()));
                        HotelDetailsUIState::set_loading(false);
                        Some(details)
                    }
                    None => {
                        log!("Hotel details resource: Failed to load hotel details");
                        HotelDetailsUIState::set_error(Some(
                            "Failed to load hotel details".to_string(),
                        ));
                        HotelDetailsUIState::set_loading(false);
                        None
                    }
                }
            }
        },
    );

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
            // <!-- Use hotel_facilities instead of amenities for LiteAPI compatibility -->
            // <!-- LiteAPI maps facility_ids to hotel_facilities, while amenities remains empty -->
            convert_to_amenities(hotel_details.hotel_facilities)
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

            // <!-- Use resource pattern like prebook_resource in block_room_v1.rs -->
            // <!-- The resource automatically triggers data loading when dependencies change -->
            <Suspense fallback=move || view! { <></> }>
                {move || {
                // Trigger the resource loading but don't render anything
                let _ = hotel_details_resource.get();
                view! { <></> }
            }}
            </Suspense>

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
                    <Show when=loaded fallback=|| view! {
                        <div class="w-full max-w-4xl mx-auto py-4 px-2 md:py-8 md:px-0">
                            <div class="bg-white rounded-xl shadow-md p-6">
                                <div class="text-xl font-semibold text-gray-600 text-center">
                                    No hotel data available
                                </div>
                            </div>
                        </div>
                    }>
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
                                        <div class="mb-2 text-gray-700" inner_html=move || description_signal()></div>
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

                                <div class="w-full md:w-2/5 mt-8 md:mt-0 flex flex-col space-y-4">
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

                                    // <!-- Pricing and Booking Section -->
                                    <div class="bg-white rounded-xl shadow-md">
                                        <PricingBookNowV1 />
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
        // todo: for now, we are showing a bullet only
            <Icon class="inline text-xl text-gray-600" icon=icondata::BsDot />
            <span class="inline ml-2 text-sm text-gray-700">{text}</span>
        </div>
    }
}

/// **Phase 2 Core Component: Room Selection Counter**
///
/// **Purpose**: Individual room type selector with increment/decrement controls
/// **Props**: room_type (display name), room_price (per night), room_unique_id (for state tracking)
/// **State Integration**: Uses HotelDetailsUIState for reactive room count management
/// **UI Design**: Adapted from original hotel_details.rs NumberCounterWrapperV2 CSS patterns
/// **Mobile Optimized**: Responsive layout with proper touch targets
#[component]
pub fn RoomCounterV1(room_type: String, room_price: f64, room_unique_id: String) -> impl IntoView {
    let hotel_details_state: HotelDetailsUIState = expect_context();

    // **Reactive State Management**:
    // Tracks room count for this specific room type using room_unique_id as key
    // Uses create_memo for efficient reactivity - only updates when selection changes
    let room_count = create_memo({
        let room_key = room_unique_id.clone();
        move |_| {
            hotel_details_state
                .selected_rooms
                .get()
                .get(&room_key)
                .copied()
                .unwrap_or(0)
        }
    });

    let is_at_minimum = move || room_count() == 0;

    // <!-- Room validation signals -->
    let is_at_maximum = create_memo(move |_| {
        // Disable increment if we're at the global room limit
        HotelDetailsUIState::is_at_room_selection_limit()
    });

    // Clone for closures
    let room_key_inc = room_unique_id.clone();
    let room_key_dec = room_unique_id.clone();

    let increment = move |_| {
        HotelDetailsUIState::increment_room_counter(room_key_inc.clone());
    };

    let decrement = move |_| {
        if room_count() > 0 {
            HotelDetailsUIState::decrement_room_counter(room_key_dec.clone());
        }
    };
    // / **UI Layout**: Adapted from original hotel_details.rs NumberCounterWrapperV2
    // / **CSS Strategy**: Robust text wrapping with flex-1 min-w-0, flex-shrink-0 for counter
    // / **Responsive**: Works on mobile and desktop with proper touch targets

    view! {
        <div class="flex flex-row items-start justify-between border-b border-gray-300 py-2">
            // <!-- Robust wrap: flex-1 min-w-0 for text, flex-shrink-0 for counter, items-start for top align -->
            <p class="w-0 flex-1 min-w-0 font-medium text-sm md:text-base break-words whitespace-normal">
                {format!("{} - ${:.2}/night", room_type, room_price)}
            </p>
            <div class="flex-shrink-0">
                // <!-- Original CSS styling with my functional implementation -->
                <div class="flex items-center justify-between mt-2 md:mt-4">
                    <div class="flex items-center space-x-1">
                        <button
                            class="ps-2 py-1 text-2xl disabled:opacity-50 disabled:cursor-not-allowed"
                            disabled=is_at_minimum
                            on:click=decrement
                        >
                            {"\u{2003}\u{2003}\u{2003}\u{2003}-"}
                        </button>
                        <p class="text-center w-6">{move || room_count()}</p>
                        <button
                            class="py-1 text-2xl disabled:opacity-50 disabled:cursor-not-allowed"
                            disabled=is_at_maximum
                            on:click=increment
                        >
                            "+"
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}

/// **Phase 2 Core Component: Pricing Calculations & Booking Action**
///
/// **Purpose**: Shows price breakdown and handles "Book Now" navigation
/// **Calculations**: Room price × nights × quantity = total (reactive)
/// **Navigation**: Integrates with Block Room flow via AppRoutes::BlockRoom
/// **State Dependencies**: UISearchCtx (dates), HotelDetailsUIState (selections)
/// **UX Features**: Loading states, disabled states, validation
#[component]
pub fn PricingBreakdownV1() -> impl IntoView {
    let hotel_details_state: HotelDetailsUIState = expect_context();
    let ui_search_ctx: UISearchCtx = expect_context();
    let navigate = use_navigate();

    // Create loading state for the Book Now button
    let booking_loading = create_rw_signal(false);

    // Reactive signals for pricing calculations
    let date_range = move || ui_search_ctx.date_range.get();

    // Calculate number of nights
    let number_of_nights = move || date_range().no_of_nights();

    // Calculate total selected rooms
    let total_selected_rooms = move || HotelDetailsUIState::total_selected_rooms();

    // Calculate subtotal using helper method
    let subtotal = move || HotelDetailsUIState::calculate_subtotal_for_nights();

    // Check if any rooms are selected
    let has_rooms_selected = move || total_selected_rooms() > 0;

    // Create memo for button disabled state
    let button_disabled = create_memo(move |_| !has_rooms_selected() || booking_loading.get());

    // Create action for Book Now button
    let book_now_action = create_action(move |_: &()| {
        let navigate = navigate.clone();
        async move {
            booking_loading.set(true);

            // <!-- Pass room selection data to BlockRoomUIState -->
            // Get selected rooms with quantities
            let selected_rooms = HotelDetailsUIState::get_selected_rooms();
            let available_rooms = HotelDetailsUIState::get_available_rooms();
            let hotel_details = HotelDetailsUIState::get_hotel_details();

            // Create room selection summary for block room page
            let mut room_selection_summary = Vec::new();
            let mut selected_rooms_with_data = std::collections::HashMap::new();

            for (room_id, quantity) in selected_rooms.iter() {
                if *quantity > 0 {
                    // Find the corresponding room data
                    if let Some(room_data) = available_rooms
                        .iter()
                        .find(|r| &r.room_unique_id == room_id)
                    {
                        // Create summary entry
                        let summary = RoomSelectionSummary {
                            room_id: room_id.clone(),
                            room_name: room_data.room_name.clone(),
                            quantity: *quantity,
                            price_per_night: HotelDetailsUIState::total_room_price()
                                / HotelDetailsUIState::total_selected_rooms() as f64,
                            room_data: room_data.clone(),
                        };
                        room_selection_summary.push(summary);
                        selected_rooms_with_data
                            .insert(room_id.clone(), (*quantity, room_data.clone()));
                    }
                }
            }

            // Pass data to BlockRoomUIState
            BlockRoomUIState::set_selected_rooms(selected_rooms_with_data);
            BlockRoomUIState::set_hotel_context(hotel_details.clone());
            BlockRoomUIState::set_room_selection_summary(room_selection_summary);

            // Also populate HotelInfoCtx for backward compatibility with block room page
            if let Some(ref hotel_info) = hotel_details {
                use crate::view_state_layer::view_state::HotelInfoCtx;
                let hotel_image = hotel_info.images.first().cloned().unwrap_or_default();
                HotelInfoCtx::set_selected_hotel_details(
                    hotel_info.hotel_code.clone(),
                    hotel_info.hotel_name.clone(),
                    hotel_image,
                    hotel_info.address.clone(),
                );
                log!(
                    "Populated HotelInfoCtx from hotel details: {}, {}",
                    hotel_info.hotel_name,
                    hotel_info.address
                );
            }

            // Navigate to block room page
            let block_room_url = AppRoutes::BlockRoom.to_string();
            navigate(block_room_url, Default::default());

            booking_loading.set(false);
        }
    });

    let on_book_now = move |_| {
        if has_rooms_selected() {
            book_now_action.dispatch(());
        }
    };

    view! {
        <div class="bg-white rounded-xl shadow-md p-6">
            <div class="text-xl mb-4 font-semibold">Price Breakdown</div>

            <Show
                when=has_rooms_selected
                fallback=|| view! {
                    <div class="text-gray-500 text-center py-4">
                        "Select rooms to see pricing"
                    </div>
                }
            >
                <div class="space-y-3">
                    // <!-- Price calculation display - breakdown per room type -->
                    // <For
                    //     each=move || HotelDetailsUIState::get_selected_rooms_with_data()
                    //     key=|(room_option, _)| room_option.room_data.room_unique_id.clone()
                    //     let:room_data
                    // >
                    //     {
                    //         let (room_option, quantity) = room_data;
                    //         let room_name = room_option.room_data.room_name.clone();
                    //         let room_price = room_option.price.room_price;
                    //
                    //         view! {
                    //             <div class="flex justify-between items-center text-gray-700">
                    //                 <span class="text-sm">
                    //                     {move || {
                    //                         let nights = number_of_nights();
                    //                         HotelDetailsUIState::format_room_breakdown_text(&room_name, quantity, nights)
                    //                     }}
                    //                 </span>
                    //                 <span class="font-medium text-sm">
                    //                     {move || {
                    //                         let nights = number_of_nights();
                    //                         let line_total = HotelDetailsUIState::calculate_room_line_total(&room_option, quantity, nights);
                    //                         format!("${:.2}", line_total)
                    //                     }}
                    //                 </span>
                    //             </div>
                    //         }
                    //     }
                    // </For>

                    // <!-- Divider -->
                    // <div class="border-t border-gray-200 my-3"></div>

                    // <!-- Total -->
                    <div class="flex justify-between items-center text-lg font-semibold">
                        <span>"Total"</span>
                        <span class="text-blue-600">
                            {move || format!("${:.2}", subtotal())}
                        </span>
                    </div>

                    // <!-- Crypto payment info -->
                    <div class="mt-4 p-3 bg-blue-50 rounded-lg">
                        <div class="flex items-center text-sm text-blue-700">
                            <svg class="w-4 h-4 mr-2" fill="currentColor" viewBox="0 0 20 20">
                                <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd"></path>
                            </svg>
                            "Cryptocurrency payments accepted!"
                        </div>
                    </div>

                    // <!-- Book Now Button -->
                    <div class="mt-6">
                        <LoadingButton
                            is_loading=booking_loading.into()
                            loading_text="Processing..."
                            on_click=on_book_now
                            class="w-full bg-blue-600 hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed text-white font-semibold py-3 px-6 rounded-xl transition-colors duration-200"
                            disabled=button_disabled
                        >
                            "Book Now"
                        </LoadingButton>
                    </div>
                </div>
            </Show>
        </div>
    }
}

/// **Phase 2 Main Component: Complete Pricing & Booking Interface**
// /
// / **Purpose**: Main pricing component integrating all room selection functionality
// / **Integration**: Combines price display, search context, room selection, and booking action
// / **API Integration**: Uses real hotel rates data from get_hotel_rates() with fallback to mock
// / **State Management**: Reactive pricing calculations with HotelDetailsUIState
// / **UX Features**: Loading skeletons, error handling, mobile optimization
// / **Navigation**: Seamless flow to Block Room page for booking completion
#[component]
pub fn PricingBookNowV1() -> impl IntoView {
    let hotel_details_state: HotelDetailsUIState = expect_context();
    let ui_search_ctx: UISearchCtx = expect_context();

    // **Smart Room Data Management**:
    // **Primary**: Uses real room data from get_hotel_rates() API
    // **Fallback**: Mock data when API is loading or unavailable
    // **Price Source**: Extracts room_price from hotel_details.all_rooms
    // **Data Structure**: (room_name, price_per_night, room_unique_id)
    let available_rooms = move || {
        if let Some(hotel_details) = hotel_details_state.hotel_details.get() {
            // Use real room data from all_rooms with individual pricing
            hotel_details
                .all_rooms
                .into_iter()
                .take(5) // <!-- Limit to maximum 5 rooms for display -->
                .map(|room_option| {
                    (
                        room_option.room_data.room_name.clone(),
                        room_option.price.room_price,
                        room_option.room_data.room_unique_id.clone(),
                    )
                })
                .collect::<Vec<_>>()
        } else {
            // No fallback rooms - show empty list when no hotel data available
            vec![]
        }
    };

    let is_rooms_loading = move || hotel_details_state.loading.get();

    // Reactive signals for display
    let total_price = create_memo(move |_| HotelDetailsUIState::total_room_price());
    let has_price = create_memo(move |_| total_price.get() > 0.0);
    let date_range = move || ui_search_ctx.date_range.get();
    let guests = ui_search_ctx.guests;

    // Format dates for display
    let check_in_display = move || {
        let range = date_range();
        if range.start == (0, 0, 0) {
            "Check-in".to_string()
        } else {
            format!(
                "{:04}-{:02}-{:02}",
                range.start.0, range.start.1, range.start.2
            )
        }
    };

    let check_out_display = move || {
        let range = date_range();
        if range.end == (0, 0, 0) {
            "Check-out".to_string()
        } else {
            format!("{:04}-{:02}-{:02}", range.end.0, range.end.1, range.end.2)
        }
    };

    let adults_count = move || guests.adults.get();
    let rooms_count = move || guests.rooms.get();

    view! {
        <div class="flex flex-col space-y-4 shadow-lg rounded-xl p-4 md:p-8 bg-white">
            // <!-- Total Price Display (if price > 0) -->
            // <Show when=has_price>
            //     <div class="bg-blue-50 rounded-lg p-4 mb-4">
            //         <div class="text-2xl font-bold text-blue-800 text-center">
            //             {move || format!("Total: ${:.2}", total_price.get())}
            //         </div>
            //     </div>
            // </Show>

            // <!-- Search Context Summary -->
            <div class="bg-gray-50 rounded-lg p-4 space-y-3">

                // <!-- Dates -->
                <div class="flex items-center space-x-2">
                    <Icon class="text-blue-600 text-lg" icon=icondata::BiCalendarRegular />
                    <span class="text-gray-700">
                        {check_in_display} " to " {check_out_display}
                    </span>
                </div>

                // <!-- Adults -->
                <div class="flex items-center space-x-2">
                    <Icon class="text-blue-600 text-lg" icon=icondata::BiUserRegular />
                    <span class="text-gray-700">
                        {adults_count} {move || if adults_count() == 1 { "adult" } else { "adults" }}
                    </span>
                </div>

                // <!-- Rooms -->
                <div class="flex items-center space-x-2">
                    <Icon class="text-blue-600 text-lg" icon=icondata::RiHomeSmile2BuildingsLine />
                    <span class="text-gray-700">
                        {rooms_count} {move || if rooms_count() == 1 { "room" } else { "rooms" }}
                    </span>
                </div>
            </div>

            // <!-- Room Selection Interface -->
            <div class="space-y-4">
                <div class="text-lg font-semibold text-gray-800">Select room type:</div>

                // <!-- Room Type Listing -->
                <div class="space-y-3">
                    <Show
                        when=move || !is_rooms_loading()
                        fallback=|| view! { <RoomSelectionSkeleton /> }
                    >
                        <For
                            each=available_rooms
                            key=|(_, _, room_id)| room_id.clone()
                            let:room_data
                        >
                            <RoomCounterV1
                                room_type=room_data.0.to_string()
                                room_price=room_data.1
                                room_unique_id=room_data.2.to_string()
                            />
                        </For>
                    </Show>
                </div>
            </div>

            // <!-- Pricing Breakdown Section -->
            <div class="mt-6">
                <PricingBreakdownV1 />
            </div>
        </div>
    }
}
