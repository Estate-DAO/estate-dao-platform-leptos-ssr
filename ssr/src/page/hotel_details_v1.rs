use leptos::*;
use leptos_icons::Icon;
use leptos_router::{use_navigate, use_query_map, NavigateOptions};

use crate::api::auth::auth_state::AuthStateSignal;
use crate::api::client_side_api::ClientSideApiClient;
use crate::api::consts::ENFORCE_SINGLE_ROOM_TYPE_BOOKING;
use crate::app::AppRoutes;
use crate::component::{loading_button::LoadingButton, FullScreenSpinnerGray};
use crate::component::{Footer, ImageLightbox};
use crate::domain::{
    DomainHotelCodeId, DomainHotelInfoCriteria, DomainHotelSearchCriteria, DomainRoomGuest,
    DomainRoomOccupancy, DomainRoomOption, DomainStaticRoom,
};
use crate::log;
use crate::page::{
    add_to_wishlist_action, HotelDetailsParams, HotelListNavbar, InputGroupContainer,
};
use crate::utils::query_params::QueryParamsSync;
use crate::view_state_layer::input_group_state::InputGroupState;
use crate::view_state_layer::ui_block_room::{BlockRoomUIState, RoomSelectionSummary};
use crate::view_state_layer::ui_hotel_details::HotelDetailsUIState;
use crate::view_state_layer::ui_search_state::UISearchCtx;
use crate::view_state_layer::view_state::HotelInfoCtx;
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;

use std::time::Duration;

// <!-- Configuration constant for number of skeleton rooms to display during loading -->
const NUMBER_OF_ROOMS: usize = 5;

#[derive(Clone)]
pub struct Amenity {
    icon: icondata::Icon,
    text: String,
}

#[derive(Clone)]
struct OfferGroup {
    offer_id: String,
    mapped_room_id: Option<u32>,
    rates: Vec<DomainRoomOption>,
    room_names: Vec<String>,
}

#[derive(Clone)]
struct RoomCard {
    mapped_room_id: Option<u32>,
    room_names: Vec<String>,
    card_title: String,
    rates: Vec<DomainRoomOption>,
}

#[derive(Clone, Hash, PartialEq, Eq)]
struct RateRowKey {
    mapped_room_id: Option<u32>,
    occupancy_number: Option<u32>,
    meal_plan: String,
    price_bits: u64,
}

