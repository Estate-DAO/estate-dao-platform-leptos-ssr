use leptos::*;
use leptos_icons::Icon;
use leptos_router::{use_navigate, use_query_map};

use crate::api::client_side_api::{ClientSideApiClient, Place};
use crate::app::AppRoutes;
use crate::component::ImageLightbox;
use crate::component::{loading_button::LoadingButton, FullScreenSpinnerGray, Navbar, StarRating};
use crate::domain::{
    DomainHotelCodeId, DomainHotelInfoCriteria, DomainHotelSearchCriteria, DomainRoomGuest,
};
use crate::log;
use crate::page::{HotelDetailsParams, HotelListNavbar, InputGroupContainer};
use crate::utils::query_params::QueryParamsSync;
use crate::view_state_layer::input_group_state::InputGroupState;
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
                hotel_params.sync_to_app_state();
            } else {
                log!("[HotelDetailsV1Page] Could not parse hotel params from URL.");
            }
        }
    });

    // <!-- Function to update URL with current search state -->
    // This can be called when navigating to this page from hotel list
    let update_url_with_current_state = move || {
        if let Some(current_params) = HotelDetailsParams::from_current_context() {
            current_params.update_url(); // Now uses individual query params
            log!(
                "Updated URL with current hotel details state (individual params): {:?}",
                current_params
            );
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
            if hotel_code.is_empty() || date_range.start == (0, 0, 0) {
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

            match client.get_hotel_rates(criteria).await {
                Ok(rates) => {
                    HotelDetailsUIState::set_rates(Some(rates.clone()));
                    HotelDetailsUIState::set_rates_loading(false);
                    Some(rates)
                }
                Err(e) => {
                    HotelDetailsUIState::set_error(Some(e));
                    HotelDetailsUIState::set_rates_loading(false);
                    None
                }
            }
        },
    );

    let is_loading =
        move || hotel_details_state.loading.get() || hotel_details_state.rates_loading.get();
    let error_message = move || hotel_details_state.error.get();

    let loaded = move || static_details_resource.get().and_then(|d| d).is_some();

    let hotel_name_signal = move || {
        if let Some(hotel_details) = hotel_details_state.static_details.get() {
            hotel_details.hotel_name
        } else {
            "".to_string()
        }
    };

    let star_rating_signal = move || {
        if let Some(hotel_details) = hotel_details_state.static_details.get() {
            hotel_details.star_rating
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
        <section class="relative min-h-screen bg-gray-50">
            <HotelListNavbar />
            <div class={
            let is_input_expanded = move || InputGroupState::is_open_show_full_input();
            move || format!(
                "transition-all duration-300 {}",
                if is_input_expanded() {
                    // Larger spacer when input is expanded on mobile/tablet, normal on desktop
                    "h-96 sm:h-96 md:h-80 lg:h-48"
                } else {
                    // Normal spacer when collapsed on all screens
                    "h-24"
                }
            )
            }></div>

            // <Navbar />
            // <Show when=move || !open_image_viewer.get()>
            //     <div class="flex flex-col items-center mt-6 p-4">
            //         <InputGroupContainer default_expanded=false allow_outside_click_collapse=true />
            //     </div>
            // </Show>

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

            <Show
                when=move || !is_loading()
                fallback=FullScreenSpinnerGray
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
                            <HotelImages open_image_viewer/>
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
                                            let range = ui_search_ctx.date_range.get();
                                            if range.start == (0,0,0) {
                                                "N/A".to_string()
                                            } else {
                                                format!("{:04}-{:02}-{:02}", range.start.0, range.start.1, range.start.2)
                                            }
                                        }}</div>
                                        <div class="text-sm text-gray-600">Check-out: {move || {
                                            let range = ui_search_ctx.date_range.get();
                                            if range.end == (0,0,0) {
                                                "N/A".to_string()
                                            } else {
                                                format!("{:04}-{:02}-{:02}", range.end.0, range.end.1, range.end.2)
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
pub fn HotelImages(open_image_viewer: RwSignal<bool>) -> impl IntoView {
    let hotel_details_state: HotelDetailsUIState = expect_context();

    // let (show_viewer, set_show_viewer) = create_signal(false);
    let (selected_index, set_selected_index) = create_signal(0);

    let images_signal = create_memo(move |_| {
        if let Some(hotel_details) = hotel_details_state.static_details.get() {
            let mut images = hotel_details.images.clone();
            if images.len() < 6 {
                let repeat_count = 6 - images.len();
                let repeated_images = images.clone();
                images.extend(repeated_images.into_iter().take(repeat_count));
            }
            images
        } else {
            vec![]
        }
    });

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
                            class="w-full h-64 rounded-xl object-cover cursor-pointer"
                            on:click=move |_| {
                                set_selected_index(0);
                                open_image_viewer.set(true);
                            }
                        />
                    </div>
                    <div class="hidden sm:flex flex-col space-y-3">
                        <div class="flex flex-col sm:flex-row space-y-3 sm:space-y-0 sm:space-x-3">
                            <img
                                src=move || images_signal()[0].clone()
                                alt="Hotel"
                                class="w-full sm:w-3/5 h-64 sm:h-96 rounded-xl object-cover cursor-pointer"
                                on:click=move |_| {
                                    set_selected_index(0);
                                    open_image_viewer.set(true);
                                }
                            />
                            <div class="flex flex-row sm:flex-col space-x-3 sm:space-x-0 sm:space-y-3 w-full sm:w-2/5">
                                <img
                                    src=move || images_signal().get(1).cloned().unwrap_or_else(|| images_signal()[0].clone())
                                    alt="Hotel"
                                    class="w-1/2 sm:w-full h-32 sm:h-[186px] rounded-xl object-cover sm:object-fill cursor-pointer"
                                    on:click=move |_| {
                                        set_selected_index(1);
                                        open_image_viewer.set(true);
                                    }
                                />
                                <img
                                    src=move || images_signal().get(2).cloned().unwrap_or_else(|| images_signal()[0].clone())
                                    alt="Hotel"
                                    class="w-1/2 sm:w-full h-32 sm:h-[186px] rounded-xl object-cover sm:object-fill cursor-pointer"
                                    on:click=move |_| {
                                        set_selected_index(2);
                                        open_image_viewer.set(true);
                                    }
                                />
                            </div>
                        </div>
                        <div class="flex justify-between space-x-3">
                            <img
                                src=move || images_signal().get(3).cloned().unwrap_or_else(|| images_signal()[0].clone())
                                alt="Hotel"
                                class="w-72 h-48 rounded-xl object-cover cursor-pointer"
                                on:click=move |_| {
                                    set_selected_index(3);
                                    open_image_viewer.set(true);
                                }
                            />
                            <img
                                src=move || images_signal().get(4).cloned().unwrap_or_else(|| images_signal()[0].clone())
                                alt="Hotel"
                                class="w-72 h-48 rounded-xl object-cover cursor-pointer"
                                on:click=move |_| {
                                    set_selected_index(4);
                                    open_image_viewer.set(true);
                                }
                            />
                            <div class="relative w-72 h-48 rounded-xl">
                                <img
                                    src=move || images_signal().get(5).cloned().unwrap_or_else(|| images_signal()[0].clone())
                                    alt="Hotel"
                                    class="object-cover h-full w-full rounded-xl cursor-pointer"
                                    on:click=move |_| {
                                        set_selected_index(5);
                                        open_image_viewer.set(true);
                                    }
                                />
                            </div>
                        </div>
                    </div>

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
        if let Some(rates) = hotel_details_state.rates.get() {
            // Use real room data from all_rooms with individual pricing
            rates
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