impl RateRowKey {
    fn from_rate(rate: &DomainRoomOption, include_occupancy: bool) -> Self {
        let mapped_room = (rate.mapped_room_id != 0).then_some(rate.mapped_room_id);
        let meal_plan = rate
            .meal_plan
            .clone()
            .unwrap_or_else(|| "Room Only".to_string());
        RateRowKey {
            mapped_room_id: mapped_room,
            occupancy_number: if include_occupancy {
                rate.room_data.occupancy_number
            } else {
                None
            },
            meal_plan,
            price_bits: rate.price.room_price.to_bits(),
        }
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
struct OfferRoomSignature {
    mapped_room_id: u32,
    occupancy_number: Option<u32>,
    meal_plan: String,
    price_bits: u64,
}

#[derive(Default, Clone)]
struct RoomDetailsLookup {
    by_id: HashMap<u32, DomainStaticRoom>,
    by_name: HashMap<String, DomainStaticRoom>,
}

#[derive(Clone, Copy)]
struct GuestReviewSnippet {
    name: &'static str,
    stay_info: &'static str,
    rating: f32,
    title: &'static str,
    body: &'static str,
    tags: &'static [&'static str],
}

const REVIEW_CATEGORY_SCORES: [(&str, f32); 6] = [
    ("Cleanliness", 7.4),
    ("Room Quality", 9.3),
    ("Location", 7.4),
    ("Food & Beverage", 9.3),
    ("Service", 9.3),
    ("Amenities", 9.3),
];

const REVIEW_HIGHLIGHT_TAGS: [&str; 3] = ["Central Location", "Great Breakfast", "Polite Staff"];

const SAMPLE_REVIEWS: [GuestReviewSnippet; 3] = [
    GuestReviewSnippet {
        name: "Raghunathan Sharma",
        stay_info: "2 night stay · Couple",
        rating: 7.1,
        title: "Great hospitality and convenient location",
        body: "Small room but spotless with friendly staff and hot water. Indian restaurant onsite was lovely and breakfast was amazing. Close to most attractions in the city so we could walk everywhere.",
        tags: &["Central Location", "Great Breakfast"],
    },
    GuestReviewSnippet {
        name: "Haider Nair",
        stay_info: "2 night stay · Family",
        rating: 7.0,
        title: "Comfortable base for a short break",
        body: "Room was compact yet very comfortable. Shower was clean with hot water. Team helped with late check-in and arranged cabs. Excellent value for the location.",
        tags: &["Helpful Staff", "Value for Money"],
    },
    GuestReviewSnippet {
        name: "Gorden",
        stay_info: "2 night stay · Solo traveller",
        rating: 8.2,
        title: "Would stay again",
        body: "Super friendly service and solid WiFi for remote work. Easy to get around and plenty of food options around the property. Perfect for a quick work trip.",
        tags: &["Fast WiFi", "Great Service"],
    },
];

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
                .unwrap_or(icondata::IoCheckmark);

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

fn normalized_room_key(value: &str) -> String {
    value
        .to_lowercase()
        .chars()
        .map(|ch| if ch.is_alphanumeric() { ch } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn room_details_lookup_for_state(state: &HotelDetailsUIState) -> RoomDetailsLookup {
    if let Some(details) = state.static_details.get() {
        let mut by_id = HashMap::new();
        let mut by_name = HashMap::new();
        for room in details.rooms.into_iter() {
            let normalized = normalized_room_key(&room.room_name);
            by_name.insert(normalized, room.clone());
            if let Ok(id) = room.room_id.trim().parse::<u32>() {
                by_id.insert(id, room);
            }
        }
        RoomDetailsLookup { by_id, by_name }
    } else {
        RoomDetailsLookup::default()
    }
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
    let query_map = use_query_map();

    // <!-- Query params handling for shareable URLs -->
    // Sync query params with state on page load (URL → State)
    create_effect(move |_| {
        let params = query_map.get();
        if !params.0.is_empty() {
            log!(
                "[HotelDetailsV1Page] Found query params in URL: {:?}",
                params
            );

            // Convert leptos_router params to HashMap
            let params_map: std::collections::HashMap<String, String> = params
                .0
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();

            // Parse params and sync to app state.
            // The logic is now much simpler and completely decoupled from 'place' searching.
            if let Some(hotel_params) = HotelDetailsParams::from_query_params(&params_map) {
                log!(
                    "[HotelDetailsV1Page] Parsed hotel params from URL: {:?}",
                    hotel_params
                );

                // Get placeId before syncing (for async fetch)
                let place_id_to_fetch = hotel_params.place_id.clone();

                hotel_params.sync_to_app_state();

                // Fetch place details asynchronously if placeId is present
                if let Some(place_id) = place_id_to_fetch {
                    spawn_local(async move {
                        log!(
                            "[HotelDetailsV1Page] Fetching place details for placeId: {}",
                            place_id
                        );
                        let api_client = ClientSideApiClient::new();
                        match api_client.get_place_details_by_id(place_id.clone()).await {
                            Ok(place_data) => {
                                // Extract display name from address components
                                // Combine locality + country for consistent display (e.g., "Bali, Indonesia")
                                let locality = place_data
                                    .address_components
                                    .iter()
                                    .find(|c| c.types.contains(&"locality".to_string()))
                                    .or_else(|| {
                                        place_data.address_components.iter().find(|c| {
                                            c.types.contains(
                                                &"administrative_area_level_1".to_string(),
                                            )
                                        })
                                    })
                                    .map(|c| c.long_text.clone());

                                let country = place_data
                                    .address_components
                                    .iter()
                                    .find(|c| c.types.contains(&"country".to_string()))
                                    .map(|c| c.long_text.clone());

                                let display_name = match (locality, country) {
                                    (Some(loc), Some(ctry)) => format!("{}, {}", loc, ctry),
                                    (Some(loc), None) => loc,
                                    (None, Some(ctry)) => ctry,
                                    (None, None) => "Unknown Location".to_string(),
                                };

                                let place = crate::api::client_side_api::Place {
                                    place_id: place_id.clone(),
                                    display_name,
                                    formatted_address: String::new(),
                                };
                                UISearchCtx::set_place(place);
                                log!(
                                    "[HotelDetailsV1Page] Successfully set place context from API"
                                );
                            }
                            Err(e) => {
                                log!("[HotelDetailsV1Page] Failed to fetch place details: {}", e);
                            }
                        }
                    });
                }
            } else {
                log!("[HotelDetailsV1Page] Could not parse hotel params from URL.");
            }
        }
    });

    // <!-- Function to update URL with current search state -->
    // This can be called when navigating to this page from hotel list
    // Preserves placeId from URL even if place context hasn't been set yet
    let update_url_with_current_state = move || {
        if let Some(mut current_params) = HotelDetailsParams::from_current_context() {
            // Preserve placeId from URL if not already in context
            // This handles the race condition where async place fetch hasn't completed
            // Preserve placeId from URL if not already in context
            // This handles the race condition where async place fetch hasn't completed
            let url_params = query_map.get();
            if current_params.place_id.is_none() {
                if let Some(place_id) = url_params.0.get("placeId") {
                    current_params.place_id = Some(place_id.clone());
                }
            }

            // Check if params actually changed to avoid infinite loops
            let new_params = current_params.to_query_params();
            let mut changed = false;

            // Simple check: if sizes differ, or any key-value pair differs
            if url_params.0.len() != new_params.len() {
                changed = true;
            } else {
                for (k, v) in &new_params {
                    if url_params.0.get(k) != Some(v) {
                        changed = true;
                        break;
                    }
                }
            }

            if changed {
                current_params.update_url(); // Now uses individual query params
                log!(
                    "Updated URL with current hotel details state (individual params): {:?}",
                    current_params
                );
            }
        }
    };

    // <!-- Auto-update URL when essential data becomes available -->
    create_effect(move |_| {
        let hotel_code = hotel_info_ctx.hotel_code.get();
        let date_range = ui_search_ctx.date_range.get();

        let has_essential_data =
            !hotel_code.is_empty() && date_range.start != (0, 0, 0) && date_range.end != (0, 0, 0);

        if has_essential_data {
            // Only update URL if query params are empty (to avoid infinite loops)
            let current_params = query_map.get();
            if current_params.0.is_empty() {
                log!("[HotelDetailsV1Page] Auto-update: URL is empty, updating with current state");
                update_url_with_current_state();
            }
        }
    });

    // Effect to update URL when dates or guests change from UI (State → URL)
    create_effect(move |_| {
        // Depend on all relevant signals
        let hotel_code = hotel_info_ctx.hotel_code.get();
        let date_range = ui_search_ctx.date_range.get();
        let adults = ui_search_ctx.guests.adults.get();
        let children = ui_search_ctx.guests.children.get();
        let rooms = ui_search_ctx.guests.rooms.get();

        // Only update URL if we have essential data and URL params are not empty
        // (to avoid updating on initial load)
        let has_essential_data =
            !hotel_code.is_empty() && date_range.start != (0, 0, 0) && date_range.end != (0, 0, 0);

        let current_params = query_map.get();
        let has_url_params = !current_params.0.is_empty();

        if has_essential_data && has_url_params {
            log!(
                "[HotelDetailsV1Page] UI state changed, updating URL: dates={:?}, adults={}, children={}, rooms={}",
                date_range,
                adults,
                children,
                rooms,
            );
            update_url_with_current_state();
        }
    });

    // Create resource to fetch hotel details when page loads
    // Following the pattern from block_room_v1.rs prebook_resource
    // Enhanced to work with query params for shareable URLs
    let static_details_resource = create_resource(
        move || hotel_info_ctx.hotel_code.get(),
        |hotel_code| async move {
            if hotel_code.is_empty() {
                return None;
            }
            let client = ClientSideApiClient::new();
            match client
                .get_hotel_static_details(DomainHotelCodeId {
                    hotel_id: hotel_code,
                })
                .await
            {
                Ok(details) => {
                    HotelDetailsUIState::set_static_details(Some(details.clone()));
                    Some(details)
                }
                Err(e) => {
                    HotelDetailsUIState::set_error(Some(e));
                    None
                }
            }
        },
    );

    let rates_resource = create_resource(
        move || {
            (
                hotel_info_ctx.hotel_code.get(),
                ui_search_ctx.date_range.get(),
                ui_search_ctx.guests.adults.get(),
                ui_search_ctx.guests.children.get(),
                ui_search_ctx.guests.rooms.get(),
                ui_search_ctx.guests.children_ages.get_signal().get(),
            )
        },
        move |(hotel_code, date_range, adults, children, rooms, children_ages)| async move {
            if hotel_code.is_empty() || date_range.start == (0, 0, 0) || date_range.end == (0, 0, 0)
            {
                return None;
            }

            HotelDetailsUIState::set_rates_loading(true);
            let client = ClientSideApiClient::new();

            let room_guests = vec![DomainRoomGuest {
                no_of_adults: adults,
                no_of_children: children,
                children_ages: if children > 0 {
                    Some(
                        children_ages
                            .into_iter()
                            .map(|age| age.to_string())
                            .collect(),
                    )
                } else {
                    None
                },
            }];

            let search_criteria = DomainHotelSearchCriteria {
                place_id: "".to_string(),
                check_in_date: (date_range.start.0, date_range.start.1, date_range.start.2),
                check_out_date: (date_range.end.0, date_range.end.1, date_range.end.2),
                no_of_nights: date_range.no_of_nights(),
                no_of_rooms: rooms,
                room_guests,
                guest_nationality: "US".to_string(),
                ..Default::default()
            };

            let criteria = DomainHotelInfoCriteria {
                token: hotel_code.clone(),
                hotel_ids: vec![hotel_code],
                search_criteria,
            };
            let mut retries_left = crate::api::consts::API_RETRY_COUNT;
            loop {
                if retries_left == 0 {
                    break None;
                }
                match client.get_hotel_rates(criteria.clone()).await {
                    Ok(rates) => {
                        HotelDetailsUIState::set_rates(Some(rates.clone()));
                        HotelDetailsUIState::set_rates_loading(false);
                        HotelDetailsUIState::set_error(None);
                        break Some(rates);
                    }
                    Err(e) => {
                        HotelDetailsUIState::set_error(Some(e));
                        HotelDetailsUIState::set_rates_loading(false);
                        retries_left -= 1;
                    }
                }
            }
        },
    );

    let is_loading =
        move || /* hotel_details_state.loading.get() ||*/  hotel_details_state.rates_loading.get();
    let error_message = move || hotel_details_state.error.get();

    let loaded = move || static_details_resource.get().and_then(|d| d).is_some();

    let hotel_name_signal = move || {
        if let Some(hotel_details) = hotel_details_state.static_details.get() {
            hotel_details.hotel_name
        } else {
            "".to_string()
        }
    };

    let hotel_code_signal = move || {
        if let Some(hotel_details) = hotel_details_state.static_details.get() {
            hotel_details.hotel_code
        } else {
            "".to_string()
        }
    };

    let star_rating_signal = move || {
        if let Some(hotel_details) = hotel_details_state.static_details.get() {
            hotel_details.star_rating as u8
        } else {
            0
        }
    };

    let description_signal = move || {
        if let Some(hotel_details) = hotel_details_state.static_details.get() {
            hotel_details.description
        } else {
            "".to_string()
        }
    };

    let address_signal = move || {
        if let Some(hotel_details) = hotel_details_state.static_details.get() {
            hotel_details.address
        } else {
            "".to_string()
        }
    };

    let amenities_signal = move || {
        if let Some(hotel_details) = hotel_details_state.static_details.get() {
            // <!-- Use hotel_facilities instead of amenities for LiteAPI compatibility -->
            // <!-- LiteAPI maps facility_ids to hotel_facilities, while amenities remains empty -->
            convert_to_amenities(hotel_details.hotel_facilities)
        } else {
            vec![]
        }
    };

    let images_signal = move || {
        if let Some(hotel_details) = hotel_details_state.static_details.get() {
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

    let open_image_viewer = RwSignal::new(false);

    view! {
        <section class="relative min-h-screen bg-gray-50 pt-16 md:pt-16">
            <HotelListNavbar />
            <div class="lg:hidden px-4 py-4 mb-4">
                <InputGroupContainer
                    default_expanded=false
                    allow_outside_click_collapse=true
                />
            </div>

            // <!-- Use resource pattern like prebook_resource in block_room_v1.rs -->
            // <!-- The resource automatically triggers data loading when dependencies change -->
            <Suspense fallback=move || view! { <></> }>
                {move || {
                // Trigger the resource loading but don't render anything
                let _ = static_details_resource.get();
                let _ = rates_resource.get();
                view! { <></> }
            }}
            </Suspense>
            <Show when=loaded /* fallback=|| view! {
                // <div class="w-full max-w-4xl mx-auto py-4 px-2 md:py-8 md:px-0">
                //     <div class="bg-white rounded-xl shadow-md p-6">
                //         <div class="text-xl font-semibold text-gray-600 text-center">
                //             No hotel data available
                //         </div>
                //     </div>
                // </div>
            } */>
                <HotelDetailsHeader hotel_name_signal=hotel_name_signal() star_rating_signal=star_rating_signal() address_signal=address_signal() hotel_code=hotel_code_signal() />
                <HotelImages open_image_viewer/>
                <DetailsSubnav />
                <OverviewSection
                    description_html=description_signal()
                    address=address_signal()
                    amenities=amenities_signal()
                />

                <SelectRoomSection />
                <GuestReviewsSection />
                <PolicyRulesSection address=address_signal() />
                <SiteFooter />
            </Show>
            <Show when=move || error_message().is_some()>
                <div class="w-full max-w-4xl mx-auto py-4 px-2 md:py-8 md:px-0">
                    // <div class="bg-white rounded-xl shadow-md p-6">
                        {
                            let error = error_message().unwrap_or_else(|| "Unknown error occurred".to_string());
                            if error.contains("No room types or rates available") || error.contains("fully booked") {
                                view! {
                                    <div class="flex items-start gap-4 border border-red-300 rounded-xl bg-red-50 p-4">
                                        <svg xmlns="http://www.w3.org/2000/svg" class="w-6 h-6 flex-shrink-0 text-red-500" viewBox="0 0 18 19" fill="none">
                                            <path d="M5.625 2.75V5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"></path>
                                            <path d="M12.375 2.75V5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"></path>
                                            <path d="M15.75 11.9839V6.125C15.75 5.52826 15.5129 4.95597 15.091 4.53401C14.669 4.11205 14.0967 3.875 13.5 3.875H4.5C3.90326 3.875 3.33097 4.11205 2.90901 4.53401C2.48705 4.95597 2.25 5.52826 2.25 6.125V6.7352" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"></path>
                                            <path d="M2.25 9.64551V13.9989C2.25 14.5956 2.48705 15.1679 2.90901 15.5899C3.33097 16.0118 3.90326 16.2489 4.5 16.2489H13.5C14.0967 16.2489 14.669 16.0118 15.091 15.5899C15.329 15.3518 15.5082 15.066 15.6192 14.7549" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"></path>
                                            <path d="M0.759766 6.27441L17.2404 12.7237" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"></path>
                                        </svg>

                                        <div>
                                            <div class="text-red-600 font-semibold text-base mb-1">
                                                This hotel is fully booked for your selected dates
                                            </div>
                                            <div class="text-gray-700 text-sm">
                                                Please try different dates or check other hotels in the area.
                                            </div>
                                        </div>
                                    </div>
                                }
                            } else {
                                view! {
                                    <div>
                                        <div class="text-xl font-semibold text-red-600 mb-2">Error</div>
                                        <div class="text-gray-700">
                                            {error}
                                        </div>
                                    </div>
                                }
                            }
                        }
                    // </div>
                </div>
            </Show>

        </section>
    }
}

#[component]
pub fn HotelDetailsHeader(
    #[prop(into)] hotel_name_signal: String,
    #[prop(into)] address_signal: String,
    #[prop(into)] star_rating_signal: u8,
    #[prop(into)] hotel_code: String,
) -> impl IntoView {
    let wishlist_code = hotel_code.clone();
    let toggle_wishlist_action = add_to_wishlist_action(hotel_code.clone());
    let is_wishlisted = move || AuthStateSignal::check_if_added_to_wishlist(&wishlist_code);
    let heart_d = "M19.62 27.81C19.28 27.93 18.72 27.93 18.38 27.81C15.48 26.82 9 22.69 9 15.69C9 12.6 11.49 10.1 14.56 10.1C16.38 10.1 17.99 10.98 19 12.34C20.01 10.98 21.63 10.1 23.44 10.1C26.51 10.1 29 12.6 29 15.69C29 22.69 22.52 26.82 19.62 27.81Z";
    view! {
        <div class="my-4 w-full max-w-7xl mx-auto px-4 pt-4 pb-2 lg:pt-2 lg:pb-0">
            {/* on small: actions drop under title; on md+: they sit on the right */}
            <div class="py-2 flex flex-col md:flex-row md:items-start md:justify-between gap-4">
                <div class="min-w-0">
                    // {/* tiny blue stars + rating */}
                    // <div class="flex items-center gap-2 text-blue-600">
                    //     <StarRating rating=move || star_rating_signal />
                    //     // <span class="text-sm"> {format!("{}.0", star_rating_signal)} </span>
                    // </div>
                    <h1 class="mt-1 text-3xl md:text-4xl font-semibold tracking-tight text-gray-900 break-words">
                        {hotel_name_signal}
                    </h1>

                    {/* address row */}
                    <div class="flex flex-wrap items-center gap-3 text-sm text-gray-700">
                        <svg class="w-4 h-4 text-blue-600" viewBox="0 0 24 24" fill="currentColor">
                            <path d="M12 2C8.134 2 5 5.134 5 9c0 5.25 7 13 7 13s7-7.75 7-13c0-3.866-3.134-7-7-7zm0 9.5a2.5 2.5 0 1 1 0-5 2.5 2.5 0 0 1 0 5z"/>
                        </svg>
                        <span class="truncate">{address_signal}</span>
                        <span class="text-gray-300">{"|"}</span>
                        <a href="#map" class="text-blue-600 hover:underline">"Show in Map"</a>
                    </div>
                </div>

                {/* actions */}
                <div class="flex items-center gap-3 md:self-start">
                    <button
                        class="inline-flex items-center gap-2 rounded-xl border border-gray-200 px-4 py-2 text-sm bg-white hover:bg-gray-50 group"
                        on:click=move |_| toggle_wishlist_action.dispatch(())
                    >
                        {move || {
                            let is_wishlisted = is_wishlisted();
                            view! {
                                <div class="flex items-center gap-2">
                                    <svg width="38" height="38" viewBox="0 0 38 38" xmlns="http://www.w3.org/2000/svg">
                                        <circle cx="19" cy="19" r="19" fill="white" />

                                        // FILL heart (underneath)
                                        <path
                                            d=heart_d
                                            fill="currentColor"
                                            stroke="none"
                                            class={
                                                if is_wishlisted {
                                                    "text-red-500 group-hover:text-blue-600 transition-colors"
                                                } else {
                                                    "text-transparent group-hover:text-blue-600 transition-colors"
                                                }
                                            }
                                        />

                                        // STROKE heart (outline, always gray)
                                        <path
                                            d=heart_d
                                            fill="none"
                                            stroke="#45556C"
                                            stroke-width="2"
                                            stroke-linecap="round"
                                            stroke-linejoin="round"
                                            class={
                                                if is_wishlisted {
                                                    "stroke-red-500 group-hover:stroke-blue-600 transition-colors"
                                                } else {
                                                    "stroke-[#45556C] group-hover:stroke-blue-600 transition-colors"
                                                }
                                            }
                                        />
                                    </svg>
                                    <Show when=move || !is_wishlisted ><span class="font-semibold text-gray-700">"Add to Wishlist"</span></Show>
                                </div>
                            }
                        }}
                    </button>
                    <a href="#rooms" class="rounded-xl bg-blue-600 text-white px-4 py-2 text-sm font-medium hover:bg-blue-700 inline-flex items-center gap-2">
                        "Select A Room"
                        <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                            <path d="M5 12h14M13 5l7 7-7 7"/>
                        </svg>
                    </a>
                </div>
            </div>
        </div>
    }
}

/* ===================== Image Gallery ===================== */

#[component]
pub fn HotelImages(open_image_viewer: RwSignal<bool>) -> impl IntoView {
    let hotel_details_state: HotelDetailsUIState = expect_context();

    // index used for the big image on the page
    let (main_index, set_main_index) = create_signal(0);
    // index to start from when opening the lightbox
    let (selected_index, set_selected_index) = create_signal(0);

    let images_signal = create_memo(move |_| {
        if let Some(h) = hotel_details_state.static_details.get() {
            let mut images = h.images.clone();
            if images.len() < 6 {
                let repeat = 6 - images.len();
                let dup = images.clone();
                images.extend(dup.into_iter().take(repeat));
            }
            images
        } else {
            vec![]
        }
    });

    // open viewer from current/explicit index
    let open_from = move |i: usize| {
        set_selected_index(i);
        open_image_viewer.set(true);
    };

    // go next/prev on the PAGE (updates the big image)
    let next = move |_| {
        let n = images_signal().len();
        if n > 0 {
            set_main_index((main_index.get() + 1) % n);
        }
    };
    let prev = move |_| {
        let n = images_signal().len();
        if n > 0 {
            set_main_index((main_index.get() + n - 1) % n);
        }
    };

    move || {
        if images_signal().is_empty() {
            view! { <div class="text-gray-500 text-center py-8">"No images available"</div> }
        } else {
            view! {
                <div class="py-2  w-full max-w-7xl mx-auto px-4">
                    {/* Mobile: one big image */}
                    <div class="md:hidden">
                        <div class="relative rounded-2xl overflow-hidden aspect-[16/9] bg-gray-100">
                            <img
                                src=move || images_signal()[main_index.get()].clone()
                                alt="Hotel"
                                class="absolute inset-0 h-full w-full object-cover"
                                on:click=move |_| open_from(main_index.get())
                            />
                        </div>
                    </div>

                    {/* Desktop / tablet */}
                    <div class="hidden md:grid grid-cols-12 gap-6">
                        {/* Left: main image (driven by main_index) */}
                        <div class="col-span-9 relative rounded-2xl overflow-hidden bg-gray-100 h-full min-h-0">
                            <img
                                src=move || images_signal()[main_index.get()].clone()
                                alt="Hotel main"
                                class="absolute inset-0 h-full w-full object-cover"
                                on:click=move |_| open_from(main_index.get())
                            />
                            {/* arrows */}
                            <button
                                class="absolute left-4 top-1/2 -translate-y-1/2 grid place-items-center w-10 h-10 rounded-full bg-white/95 hover:bg-white shadow"
                                on:click=prev aria-label="Previous"
                            >
                                <span class="text-lg">"‹"</span>
                            </button>
                            <button
                                class="absolute right-4 top-1/2 -translate-y-1/2 grid place-items-center w-10 h-10 rounded-full bg-white/95 hover:bg-white shadow"
                                on:click=next aria-label="Next"
                            >
                                <span class="text-lg">"›"</span>
                            </button>
                        </div>

                        {/* Right: vertical stack of 3 thumbs (static) */}
                        <div class="col-span-3 flex flex-col gap-6">
                            <img
                                src=move || images_signal().get(1).cloned().unwrap_or_else(|| images_signal()[0].clone())
                                alt="Thumb 1"
                                class="w-full aspect-[16/9] rounded-xl object-cover cursor-pointer"
                                on:click=move |_| set_main_index(1)  // change main image
                            />
                            <img
                                src=move || images_signal().get(2).cloned().unwrap_or_else(|| images_signal()[0].clone())
                                alt="Thumb 2"
                                class="w-full aspect-[16/9] rounded-xl object-cover cursor-pointer"
                                on:click=move |_| set_main_index(2)
                            />
                            <div class="relative">
                                <img
                                    src=move || images_signal().get(3).cloned().unwrap_or_else(|| images_signal()[0].clone())
                                    alt="Thumb 3"
                                    class="w-full aspect-[16/9] rounded-xl object-cover cursor-pointer"
                                    on:click=move |_| set_main_index(3)
                                />
                                {/* "Show All Pictures" overlay opens viewer at current main_index */}
                                <button
                                    class="absolute left-4 right-4 top-1/2 -translate-y-1/2 rounded-xl bg-white/95 px-4 py-3 text-sm leading-tight shadow hover:bg-white"
                                    on:click=move |_| open_from(main_index.get())
                                >
                                    <div class="flex flex-col items-center">
                                        <span>"Show All Pictures"</span>
                                    </div>
                                </button>
                            </div>
                        </div>
                    </div>

                    {/* Lightbox uses selected_index */}
                    {move || open_image_viewer.get().then(|| {
                        view! {
                            <ImageLightbox
                                images=images_signal()
                                initial_index=selected_index.get()
                                loop_images=true
                                on_close=Callback::new(move |_| open_image_viewer.set(false))
                            />
                        }
                    })}
                </div>
            }
        }
    }
}

/* ---------------- Sub-nav tabs ---------------- */

#[component]
pub fn DetailsSubnav() -> impl IntoView {
    let tabs = vec![
        ("overview", "Overview"),
        ("facilities", "Facilities"),
        ("rooms", "Select A Room"),
        ("reviews", "Guest Reviews"),
        ("rules", "Rules & Policies"),
    ];
    view! {
        <div class="w-full max-w-7xl mx-auto px-4 pt-2 md:pt-4">
            <div class="flex items-center gap-8 text-gray-600 text-sm md:text-base overflow-x-auto">
                <For each=move || tabs.clone() key=|(id, _)| id.to_string() let:tab>
                    {let (id, label) = tab;
                    view!{
                        <a href={format!("#{id}")} class="relative py-3 whitespace-nowrap hover:text-gray-900 group">
                            <span class="">{label}</span>
                            <span class="absolute left-0 -bottom-[1px] h-[3px] w-0 bg-blue-500 rounded-full group-hover:w-full transition-all"></span>
                        </a>
                    }}
                </For>
            </div>
            <div class="mt-3 h-px w-full bg-gray-200"></div>
        </div>
    }
}

#[component]
pub fn SectionTitle(#[prop(into)] id: String, #[prop(into)] title: String) -> impl IntoView {
    view! {
        // <div id=id class="scroll-mt-24"/>
        <h2 id=id  class="scroll-mt-24 pl-4 border-l-4 border-blue-500 text-2xl md:text-[28px] font-semibold text-gray-900">
            {title}
        </h2>
    }
}

#[component]
pub fn MapBlock(#[prop(into)] address: String) -> impl IntoView {
    let hotel_details_state: HotelDetailsUIState = expect_context();
    let is_map_expanded = create_rw_signal(false);

    let location_signal = move || {
        if let Some(hotel_details) = hotel_details_state.static_details.get() {
            hotel_details.location
        } else {
            None
        }
    };

    // Generate map URL
    let map_url_signal = move || {
        location_signal().map(|location| {
            format!(
                "https://www.openstreetmap.org/export/embed.html?bbox={},{},{},{}&layer=mapnik&marker={:.6},{:.6}",
                location.longitude - 0.01,
                location.latitude - 0.01,
                location.longitude + 0.01,
                location.latitude + 0.01,
                location.latitude,
                location.longitude
            )
        })
    };

    // Generate expanded map URL (larger bounding box for better view)
    let expanded_map_url_signal = move || {
        location_signal().map(|location| {
            format!(
                "https://www.openstreetmap.org/export/embed.html?bbox={},{},{},{}&layer=mapnik&marker={:.6},{:.6}",
                location.longitude - 0.05,
                location.latitude - 0.05,
                location.longitude + 0.05,
                location.latitude + 0.05,
                location.latitude,
                location.longitude
            )
        })
    };

    view! {
        <div id="map" class="scroll-mt-24 mt-4">
            <div class="flex items-center justify-between">
                <span class="font-semibold text-gray-800">Map</span>
                <button
                    type="button"
                    class="text-sm text-blue-600 hover:underline cursor-pointer flex items-center gap-1"
                    on:click=move |_| is_map_expanded.set(true)
                >
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 8V4m0 0h4M4 4l5 5m11-1V4m0 0h-4m4 0l-5 5M4 16v4m0 0h4m-4 0l5-5m11 5l-5-5m5 5v-4m0 4h-4"/>
                    </svg>
                    "Show Map"
                </button>
            </div>

            <div class="mt-2 rounded-xl overflow-hidden bg-gray-200">
                <Show
                    when=move || location_signal().is_some()
                    fallback=|| view! {
                        <div class="w-full aspect-[21/9] bg-[url('/img/map-placeholder.webp')] bg-cover bg-center"></div>
                    }
                >
                    {move || map_url_signal().map(|url| view! {
                        <iframe
                            class="w-full"
                            frameborder="0"
                            scrolling="no"
                            marginheight="0"
                            marginwidth="0"
                            src=url
                        ></iframe>
                    })}
                </Show>
            </div>

            <div class="mt-3 flex items-start gap-2 text-gray-700">
                <svg class="w-[18px] h-[18px] text-blue-600 mt-[2px]" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M12 2C8.134 2 5 5.134 5 9c0 5.25 7 13 7 13s7-7.75 7-13c0-3.866-3.134-7-7-7zm0 9.5a2.5 2.5 0 1 1 0-5 2.5 2.5 0 0 1 0 5z"/>
                </svg>
                <span class="text-sm md:text-base">{address.clone()}</span>
            </div>
        </div>

        // Expanded Map Modal
        <Show when=move || is_map_expanded.get()>
            <div class="fixed inset-0 z-50 flex items-center justify-center">
                // Backdrop
                <div
                    class="absolute inset-0 bg-black/60 backdrop-blur-sm"
                    on:click=move |_| is_map_expanded.set(false)
                ></div>

                // Modal content
                <div class="relative z-10 w-[95vw] h-[85vh] max-w-6xl bg-white rounded-2xl shadow-2xl overflow-hidden flex flex-col">
                    // Header
                    <div class="flex items-center justify-between px-6 py-4 border-b border-gray-200 bg-gray-50">
                        <div class="flex items-center gap-2">
                            <svg class="w-5 h-5 text-blue-600" viewBox="0 0 24 24" fill="currentColor">
                                <path d="M12 2C8.134 2 5 5.134 5 9c0 5.25 7 13 7 13s7-7.75 7-13c0-3.866-3.134-7-7-7zm0 9.5a2.5 2.5 0 1 1 0-5 2.5 2.5 0 0 1 0 5z"/>
                            </svg>
                            <span class="font-semibold text-gray-800 text-lg">Hotel Location</span>
                        </div>
                        <button
                            type="button"
                            class="p-2 rounded-full hover:bg-gray-200 transition-colors cursor-pointer"
                            on:click=move |_| is_map_expanded.set(false)
                            aria-label="Close map"
                        >
                            <svg class="w-6 h-6 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
                            </svg>
                        </button>
                    </div>

                    // Map iframe
                    <div class="flex-1 bg-gray-200">
                        <Show
                            when=move || location_signal().is_some()
                            fallback=|| view! {
                                <div class="w-full h-full bg-[url('/img/map-placeholder.webp')] bg-cover bg-center flex items-center justify-center">
                                    <span class="text-gray-600 bg-white/80 px-4 py-2 rounded-lg">Map not available</span>
                                </div>
                            }
                        >
                            {move || expanded_map_url_signal().map(|url| view! {
                                <iframe
                                    class="w-full h-full"
                                    frameborder="0"
                                    scrolling="no"
                                    marginheight="0"
                                    marginwidth="0"
                                    src=url
                                ></iframe>
                            })}
                        </Show>
                    </div>

                    // Footer with address
                    <div class="px-6 py-3 border-t border-gray-200 bg-gray-50">
                        <div class="flex items-center gap-2 text-gray-700">
                            <svg class="w-4 h-4 text-blue-600 flex-shrink-0" viewBox="0 0 24 24" fill="currentColor">
                                <path d="M12 2C8.134 2 5 5.134 5 9c0 5.25 7 13 7 13s7-7.75 7-13c0-3.866-3.134-7-7-7zm0 9.5a2.5 2.5 0 1 1 0-5 2.5 2.5 0 0 1 0 5z"/>
                            </svg>
                            <span class="text-sm">{address.clone()}</span>
                        </div>
                    </div>
                </div>
            </div>
        </Show>
    }
}

#[component]
pub fn FacilityChips(amenities: Vec<Amenity>) -> impl IntoView {
    let is_expanded = create_rw_signal(false);
    let total_count = amenities.len();
    let top = amenities.iter().take(10).cloned().collect::<Vec<_>>();
    let all_amenities = store_value(amenities);

    view! {
        <div class="flex items-center justify-between">
            <span class="font-semibold text-gray-800">Most Popular facilities</span>
            <Show when=move || { total_count > 10 }>
                <button
                    type="button"
                    class="text-sm text-blue-600 hover:underline cursor-pointer flex items-center gap-1"
                    on:click=move |_| is_expanded.set(true)
                >
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"/>
                    </svg>
                    {format!("Show All ({})", total_count)}
                </button>
            </Show>
        </div>

        <div class="mt-4 flex flex-wrap gap-x-6 gap-y-4">
            <For each=move || top.clone() key=|a| a.text.clone() let:item>
                <div class="inline-flex items-center text-gray-800">
                    <div class="w-2 h-2 bg-blue-600 rounded-full mr-2"></div>
                    <span class="text-sm md:text-base">{item.text}</span>
                </div>
            </For>
        </div>

        // Expanded Facilities Modal
        <Show when=move || is_expanded.get()>
            <div class="fixed inset-0 z-50 flex items-center justify-center">
                // Backdrop
                <div
                    class="absolute inset-0 bg-black/60 backdrop-blur-sm"
                    on:click=move |_| is_expanded.set(false)
                ></div>

                // Modal content
                <div class="relative z-10 w-[95vw] max-w-2xl max-h-[85vh] bg-white rounded-2xl shadow-2xl overflow-hidden flex flex-col">
                    // Header
                    <div class="flex items-center justify-between px-6 py-4 border-b border-gray-200 bg-gray-50">
                        <div class="flex items-center gap-2">
                            <svg class="w-5 h-5 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4"/>
                            </svg>
                            <span class="font-semibold text-gray-800 text-lg">{format!("All Facilities ({})", total_count)}</span>
                        </div>
                        <button
                            type="button"
                            class="p-2 rounded-full hover:bg-gray-200 transition-colors cursor-pointer"
                            on:click=move |_| is_expanded.set(false)
                            aria-label="Close"
                        >
                            <svg class="w-6 h-6 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
                            </svg>
                        </button>
                    </div>

                    // Facilities grid
                    <div class="flex-1 overflow-y-auto p-6">
                        <div class="grid grid-cols-1 sm:grid-cols-2 gap-x-8 gap-y-3">
                            <For each=move || all_amenities.get_value() key=|a| a.text.clone() let:item>
                                <div class="inline-flex items-center text-gray-800">
                                    <Icon icon=item.icon class="w-5 h-5 text-blue-600 mr-3 flex-shrink-0"/>
                                    <span class="text-sm md:text-base">{item.text}</span>
                                </div>
                            </For>
                        </div>
                    </div>
                </div>
            </div>
        </Show>
    }
}

#[component]
pub fn OverviewSection(
    #[prop(into)] description_html: String,
    #[prop(into)] address: String,
    amenities: Vec<Amenity>,
) -> impl IntoView {
    view! {
        <section class="w-full max-w-7xl mx-auto px-4 mt-6 md:mt-8">
            <SectionTitle id="overview" title="Overview"/>
            <div class="mt-3 text-gray-700 leading-7" inner_html=description_html></div>

            <MapBlock address=address />

            <div class="mt-10">
                <SectionTitle id="facilities" title="Facility"/>
                <div class="mt-4">
                    <FacilityChips amenities/>
                </div>
            </div>
        </section>
    }
}

fn group_room_options_by_type(rates: Vec<DomainRoomOption>) -> Vec<OfferGroup> {
    let mut grouped_by_offer: BTreeMap<String, OfferGroup> = BTreeMap::new();
    let mut offer_room_signatures: HashMap<String, HashSet<OfferRoomSignature>> = HashMap::new();

    let mut seen_rate_keys = HashSet::new();

    for rate in rates {
        if !seen_rate_keys.insert(rate.room_data.rate_key.clone()) {
            continue;
        }
        let offer_key = rate.room_data.offer_id.clone();
        let signature = OfferRoomSignature {
            mapped_room_id: rate.mapped_room_id,
            occupancy_number: rate.room_data.occupancy_number,
            meal_plan: rate.meal_plan.clone().unwrap_or_else(String::new),
            price_bits: rate.price.room_price.to_bits(),
        };
        let set = offer_room_signatures
            .entry(offer_key.clone())
            .or_insert_with(HashSet::new);

        // Disable deduplication to ensure all rooms in the offer are counted
        // This is necessary because identical rooms (same price/signature) must still contribute to the total offer price
        // if !set.insert(signature) {
        //     continue;
        // }
        set.insert(signature);
        grouped_by_offer
            .entry(offer_key.clone())
            .and_modify(|entry| {
                if entry.mapped_room_id.is_none() && rate.mapped_room_id != 0 {
                    entry.mapped_room_id = Some(rate.mapped_room_id);
                }
                entry.rates.push(rate.clone());
                if !entry
                    .room_names
                    .iter()
                    .any(|name| name == &rate.room_data.room_name)
                {
                    entry.room_names.push(rate.room_data.room_name.clone());
                }
            })
            .or_insert_with(|| OfferGroup {
                offer_id: offer_key.clone(),
                mapped_room_id: (rate.mapped_room_id != 0).then_some(rate.mapped_room_id),
                rates: vec![rate.clone()],
                room_names: vec![rate.room_data.room_name.clone()],
            });
    }

    let mut groups: Vec<OfferGroup> = grouped_by_offer.into_values().collect();

    for group in &mut groups {
        group.rates.sort_by(|a, b| {
            a.price
                .room_price
                .partial_cmp(&b.price.room_price)
                .unwrap_or(Ordering::Equal)
        });
    }

    groups.sort_by(|a, b| {
        let a_price = a
            .rates
            .first()
            .map(|rate| rate.price.room_price)
            .unwrap_or(f64::MAX);
        let b_price = b
            .rates
            .first()
            .map(|rate| rate.price.room_price)
            .unwrap_or(f64::MAX);
        a_price.partial_cmp(&b_price).unwrap_or(Ordering::Equal)
    });

    groups
}

fn same_mapped_room_id(offer: &OfferGroup) -> Option<u32> {
    let mut room_id: Option<u32> = None;
    for rate in &offer.rates {
        if rate.mapped_room_id == 0 {
            return None;
        }
        match room_id {
            None => room_id = Some(rate.mapped_room_id),
            Some(existing) if existing == rate.mapped_room_id => {}
            _ => return None,
        }
    }
    room_id
}

fn build_type_b_title(rates: &[DomainRoomOption]) -> Option<String> {
    if rates.is_empty() {
        return None;
    }
    let mut counts: HashMap<(Option<u32>, String), (String, usize)> = HashMap::new();
    for rate in rates {
        let mapped = (rate.mapped_room_id != 0).then_some(rate.mapped_room_id);
        let normalized_name = normalized_room_key(&rate.room_data.room_name);
        let key = (mapped, normalized_name);
        let entry = counts
            .entry(key)
            .or_insert((rate.room_data.room_name.clone(), 0));
        entry.1 += 1;
    }

    let mut entries: Vec<(Option<u32>, String, usize)> = counts
        .into_iter()
        .map(|((mapped, _), (display_name, count))| (mapped, display_name, count))
        .collect();
    entries.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| a.1.cmp(&b.1)));

    let title = entries
        .into_iter()
        .map(|(_, name, count)| format!("{count} x {name}"))
        .collect::<Vec<_>>()
        .join(" + ");

    if title.is_empty() {
        None
    } else {
        Some(title)
    }
}

fn dedup_rates(rates: &[DomainRoomOption], include_occupancy: bool) -> Vec<DomainRoomOption> {
    let mut seen = HashSet::new();
    let mut deduped = Vec::new();
    for rate in rates {
        let key = RateRowKey::from_rate(rate, include_occupancy);
        if seen.insert(key) {
            deduped.push(rate.clone());
        }
    }
    deduped.sort_by(|a, b| {
        a.price
            .room_price
            .partial_cmp(&b.price.room_price)
            .unwrap_or(Ordering::Equal)
    });
    deduped
}

/// Deduplicate TYPE B rates by meal plan only, keeping the lowest price for each meal plan.
/// This prevents showing multiple rows with the same meal plan but different prices on TYPE B cards.
fn dedup_rates_by_meal_plan(rates: &[DomainRoomOption]) -> Vec<DomainRoomOption> {
    let mut best_by_meal_plan: HashMap<String, DomainRoomOption> = HashMap::new();

    for rate in rates {
        let meal_plan = rate
            .meal_plan
            .clone()
            .unwrap_or_else(|| "Room Only".to_string());

        best_by_meal_plan
            .entry(meal_plan)
            .and_modify(|existing| {
                // Keep the rate with the lower price
                if rate.price.room_price < existing.price.room_price {
                    *existing = rate.clone();
                }
            })
            .or_insert_with(|| rate.clone());
    }

    let mut deduped: Vec<DomainRoomOption> = best_by_meal_plan.into_values().collect();
    deduped.sort_by(|a, b| {
        a.price
            .room_price
            .partial_cmp(&b.price.room_price)
            .unwrap_or(Ordering::Equal)
    });
    deduped
}

fn dedup_rates_for_type_b(rates: &[DomainRoomOption]) -> Vec<DomainRoomOption> {
    let mut rates_by_offer: HashMap<String, Vec<DomainRoomOption>> = HashMap::new();
    for rate in rates {
        rates_by_offer
            .entry(rate.room_data.offer_id.clone())
            .or_default()
            .push(rate.clone());
    }

    let mut offer_representatives: Vec<DomainRoomOption> = Vec::new();
    for (_, offer_rates) in rates_by_offer {
        if let Some(first_rate) = offer_rates.first() {
            let mut rep_rate = first_rate.clone();
            // Sum NET prices (same as Cart calculation at line 1925)
            let total_price: f64 = offer_rates
                .iter()
                .map(|r| r.price_excluding_included_taxes())
                .sum();
            rep_rate.price.room_price = total_price;
            // Clear tax_lines since we already computed Net
            rep_rate.tax_lines.clear();
            offer_representatives.push(rep_rate);
        }
    }

    // Now dedup by meal plan using the existing logic (lowest price per meal plan)
    dedup_rates_by_meal_plan(&offer_representatives)
}

fn build_room_cards(mut offers: Vec<OfferGroup>) -> Vec<RoomCard> {
    let mut type_a_map: HashMap<u32, RoomCard> = HashMap::new();
    let mut type_a_order: Vec<u32> = Vec::new();
    // Group TYPE B cards by their card title (room combination)
    let mut type_b_map: HashMap<String, RoomCard> = HashMap::new();
    let mut type_b_order: Vec<String> = Vec::new();

    for offer in &offers {
        log!(
            "DEBUG: Offer '{}' has {} rates",
            offer.room_names.join(" + "),
            offer.rates.len()
        );
        for (i, r) in offer.rates.iter().enumerate() {
            log!(
                "  Rate[{}]: room_price={:.2}, tax_lines.len={}, included_taxes={:.2}",
                i,
                r.price.room_price,
                r.tax_lines.len(),
                r.included_taxes_total()
            );
        }
    }

    for offer in offers {
        let offer_rate = offer.rates.first().cloned();
        let normalized_title = offer.room_names.join(" + ");

        if let Some(mapped_id) = same_mapped_room_id(&offer) {
            // TYPE A: Single room type
            if let Some(first_rate) = offer_rate {
                // Calculate sum of Net Prices for all rooms in the offer
                // This handles cases where mixed occupancies (adults/children) result in different per-room prices
                let mut representative_rate = first_rate.clone();
                let count = offer.rates.len() as f64;
                if count > 0.0 {
                    // Sum the Net Prices (using price_excluding_included_taxes for accuracy)
                    // RoomRateRow will divide this by the number of rooms to show per-room price
                    let total_room_price: f64 = offer
                        .rates
                        .iter()
                        .map(|r| r.price_excluding_included_taxes())
                        .sum();

                    let total_tax: f64 = offer.rates.iter().map(|r| r.price.tax).sum();

                    representative_rate.price.room_price = total_room_price;
                    representative_rate.price.tax = total_tax;
                    representative_rate.price.suggested_selling_price = total_room_price;

                    // Clear tax lines to avoid double subtraction in price_excluding_included_taxes
                    // Since we already summed the Net Prices, the total is already Net.
                    representative_rate.tax_lines.clear();
                    representative_rate.price.published_price = offer
                        .rates
                        .iter()
                        .map(|r| r.price.published_price)
                        .sum::<f64>()
                        / count;
                    representative_rate.price.offered_price = offer
                        .rates
                        .iter()
                        .map(|r| r.price.offered_price)
                        .sum::<f64>()
                        / count;
                }

                let entry = type_a_map.entry(mapped_id).or_insert_with(|| RoomCard {
                    mapped_room_id: Some(mapped_id),
                    room_names: offer.room_names.clone(),
                    card_title: normalized_title.clone(),
                    rates: Vec::new(),
                });

                for name in &offer.room_names {
                    if !entry.room_names.contains(name) {
                        entry.room_names.push(name.clone());
                    }
                }
                if entry.card_title.is_empty() {
                    entry.card_title = entry.room_names.join(" + ");
                }
                entry.rates.push(representative_rate);
                if !type_a_order.contains(&mapped_id) {
                    type_a_order.push(mapped_id);
                }
            }
        } else {
            // TYPE B: Multiple room types
            let card_title =
                build_type_b_title(&offer.rates).unwrap_or_else(|| normalized_title.clone());

            // Group TYPE B cards by card_title (room combination)
            let entry = type_b_map.entry(card_title.clone()).or_insert_with(|| {
                // Track order of TYPE B cards
                if !type_b_order.contains(&card_title) {
                    type_b_order.push(card_title.clone());
                }
                RoomCard {
                    mapped_room_id: None,
                    room_names: offer.room_names.clone(),
                    card_title: card_title.clone(),
                    rates: Vec::new(),
                }
            });

            // Merge room names from this offer
            for name in &offer.room_names {
                if !entry.room_names.contains(name) {
                    entry.room_names.push(name.clone());
                }
            }

            // Add all rates from this offer to the card
            entry.rates.extend(offer.rates);
        }
    }

    // Build final card list
    let mut cards = Vec::new();

    // Add TYPE A cards in order
    for mapped_id in type_a_order {
        if let Some(mut card) = type_a_map.remove(&mapped_id) {
            if card.card_title.is_empty() {
                card.card_title = card.room_names.join(" + ");
            }
            // Deduplicate rates by meal plan, keeping the lowest price for each
            card.rates = dedup_rates_by_meal_plan(&card.rates);
            cards.push(card);
        }
    }

    // Add TYPE B cards in order, deduplicating rates by meal plan
    for card_title in type_b_order {
        if let Some(mut card) = type_b_map.remove(&card_title) {
            // Deduplicate rates by meal plan, keeping the lowest price for each
            // For TYPE B, we first sum the prices of all rooms in the offer
            card.rates = dedup_rates_for_type_b(&card.rates);
            cards.push(card);
        }
    }

    cards
}

fn grouped_rooms_for_state(state: &HotelDetailsUIState) -> Vec<RoomCard> {
    let rates = match state.rates.get() {
        Some(rates) => rates,
        None => return vec![],
    };

    if rates.is_empty() {
        return vec![];
    }

    let mut grouped = group_room_options_by_type(rates);

    if let Some(static_details) = state.static_details.get() {
        if !static_details.rooms.is_empty() {
            let mut ordered_groups = Vec::new();

            for room in &static_details.rooms {
                let room_id = room.room_id.trim().parse::<u32>().ok();
                let position = if let Some(id) = room_id {
                    grouped
                        .iter()
                        .position(|group| group.mapped_room_id == Some(id))
                } else {
                    grouped.iter().position(|group| {
                        group.room_names.iter().any(|name| {
                            normalized_room_key(name) == normalized_room_key(&room.room_name)
                        })
                    })
                };

                if let Some(pos) = position {
                    ordered_groups.push(grouped.remove(pos));
                }
            }

            if !ordered_groups.is_empty() {
                ordered_groups.extend(grouped);
                grouped = ordered_groups;
            }
        }
    }

    build_room_cards(grouped)
}

fn fallback_image_for_state(state: &HotelDetailsUIState) -> Option<String> {
    state
        .static_details
        .get()
        .and_then(|details| details.images.first().cloned())
}

fn amenity_preview_for_state(state: &HotelDetailsUIState) -> Vec<Amenity> {
    state
        .static_details
        .get()
        .map(|details| convert_to_amenities(details.hotel_facilities))
        .unwrap_or_default()
}

fn currency_symbol_for_code(code: &str) -> &str {
    match code {
        "INR" => "₹",
        "EUR" => "€",
        "GBP" => "£",
        "USD" => "$",
        _ => "$",
    }
}

fn format_currency_with_code(amount: f64, currency_code: &str) -> String {
    let symbol = currency_symbol_for_code(currency_code);
    if amount.fract() == 0.0 {
        format!("{symbol}{:.0}", amount)
    } else {
        format!("{symbol}{:.2}", amount)
    }
}

fn included_taxes_for_rate(rate: &DomainRoomOption) -> f64 {
    rate.included_taxes_total()
}

fn nightly_price_excluding_taxes(rate: &DomainRoomOption, nights: u32) -> f64 {
    let valid_nights = if nights == 0 { 1 } else { nights } as f64;
    rate.price_excluding_included_taxes() / valid_nights
}

fn format_occupancy_text(info: Option<&DomainRoomOccupancy>) -> String {
    if let Some(info) = info {
        let adults = info.adult_count.unwrap_or(0);
        let children = info.child_count.unwrap_or(0);
        let max = info.max_occupancy.unwrap_or(adults + children);
        if children > 0 {
            format!(
                "Sleeps up to {} · {} adults + {} children",
                max, adults, children
            )
        } else if adults > 0 {
            format!("Sleeps up to {} · {} adults", max, adults)
        } else {
            format!("Sleeps {}", max)
        }
    } else {
        "Occupancy details not provided".to_string()
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
        <div class="flex flex-row items-center justify-between border-b border-gray-300 py-2">
            // <!-- Robust wrap: flex-1 min-w-0 for text, flex-shrink-0 for counter, items-start for top align -->
            <p class="w-0 flex-1 min-w-0 font-medium text-sm md:text-base break-words whitespace-normal">
                {format!("{} - ${:.2}", room_type, room_price)}
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
    let ui_search_ctx: UISearchCtx = expect_context();
    let navigate = use_navigate();

    // Create loading state for the Book Now button
    let booking_loading = create_rw_signal(false);

    // Reactive signals for pricing calculations
    let date_range = move || ui_search_ctx.date_range.get();

    // Calculate number of nights
    let number_of_nights = move || date_range().no_of_nights();
    let nights = move || {
        let nights = number_of_nights();
        if nights == 0 {
            1
        } else {
            nights
        }
    };

    // Calculate total selected rooms
    let total_selected_rooms = move || HotelDetailsUIState::total_selected_rooms();

    // Check if any rooms are selected
    let has_rooms_selected = move || total_selected_rooms() > 0;

    // Create memo for button disabled state
    let button_disabled = create_memo(move |_| !has_rooms_selected() || booking_loading.get());

    let currency_code = create_memo(move |_| {
        HotelDetailsUIState::get_available_room_options()
            .first()
            .map(|opt| opt.price.currency_code.clone())
            .unwrap_or_else(|| "USD".to_string())
    });
    let resolved_selection_rows = move || {
        let selected = HotelDetailsUIState::get_selected_rooms();
        let options = HotelDetailsUIState::get_available_room_options();
        let default_code = currency_code.get();
        let nights_val = {
            let n = ui_search_ctx.date_range.get().no_of_nights();
            if n == 0 {
                1.0
            } else {
                n as f64
            }
        };

        // First pass: collect all individual rows
        let mut raw_rows = Vec::new();
        for (rate_key, quantity) in selected.into_iter().filter(|(_, qty)| *qty > 0) {
            if let Some(opt) = options
                .iter()
                .find(|opt| opt.room_data.rate_key == rate_key)
            {
                let room_name = opt.room_data.room_name.clone();
                let total_price_for_stay = opt.price_excluding_included_taxes(); // Total for ALL nights
                let price_per_night = total_price_for_stay / nights_val;
                let code = opt.price.currency_code.clone();
                let meal_plan = opt.meal_plan.clone().unwrap_or_default();
                raw_rows.push((
                    rate_key,
                    quantity,
                    room_name,
                    meal_plan,
                    price_per_night,
                    total_price_for_stay,
                    code,
                ));
            }
        }

        // Second pass: group by room_name and meal_plan
        // Key: (Name, MealPlan), Value: (TotalQty, TotalPriceSum, Code, FirstRateKey)
        let mut grouped: HashMap<(String, String), (u32, f64, f64, String, String)> =
            HashMap::new();

        for (rate_key, qty, name, meal, price, total, code) in raw_rows {
            let entry = grouped
                .entry((name, meal))
                .or_insert((0, 0.0, 0.0, code, rate_key));
            entry.0 += qty;
            entry.1 += price * qty as f64; // Accumulate per-night prices
            entry.2 += total * qty as f64; // Accumulate total prices
        }

        // Convert back to vector
        // Return: (rate_key, qty, name, meal, avg_price_per_night, code, total_for_all_nights_and_rooms)
        grouped
            .into_iter()
            .map(
                |((name, meal), (qty, total_price_per_night, total_for_stay, code, rate_key))| {
                    let avg_price = total_price_per_night / qty as f64;
                    (rate_key, qty, name, meal, avg_price, code, total_for_stay)
                },
            )
            .collect::<Vec<_>>()
    };

    // Calculate subtotal by summing line totals from resolved_selection_rows
    // This ensures the Cart total exactly matches the displayed subtotals
    let total_for_stay = move || {
        let rows = resolved_selection_rows();
        let nights_val = {
            let n = ui_search_ctx.date_range.get().no_of_nights();
            if n == 0 {
                1
            } else {
                n
            }
        };
        rows.iter()
            .fold(0.0, |acc, (_, qty, _, _, price_per_night, _, _)| {
                let rounded_price = (*price_per_night * 100.0).round() / 100.0;
                let line_total = rounded_price * *qty as f64 * nights_val as f64;
                acc + line_total
            })
    };
    let guests = ui_search_ctx.guests;
    let adults_count = move || guests.adults.get();

    // Create action for Book Now button
    let book_now_action = create_action(move |_: &()| {
        let navigate = navigate.clone();
        async move {
            booking_loading.set(true);
            let nights_val = {
                let n = ui_search_ctx.date_range.get().no_of_nights();
                if n == 0 {
                    1.0
                } else {
                    n as f64
                }
            };

            // <!-- Pass room selection data to BlockRoomUIState -->
            // Get selected rooms with quantities
            let selected_rooms = HotelDetailsUIState::get_selected_rooms();
            let available_rooms = HotelDetailsUIState::get_available_rooms();
            let room_options = HotelDetailsUIState::get_available_room_options();
            let hotel_details = HotelDetailsUIState::get_hotel_details();

            // Create room selection summary for block room page
            let mut raw_summaries = Vec::new();
            let mut selected_rooms_with_data = std::collections::HashMap::new();
            let fallback_price_per_room = {
                let total_rooms = HotelDetailsUIState::total_selected_rooms();
                if total_rooms > 0 {
                    HotelDetailsUIState::total_room_price() / total_rooms as f64
                } else {
                    0.0
                }
            };

            for (rate_key, quantity) in selected_rooms.iter() {
                if *quantity > 0 {
                    if let Some(room_option) = room_options
                        .iter()
                        .find(|opt| &opt.room_data.rate_key == rate_key)
                    {
                        raw_summaries.push((room_option, *quantity));
                        selected_rooms_with_data
                            .insert(rate_key.clone(), (*quantity, room_option.room_data.clone()));
                    } else if let Some(room_data) =
                        available_rooms.iter().find(|r| &r.rate_key == rate_key)
                    {
                        selected_rooms_with_data
                            .insert(rate_key.clone(), (*quantity, room_data.clone()));
                    }
                }
            }

            // Group by name and meal plan
            let mut grouped_summaries: HashMap<
                (String, String),
                (
                    u32,
                    f64,
                    crate::domain::DomainRoomData,
                    Vec<crate::domain::DomainTaxLine>,
                ),
            > = HashMap::new();

            for (opt, qty) in raw_summaries {
                let name = opt.room_data.room_name.clone();
                let meal = opt.meal_plan.clone().unwrap_or_default();
                let price_per_night = opt.price_excluding_included_taxes() / nights_val;

                let entry = grouped_summaries.entry((name, meal)).or_insert((
                    0,
                    0.0,
                    opt.room_data.clone(),
                    opt.tax_lines.clone(),
                ));
                entry.0 += qty;
                entry.1 += price_per_night * qty as f64;
            }

            let room_selection_summary: Vec<RoomSelectionSummary> = grouped_summaries
                .into_iter()
                .map(|((name, meal), (qty, total_price, room_data, tax_lines))| {
                    let avg_price = total_price / qty as f64;
                    RoomSelectionSummary {
                        room_id: room_data.rate_key.clone(), // Use representative ID
                        room_name: name,
                        meal_plan: Some(meal),
                        quantity: qty,
                        price_per_night: avg_price,
                        tax_lines,
                        room_data,
                    }
                })
                .collect();

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
        <div id="cart-section" class="scroll-mt-24 bg-gray-50 border border-gray-200 rounded-2xl shadow-sm p-5 space-y-5">
            <div class="flex items-start justify-between gap-3">
                <div>
                    <div class="text-lg font-semibold text-gray-900">"Cart"</div>
                    <p class="text-sm text-gray-500">
                        {move || {
                            if has_rooms_selected() {
                                format!(
                                    "{} room{} selected",
                                    total_selected_rooms(),
                                    if total_selected_rooms() == 1 { "" } else { "s" }
                                )
                            } else {
                                "Pick a room to continue with your booking.".to_string()
                            }
                        }}
                    </p>
                </div>
            </div>

            <Show
                when=has_rooms_selected
                fallback=|| view! {
                    <div class="border border-dashed border-gray-200 rounded-xl bg-white text-sm text-gray-600 px-4 py-5 text-center">
                        "You haven't selected any rooms yet."
                    </div>
                }
            >
                <div class="space-y-4">
                    <div class="space-y-3">
                        {move || {
                            resolved_selection_rows()
                                .into_iter()
                                .map(|selected| {
                                    let (_, quantity, room_name, meal_plan, price_per_night, code, _) =
                                        selected;
                                    let nights = nights();
                                    // Round price to 2 decimals to match what's displayed, then calculate total
                                    let rounded_price = (price_per_night * 100.0).round() / 100.0;
                                    let line_total = rounded_price * quantity as f64 * nights as f64;
                                    let display_name = if meal_plan.is_empty() {
                                        room_name
                                    } else {
                                        format!("{room_name} ({meal_plan})")
                                    };
                                    view! {
                                        <div class="rounded-xl border border-gray-200 bg-white px-4 py-4 space-y-3">
                                            <div class="text-base font-semibold text-gray-900">{display_name}</div>
                                            <div class="grid grid-cols-2 gap-y-2 text-sm text-gray-700">
                                                <span>"Price Per Night"</span>
                                                <span class="text-right font-medium text-gray-900">{format_currency_with_code(price_per_night, &code)}</span>
                                                <span>"Total Rooms"</span>
                                                <span class="text-right font-medium text-gray-900">{format!("x{}", quantity)}</span>
                                                <span>"Total Nights"</span>
                                                <span class="text-right font-medium text-gray-900">{format!("x{}", nights)}</span>
                                                <span>"Sub total"</span>
                                                <span class="text-right font-semibold text-gray-900">{format_currency_with_code(line_total, &code)}</span>
                                            </div>
                                        </div>
                                    }
                                })
                                .collect_view()
                        }}
                    </div>

                    <div class="pt-2">
                        <div class="text-3xl font-semibold text-gray-900">
                            {move || format_currency_with_code(total_for_stay(), &currency_code.get())}
                        </div>
                        <p class="text-sm text-gray-600 mt-1">
                            {move || {
                                let nights = nights();
                                let rooms = total_selected_rooms();
                                let adults = adults_count();
                                format!(
                                    "{} Night{}, {} Room{}, {} Adult{}",
                                    nights,
                                    if nights == 1 { "" } else { "s" },
                                    rooms,
                                    if rooms == 1 { "" } else { "s" },
                                    adults,
                                    if adults == 1 { "" } else { "s" }
                                )
                            }}
                        </p>
                    </div>
                </div>
            </Show>

            <LoadingButton
                is_loading=booking_loading.into()
                loading_text="Processing..."
                on_click=on_book_now
                class="w-full bg-blue-600 hover:bg-blue-700 disabled:bg-gray-300 disabled:text-gray-600 disabled:hover:bg-gray-300 disabled:cursor-not-allowed text-white font-semibold py-3 px-6 rounded-xl transition-colors duration-200"
                disabled=button_disabled
            >
                "Continue Booking"
            </LoadingButton>
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
    view! {
        <div class="lg:sticky lg:top-24">
            <PricingBreakdownV1 />
        </div>
    }
}

#[component]
fn RoomRateRow(room_id: String, rate: DomainRoomOption) -> impl IntoView {
    let hotel_details_state: HotelDetailsUIState = expect_context();
    let ui_search_ctx: UISearchCtx = expect_context();

    let nights = move || {
        let nights = ui_search_ctx.date_range.get().no_of_nights();
        if nights == 0 {
            1
        } else {
            nights
        }
    };

    let currency_code = rate.price.currency_code.clone();

    // Calculate price per room per night
    let requested_rooms = ui_search_ctx.guests.rooms.get();
    let total_price_per_night = nightly_price_excluding_taxes(&rate, nights());
    // Ensure we handle case where requested_rooms might be 0 (though unlikely in valid state)
    let valid_rooms = if requested_rooms == 0 {
        1
    } else {
        requested_rooms
    };
    let base_price_per_night = total_price_per_night / valid_rooms as f64;

    let price_text = format_currency_with_code(base_price_per_night, &currency_code);
    let meal_plan = rate
        .meal_plan
        .clone()
        .unwrap_or_else(|| "Room Only".to_string());
    let occupancy = format_occupancy_text(rate.occupancy_info.as_ref());
    let room_key = room_id.clone();

    let selection_key = room_key.clone();
    let selection_offer_id = rate.room_data.offer_id.clone();
    let selection_count = create_memo(move |_| {
        let selected_rooms = hotel_details_state.selected_rooms.get();
        if !selection_offer_id.is_empty() {
            let available_options = HotelDetailsUIState::get_available_room_options();
            selected_rooms
                .iter()
                .filter_map(|(rate_key, qty)| {
                    available_options
                        .iter()
                        .find(|option| option.room_data.rate_key == *rate_key)
                        .filter(|option| option.room_data.offer_id == selection_offer_id)
                        .map(|_| *qty)
                })
                .sum::<u32>()
        } else {
            selected_rooms.get(&selection_key).copied().unwrap_or(0)
        }
    });

    let dec_key = room_key.clone();
    let inc_key = room_key.clone();
    let rooms_requested_signal = ui_search_ctx.guests.rooms;

    let offer_id = rate.room_data.offer_id.clone();
    let select_room = Action::new(move |_: &()| {
        if ENFORCE_SINGLE_ROOM_TYPE_BOOKING {
            // Multi-room selection logic: find all rooms in this offer
            let all_options = HotelDetailsUIState::get_available_room_options();
            let mut selections: HashMap<String, u32> = HashMap::new();

            for option in all_options {
                if option.room_data.offer_id == offer_id {
                    let key = option.room_data.rate_key.clone();
                    *selections.entry(key).or_insert(0) += 1;
                }
            }

            let selection_vec: Vec<(String, u32)> = selections.into_iter().collect();
            HotelDetailsUIState::set_multi_room_selection(selection_vec);
        } else {
            HotelDetailsUIState::increment_room_counter(room_key.clone());
        }
        async {}
    });
    let decrement = Action::new(move |_: &()| {
        if ENFORCE_SINGLE_ROOM_TYPE_BOOKING {
            let current = HotelDetailsUIState::get_selected_rooms()
                .get(&dec_key)
                .copied()
                .unwrap_or(0);
            let new_qty = 0;
            HotelDetailsUIState::set_single_room_selection(dec_key.clone(), new_qty);
        } else {
            HotelDetailsUIState::decrement_room_counter(dec_key.clone());
        }
        async {}
    });
    let increment = Action::new(move |_: &()| {
        if ENFORCE_SINGLE_ROOM_TYPE_BOOKING {
            if HotelDetailsUIState::can_increment_room_selection() {
                let current = HotelDetailsUIState::get_selected_rooms()
                    .get(&inc_key)
                    .copied()
                    .unwrap_or(0);
                HotelDetailsUIState::set_single_room_selection(inc_key.clone(), current + 1);
            }
        } else {
            HotelDetailsUIState::increment_room_counter(inc_key.clone());
        }
        async {}
    });

    let mut rate_details = Vec::new();
    let lower_meal = meal_plan.to_lowercase();
    let meal_desc = if lower_meal.contains("full board") || lower_meal.contains("(fb)") {
        "Breakfast, lunch, and dinner included"
    } else if lower_meal.contains("half board") || lower_meal.contains("(hb)") {
        "Breakfast and dinner included"
    } else if lower_meal.contains("breakfast") || lower_meal.contains("(bi)") {
        "Breakfast included"
    } else {
        "No meals included"
    };
    rate_details.push(meal_desc.to_string());
    rate_details.push("Non-Refundable".to_string());

    view! {
        <div class="flex flex-col md:grid md:grid-cols-[1.5fr_1fr_auto] md:items-stretch gap-4 md:gap-0">
            <div class="space-y-2 md:pr-6">
                <p class="text-base font-semibold text-gray-900">{meal_plan}</p>
                <ul class="list-disc list-inside text-sm text-gray-700 space-y-1">
                    <For
                        each=move || rate_details.clone()
                        key=|item| item.clone()
                        let:item
                    >
                        <li>{item}</li>
                    </For>
                </ul>
            </div>
            <div class="md:border-l md:border-gray-200 md:px-6 text-left md:text-center space-y-1 flex flex-col justify-center md:h-full">
                <p class="text-2xl font-semibold text-gray-900">{price_text}</p>
                <p class="text-[11px] text-gray-500">
                    // {move || {
                    //     let nights = nights();
                    //     format!(
                    //         "({} night{}, 1 Room excl. taxes (payable at property))",
                    //         nights,
                    //         if nights == 1 { "" } else { "s" }
                    //     )
                    // }}
                    (price per night)
                </p>
            </div>
            <div class="md:border-l md:border-gray-200 md:pl-6 flex items-center justify-start md:justify-end w-full md:h-full">
                <Show
                    when=move || selection_count.get() == 0
                    fallback=move ||
                        view! {
                            <div class="inline-flex items-center overflow-hidden rounded-lg border border-blue-100 bg-blue-50 text-blue-700">
                                <button
                                    class="px-3 py-2 text-lg hover:bg-blue-100 disabled:opacity-50 disabled:cursor-not-allowed"
                                    disabled=move || selection_count.get() == 0
                                    on:click=move|_|decrement.dispatch(())
                                >
                                    "−"
                                </button>
                                <span class="px-3 text-sm font-semibold">{move || selection_count.get()}</span>
                                <button
                                    class="px-3 py-2 text-lg hover:bg-blue-100 disabled:opacity-50 disabled:cursor-not-allowed"
                                    disabled=move || HotelDetailsUIState::is_at_room_selection_limit()
                                    on:click=move|_|increment.dispatch(())
                                >
                                    "+"
                                </button>
                            </div>
                        }

                >
                    <button
                        class="inline-flex items-center justify-center rounded-xl bg-blue-600 px-5 py-2.5 text-sm font-semibold text-white hover:bg-blue-700 transition-colors duration-150 w-full md:w-auto"
                        on:click=move|_| {
                            select_room.dispatch(());
                            set_timeout(move || {
                                if let Some(element) = document().get_element_by_id("cart-section") {
                                    element.scroll_into_view();
                                }
                            }, Duration::from_secs(1));
                        }
                    >
                        "Select Room"
                    </button>
                </Show>
            </div>
        </div>
    }
}

#[component]
fn RoomTypeCard(
    room_group: RoomCard,
    fallback_image: Option<String>,
    amenity_preview: Vec<Amenity>,
    room_details: Option<DomainStaticRoom>,
    #[prop(optional)] is_recommended: bool,
) -> impl IntoView {
    let open_image_viewer = RwSignal::new(false);
    let RoomCard {
        mapped_room_id: _,
        room_names,
        card_title,
        rates,
    } = room_group;

    let rates_for_render = rates.clone();

    let room_images = room_details
        .as_ref()
        .map(|details| details.photos.clone())
        .unwrap_or_default();

    let hero_image = room_details
        .as_ref()
        .and_then(|details| details.photos.first().cloned())
        .or_else(|| fallback_image.clone())
        .unwrap_or_else(|| "/img/home.png".to_string());

    let occupancy_from_rates = rates_for_render
        .iter()
        .find_map(|rate| rate.occupancy_info.as_ref());
    let occupancy_summary = room_details.as_ref().and_then(|details| {
        details.max_occupancy.map(|occ| {
            let adults = details.max_adults.unwrap_or(occ);
            let children = details.max_children.unwrap_or(0);
            if children > 0 {
                format!("Sleeps {occ} · {adults} adults + {children} children")
            } else {
                format!("Sleeps {occ} · {adults} adults")
            }
        })
    });
    let occupancy_text = if let Some(info) = occupancy_from_rates {
        format_occupancy_text(Some(info))
    } else if let Some(summary) = occupancy_summary.clone() {
        summary
    } else {
        "Sleeps up to 2 guests".to_string()
    };

    let room_specific_amenities = room_details
        .as_ref()
        .map(|details| convert_to_amenities(details.amenities.clone()))
        .unwrap_or_default();

    // Store all amenities for modal display
    let all_room_amenities = store_value(if room_specific_amenities.is_empty() {
        amenity_preview.clone()
    } else {
        room_specific_amenities.clone()
    });

    let amenities_for_render = if room_specific_amenities.is_empty() {
        amenity_preview.into_iter().take(5).collect::<Vec<_>>()
    } else {
        room_specific_amenities
            .into_iter()
            .take(5)
            .collect::<Vec<_>>()
    };

    let room_display_name = if card_title.is_empty() {
        if room_names.len() == 1 {
            room_names[0].clone()
        } else {
            room_names.join(" + ")
        }
    } else {
        card_title.clone()
    };
    let room_display_name_for_modal = room_display_name.clone();

    let room_size_text = room_details.as_ref().and_then(|details| {
        details.room_size_square.map(|size| {
            let unit = details
                .room_size_unit
                .clone()
                .unwrap_or_else(|| "m²".to_string());
            format!("{size:.0} {unit}")
        })
    });
    let room_size_text_for_modal = store_value(room_size_text.clone());

    let bed_text = room_details
        .as_ref()
        .and_then(|details| details.bed_types.first().cloned());
    let bed_text_for_modal = store_value(bed_text.clone());

    let occupancy_text_for_modal = store_value(occupancy_text.clone());

    let quick_facts: Vec<(usize, icondata::Icon, String)> = {
        let mut facts: Vec<(icondata::Icon, String)> = Vec::new();
        if let Some(size) = room_size_text.clone() {
            facts.push((icondata::FaRulerCombinedSolid, size));
        }
        facts.push((icondata::BiUserRegular, occupancy_text.clone()));
        if let Some(bed) = bed_text {
            facts.push((icondata::FaBedSolid, bed));
        }
        facts
            .into_iter()
            .enumerate()
            .map(|(idx, (icon, label))| (idx, icon, label))
            .collect()
    };

    // Modal state for room details
    let show_room_details_modal = create_rw_signal(false);
    let room_images_for_modal = store_value(room_images.clone());
    let total_amenities_count = all_room_amenities.get_value().len();

    view! {
        {move || open_image_viewer.get().then(|| {
            let room_images = room_images.clone();
            view! {
                <ImageLightbox
                    images=room_images
                    initial_index=0
                    loop_images=true
                    on_close=Callback::new(move |_| open_image_viewer.set(false))
                />
            }
        })}

        // Room Details Modal
        <Show when=move || show_room_details_modal.get()>
            <div class="fixed inset-0 z-50 flex items-center justify-center">
                // Backdrop
                <div
                    class="absolute inset-0 bg-black/60 backdrop-blur-sm"
                    on:click=move |_| show_room_details_modal.set(false)
                ></div>

                // Modal content
                <div class="relative z-10 w-[95vw] max-w-3xl max-h-[90vh] bg-white rounded-2xl shadow-2xl overflow-hidden flex flex-col">
                    // Header
                    <div class="flex items-center justify-between px-6 py-4 border-b border-gray-200 bg-gray-50">
                        <h3 class="font-semibold text-gray-900 text-lg">{room_display_name_for_modal.clone()}</h3>
                        <button
                            type="button"
                            class="p-2 rounded-full hover:bg-gray-200 transition-colors cursor-pointer"
                            on:click=move |_| show_room_details_modal.set(false)
                            aria-label="Close"
                        >
                            <svg class="w-6 h-6 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
                            </svg>
                        </button>
                    </div>

                    // Content
                    <div class="flex-1 overflow-y-auto p-6 space-y-6">
                        // Room images preview
                        <Show when=move || { !room_images_for_modal.get_value().is_empty() }>
                            <div class="flex gap-2 overflow-x-auto pb-2">
                                {move || room_images_for_modal.get_value().iter().take(4).cloned().collect::<Vec<_>>().into_iter().map(|img| {
                                    view! {
                                        <div class="flex-shrink-0 w-32 h-24 rounded-lg overflow-hidden bg-gray-100">
                                            <img src=img class="w-full h-full object-cover"/>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        </Show>

                        // Room Facts
                        <div class="space-y-3">
                            <h4 class="text-sm font-semibold text-gray-800">Room Details</h4>
                            <div class="flex flex-wrap gap-4">
                                <Show when=move || room_size_text_for_modal.get_value().is_some()>
                                    <span class="inline-flex items-center gap-2 text-sm text-gray-700 bg-gray-100 px-3 py-1.5 rounded-full">
                                        <Icon class="text-blue-500" icon=icondata::FaRulerCombinedSolid/>
                                        {move || room_size_text_for_modal.get_value().unwrap_or_default()}
                                    </span>
                                </Show>
                                <span class="inline-flex items-center gap-2 text-sm text-gray-700 bg-gray-100 px-3 py-1.5 rounded-full">
                                    <Icon class="text-blue-500" icon=icondata::BiUserRegular/>
                                    {move || occupancy_text_for_modal.get_value()}
                                </span>
                                <Show when=move || bed_text_for_modal.get_value().is_some()>
                                    <span class="inline-flex items-center gap-2 text-sm text-gray-700 bg-gray-100 px-3 py-1.5 rounded-full">
                                        <Icon class="text-blue-500" icon=icondata::FaBedSolid/>
                                        {move || bed_text_for_modal.get_value().unwrap_or_default()}
                                    </span>
                                </Show>
                            </div>
                        </div>

                        // All Amenities
                        <div class="space-y-3">
                            <h4 class="text-sm font-semibold text-gray-800">{format!("Room Amenities ({})", total_amenities_count)}</h4>
                            <div class="grid grid-cols-1 sm:grid-cols-2 gap-x-6 gap-y-2">
                                <For each=move || all_room_amenities.get_value() key=|a| a.text.clone() let:amenity>
                                    <div class="inline-flex items-center text-gray-700">
                                        <Icon class="w-5 h-5 text-blue-500 mr-3 flex-shrink-0" icon=amenity.icon/>
                                        <span class="text-sm">{amenity.text}</span>
                                    </div>
                                </For>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </Show>
        <div class="bg-[#f9f9f9] border border-gray-200 rounded-2xl shadow-sm overflow-hidden">
            <div class="px-5 pt-5 pb-0">
                <div class="flex items-center justify-between gap-3">
                    <h3 class="text-xl font-semibold text-gray-900">{room_display_name}</h3>
                    <Show when=move || is_recommended>
                        <span class="inline-flex items-center gap-1 rounded-full bg-blue-50 text-blue-700 text-xs font-semibold px-3 py-1">
                            <Icon class="text-sm" icon=icondata::FaThumbsUpSolid />
                            "Recommended"
                        </span>
                    </Show>
                </div>

                <div class="mt-5 flex flex-col lg:grid lg:grid-cols-[320px_1fr] items-start gap-5">
                    <button
                        type="button"
                        class="w-full text-left"
                        on:click=move |_| open_image_viewer.set(true)>
                        <div class="w-full h-48 md:h-56 rounded-xl overflow-hidden bg-gray-100 shadow-sm">
                            <img
                                src=hero_image.clone()
                                // alt={format!("{} photo", room_display_name)}
                                class="w-full h-full object-cover"
                            />
                        </div>
                    </button>

                    <div class="w-full flex flex-col gap-5">
                        <div class="space-y-3">
                            <p class="text-sm font-semibold text-gray-800">"Room Details"</p>
                            <div class="flex flex-wrap items-center gap-4">
                                <For
                                    each=move || quick_facts.clone()
                                    key=|(idx, _, _)| *idx
                                    let:fact
                                >
                                    {let (_, icon, label) = fact;
                                    view! {
                                        <span class="inline-flex items-center gap-2 text-sm text-gray-700">
                                            <Icon class="text-blue-500 text-base" icon=icon />
                                            {label}
                                        </span>
                                    }}
                                </For>
                            </div>
                        </div>
                        <div class="space-y-2">
                            <p class="text-sm font-semibold text-gray-800">"Amenities"</p>
                            <div class="flex flex-wrap items-center gap-3 text-sm text-gray-700">
                                <For
                                    each=move || amenities_for_render.clone()
                                    key=|amenity| amenity.text.clone()
                                    let:amenity
                                >
                                    <span class="inline-flex items-center gap-2">
                                        <Icon class="text-blue-500 text-sm" icon=amenity.icon />
                                        {amenity.text.clone()}
                                    </span>
                                </For>
                                <button
                                    type="button"
                                    class="text-sm font-semibold text-blue-600 hover:underline cursor-pointer"
                                    on:click=move |_| show_room_details_modal.set(true)
                                >
                                    "See All Details"
                                </button>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <div class="mt-5 border-t border-gray-200">
                <For
                    each=move || rates_for_render.clone()
                    key=|rate| rate.room_data.rate_key.clone()
                    let:rate
                >
                    <div class="px-5 py-4 border-b border-gray-200 last:border-b-0 bg-white">
                        <RoomRateRow
                            room_id=rate.room_data.rate_key.clone()
                            rate=rate
                        />
                    </div>
                </For>
        </div>
        </div>
    }
}

#[component]
pub fn SelectRoomSection() -> impl IntoView {
    let hotel_details_state: HotelDetailsUIState = expect_context();

    let total_room_types_state = hotel_details_state.clone();
    let total_room_types = move || grouped_rooms_for_state(&total_room_types_state).len();

    let total_offers_state = hotel_details_state.clone();
    let total_offers = move || {
        grouped_rooms_for_state(&total_offers_state)
            .iter()
            .map(|card| card.rates.len())
            .sum::<usize>()
    };

    view! {
        <section class="w-full max-w-7xl mx-auto px-4 mt-10">
            <div class="flex flex-col md:flex-row md:items-end md:justify-between gap-3">
                <SectionTitle id="rooms" title="Select A Room" />
                <p class="text-sm text-gray-500">
                    {move || format!("{} Room types | {} Offers", total_room_types(), total_offers())}
                </p>
            </div>
            <div class="mt-6 grid lg:grid-cols-[minmax(0,2fr)_minmax(320px,1fr)] gap-6 items-start">
                <div class="space-y-6">
                    <div>
                        <Show
                            when={
                                let state = hotel_details_state.clone();
                                move || !grouped_rooms_for_state(&state).is_empty()
                            }
                            fallback=move || {
                                if hotel_details_state.rates_loading.get() {
                                    view! { <RoomSelectionSkeleton /> }.into_view()
                                } else {
                                    view! {
                                        <div class="bg-white rounded-2xl border border-dashed border-gray-200 p-6 text-center text-gray-500">
                                            "No rooms available for the selected dates."
                                        </div>
                                    }
                                    .into_view()
                                }
                            }
                        >
                            <div class="space-y-6">
                                {
                                    let state = hotel_details_state.clone();
                                move || {
                                    let fallback = fallback_image_for_state(&state);
                                    let amenities = amenity_preview_for_state(&state);
                                    let room_lookup = Arc::new(room_details_lookup_for_state(&state));
                                    grouped_rooms_for_state(&state)
                                        .into_iter()
                                        .enumerate()
                                        .map(|(idx, group)| {
                                            let fallback_clone = fallback.clone();
                                            let amenities_clone = amenities.clone();
                                            let lookup = room_lookup.clone();
                                            let room_details = group
                                                .mapped_room_id
                                                .and_then(|id| lookup.by_id.get(&id).cloned())
                                                .or_else(|| {
                                                    group
                                                        .room_names
                                                        .iter()
                                                        .find_map(|name| {
                                                            lookup
                                                                .by_name
                                                                .get(&normalized_room_key(name))
                                                                .cloned()
                                                        })
                                                });
                                            view! {
                                                <RoomTypeCard
                                                    room_group=group
                                                    fallback_image=fallback_clone.clone()
                                                    amenity_preview=amenities_clone.clone()
                                                    room_details=room_details
                                                    is_recommended=idx == 0
                                                />
                                            }
                                        })
                                        .collect_view()
                                }
                                }
                            </div>
                        </Show>
                    </div>
                </div>

                <PricingBookNowV1 />
            </div>
        </section>
    }
}

fn rating_label_for_score(score: f64) -> &'static str {
    if score >= 9.0 {
        "Superb"
    } else if score >= 8.5 {
        "Excellent"
    } else if score >= 8.0 {
        "Very Good"
    } else if score >= 7.0 {
        "Good"
    } else if score >= 6.0 {
        "Pleasant"
    } else {
        "Okay"
    }
}

#[component]
pub fn GuestReviewsSection() -> impl IntoView {
    let hotel_details_state: HotelDetailsUIState = expect_context();
    let show_all_categories = create_rw_signal(false);

    let summary_score = hotel_details_state
        .static_details
        .get()
        .and_then(|d| d.rating)
        .unwrap_or(7.1);
    let summary_label = rating_label_for_score(summary_score);
    let total_reviews = hotel_details_state
        .static_details
        .get()
        .and_then(|d| d.review_count)
        .unwrap_or(1136);
    let review_cards = SAMPLE_REVIEWS.to_vec();
    let categories_from_api = hotel_details_state
        .static_details
        .get()
        .map(|d| d.categories.clone())
        .unwrap_or_default();
    let mut categories = if categories_from_api.is_empty() {
        REVIEW_CATEGORY_SCORES
            .iter()
            .map(|(name, score)| (name.to_string(), *score))
            .collect::<Vec<_>>()
    } else {
        categories_from_api
            .into_iter()
            .map(|c| (c.name, c.rating))
            .collect::<Vec<_>>()
    };
    categories.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Store all categories and create preview
    let all_categories = store_value(categories.clone());
    let total_categories_count = all_categories.get_value().len();
    let preview_categories: Vec<(String, f32)> = categories.into_iter().take(6).collect();

    let highlight_tag_vec = {
        let cats = all_categories.get_value();
        if !cats.is_empty() {
            cats.iter()
                .take(3)
                .map(|(name, _)| name.clone())
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        }
    };

    view! {
        <section class="w-full max-w-7xl mx-auto px-4 mt-12">
            <SectionTitle id="reviews" title="Guest Reviews" />
            <div class="mt-6 gap-6 lg:grid-cols-[minmax(0,1.3fr)_minmax(0,1fr)]">
                <div class="bg-white rounded-2xl shadow-sm border border-gray-100 p-6 space-y-6">
                    <div class="flex items-start gap-4 md:gap-6">
                        <div class="flex items-center justify-center bg-yellow-400 text-white font-semibold text-xl w-12 h-12 rounded-lg">
                            {format!("{summary_score:.1}")}
                        </div>
                        <div>
                            <p class="text-lg font-semibold text-gray-900">{summary_label}</p>
                            <p class="text-sm text-gray-500">"Based on " {total_reviews} " reviews"</p>
                        </div>
                    </div>

                    <div class="space-y-3">
                        <div class="flex items-center justify-between">
                            <p class="text-sm font-semibold text-gray-900">"Categories"</p>
                            <Show when=move || { total_categories_count > 6 }>
                                <button
                                    class="text-sm font-semibold text-blue-600 hover:underline cursor-pointer flex items-center gap-1"
                                    type="button"
                                    on:click=move |_| show_all_categories.set(true)
                                >
                                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"/>
                                    </svg>
                                    {format!("Show All ({})", total_categories_count)}
                                </button>
                            </Show>
                        </div>
                        <div class="grid gap-3 md:grid-cols-3">
                            {
                                move || {
                                    preview_categories
                                        .clone()
                                        .into_iter()
                                        .map(|(label, score)| {
                                            let percent = (score / 10.0 * 100.0).clamp(0.0, 100.0);
                                            view! {
                                                <div class="space-y-1">
                                                    <div class="flex items-center justify-between text-sm text-gray-700">
                                                        <span>{label}</span>
                                                        <span>{format!("{score:.1}")}</span>
                                                    </div>
                                                    <div class="h-2 rounded-full bg-gray-100 overflow-hidden">
                                                        <div
                                                            class="h-full bg-blue-500 rounded-full"
                                                            style=move || format!("width: {percent:.0}%;")
                                                        ></div>
                                                    </div>
                                                </div>
                                            }
                                        })
                                        .collect_view()
                                }
                            }
                        </div>
                    </div>
                    <div class="flex flex-wrap gap-2">
                        {move || {
                            highlight_tag_vec
                                .clone()
                                .into_iter()
                                .map(|tag| {
                                    view! {
                                        <span class="px-3 py-1 rounded-full bg-blue-50 text-blue-700 text-xs font-medium">{tag}</span>
                                    }
                                })
                                .collect_view()
                        }}
                    </div>
                </div>

                // Categories Modal
                <Show when=move || show_all_categories.get()>
                    <div class="fixed inset-0 z-50 flex items-center justify-center">
                        // Backdrop
                        <div
                            class="absolute inset-0 bg-black/60 backdrop-blur-sm"
                            on:click=move |_| show_all_categories.set(false)
                        ></div>

                        // Modal content
                        <div class="relative z-10 w-[95vw] max-w-2xl max-h-[85vh] bg-white rounded-2xl shadow-2xl overflow-hidden flex flex-col">
                            // Header
                            <div class="flex items-center justify-between px-6 py-4 border-b border-gray-200 bg-gray-50">
                                <div class="flex items-center gap-3">
                                    <div class="flex items-center justify-center bg-yellow-400 text-white font-semibold text-lg w-10 h-10 rounded-lg">
                                        {format!("{summary_score:.1}")}
                                    </div>
                                    <div>
                                        <span class="font-semibold text-gray-800 text-lg">{format!("All Categories ({})", total_categories_count)}</span>
                                    </div>
                                </div>
                                <button
                                    type="button"
                                    class="p-2 rounded-full hover:bg-gray-200 transition-colors cursor-pointer"
                                    on:click=move |_| show_all_categories.set(false)
                                    aria-label="Close"
                                >
                                    <svg class="w-6 h-6 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
                                    </svg>
                                </button>
                            </div>

                            // Categories grid
                            <div class="flex-1 overflow-y-auto p-6">
                                <div class="grid gap-4 md:grid-cols-2">
                                    <For each=move || all_categories.get_value() key=|(label, _)| label.clone() let:item>
                                        {
                                            let (label, score) = item;
                                            let percent = (score / 10.0 * 100.0).clamp(0.0, 100.0);
                                            view! {
                                                <div class="space-y-1">
                                                    <div class="flex items-center justify-between text-sm text-gray-700">
                                                        <span class="font-medium">{label}</span>
                                                        <span class="font-semibold text-gray-900">{format!("{score:.1}")}</span>
                                                    </div>
                                                    <div class="h-2.5 rounded-full bg-gray-100 overflow-hidden">
                                                        <div
                                                            class="h-full bg-blue-500 rounded-full transition-all duration-300"
                                                            style=format!("width: {percent:.0}%;")
                                                        ></div>
                                                    </div>
                                                </div>
                                            }
                                        }
                                    </For>
                                </div>
                            </div>
                        </div>
                    </div>
                </Show>
                // <div class="space-y-4">
                //     {
                //         move || {
                //             review_cards
                //                 .clone()
                //                 .into_iter()
                //                 .map(|review| {
                //                     view! {
                //                         <div class="bg-white rounded-2xl border border-gray-100 p-5 space-y-3">
                //                             <div class="flex items-center justify-between">
                //                                 <div>
                //                                     <p class="font-semibold text-gray-900">{review.name}</p>
                //                                     <p class="text-xs text-gray-500">{review.stay_info}</p>
                //                                 </div>
                //                                 <span class="px-3 py-1 rounded-full bg-blue-50 text-blue-700 text-sm font-semibold">{format!("{:.1}", review.rating)}</span>
                //                             </div>
                //                             <p class="text-sm font-semibold text-gray-800">{review.title}</p>
                //                             <p class="text-sm text-gray-600 leading-6">{review.body}</p>
                //                             <div class="flex flex-wrap gap-2">
                //                                 {review.tags.iter().map(|tag| view! {
                //                                     <span class="text-xs text-gray-500 border border-gray-200 rounded-full px-3 py-1">{*tag}</span>
                //                                 }).collect_view()}
                //                             </div>
                //                             <button class="text-sm text-blue-600 hover:underline">"Read More"</button>
                //                         </div>
                //                     }
                //                 })
                //                 .collect_view()
                //         }
                //     }
                //     <button class="w-full rounded-xl border border-gray-200 py-3 text-sm font-semibold text-blue-600 hover:border-blue-400">
                //         "View All Reviews"
                //     </button>
                // </div>
            </div>
        </section>
    }
}

#[component]
pub fn PolicyRulesSection(#[prop(into)] address: String) -> impl IntoView {
    let hotel_details_state: HotelDetailsUIState = expect_context();
    let static_details = hotel_details_state.static_details.get();
    let static_details_clone = static_details.clone();
    let policies = Memo::new(move |_| {
        static_details
            .as_ref()
            .map(|d| d.policies.clone())
            .unwrap_or_default()
    });

    let is_policies_empty = move || policies.get().is_empty();
    let check_times = static_details_clone
        .as_ref()
        .and_then(|d| d.checkin_checkout_times.clone());

    view! {
        <section class="w-full max-w-7xl mx-auto px-4 mt-12">
            <SectionTitle id="rules" title="Policy & Rules" />
            <div class="mt-4 grid gap-6 md:grid-cols-2">
                <div class="bg-white rounded-2xl border border-gray-100 p-6 space-y-4">
                    <p class="text-gray-700 leading-7">
                        "Please review the key policies for this property before confirming your stay. "
                        "Property address: " {address.clone()} "."
                    </p>
                    <Show
                        when=move || !is_policies_empty()
                        fallback=|| view! {
                            <ul class="space-y-3 text-sm text-gray-700">
                                <li>
                                    <span class="font-semibold text-gray-900">"Children & extra beds: "</span>
                                    "Children are welcome. Extra beds depend on the room you choose; please check the individual room capacity."
                                </li>
                                <li>
                                    <span class="font-semibold text-gray-900">"Pets: "</span>
                                    "Pets are not allowed at this property."
                                </li>
                                <li>
                                    <span class="font-semibold text-gray-900">"Payment methods: "</span>
                                    "Major cards and digital payments accepted at the front desk."
                                </li>
                            </ul>
                        }
                    >
                        <div class="space-y-3">
                            {move || {
                                use std::collections::HashMap;

                                // Group policies by name
                                let mut grouped: HashMap<String, Vec<String>> = HashMap::new();
                                for policy in policies.get() {
                                    grouped.entry(policy.name.clone())
                                        .or_insert_with(Vec::new)
                                        .push(if policy.description.trim().is_empty() {
                                            "Details not provided.".to_string()
                                        } else {
                                            policy.description.clone()
                                        });
                                }

                                // Convert to sorted vec for consistent ordering
                                let mut policy_groups: Vec<(String, Vec<String>)> = grouped.into_iter().collect();
                                policy_groups.sort_by(|a, b| a.0.cmp(&b.0));

                                policy_groups.into_iter().map(|(name, descriptions)| {
                                    view! {
                                        <div>
                                            <p class="font-semibold text-gray-900 text-sm">{name}</p>
                                            <ul class="space-y-1">
                                                {descriptions.into_iter().map(|desc| {
                                                    view! {
                                                        <li class="text-sm text-gray-700 whitespace-pre-line">
                                                            {desc}
                                                        </li>
                                                    }
                                                }).collect_view()}
                                            </ul>
                                        </div>
                                    }
                                }).collect_view()
                            }}
                        </div>
                    </Show>
                </div>
                <div class="bg-white rounded-2xl border border-gray-100 p-6 space-y-4">
                    <div>
                        <p class="text-sm font-semibold text-gray-900 uppercase tracking-wide">
                            "Check-in / Check-out"
                        </p>
                        <p class="text-sm text-gray-700 mt-1">
                            {move || {
                                if let Some(times) = check_times.clone() {
                                    let checkin = if times.checkin.is_empty() { "03:00 PM".to_string() } else { times.checkin };
                                    let checkout = if times.checkout.is_empty() { "12:00 PM".to_string() } else { times.checkout };
                                    format!("Check-in from {checkin} · Check-out until {checkout}")
                                } else {
                                    "Check-in from 03:00 PM · Check-out until 12:00 PM".to_string()
                                }
                            }}
                        </p>
                    </div>
                    <div>
                        <p class="text-sm font-semibold text-gray-900 uppercase tracking-wide">
                            "Important Info"
                        </p>
                        <p class="text-sm text-gray-700 mt-1">
                            "Policies vary by room type and rate plan. Please review specific rate details before booking."
                        </p>
                    </div>
                    <div>
                        <p class="text-sm font-semibold text-gray-900 uppercase tracking-wide">
                            "Cancellation / Prepayment"
                        </p>
                        <p class="text-sm text-gray-700 mt-1">
                            "Cancellation and prepayment policies vary according to the room rate selected. Review the plan before confirming."
                        </p>
                    </div>
                </div>
            </div>
        </section>
    }
}

#[component]
pub fn SiteFooter() -> impl IntoView {
    view! {
        // <footer class="mt-16 border-t border-gray-200 py-10">
        //     <div class="w-full max-w-7xl mx-auto px-4 flex flex-col md:flex-row md:items-center md:justify-between gap-3 text-sm text-gray-500">
        //         <span>"Copyright © 2024 EstateDAO. All Rights Reserved."</span>
        //         <div class="flex items-center gap-4">
        //             <a href="#" class="hover:text-blue-600">"Twitter"</a>
        //             <a href="#" class="hover:text-blue-600">"Facebook"</a>
        //             <a href="#" class="hover:text-blue-600">"Instagram"</a>
        //         </div>
        //     </div>
        // </footer>
        <div class="mt-16 border-t border-gray-200">
            <Footer />
        </div>
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        DomainCurrencyAmount, DomainDetailedPrice, DomainPrice, DomainRoomData, DomainRoomOption,
    };

    // Helper function to create a test DomainRoomOption
    fn create_test_room_option(
        mapped_room_id: u32,
        room_name: &str,
        meal_plan: Option<&str>,
        price: f64,
        rate_key: &str,
    ) -> DomainRoomOption {
        DomainRoomOption {
            mapped_room_id,
            price: DomainDetailedPrice {
                published_price: price,
                published_price_rounded_off: price,
                offered_price: price,
                offered_price_rounded_off: price,
                suggested_selling_price: price,
                suggested_selling_price_rounded_off: price,
                room_price: price,
                tax: 0.0,
                extra_guest_charge: 0.0,
                child_charge: 0.0,
                other_charges: 0.0,
                currency_code: "USD".to_string(),
            },
            tax_lines: vec![],
            offer_retail_rate: Some(DomainCurrencyAmount {
                amount: price,
                currency_code: "USD".to_string(),
            }),
            room_data: DomainRoomData {
                mapped_room_id,
                occupancy_number: Some(1),
                room_name: room_name.to_string(),
                room_unique_id: "test_unique_id".to_string(),
                rate_key: rate_key.to_string(),
                offer_id: "test_offer".to_string(),
            },
            meal_plan: meal_plan.map(String::from),
            occupancy_info: Some(DomainRoomOccupancy {
                max_occupancy: Some(2),
                adult_count: Some(1),
                child_count: Some(0),
            }),
            cancellation_policies: None,
            // perks: vec![],
            promotions: None,
            remarks: None,
        }
    }

    #[test]
    fn test_dedup_rates_by_meal_plan_removes_duplicates() {
        // Create rates with duplicate "Room Only" meal plans but different prices
        let rates = vec![
            create_test_room_option(1, "Rover Room", Some("Room Only (RO)"), 79.43, "rate1"),
            create_test_room_option(1, "Rover Room", Some("Room Only (RO)"), 76.46, "rate2"),
            create_test_room_option(
                1,
                "Rover Room",
                Some("Breakfast Included (BI)"),
                99.09,
                "rate3",
            ),
        ];

        let deduped = dedup_rates_by_meal_plan(&rates);

        // Should have only 2 rates: 1 Room Only (lowest price) + 1 Breakfast
        assert_eq!(
            deduped.len(),
            2,
            "Should deduplicate to 2 unique meal plans"
        );

        // Find the Room Only rate
        let room_only = deduped
            .iter()
            .find(|r| {
                r.meal_plan
                    .as_ref()
                    .map_or(false, |mp| mp.contains("Room Only"))
            })
            .expect("Should have Room Only rate");

        // Should keep the lowest price (76.46)
        assert_eq!(
            room_only.price.room_price, 76.46,
            "Should keep the lowest price for Room Only"
        );

        // Find the Breakfast rate
        let breakfast = deduped
            .iter()
            .find(|r| {
                r.meal_plan
                    .as_ref()
                    .map_or(false, |mp| mp.contains("Breakfast"))
            })
            .expect("Should have Breakfast rate");

        assert_eq!(
            breakfast.price.room_price, 99.09,
            "Should keep the Breakfast rate"
        );
    }

    #[test]
    fn test_dedup_rates_by_meal_plan_handles_none_meal_plan() {
        // Test with None meal plans (should default to "Room Only")
        let rates = vec![
            create_test_room_option(1, "Test Room", None, 100.0, "rate1"),
            create_test_room_option(1, "Test Room", None, 80.0, "rate2"),
        ];

        let deduped = dedup_rates_by_meal_plan(&rates);

        // Should have only 1 rate with the lowest price
        assert_eq!(deduped.len(), 1, "Should deduplicate None meal plans");
        assert_eq!(
            deduped[0].price.room_price, 80.0,
            "Should keep the lowest price"
        );
    }

    #[test]
    fn test_dedup_rates_by_meal_plan_sorts_by_price() {
        let rates = vec![
            create_test_room_option(1, "Test Room", Some("Full Board (FB)"), 145.05, "rate1"),
            create_test_room_option(1, "Test Room", Some("Room Only (RO)"), 79.43, "rate2"),
            create_test_room_option(
                1,
                "Test Room",
                Some("Breakfast Included (BI)"),
                99.09,
                "rate3",
            ),
            create_test_room_option(1, "Test Room", Some("Half Board (HB)"), 122.07, "rate4"),
        ];

        let deduped = dedup_rates_by_meal_plan(&rates);

        // Should be sorted by price (ascending)
        assert_eq!(deduped.len(), 4, "Should have 4 unique meal plans");
        assert_eq!(
            deduped[0].price.room_price, 79.43,
            "First should be Room Only (cheapest)"
        );
        assert_eq!(
            deduped[1].price.room_price, 99.09,
            "Second should be Breakfast"
        );
        assert_eq!(
            deduped[2].price.room_price, 122.07,
            "Third should be Half Board"
        );
        assert_eq!(
            deduped[3].price.room_price, 145.05,
            "Fourth should be Full Board"
        );
    }

    #[test]
    fn test_dedup_rates_by_meal_plan_multiple_duplicates_same_meal_plan() {
        // Test with multiple duplicates of the same meal plan
        let rates = vec![
            create_test_room_option(1, "Test Room", Some("Room Only (RO)"), 250.14, "rate1"),
            create_test_room_option(1, "Test Room", Some("Room Only (RO)"), 250.8, "rate2"),
            create_test_room_option(1, "Test Room", Some("Room Only (RO)"), 259.79, "rate3"),
            create_test_room_option(1, "Test Room", Some("Room Only (RO)"), 245.0, "rate4"), // Lowest
        ];

        let deduped = dedup_rates_by_meal_plan(&rates);

        assert_eq!(deduped.len(), 1, "Should have only 1 rate");
        assert_eq!(
            deduped[0].price.room_price, 245.0,
            "Should keep the absolute lowest price"
        );
    }

    #[test]
    fn test_build_room_cards_deduplicates_type_a_rates() {
        // Create an OfferGroup representing a TYPE A card (same mapped_room_id)
        let offer1 = OfferGroup {
            offer_id: "offer1".to_string(),
            mapped_room_id: Some(1321373764),
            rates: vec![create_test_room_option(
                1321373764,
                "Rover Room",
                Some("Room Only (RO)"),
                79.43,
                "rate1",
            )],
            room_names: vec!["Rover Room".to_string()],
        };

        let offer2 = OfferGroup {
            offer_id: "offer2".to_string(),
            mapped_room_id: Some(1321373764),
            rates: vec![create_test_room_option(
                1321373764,
                "Rover Room",
                Some("Room Only (RO)"),
                76.46,
                "rate2",
            )],
            room_names: vec!["Rover Room".to_string()],
        };

        let offers = vec![offer1, offer2];
        let cards = build_room_cards(offers);

        // Should have only 1 card for this room type
        assert_eq!(cards.len(), 1, "Should have 1 TYPE A card");

        let card = &cards[0];
        assert_eq!(
            card.mapped_room_id,
            Some(1321373764),
            "Should have correct mapped_room_id"
        );

        // The card should have only 1 rate (deduplicated)
        assert_eq!(
            card.rates.len(),
            1,
            "Should have only 1 rate after deduplication"
        );

        // Should keep the lowest price
        assert_eq!(
            card.rates[0].price.room_price, 76.46,
            "Should keep the lowest price"
        );
    }

    #[test]
    fn test_build_room_cards_preserves_different_meal_plans() {
        // Create offers with same room but different meal plans
        let offer1 = OfferGroup {
            offer_id: "offer1".to_string(),
            mapped_room_id: Some(1321373764),
            rates: vec![create_test_room_option(
                1321373764,
                "Rover Room",
                Some("Room Only (RO)"),
                79.43,
                "rate1",
            )],
            room_names: vec!["Rover Room".to_string()],
        };

        let offer2 = OfferGroup {
            offer_id: "offer2".to_string(),
            mapped_room_id: Some(1321373764),
            rates: vec![create_test_room_option(
                1321373764,
                "Rover Room",
                Some("Breakfast Included (BI)"),
                99.09,
                "rate2",
            )],
            room_names: vec!["Rover Room".to_string()],
        };

        let offers = vec![offer1, offer2];
        let cards = build_room_cards(offers);

        assert_eq!(cards.len(), 1, "Should have 1 TYPE A card");

        let card = &cards[0];
        // Should have 2 rates (different meal plans)
        assert_eq!(
            card.rates.len(),
            2,
            "Should have 2 rates with different meal plans"
        );

        // Verify both meal plans are present
        let meal_plans: Vec<String> = card
            .rates
            .iter()
            .filter_map(|r| r.meal_plan.clone())
            .collect();

        assert!(
            meal_plans.iter().any(|mp| mp.contains("Room Only")),
            "Should have Room Only"
        );
        assert!(
            meal_plans.iter().any(|mp| mp.contains("Breakfast")),
            "Should have Breakfast"
        );
    }
}
