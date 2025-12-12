use leptos::*;
use leptos_icons::Icon;
use leptos_router::use_navigate;
use leptos_use::{use_interval_fn, utils::Pausable};

use crate::api::auth::auth_state::{AuthState, AuthStateSignal};
use crate::api::client_side_api::{ClientSideApiClient, SendOtpResponse, VerifyOtpResponse};
use crate::api::consts::{get_ipn_callback_url, get_payments_url_v2};
use crate::api::payments::{create_domain_request, PaymentProvider};
use crate::app::AppRoutes;
use crate::application_services::BookingService;
use crate::component::yral_auth_provider::YralAuthProvider;
use crate::component::ChildrenAgesSignalExt;
use crate::component::{Divider, Navbar, SpinnerGray};
use crate::domain::{
    DomainAdultDetail, DomainBlockRoomRequest, DomainChildDetail, DomainDestination,
    DomainHotelDetails, DomainHotelInfoCriteria, DomainHotelSearchCriteria, DomainRoomData,
    DomainRoomGuest, DomainSelectedDateRange, DomainSelectedRoomWithQuantity, DomainUserDetails,
};
use crate::log;
use crate::utils::app_reference::generate_app_reference;
use crate::utils::cookie_storage::CookieBookingStorage;
use crate::utils::{app_reference::BookingId, BackendIntegrationHelper};
use crate::view_state_layer::booking_id_state::BookingIdState;
use crate::view_state_layer::email_verification_state::EmailVerificationState;
use crate::view_state_layer::hotel_details_state::PricingBookNowState;
use crate::view_state_layer::ui_block_room::{
    AdultDetail, BlockRoomUIState, ChildDetail, RoomSelectionSummary,
};
use crate::view_state_layer::ui_hotel_details::HotelDetailsUIState;
use crate::view_state_layer::ui_search_state::UISearchCtx;
use crate::view_state_layer::view_state::HotelInfoCtx;
use std::collections::HashMap;

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

fn format_tax_label(label: &str) -> String {
    label
        .replace('_', " ")
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => "".to_string(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn round_up_to_cents(value: f64) -> f64 {
    (value * 100.0).ceil() / 100.0
}

fn aggregate_tax_summary(
    rooms: Vec<RoomSelectionSummary>,
    include: bool,
    multiply_by_quantity: bool,
) -> Vec<(String, String, f64)> {
    let mut aggregated: HashMap<(String, String), f64> = HashMap::new();
    for room in rooms {
        for tax in room
            .tax_lines
            .iter()
            .filter(|line| line.included == include)
        {
            if tax.amount <= 0.0 {
                continue;
            }
            let total_amount = if multiply_by_quantity {
                tax.amount * room.quantity as f64
            } else {
                tax.amount
            };
            let key = (tax.description.clone(), tax.currency_code.clone());
            *aggregated.entry(key).or_insert(0.0) += total_amount;
        }
    }
    let mut entries: Vec<_> = aggregated
        .into_iter()
        .map(|((description, currency), amount)| (description, currency, amount))
        .collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    entries
}

#[component]
pub fn BlockRoomV1Page() -> impl IntoView {
    // Initialize form validation on page load - button will be disabled until form is valid
    BlockRoomUIState::validate_form();

    let block_room_state: BlockRoomUIState = expect_context();
    let ui_search_ctx: UISearchCtx = expect_context();
    let hotel_info_ctx: HotelInfoCtx = expect_context();
    let auth_state_signal: AuthStateSignal = expect_context();
    let navigate = use_navigate();

    // Initialize form data on mount - only once
    let (initialized, set_initialized) = create_signal(false);

    // Create booking ID signal - uses centralized BookingIdState
    let booking_id_signal = create_memo(move |_| {
        // Get the primary adult email
        let adults_list = block_room_state.adults.get();
        if let Some(email) = adults_list.first().and_then(|adult| adult.email.clone()) {
            // Use centralized booking ID management
            BookingIdState::get_or_create_booking_id(email)
        } else {
            None
        }
    });

    create_effect(move |_| {
        let rooms_count = ui_search_ctx.guests.rooms.get() as usize;
        let required_primary_contacts = rooms_count.max(1);
        let children_count = ui_search_ctx.guests.children.get() as usize;
        let children_ages = ui_search_ctx.guests.children_ages.clone();

        // Initialize adults and children only once
        if !initialized.get_untracked() {
            log!(
                "Initializing form data for the first time - adults: {}, rooms: {}, children: {}",
                required_primary_contacts,
                rooms_count,
                children_count
            );
            BlockRoomUIState::create_adults(required_primary_contacts);
            BlockRoomUIState::create_children(children_count);
            set_initialized.set(true);
        } else {
            log!("Skipping form data initialization - already initialized");
        }

        // Set pricing data from HotelDetailsUIState (correct source) instead of PricingBookNowState
        let room_price_from_pricing_book_now =
            PricingBookNowState::total_room_price_for_all_user_selected_rooms();
        let room_price_from_hotel_details = HotelDetailsUIState::total_room_price();

        // Use the HotelDetailsUIState data since it has the correct pricing
        let room_price = if room_price_from_hotel_details > 0.0 {
            room_price_from_hotel_details
        } else {
            room_price_from_pricing_book_now
        };

        let date_range = ui_search_ctx.date_range.get();
        let num_nights = date_range.no_of_nights();

        log!("BlockRoomV1Page pricing initialization:");
        log!(
            "  room_price from PricingBookNowState: {}",
            room_price_from_pricing_book_now
        );
        log!(
            "  room_price from HotelDetailsUIState: {}",
            room_price_from_hotel_details
        );
        log!("  final room_price selected: {}", room_price);
        log!("  num_nights: {}", num_nights);

        BlockRoomUIState::set_room_price(room_price);
        BlockRoomUIState::set_num_nights(num_nights);
        let total = BlockRoomUIState::calculate_total_price();

        log!("  calculated total: {}", total);
        log!(
            "  final room_price in state: {}",
            BlockRoomUIState::get_room_price()
        );

        // Also try to get room selection summary to see if it's populated
        let room_summary = BlockRoomUIState::get_room_selection_summary_untracked();
        log!("  room_selection_summary length: {}", room_summary.len());
        for (i, room) in room_summary.iter().enumerate() {
            log!(
                "    Room {}: {} x{} @ ${:.2}/night",
                i + 1,
                room.display_name(),
                room.quantity,
                room.price_per_night
            );
        }

        log!(
            "BlockRoomV1Page initialized - adults: {}, children: {}, room_price: {}, nights: {}",
            required_primary_contacts,
            children_count,
            room_price,
            num_nights
        );
    });

    // Navigation handler
    let go_back_to_details = move |_: ev::MouseEvent| {
        // Instead of this let's go to browser history back
        web_sys::window()
            .and_then(|win| win.history().ok())
            .and_then(|history| history.back().ok());
        // let _ = navigate(AppRoutes::HotelDetails.to_string(), Default::default());
    };

    // Get reactive signals using static methods
    let room_price = move || BlockRoomUIState::get_room_price();
    let total_price = move || BlockRoomUIState::get_total_price();
    let num_nights = move || BlockRoomUIState::get_num_nights();
    let room_summary = move || BlockRoomUIState::get_room_selection_summary();

    // Create resource to call prebook API when page loads
    // Following the pattern from payment_handler.rs
    //
    // NOTE: This can be simplified using BookingService:
    // let booking_service = BookingService::from_ui_context(LiteApiAdapter::default());
    // let response = booking_service.block_room_and_save_to_backend(booking_id, hotel_token).await;
    // let prebook_resource = create_resource(
    //     move || {
    //         // Wait for essential data to be ready before calling API
    //         let room_price_val = room_price();
    //         let adults_list = BlockRoomUIState::get_adults();
    //         let hotel_code_val = hotel_info_ctx.hotel_code.get();

    //         let has_room_price = room_price_val > 0.0;
    //         let has_adults = !adults_list.is_empty();
    //         let has_hotel_code = !hotel_code_val.is_empty();

    //         log!("Prebook resource readiness check:");
    //         log!("  room_price: {} (has_room_price: {})", room_price_val, has_room_price);
    //         log!("  adults_count: {} (has_adults: {})", adults_list.len(), has_adults);
    //         log!("  hotel_code: '{}' (has_hotel_code: {})", hotel_code_val, has_hotel_code);
    //         log!("  overall_ready: {}", has_room_price && has_adults && has_hotel_code);

    //         // Return true when ready to call API
    //         has_room_price && has_adults && has_hotel_code
    //     },
    //     move |is_ready| async move {
    //         if !is_ready {
    //             log!("Prebook resource: Not ready yet, waiting for data... - {}", is_ready);
    //             return None;
    //         }

    //         log!("Prebook resource: Page data ready, calling prebook API...");

    //         // Build prebook request
    //         match build_block_room_request().await {
    //             Some(request) => {
    //                 let client = ClientSideApiClient::new();
    //                 log!(
    //                     "Prebook resource: Making API call with request: {:?}",
    //                     request
    //                 );

    //                 match client.block_room(request).await {
    //                     Some(response) => {
    //                         log!("Prebook resource: Success - {:?}", response);

    //                         // Update pricing with data from block room API response
    //                         let api_total_price = response.total_price.room_price;
    //                         let api_room_price = if !response.blocked_rooms.is_empty() {
    //                             response.blocked_rooms[0].price.room_price
    //                         } else {
    //                             response.total_price.room_price
    //                         };

    //                         log!(
    //                             "Updating pricing from API - Total: ${:.2}, Room: ${:.2}",
    //                             api_total_price,
    //                             api_room_price
    //                         );

    //                         // Log price change if any
    //                         if BlockRoomUIState::has_price_changed_from_original(api_total_price) {
    //                             let difference =
    //                                 BlockRoomUIState::get_price_difference(api_total_price);
    //                             log!("Price changed from original by ${:.2}", difference);
    //                         }

    //                         // Save to backend after successful prebook
    //                         if let Err(e) = save_booking_to_backend(&response.block_id).await {
    //                             log!("Prebook resource: Failed to save to backend: {}", e);
    //                             // Batch update all state changes to avoid borrow conflicts
    //                             BlockRoomUIState::batch_update_on_success_with_backend_error(
    //                                 response.block_id.clone(),
    //                                 api_total_price,
    //                                 api_room_price,
    //                                 Some(format!("Room reserved but booking save failed: {}", e))
    //                             );
    //                             return Some(response.block_id);
    //                         }

    //                         // Batch update all state changes to avoid borrow conflicts
    //                         BlockRoomUIState::batch_update_on_success(
    //                             response.block_id.clone(),
    //                             api_total_price,
    //                             api_room_price,
    //                         );
    //                         Some(response.block_id)
    //                     }
    //                     None => {
    //                         log!("Prebook resource: API call failed");
    //                         BlockRoomUIState::batch_update_on_error(
    //                             Some("server".to_string()),
    //                             Some("Unable to reserve your room. Please try again.".to_string()),
    //                             Some("Prebook API returned None response".to_string()),
    //                         );
    //                         None
    //                     }
    //                 }
    //             }
    //             None => {
    //                 log!("Prebook resource: Failed to build request");
    //                 BlockRoomUIState::batch_update_on_error(
    //                     Some("validation".to_string()),
    //                     Some("Invalid booking information. Please check your details.".to_string()),
    //                     Some("Failed to build prebook request - missing required data".to_string()),
    //                 );
    //                 None
    //             }
    //         }
    //     },
    // );

    // <!-- Calculate totals from room selections -->
    let calculated_total = move || {
        let summary = room_summary();
        let nights = num_nights();
        summary
            .iter()
            .map(|room| {
                // Round price to 2 decimals to match displayed value
                let rounded_price = (room.price_per_night * 100.0).round() / 100.0;
                rounded_price * room.quantity as f64 * nights as f64
            })
            .sum::<f64>()
    };

    let rooms_total_per_night = move || {
        room_summary()
            .iter()
            .map(|room| room.price_per_night * room.quantity as f64)
            .sum::<f64>()
    };
    let num_rooms = move || ui_search_ctx.guests.rooms.get();
    let adult_count = move || {
        ui_search_ctx
            .guests
            .adults
            .get()
            .max(ui_search_ctx.guests.rooms.get())
    };
    let child_count = move || ui_search_ctx.guests.children.get();

    // Hotel info signals with enhanced data flow - prioritize BlockRoomUIState, then HotelDetailsUIState, then HotelInfoCtx
    let resolved_hotel_details = move || {
        BlockRoomUIState::get_hotel_context().or_else(|| HotelDetailsUIState::get_hotel_details())
    };

    let hotel_name = move || {
        resolved_hotel_details()
            .map(|h| h.hotel_name)
            .filter(|name| !name.trim().is_empty())
            .or_else(|| {
                let name = hotel_info_ctx.selected_hotel_name.get();
                if name.trim().is_empty() {
                    None
                } else {
                    Some(name)
                }
            })
            .unwrap_or_else(|| "Hotel Name".to_string())
    };

    let hotel_address = move || {
        resolved_hotel_details()
            .map(|h| h.address)
            .filter(|addr| !addr.trim().is_empty())
            .or_else(|| {
                let address = hotel_info_ctx.selected_hotel_location.get();
                if address.trim().is_empty() {
                    None
                } else {
                    Some(address)
                }
            })
            .unwrap_or_else(|| "".to_string())
    };

    let hotel_rating_and_reviews = move || {
        resolved_hotel_details()
            .map(|h| (h.rating, h.review_count))
            .unwrap_or((None, None))
    };

    let hotel_image = move || {
        resolved_hotel_details()
            .and_then(|h| h.images.first().cloned())
            .filter(|img| !img.is_empty())
            .or_else(|| {
                let img = hotel_info_ctx.selected_hotel_image.get();
                if img.is_empty() {
                    None
                } else {
                    Some(img)
                }
            })
            .unwrap_or_else(|| "/img/home.webp".to_string())
    };

    // Ensure hotel context is populated on this page if it was missed during navigation
    create_effect(move |_| {
        if BlockRoomUIState::get_hotel_context().is_none() {
            if let Some(details) = HotelDetailsUIState::get_hotel_details() {
                BlockRoomUIState::set_hotel_context(Some(details.clone()));
            }
        }
    });

    // Date formatting
    let checkin_date = move || ui_search_ctx.date_range.get().dd_month_yyyy_start();
    let checkout_date = move || ui_search_ctx.date_range.get().dd_month_yyyy_end();
    let formatted_nights = move || ui_search_ctx.date_range.get().formatted_nights();

    view! {
        <section class="relative min-h-screen bg-[#f7f7f7]">
            <Navbar />

            <div class="max-w-7xl mx-auto px-4 sm:px-8">
                <div class="flex items-center py-3 md:py-5">
                    <span class="inline-flex items-center cursor-pointer" on:click=go_back_to_details>
                        <Icon icon=icondata::AiArrowLeftOutlined class="text-black font-light" />
                    </span>
                    <h1 class="ml-2 sm:ml-4 text-xl sm:text-2xl font-semibold text-gray-800">"You're just one step away!"</h1>
                </div>

                <div class="relative flex flex-col lg:flex-row min-h-[calc(100vh-5rem)] items-start justify-between pb-14 gap-6 sm:gap-8">
                    <div class="w-full lg:w-3/5 flex flex-col gap-5">
                        // Hotel information card
                        <div class="p-6 sm:p-7 bg-white rounded-xl border border-gray-200 w-full space-y-5">
                            <div class="flex items-start gap-4 sm:gap-5">
                                <img
                                    src=hotel_image
                                    alt=hotel_name
                                    class="h-16 w-16 sm:h-20 sm:w-20 rounded-xl object-cover flex-shrink-0 border border-gray-200"
                                />
                                <div class="flex-1 min-w-0 space-y-2">
                                    <div class="text-lg sm:text-xl font-semibold text-gray-900 leading-tight truncate">
                                        {hotel_name}
                                    </div>
                                    {move || {
                                        let address = hotel_address();
                                        (!address.trim().is_empty()).then(|| {
                                            view! {
                                                <div class="flex items-center gap-2 text-sm text-gray-600 leading-tight">
                                                    <Icon icon=icondata::AiEnvironmentOutlined class="text-blue-500 text-base" />
                                                    <span class="truncate">{address}</span>
                                                </div>
                                            }
                                        })
                                    }}
                                    {move || {
                                        let (rating, reviews) = hotel_rating_and_reviews();
                                        (rating.is_some() || reviews.is_some()).then(|| {
                                            let badge = rating
                                                .map(|r| format!("{:.1}", r))
                                                .unwrap_or_else(|| "--".to_string());
                                            let review_text = match (rating, reviews) {
                                                (Some(_), Some(c)) => format!("Good ({c})"),
                                                (Some(_), None) => "Good".to_string(),
                                                (None, Some(c)) => format!("Reviews ({c})"),
                                                (None, None) => "".to_string(),
                                            };
                                            view! {
                                                <div class="flex items-center gap-3 text-sm text-gray-700 leading-tight">
                                                    <span class="inline-flex items-center rounded-md bg-amber-200 text-amber-800 px-2 py-0.5 text-xs font-semibold">
                                                        {badge}
                                                    </span>
                                                    <span class="text-gray-700">{review_text}</span>
                                                </div>
                                            }
                                        })
                                    }}
                                </div>
                            </div>

                            <Divider class="border-gray-200".into() />

                            <div class="space-y-1.5">
                                <p class="text-sm text-gray-500">"Check-in & Check-out"</p>
                                <p class="text-lg font-semibold text-gray-900 leading-tight">
                                    {move || format!("{} - {} ( {} )", checkin_date(), checkout_date(), formatted_nights())}
                                </p>
                            </div>

                            <Divider class="border-gray-200".into() />

                            <div class="space-y-1.5">
                                <p class="text-sm text-gray-500">"Guests & Room"</p>
                                <p class="text-lg font-semibold text-gray-900 leading-tight">
                                    {move || {
                                        let rooms = num_rooms();
                                        let adults = adult_count();
                                        let children = child_count();
                                        let mut parts = vec![
                                            format!("{} Room{}", rooms, if rooms == 1 { "" } else { "s" }),
                                            format!("{} Adult{}", adults, if adults == 1 { "" } else { "s" }),
                                        ];
                                        if children > 0 {
                                            parts.push(format!("{} Child{}", children, if children == 1 { "" } else { "ren" }));
                                        }
                                        parts.join(" • ")
                                    }}
                                </p>
                            </div>
                        </div>

                        // Login prompt for non-logged users or Guest form for logged users
                        <AuthGatedGuestForm />

                        // Promo code entry (temporarily disabled)
                        // <PromoCodeSection />

                        // Mobile price card
                        <EnhancedPricingDisplay mobile=true booking_id_signal=booking_id_signal />
                    </div>

                    // Right side pricing
                    <div class="w-full lg:w-2/5 flex flex-col gap-4 lg:sticky lg:top-24">
                        <EnhancedPricingDisplay mobile=false booking_id_signal=booking_id_signal />
                    </div>
                </div>

                // Payment Modal
                <PaymentModal />
            </div>
        </section>
    }
}

// <!-- Phase 2.2: Enhanced Pricing Display Component -->
#[component]
pub fn EnhancedPricingDisplay(
    mobile: bool,
    #[prop(into)] booking_id_signal: Signal<Option<BookingId>>,
) -> impl IntoView {
    let room_summary = move || BlockRoomUIState::get_room_selection_summary();
    let num_nights = move || BlockRoomUIState::get_num_nights();

    // Calculate totals from room selections
    let rooms_total_per_night = move || {
        room_summary()
            .iter()
            .map(|room| room.price_per_night * room.quantity as f64)
            .sum::<f64>()
    };

    let base_total = move || {
        let summary = room_summary();
        let nights = num_nights();
        summary
            .iter()
            .map(|room| {
                // Round price to 2 decimals to match displayed value
                let rounded_price = (room.price_per_night * 100.0).round() / 100.0;
                rounded_price * room.quantity as f64 * nights as f64
            })
            .sum::<f64>()
    };

    let container_class = if mobile {
        "lg:hidden w-full rounded-2xl border border-gray-200 bg-white p-5 sm:p-6 shadow-md flex flex-col items-stretch space-y-4"
    } else {
        "hidden lg:flex w-full rounded-2xl border border-gray-200 bg-white p-5 sm:p-7 shadow-md flex-col items-stretch space-y-4"
    };

    let included_taxes_summary =
        create_memo(move |_| aggregate_tax_summary(room_summary(), true, true));
    let excluded_taxes_summary =
        create_memo(move |_| aggregate_tax_summary(room_summary(), false, false));

    let included_taxes_total = move || {
        included_taxes_summary()
            .iter()
            .map(|entry| entry.2)
            .sum::<f64>()
    };

    let platform_markup_total = move || 0.0;

    let excluded_taxes_total = move || {
        excluded_taxes_summary()
            .iter()
            .map(|entry| entry.2)
            .sum::<f64>()
    };

    let included_tax_currency = move || {
        included_taxes_summary()
            .first()
            .map(|entry| entry.1.clone())
            .unwrap_or_else(|| "USD".to_string())
    };

    let platform_markup_currency = move || "USD".to_string();

    let excluded_tax_currency = move || {
        excluded_taxes_summary()
            .first()
            .map(|entry| entry.1.clone())
            .unwrap_or_else(|| "USD".to_string())
    };

    let total_with_included = move || base_total() + included_taxes_total();
    let rounded_total_with_included = move || round_up_to_cents(total_with_included());

    let format_currency = |val: f64| format!("${:.2}", val);

    view! {
        <div class=container_class>
            <div class="text-xs text-gray-500">"Price Breakup"</div>

            // <!-- Per-room breakdown -->
            <div class="price-breakdown space-y-3">
                <Show when=move || !room_summary().is_empty()>
                    {move || room_summary().into_iter().map(|room| {
                        let display_name = room.display_name();
                        let price_per_night = room.price_per_night;
                        let quantity = room.quantity;
                        // Round price to 2 decimals to match displayed value, then calculate total
                        let rounded_price = (price_per_night * 100.0).round() / 100.0;
                        let line_total = rounded_price * quantity as f64 * num_nights() as f64;
                        view! {
                            <div class="flex justify-between items-start text-sm">
                                <div class="flex-1 min-w-0 space-y-0.5">
                                    <div class="font-medium text-gray-800 break-words">{display_name} " × " {num_nights()} " Nights"</div>
                                    <div class="text-xs text-gray-500">
                                        {let nights = num_nights();
                                        let nights_label = if nights == 1 {
                                            "1 night".to_string()
                                        } else {
                                            format!("{nights} nights")
                                        };
                                        let rooms_label = if quantity == 1 {
                                            "1 room".to_string()
                                        } else {
                                            format!("{quantity} rooms")
                                        };
                                        view! {
                                            "(" {format_currency(price_per_night)} "/night × " {nights_label} " × " {rooms_label} ")"
                                        }}
                                    </div>
                                </div>
                                <div class="text-sm font-semibold text-gray-900 ml-3">
                                    {format_currency(line_total)}
                                </div>
                            </div>
                        }
                    }).collect::<Vec<_>>()}
                </Show>

                // <!-- Fallback when no rooms selected -->
                <Show when=move || room_summary().is_empty()>
                    <div class="flex justify-between items-center text-base">
                        <span class="text-gray-700">
                            {move || format!("${:.2} x {} nights", rooms_total_per_night(), num_nights())}
                        </span>
                        <span class="font-semibold">
                            {move || format_currency(base_total())}
                        </span>
                    </div>
                </Show>

                // <!-- Taxes and fees -->
                <div class="space-y-1 pt-1 border-t border-dashed border-gray-200">
                    <div class="flex justify-between items-center text-sm">
                        <span class="text-gray-600">"Taxes and fees"</span>
                        <span class="font-semibold text-gray-900">
                            {move || format_currency_with_code(included_taxes_total(), &included_tax_currency())}
                        </span>
                    </div>
                    <Show when=move || !included_taxes_summary().is_empty()>
                        <div class="space-y-1 text-xs text-gray-500">
                            {move || included_taxes_summary()
                                .iter()
                                .map(|(description, currency, amount)| {
                                    let desc = format_tax_label(description);
                                    let currency_code = currency.clone();
                                    view! {
                                        <div class="flex justify-between">
                                            <span>{desc}</span>
                                            <span>{format_currency_with_code(*amount, &currency_code)}</span>
                                        </div>
                                    }
                                })
                                .collect::<Vec<_>>() }
                        </div>
                    </Show>
                </div>
            </div>

            <Divider class="my-2".into() />
                // <!-- Platform and Markup fees -->
            <div class="space-y-1">
                <div class="flex justify-between items-center text-sm">
                    <span class="text-gray-600">"Platform and Markup fees"</span>
                    <span class="font-semibold text-gray-900">
                        {move || format_currency_with_code(platform_markup_total(), &platform_markup_currency())}
                    </span>
                </div>
            </div>

            <Divider class="my-2".into() />

            // <!-- Total -->
            <div class="flex justify-between items-center font-bold text-lg">
                <span>Total</span>
                <span class="text-2xl">
                    {move || format_currency_with_code(rounded_total_with_included(), &included_tax_currency())}
                </span>
            </div>

            <Show when=move || (excluded_taxes_total().abs() > 0.0)>
                <div class="mt-2 space-y-1 text-sm">
                    <div class="flex justify-between items-center text-sm text-gray-600">
                        <span>"Local fees (pay at property)"</span>
                        <span class="font-semibold text-gray-900">
                            {move || format_currency_with_code(excluded_taxes_total(), &excluded_tax_currency())}
                        </span>
                    </div>
                    <Show when=move || !excluded_taxes_summary().is_empty()>
                        <div class="space-y-1 text-xs text-gray-500">
                                {move || excluded_taxes_summary()
                                    .iter()
                                    .map(|(description, currency, amount)| {
                                        let desc = format_tax_label(description);
                                        let currency_code = currency.clone();
                                        view! {
                                            <div class="flex justify-between">
                                            <span>{desc}</span>
                                            <span>{format_currency_with_code(*amount, &currency_code)}</span>
                                        </div>
                                    }
                                })
                                .collect::<Vec<_>>() }
                        </div>
                    </Show>
                </div>
            </Show>

            <div class="pt-2 space-y-3">
                <TermsCheckbox />
                <ConfirmButton mobile=mobile booking_id_signal=booking_id_signal />
            </div>
        </div>
    }
}

// <!-- Phase 2.1: Room Selection Summary Component -->
#[component]
pub fn SelectedRoomsSummary() -> impl IntoView {
    let room_summary = move || BlockRoomUIState::get_room_selection_summary();

    view! {
        <div class="bg-white rounded-3xl border border-gray-200 shadow-md p-5 sm:p-6">
            <h3 class="text-lg sm:text-xl font-bold mb-4">Selected Rooms</h3>

            <Show
                when=move || !room_summary().is_empty()
                fallback=move || view! {
                    <div class="text-gray-500 text-center py-4">
                        "No rooms selected"
                    </div>
                }
            >
                <div class="space-y-4">
                    {move || room_summary().into_iter().map(|room| {
                        view! {
                            <RoomSummaryCard room=room />
                        }
                    }).collect::<Vec<_>>()}
                </div>

                // <!-- Room selection totals -->
                <div class="mt-4 pt-4 border-t border-gray-200">
                    <div class="flex justify-between items-center text-sm text-gray-600">
                        <span>Total Rooms Selected:</span>
                        <span class="font-semibold">
                            {move || room_summary().iter().map(|r| r.quantity).sum::<u32>()}
                        </span>
                    </div>
                </div>
            </Show>
        </div>
    }
}

// <!-- Individual room summary card component -->
#[component]
pub fn RoomSummaryCard(room: RoomSelectionSummary) -> impl IntoView {
    let display_name = room.display_name();
    let quantity = room.quantity;
    let price_per_night = room.price_per_night;
    view! {
        <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between border border-gray-200 rounded-2xl p-3 sm:p-4 bg-gray-50">
            // <!-- Room details -->
            <div class="flex-1 min-w-0 mb-2 sm:mb-0">
                <div class="font-semibold text-base min-w-0 break-words whitespace-normal truncate">
                    {display_name.clone()}
                </div>
                <div class="text-sm text-gray-600 flex items-center gap-2 mt-1">
                    <span class="bg-blue-100 text-blue-800 px-2 py-1 rounded-full text-xs font-medium">
                        {format!("{} room{}", quantity, if quantity == 1 { "" } else { "s" })}
                    </span>
                    <span class="text-gray-500">"•"</span>
                    <span>${format!("{:.2}", price_per_night)} /night</span>
                </div>
            </div>

            // <!-- Price display -->
            <div class="flex flex-col items-start sm:items-end sm:text-right">
                <div class="text-lg font-bold">
                    ${format!("{:.2}", price_per_night * quantity as f64)}
                    <span class="text-sm font-normal text-gray-600 ml-1">/night</span>
                </div>
                <div class="text-xs text-gray-500">
                    {format!("${:.2} × {}", price_per_night, quantity)}
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn AuthGatedGuestForm() -> impl IntoView {
    use crate::api::consts::USER_IDENTITY;
    use codee::string::JsonSerdeCodec;
    use leptos_use::{use_cookie_with_options, UseCookieOptions};

    // Use AuthStateSignal pattern (same as base_route.rs and my_bookings.rs)
    // let auth_state_signal: AuthStateSignal = expect_context();

    // // Also monitor the USER_IDENTITY cookie directly (same as navbar pattern)
    // let (stored_identity, _) = use_cookie_with_options::<NewIdentity, JsonSerdeCodec>(
    //     USER_IDENTITY,
    //     UseCookieOptions::default()
    //         .path("/")
    //         .same_site(leptos_use::SameSite::Lax)
    //         .http_only(false)
    //         .secure(false),
    // );

    // Get user email from identity for auto-fill
    let user_email = Signal::derive(move || AuthStateSignal::auth_state().get().email);

    // crate::log!(
    //     "AUTH_FLOW: block_room - AuthGatedGuestForm initialized - cookie_identity: {}, auth_signal_identity: {}",
    //     stored_identity.get().is_some(),
    //     auth_state_signal.get().user_identity.get().is_some()
    // );

    // Return the reactive view - use move closure for reactivity
    // Return the reactive view - use move closure for reactivity
    move || {
        // Check auth state from both sources (cookie takes priority)
        let is_logged_in = user_email.get().is_some();

        crate::log!(
            "AUTH_FLOW: block_room - AuthGatedGuestForm render check - is_logged_in: {}, email: {:?}",
            is_logged_in,
            user_email.get()
        );

        if is_logged_in {
            // User is logged in, show guest form
            view! { <GuestForm user_email=user_email /> }.into_view()
        } else {
            // User is not logged in, show login prompt
            view! { <LoginPrompt /> }.into_view()
        }
    }
}

#[component]
pub fn LoginPrompt() -> impl IntoView {
    view! {
        <div class="bg-white rounded-3xl border border-gray-200 shadow-md p-6 text-center">
            <div class="mb-4">
                <Icon icon=icondata::AiUserOutlined class="text-gray-400 text-4xl mx-auto mb-3" />
                <h3 class="text-lg font-semibold text-gray-900 mb-2">
                    "Login Required"
                </h3>
                <p class="text-gray-600 text-sm mb-4">
                    "Please login to continue with your booking"
                </p>
            </div>
            // <!-- Centered login button with improved mobile styling -->
            <div class="w-full flex justify-center">
                <YralAuthProvider />
            </div>
        </div>
    }
}

#[component]
pub fn GuestForm(#[prop(into)] user_email: Signal<Option<String>>) -> impl IntoView {
    let block_room_state: BlockRoomUIState = expect_context();
    let ui_search_ctx: UISearchCtx = expect_context();

    let adult_count = move || ui_search_ctx.guests.rooms.get();
    let child_count = move || ui_search_ctx.guests.children.get();
    let children_ages = ui_search_ctx.guests.children_ages.clone();

    view! {
        <div class="guest-form mt-4 space-y-5 sm:space-y-6">
            {move || {
                let rooms = ui_search_ctx.guests.rooms.get();
                let adults = BlockRoomUIState::get_adults().len() as u32;

                (rooms > adults).then_some(view! {
                    <div class="rounded-md bg-amber-50 text-amber-800 text-sm px-3 py-2">
                        {format!(
                            "{} room(s) selected. Please add a primary adult for each room ({} required).",
                            rooms, rooms
                        )}
                    </div>
                })
            }}

            // Adults
            {(0..adult_count())
                .map(|i| {
                    view! {
                        <AdultFormSection index=i user_email=user_email />
                    }
                })
                .collect::<Vec<_>>()}

            // Children
            {(0..child_count())
                .map(|i| {
                    view! {
                        <ChildFormSection index=i />
                    }
                })
                .collect::<Vec<_>>()}
        </div>
    }
}

#[component]
pub fn AdultFormSection(
    index: u32,
    #[prop(into)] user_email: Signal<Option<String>>,
) -> impl IntoView {
    // Auto-fill email for primary adult (index 0) when user email is available
    create_effect(move |_| {
        if index == 0 {
            if let Some(email) = user_email.get() {
                // Check if email field is empty before auto-filling
                let adults_list = BlockRoomUIState::get_adults_untracked();
                if let Some(adult) = adults_list.get(index as usize) {
                    if adult.email.is_none() || adult.email.as_ref().map_or(true, |e| e.is_empty())
                    {
                        log!(
                            "Auto-filling primary adult email from logged-in user identity: {}",
                            email
                        );
                        BlockRoomUIState::update_adult(index as usize, "email", email);
                        BlockRoomUIState::validate_form();
                    }
                }
            }
        }
    });

    let update_adult = move |field: &str, value: String| {
        log!(
            "AdultFormSection update_adult called - index: {}, field: '{}', value: '{}'",
            index,
            field,
            value
        );
        BlockRoomUIState::update_adult(index as usize, field, value.clone());
        BlockRoomUIState::validate_form();

        // Debug: Check if the update actually worked
        let adults_list = BlockRoomUIState::get_adults_untracked();
        if let Some(adult) = adults_list.get(index as usize) {
            log!(
                "After update - Adult {}: first_name='{}', email={:?}, phone={:?}",
                index,
                adult.first_name,
                adult.email,
                adult.phone
            );
        }
    };

    // Create reactive signal for current email value to show in input
    let current_email = Signal::derive(move || {
        let adults_list = BlockRoomUIState::get_adults();
        adults_list
            .get(index as usize)
            .and_then(|adult| adult.email.clone())
            .unwrap_or_default()
    });

    let email_valid = Signal::derive(move || {
        let email = current_email.get();
        !email.trim().is_empty() && BlockRoomUIState::is_valid_email(&email)
    });

    let room_label = Signal::derive(move || {
        let mut labels = vec![];
        for room in BlockRoomUIState::get_room_selection_summary() {
            for _ in 0..room.quantity {
                labels.push(room.display_name());
            }
        }
        labels
            .get(index as usize)
            .cloned()
            .unwrap_or_else(|| format!("Room {}", index + 1))
    });

    let (country_code, set_country_code) = create_signal("+91".to_string());
    let (phone_number, set_phone_number) = create_signal(String::new());

    let combine_phone = |code: &str, number: &str| {
        if number.trim().is_empty() {
            String::new()
        } else {
            format!("{code} {number}")
        }
    };

    view! {
        <div class="person-details bg-gray-50 rounded-2xl border border-gray-200 shadow-sm p-6 sm:p-7 space-y-5">
            <div class="space-y-1">
                <h3 class="text-lg sm:text-xl font-semibold text-gray-900">
                    {move || {
                        let descriptor = if index == 0 {
                            "Primary Guest Detail"
                        } else {
                            "Guest Details"
                        };
                        format!("{}, {}", room_label.get(), descriptor)
                    }}
                </h3>
            </div>

            <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 sm:gap-5">
                <div class="space-y-2">
                    <label class="text-sm font-medium text-gray-800">
                        "First Name"
                        <span class="text-red-500">*</span>
                    </label>
                    <input
                        type="text"
                        placeholder="Enter Name"
                        class="w-full rounded-xl border border-gray-200 bg-gray-50 p-3.5 text-sm text-gray-900 placeholder:text-gray-400 focus:bg-white focus:border-blue-500 focus:ring-2 focus:ring-blue-100 transition-colors"
                        required=true
                        on:input=move |ev| {
                            let value = event_target_value(&ev);
                            update_adult("first_name", value.clone());

                            if value.trim().is_empty() {
                                // spot reserved for inline validation messaging
                            }
                        }
                        on:blur=move |_| {
                            BlockRoomUIState::validate_form();
                        }
                    />
                </div>
                <div class="space-y-2">
                    <label class="text-sm font-medium text-gray-800">
                        "Last Name"
                        <span class="text-red-500">*</span>
                    </label>
                    <input
                        type="text"
                        placeholder="Enter Surname"
                        class="w-full rounded-xl border border-gray-200 bg-gray-50 p-3.5 text-sm text-gray-900 placeholder:text-gray-400 focus:bg-white focus:border-blue-500 focus:ring-2 focus:ring-blue-100 transition-colors"
                        required=true
                        on:input=move |ev| {
                            update_adult("last_name", event_target_value(&ev));
                        }
                    />
                </div>
            </div>

            // Primary adult gets email and phone fields
            {move || {
                if index == 0 {
                    view! {
                        <div class="grid grid-cols-1 gap-4 sm:gap-5">
                            <div class="space-y-2">
                                <label class="text-sm font-medium text-gray-800">
                                    "Email ID"
                                    <span class="text-red-500">*</span>
                                </label>
                                <div class="relative">
                                    <input
                                        type="email"
                                        placeholder="Enter Email"
                                        class=move || {
                                            let base = "w-full rounded-xl border border-gray-200 p-3.5 text-sm text-gray-900 placeholder:text-gray-400 pr-11";
                                            if user_email.get().is_some() {
                                                format!("{base} bg-gray-100 cursor-not-allowed")
                                            } else {
                                                format!("{base} bg-gray-50 focus:bg-white focus:border-blue-500 focus:ring-2 focus:ring-blue-100 transition-colors")
                                            }
                                        }
                                        required=true
                                        value=move || current_email.get()
                                        readonly=move || user_email.get().is_some()
                                        disabled=move || user_email.get().is_some()
                                        on:input=move |ev| {
                                            if user_email.get().is_none() {
                                                let value = event_target_value(&ev);
                                                update_adult("email", value.clone());
                                            }
                                        }
                                        on:blur=move |_| {
                                            if user_email.get().is_none() {
                                                BlockRoomUIState::validate_form();
                                            }
                                        }
                                    />
                                    <Show when=move || email_valid.get()>
                                        <span class="absolute inset-y-0 right-3 flex items-center text-green-500">
                                            <Icon icon=icondata::AiCheckCircleOutlined class="text-xl" />
                                        </span>
                                    </Show>
                                </div>
                            </div>

                            <div class="space-y-2">
                                <label class="text-sm font-medium text-gray-800">
                                    "Phone Number"
                                    <span class="text-red-500">*</span>
                                </label>
                                <div class="flex flex-col sm:flex-row gap-3">
                                    <select
                                        class="sm:w-28 w-full rounded-xl border border-gray-200 bg-white p-3.5 text-sm text-gray-900 focus:border-blue-500 focus:ring-2 focus:ring-blue-100 transition-colors"
                                        value=move || country_code.get()
                                        on:input=move |ev| {
                                            let value = event_target_value(&ev);
                                            let combined = combine_phone(&value, &phone_number.get_untracked());
                                            set_country_code.set(value.clone());
                                            update_adult("phone", combined);
                                        }
                                    >
                                        <option value="+91">"India +91"</option>
                                        <option value="+1">"USA +1"</option>
                                        <option value="+44">"UK +44"</option>
                                        <option value="+61">"AU +61"</option>
                                    </select>
                                    <input
                                        type="tel"
                                        placeholder="Enter Number"
                                        class="flex-1 rounded-xl border border-gray-200 bg-gray-50 p-3.5 text-sm text-gray-900 placeholder:text-gray-400 focus:bg-white focus:border-blue-500 focus:ring-2 focus:ring-blue-100 transition-colors"
                                        required=true
                                        on:input=move |ev| {
                                            let value = event_target_value(&ev);
                                            set_phone_number.set(value.clone());
                                            let combined =
                                                combine_phone(&country_code.get_untracked(), &value);
                                            update_adult("phone", combined);

                                            if !value.trim().is_empty() && !BlockRoomUIState::is_valid_phone(&value) {
                                                // reserved for inline validation messaging
                                            }
                                        }
                                        on:blur=move |_| {
                                            BlockRoomUIState::validate_form();
                                        }
                                    />
                                </div>
                            </div>
                        </div>
                    }.into_view()
                } else {
                    view! { <div></div> }.into_view()
                }
            }}
        </div>
    }
}

#[component]
pub fn ChildFormSection(index: u32) -> impl IntoView {
    let ui_search_ctx: UISearchCtx = expect_context();

    let update_child = move |field: &str, value: String| {
        BlockRoomUIState::update_child(index as usize, field, value);
        BlockRoomUIState::validate_form();
    };

    let age_value = ui_search_ctx.guests.children_ages.get_value_at(index);

    view! {
        <div class="person-details bg-gray-50 rounded-2xl border border-gray-200 shadow-sm p-6 sm:p-7 space-y-5">
            <div class="space-y-1">
                <h3 class="text-lg sm:text-xl font-semibold text-gray-900">
                    {format!("Child {}", index + 1)}
                </h3>
            </div>
            <div class="grid grid-cols-1 sm:grid-cols-3 gap-4 sm:gap-5">
                <div class="space-y-2">
                    <label class="text-sm font-medium text-gray-800">
                        "First Name"
                        <span class="text-red-500">*</span>
                    </label>
                    <input
                        type="text"
                        placeholder="Enter Name"
                        class="w-full rounded-xl border border-gray-200 bg-gray-50 p-3.5 text-sm text-gray-900 placeholder:text-gray-400 focus:bg-white focus:border-blue-500 focus:ring-2 focus:ring-blue-100 transition-colors"
                        required=true
                        on:input=move |ev| {
                            update_child("first_name", event_target_value(&ev));
                        }
                    />
                </div>
                <div class="space-y-2">
                    <label class="text-sm font-medium text-gray-800">"Last Name"</label>
                    <input
                        type="text"
                        placeholder="Enter Surname"
                        class="w-full rounded-xl border border-gray-200 bg-gray-50 p-3.5 text-sm text-gray-900 placeholder:text-gray-400 focus:bg-white focus:border-blue-500 focus:ring-2 focus:ring-blue-100 transition-colors"
                        on:input=move |ev| {
                            update_child("last_name", event_target_value(&ev));
                        }
                    />
                </div>
                <div class="space-y-2">
                    <label class="text-sm font-medium text-gray-800">
                        "Age"
                        <span class="text-red-500">*</span>
                    </label>
                    <select
                        class="w-full rounded-xl border border-gray-200 bg-white p-3.5 text-sm text-gray-900 focus:border-blue-500 focus:ring-2 focus:ring-blue-100 transition-colors"
                        required=true
                        on:input=move |ev| {
                            update_child("age", event_target_value(&ev));
                        }
                    >
                        <option disabled selected>{age_value}</option>
                        {(1..18)
                            .map(|age| {
                                let selected = if age == age_value {
                                    "selected"
                                } else {
                                    ""
                                };
                                view! {
                                    <option value=age.to_string() {selected}>{age}</option>
                                }
                            })
                            .collect::<Vec<_>>()}
                    </select>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn TermsCheckbox() -> impl IntoView {
    view! {
        <div class="flex items-start gap-2 text-sm text-gray-700">
            <input
                type="checkbox"
                id="agree"
                class="mt-1 h-4 w-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
                on:change=move |ev| {
                    BlockRoomUIState::set_terms_accepted(event_target_checked(&ev));
                    BlockRoomUIState::validate_form();
                }
            />
            <label for="agree" class="leading-relaxed">
                "By continuing you accept our Cancellation Policy and Terms."
            </label>
        </div>
    }
}

#[component]
pub fn PromoCodeSection() -> impl IntoView {
    let (promo_code, set_promo_code) = create_signal(String::new());

    let apply_code = move |_| {
        let code = promo_code.get_untracked();
        if !code.trim().is_empty() {
            log!("Promo code apply clicked: {}", code);
        }
    };

    let apply_button_class = move || {
        if promo_code.get().trim().is_empty() {
            "w-full sm:w-28 rounded-xl bg-blue-200 text-white font-semibold px-4 py-3 text-sm cursor-not-allowed"
        } else {
            "w-full sm:w-28 rounded-xl bg-blue-400 hover:bg-blue-500 text-white font-semibold px-4 py-3 text-sm transition-colors"
        }
    };

    view! {
        <div class="bg-white rounded-3xl border border-gray-200 shadow-md p-5 sm:p-6 space-y-3">
            <h3 class="text-lg sm:text-xl font-semibold text-gray-900">"Redeem Promo Code"</h3>
            <div class="flex flex-col sm:flex-row gap-3">
                <input
                    type="text"
                    placeholder="Enter Code"
                    class="flex-1 rounded-xl border border-gray-200 bg-gray-50 p-3.5 text-sm text-gray-900 placeholder:text-gray-400 focus:bg-white focus:border-blue-500 focus:ring-2 focus:ring-blue-100 transition-colors"
                    value=move || promo_code.get()
                    on:input=move |ev| {
                        set_promo_code.set(event_target_value(&ev));
                    }
                />
                <button
                    class=apply_button_class
                    disabled=move || promo_code.get().trim().is_empty()
                    on:click=apply_code
                >
                    "Apply"
                </button>
            </div>
        </div>
    }
}

#[component]
pub fn ConfirmButton(
    mobile: bool,
    #[prop(into)] booking_id_signal: Signal<Option<BookingId>>,
) -> impl IntoView {
    let block_room_state: BlockRoomUIState = expect_context();
    let is_form_valid = move || BlockRoomUIState::get_form_valid();

    // Email verification state - use centralized state
    let email_state = EmailVerificationState::from_leptos_context();

    // Create action for integrated prebook + backend save API call
    let prebook_action = create_action(move |_: &()| async move {
        log!("Integrated prebook action triggered - calling integrated API");
        BlockRoomUIState::set_loading(true);

        // Get required data for integrated call
        let block_room_state: BlockRoomUIState = expect_context();
        let hotel_info_ctx: HotelInfoCtx = expect_context();

        // Debug logging for prebook action
        let adults_list = block_room_state.adults.get_untracked();
        log!("Prebook action - adults list: {:?}", adults_list);
        if let Some(first_adult) = adults_list.first() {
            log!(
                "Prebook action - first adult email: {:?}",
                first_adult.email
            );
            log!(
                "Prebook action - first adult first_name: '{}'",
                first_adult.first_name
            );
            log!(
                "Prebook action - first adult phone: {:?}",
                first_adult.phone
            );
        }

        let email = adults_list.first().and_then(|adult| adult.email.clone());

        let Some(email) = email else {
            log!("Integrated prebook action failed - no primary adult email");
            BlockRoomUIState::batch_update_on_error(
                Some("validation".to_string()),
                Some("Primary adult email is required".to_string()),
                Some("Missing primary adult email for booking".to_string()),
            );
            return None;
        };

        // Use the booking ID from the shared signal
        let Some(booking_id) = booking_id_signal.get_untracked() else {
            log!("Integrated prebook action failed - no booking ID available");
            BlockRoomUIState::batch_update_on_error(
                Some("validation".to_string()),
                Some("Unable to get booking reference".to_string()),
                Some("No booking ID available".to_string()),
            );
            return None;
        };

        // Note: We don't need block_room_id here since it's set AFTER successful prebook
        // The BookingService will use hotel_code as token for LiteAPI prebook call

        // Use BookingService for integrated call (block room + backend save in one call)
        let booking_service = BookingService::new();

        log!(
            "Calling integrated block room service for booking_id: {}, email: {}",
            booking_id.to_order_id(),
            email
        );

        match booking_service
            .block_room_with_backend_integration(booking_id.to_order_id(), email, None)
            .await
        {
            Ok(response) => {
                log!("Integrated prebook action: Successfully completed block room + backend save");

                // For now, we don't get detailed pricing from the integrated response
                // The UI pricing calculations are sufficient until we need API pricing updates
                let current_total = response.1.unwrap_or(BlockRoomUIState::get_total_price());
                let current_room_price = BlockRoomUIState::get_room_price();

                BlockRoomUIState::batch_update_on_success(
                    booking_id.to_order_id(),
                    current_total,
                    current_total,
                );

                log!("Integrated prebook action: After batch_update_on_success - loading: {}, block_room_called: {}", 
                     BlockRoomUIState::get_loading(), BlockRoomUIState::get_block_room_called());

                Some((booking_id.to_order_id(), current_total))
            }
            Err(e) => {
                log!(
                    "Integrated prebook action failed: {}",
                    e.technical_details()
                );
                BlockRoomUIState::batch_update_on_error(
                    Some(e.category().to_string()),
                    Some(e.user_message()),
                    Some(e.technical_details()),
                );
                None
            }
        }
    });

    let open_modal = move |_| {
        if is_form_valid() {
            if !EmailVerificationState::get_email_verified() {
                // Show email verification step first
                EmailVerificationState::start_verification_flow();
            } else {
                if let Some(email) = AuthStateSignal::auth_state().get_untracked().email {
                    generate_app_reference(email.clone());
                }
                // Email already verified, proceed with booking
                BlockRoomUIState::set_show_payment_modal(true);
                prebook_action.dispatch(());
            }
        }
    };

    // <!-- Phase 4.3: Enhanced button styling with validation feedback -->
    let button_class =
        "mt-6 w-full rounded-2xl py-3.5 text-white text-base sm:text-lg font-semibold shadow-md min-h-[48px] transition-all duration-200";

    let button_style = move || {
        if is_form_valid() {
            "bg-blue-500 hover:bg-blue-600 hover:shadow-lg"
        } else {
            "bg-blue-200 cursor-not-allowed"
        }
    };

    // Create callbacks outside the view to avoid lifetime issues
    let on_email_verified = {
        let prebook_action = prebook_action.clone();
        Callback::new(move |_: ()| {
            // Email verification is already handled in EmailVerificationState::complete_verification
            // Proceed with booking
            BlockRoomUIState::set_show_payment_modal(true);
            prebook_action.dispatch(());
        })
    };

    let on_email_cancel = Callback::new(move |_: ()| {
        // Cancellation is already handled in EmailVerificationState::cancel_verification
    });

    view! {
        <button
            class=move || format!("{} {}", button_class, button_style())
            disabled=move || !is_form_valid()
            on:click=open_modal
        >
            // <!-- Phase 4.3: Dynamic button text with validation feedback -->
            {move || {
                if is_form_valid() {
                    if EmailVerificationState::get_email_verified() {
                        "Confirm & Book"
                    } else {
                        "Verify Email & Book"
                    }
                } else {
                    "Complete Required Fields"
                }
            }}
        </button>

        // Show email verification component when needed
        <Show when=move || EmailVerificationState::get_show_verification_modal()>
            <EmailVerificationStep
                on_verified=on_email_verified
                on_cancel=on_email_cancel
            />
        </Show>
    }
}

#[component]
pub fn PaymentModal() -> impl IntoView {
    let block_room_state: BlockRoomUIState = expect_context();
    let ui_search_ctx: UISearchCtx = expect_context();
    let hotel_info_ctx: HotelInfoCtx = expect_context();

    let show_modal = move || BlockRoomUIState::get_show_payment_modal();
    // let is_loading = move || BlockRoomUIState::get_loading();
    // let block_room_called = move || BlockRoomUIState::get_block_room_called();

    let room_price = move || BlockRoomUIState::get_room_price();
    let calculated_total = move || BlockRoomUIState::get_calculated_total_from_summary();
    let num_nights = move || BlockRoomUIState::get_num_nights();

    // Note: Prebook API is now called when user clicks "Confirm & Book" button via action pattern

    let close_modal = move |_| {
        BlockRoomUIState::set_show_payment_modal(false);
    };

    view! {
        <Show when=show_modal>
            <div class="fixed inset-0 flex items-center justify-center z-50">
                <div
                    class="fixed inset-0 bg-black opacity-50"
                    on:click=close_modal
                />
                <div class="w-full max-w-lg bg-white rounded-lg p-4 sm:p-8 z-50 shadow-xl relative mx-2">
                    <button
                        class="absolute top-2 right-2 sm:top-4 sm:right-4 text-gray-500 hover:text-gray-700"
                        on:click=close_modal
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                        </svg>
                    </button>

                    // <!-- Phase 4.2: Enhanced loading states and error components -->
                    <Show
                        when=move || !BlockRoomUIState::get_loading() && BlockRoomUIState::get_block_room_called()
                        fallback=move || view! {
                            <div class="flex flex-col justify-center items-center h-40 space-y-4">
                                <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
                                <div class="text-center space-y-2">
                                    <div class="font-semibold text-lg">
                                        "Reserving Your Room"
                                    </div>
                                    <div class="text-sm text-gray-600">
                                        "Securing your reservation for 15 minutes..."
                                    </div>
                                </div>
                            </div>
                        }
                    >
                        // <!-- Phase 4.2: Enhanced error display -->
                        <Show when=move || BlockRoomUIState::get_error().is_some()>
                            <EnhancedErrorDisplay />
                        </Show>

                        <Show when=move || BlockRoomUIState::get_error().is_none()>
                            <h2 class="text-xl font-bold text-center mb-6">Payment</h2>
                            <div class="flex flex-col gap-2 mb-6">
                                // <div class="flex justify-between items-end">
                                //     <span class="text-lg font-bold">{move || format!("${:.2}", room_price())}</span>
                                //     // <span class="ml-1 text-base font-normal text-gray-600">/night</span>
                                // </div>
                                // <div class="flex justify-between items-center text-base">
                                //     // <span class="text-gray-700">
                                //     //     {move || format!("${:.2} x {} nights", room_price(), num_nights())}
                                //     // </span>
                                //     <span class="font-semibold">
                                //         {move || format!("${:.2}", room_price() * num_nights() as f64)}
                                //     </span>
                                // </div>
                                <Divider class="my-2".into() />
                                <div class="flex justify-between items-center font-bold text-lg mb-2">
                                    <span>Total</span>
                                    <span class="text-2xl">{move || format!("${:.2}", calculated_total())}</span>
                                </div>
                            </div>

                            <div class="font-bold">
                                <label>"Pay with"</label>
                                <div class="flex flex-col w-full mt-4 space-y-2">
                                    <PaymentProviderButtons />
                                </div>
                            </div>
                        </Show>
                    </Show>
                </div>
            </div>
        </Show>
    }
}

// // <!-- Phase 4.1: Specialized Amenities Display Component -->
// #[component]
// pub fn HotelAmenitiesDisplay() -> impl IntoView {
//     let block_room_state: BlockRoomUIState = expect_context();

//     let hotel_context = move || block_room_state.hotel_context.get();

//     // Amenity icon mapping based on hotel_details_v1.rs
//     let get_amenity_icon = |facility: &str| -> Option<icondata::Icon> {
//         match facility.to_lowercase().as_str() {
//             f if f.contains("wifi") || f.contains("internet") => Some(icondata::BsWifi),
//             f if f.contains("parking") => Some(icondata::FaCarSolid),
//             f if f.contains("fitness") || f.contains("gym") => Some(icondata::FaDumbbellSolid),
//             f if f.contains("pool") || f.contains("swimming") => Some(icondata::FaWaterSolid),
//             f if f.contains("spa") => Some(icondata::FaSpaSolid),
//             f if f.contains("restaurant") || f.contains("dining") => Some(icondata::FaUtensilsSolid),
//             f if f.contains("bar") || f.contains("lounge") => Some(icondata::FaGlassWaterSolid),
//             f if f.contains("business") || f.contains("meeting") => Some(icondata::FaBriefcaseSolid),
//             f if f.contains("concierge") => Some(icondata::FaBellSolid),
//             f if f.contains("laundry") => Some(icondata::FaShirtSolid),
//             f if f.contains("pet") => Some(icondata::FaPawSolid),
//             f if f.contains("air conditioning") || f.contains("ac") => Some(icondata::TbAirConditioning),
//             f if f.contains("elevator") => Some(icondata::FaElevatorSolid),
//             _ => Some(icondata::AiCheckCircleOutlined), // Default checkmark for other amenities
//         }
//     };

//     view! {
//         <Show when=move || hotel_context().is_some()>
//             <div class="mt-4">
//                 <div class="flex items-center gap-2 mb-3">
//                     <Icon icon=icondata::FaStarSolid class="text-yellow-400 text-sm" />
//                     <span class="text-xs text-gray-400 font-semibold">Hotel Amenities</span>
//                 </div>

//                 <div class="amenities-grid grid grid-cols-2 sm:grid-cols-3 gap-2">
//                     {move || {
//                         if let Some(hotel) = hotel_context() {
//                             // Take first 6 facilities for compact display
//                             hotel.hotel_facilities.iter()
//                                 .take(6)
//                                 .map(|facility| {
//                                     let icon = get_amenity_icon(facility);
//                                     view! {
//                                         <div class="flex items-center gap-2 p-2 bg-gray-50 rounded-lg">
//                                             {match icon {
//                                                 Some(icon_data) => view! {
//                                                     <Icon icon=icon_data class="text-gray-600 text-sm flex-shrink-0" />
//                                                 }.into_view(),
//                                                 None => view! {
//                                                     <div class="w-4 h-4 bg-gray-300 rounded-full flex-shrink-0"></div>
//                                                 }.into_view()
//                                             }}
//                                             <span class="text-xs text-gray-700 truncate">{facility.clone()}</span>
//                                         </div>
//                                     }
//                                 })
//                                 .collect::<Vec<_>>()
//                         } else {
//                             vec![]
//                         }
//                     }}
//                 </div>

//                 // <!-- Show more amenities if available -->
//                 <Show when=move || {
//                     hotel_context().map_or(false, |hotel| hotel.hotel_facilities.len() > 6)
//                 }>
//                     <div class="mt-2 text-center">
//                         <span class="text-xs text-blue-600 cursor-pointer hover:underline">
//                             {move || {
//                                 let remaining = hotel_context()
//                                     .map_or(0, |hotel| hotel.hotel_facilities.len().saturating_sub(6));
//                                 format!("+ {} more amenities", remaining)
//                             }}
//                         </span>
//                     </div>
//                 </Show>
//             </div>
//         </Show>
//     }
// }

// <!-- Phase 4.2: Enhanced Loading State Component -->
// #[component]
// pub fn EnhancedLoadingState() -> impl IntoView {
//     let block_room_state: BlockRoomUIState = expect_context();

//     // let is_availability_checking = move || block_room_state.availability_checking.get();
//     let is_blocking_room = move || block_room_state.loading.get();

//     view! {
//         <div class="flex flex-col justify-center items-center h-40 space-y-4">
//             // <!-- Animated spinner -->
//             <div class="relative">
//                 <SpinnerGray />
//                 // <!-- Pulsing ring animation -->
//                 <div class="absolute inset-0 border-4 border-blue-200 rounded-full animate-ping opacity-75"></div>
//             </div>

//             // <!-- Dynamic loading message -->
//             <div class="text-center space-y-2">
//                 <div class="font-semibold text-lg">
//                     {move || {
//                         if is_availability_checking() {
//                             "Checking Room Availability"
//                         } else if is_blocking_room() {
//                             "Reserving Your Room"
//                         } else {
//                             "Processing Request"
//                         }
//                     }}
//                 </div>
//                 <div class="text-sm text-gray-600">
//                     {move || {
//                         if is_availability_checking() {
//                             "Verifying room availability and pricing..."
//                         } else if is_blocking_room() {
//                             "Securing your reservation for 15 minutes..."
//                         } else {
//                             "Please wait while we process your request..."
//                         }
//                     }}
//                 </div>
//             </div>

//             // <!-- Progress dots animation -->
//             <div class="flex space-x-1">
//                 <div class="w-2 h-2 bg-blue-500 rounded-full animate-bounce"></div>
//                 <div class="w-2 h-2 bg-blue-500 rounded-full animate-bounce" style="animation-delay: 0.1s"></div>
//                 <div class="w-2 h-2 bg-blue-500 rounded-full animate-bounce" style="animation-delay: 0.2s"></div>
//             </div>
//         </div>
//     }
// }

// <!-- Phase 4.2: Enhanced Error Display Component -->
#[component]
pub fn EnhancedErrorDisplay() -> impl IntoView {
    let block_room_state: BlockRoomUIState = expect_context();

    let error_message = move || BlockRoomUIState::get_error().unwrap_or_default();
    let error_type = move || BlockRoomUIState::get_api_error_type();
    let can_retry = move || BlockRoomUIState::can_retry();

    // Error-specific icons and colors
    let get_error_display = move || match error_type().as_deref() {
        Some("room_unavailable") => (icondata::FaBedSolid, "text-orange-600", "bg-orange-50"),
        Some("network") => (icondata::BsWifi, "text-red-600", "bg-red-50"),
        Some("validation") => (
            icondata::AiExclamationCircleOutlined,
            "text-yellow-600",
            "bg-yellow-50",
        ),
        Some("server") => (icondata::FaServerSolid, "text-red-600", "bg-red-50"),
        _ => (icondata::AiWarningOutlined, "text-gray-600", "bg-gray-50"),
    };

    let retry_action = move |_| {
        if can_retry() {
            // Reset error state and try again (prebook API will recheck availability)
            BlockRoomUIState::set_error(None);
            BlockRoomUIState::set_api_error(None, None, None);
            // Modal will automatically retry due to the effect
        }
    };

    let close_and_return = move |_| {
        BlockRoomUIState::set_show_payment_modal(false);
        BlockRoomUIState::set_error(None);
        BlockRoomUIState::set_api_error(None, None, None);
    };

    view! {
        <div class=format!("p-6 rounded-lg {}", get_error_display().2)>
            <div class="flex flex-col items-center text-center space-y-4">
                // <!-- Error icon -->
                <div class=format!("w-16 h-16 rounded-full {} flex items-center justify-center", get_error_display().2)>
                    <Icon icon=get_error_display().0 class=format!("text-3xl {}", get_error_display().1) />
                </div>

                // <!-- Error title -->
                <div class="space-y-2">
                    <h3 class="text-lg font-bold text-gray-900">
                        {move || match error_type().as_deref() {
                            Some("room_unavailable") => "Room No Longer Available",
                            Some("network") => "Connection Issue",
                            Some("validation") => "Booking Information Issue",
                            Some("server") => "Service Temporarily Unavailable",
                            _ => "Something Went Wrong"
                        }}
                    </h3>

                    // <!-- Error message -->
                    <p class="text-gray-600 text-sm">
                        {error_message}
                    </p>
                </div>

                // <!-- Action buttons -->
                <div class="flex flex-col sm:flex-row gap-3 w-full">
                    <Show when=can_retry>
                        <button
                            class="flex-1 bg-blue-600 text-white px-4 py-2 rounded-lg hover:bg-blue-700 font-medium"
                            on:click=retry_action
                        >
                            "Try Again"
                        </button>
                    </Show>

                    <button
                        class="flex-1 bg-gray-100 text-gray-700 px-4 py-2 rounded-lg hover:bg-gray-200 font-medium"
                        on:click=close_and_return
                    >
                        {move || if error_type().as_deref() == Some("room_unavailable") {
                            "Select Different Room"
                        } else {
                            "Go Back"
                        }}
                    </button>
                </div>

                // <!-- Retry count indicator -->
                <Show when=move || { BlockRoomUIState::get_retry_count() > 0 }>
                    <div class="text-xs text-gray-500">
                        {move || format!("Attempt {} of 3", BlockRoomUIState::get_retry_count() + 1)}
                    </div>
                </Show>
            </div>
        </div>
    }
}

// Note: build_block_room_request and save_booking_to_backend functions removed
// The integrated server function now handles both the block room API call and backend save
// using BookingConversions::ui_to_block_room_request() on the server side

#[component]
pub fn EmailVerificationStep(
    #[prop(into)] on_verified: Callback<()>,
    #[prop(into)] on_cancel: Callback<()>,
) -> impl IntoView {
    let block_room_state: BlockRoomUIState = expect_context();
    let email_state = EmailVerificationState::from_leptos_context();

    // Get email from form
    let get_email = move || {
        let adults_list = block_room_state.adults.get();
        adults_list.first().and_then(|adult| adult.email.clone())
    };

    // Get booking ID using centralized state management
    let get_booking_id = move || {
        if let Some(email) = get_email() {
            // booking id is created during email verification itself!
            BookingIdState::create_booking_id(email).map(|booking_id| booking_id.to_order_id())
        } else {
            None
        }
    };

    // Timer for resend countdown
    let timer = use_interval_fn(
        move || {
            EmailVerificationState::tick_timer();
        },
        1000, // 1 second
    );

    // Start timer when it becomes active
    create_effect(move |_| {
        if EmailVerificationState::get_timer_active() {
            (timer.resume)();
        } else {
            (timer.pause)();
        }
    });

    // Action to send OTP
    let send_otp_action = create_action(move |_: &()| {
        let get_email = get_email.clone();
        let get_booking_id = get_booking_id.clone();
        async move {
            EmailVerificationState::start_send_otp();

            let email = get_email();
            let booking_id = get_booking_id();

            if let (Some(email), Some(booking_id)) = (email, booking_id) {
                let client = ClientSideApiClient::new();
                match client.send_otp_email(email, booking_id).await {
                    Ok(response) => {
                        EmailVerificationState::handle_send_otp_success(response);
                    }
                    Err(e) => {
                        EmailVerificationState::handle_send_otp_error(e);
                    }
                }
            } else {
                EmailVerificationState::handle_send_otp_error(
                    "Missing email or booking ID".to_string(),
                );
            }
        }
    });

    // Action to verify OTP
    let verify_otp_action = create_action(move |otp: &String| {
        let read_booking_id =
            BookingId::read_from_local_storage().map(|booking_id| booking_id.to_order_id());
        let otp = otp.clone();
        async move {
            EmailVerificationState::start_verify_otp();

            if let Some(booking_id) = read_booking_id {
                let client = ClientSideApiClient::new();
                match client.verify_otp(booking_id, otp).await {
                    Ok(response) => {
                        if EmailVerificationState::handle_verify_otp_success(response) {
                            // Verification successful - trigger callback
                            Callable::call(&on_verified, ());
                        }
                    }
                    Err(e) => {
                        EmailVerificationState::handle_verify_otp_error(e);
                    }
                }
            } else {
                EmailVerificationState::handle_verify_otp_error("Missing booking ID".to_string());
            }
        }
    });

    let send_otp = move |_| {
        send_otp_action.dispatch(());
    };

    let verify_otp = move |_| {
        if EmailVerificationState::can_verify_otp() {
            verify_otp_action.dispatch(EmailVerificationState::get_otp_value());
        }
    };

    let resend_otp = move |_| {
        if EmailVerificationState::get_can_resend() {
            EmailVerificationState::resend_otp();
            send_otp_action.dispatch(());
        }
    };

    let cancel_verification = move |_| {
        EmailVerificationState::cancel_verification();
        Callable::call(&on_cancel, ());
    };

    // Handle Enter key in OTP input
    let handle_otp_keypress = move |ev: ev::KeyboardEvent| {
        if ev.key() == "Enter" && EmailVerificationState::can_verify_otp() {
            verify_otp(ev::MouseEvent::new("click").unwrap());
        }
    };

    view! {
        <div class="email-verification-overlay fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
            <div class="bg-white rounded-lg p-6 w-full max-w-md mx-4 shadow-xl">
                // Close button
                <div class="flex justify-between items-center mb-4">
                    <h2 class="text-xl font-bold">"Verify Your Email"</h2>
                    <button
                        class="text-gray-400 hover:text-gray-600"
                        on:click=cancel_verification
                    >
                        <Icon icon=icondata::AiCloseOutlined class="text-xl" />
                    </button>
                </div>

                <Show
                    when=move || !EmailVerificationState::get_otp_sent() || !EmailVerificationState::get_show_otp_input()
                    fallback=move || view! {
                        // OTP input form
                        <div class="space-y-4">
                            <p class="text-gray-600">
                                "We've sent a 6-digit verification code to "
                                <span class="font-semibold">{move || get_email().unwrap_or_default()}</span>
                            </p>
                            <p class="text-sm text-gray-500">
                                "The code will expire in 10 minutes."
                            </p>
                            <input
                                type="text"
                                placeholder="000000"
                                maxlength="6"
                                class="w-full text-center text-2xl tracking-widest border rounded-lg p-3 focus:border-blue-500 focus:ring-2 focus:ring-blue-200"
                                value=move || EmailVerificationState::get_otp_value()
                                on:input=move |ev| {
                                    let value = event_target_value(&ev);
                                    EmailVerificationState::update_otp_value(value);
                                }
                                on:keypress=handle_otp_keypress
                                autofocus=true
                            />
                            <div class="flex space-x-3">
                                <button
                                    class="flex-1 bg-blue-600 text-white rounded-lg py-3 font-semibold hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
                                    disabled=move || !EmailVerificationState::can_verify_otp()
                                    on:click=verify_otp
                                >
                                    {move || if EmailVerificationState::get_verify_otp_loading() { "Verifying..." } else { "Verify Email" }}
                                </button>
                                <div class="flex flex-col items-center">
                                    <button
                                        class="px-4 py-3 text-blue-600 hover:text-blue-800 font-semibold disabled:text-gray-400 disabled:cursor-not-allowed"
                                        disabled=move || !EmailVerificationState::get_can_resend() || EmailVerificationState::get_send_otp_loading()
                                        on:click=resend_otp
                                    >
                                        {move || if EmailVerificationState::get_send_otp_loading() { "Sending..." } else { "Resend Code" }}
                                    </button>

                                    // Timer display
                                    <Show when=move || EmailVerificationState::get_timer_active()>
                                        <div class="flex items-center text-xs text-gray-500 mt-1">
                                            <Icon icon=icondata::AiClockCircleOutlined class="mr-1" />
                                            <span>"Resend in " {move || EmailVerificationState::format_timer()}</span>
                                        </div>
                                    </Show>
                                </div>
                            </div>
                        </div>
                    }
                >
                    // Send OTP form
                    <div class="space-y-4">
                        <p class="text-gray-600">
                            "We'll send a verification code to:"
                        </p>
                        <div class="bg-gray-50 rounded-lg p-3">
                            <span class="font-semibold">{move || get_email().unwrap_or("No email found".to_string())}</span>
                        </div>
                        <p class="text-sm text-gray-500">
                            "Please make sure this email address is correct before proceeding."
                        </p>
                        <button
                            class="w-full bg-blue-600 text-white rounded-lg py-3 font-semibold hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
                            disabled=move || EmailVerificationState::get_send_otp_loading() || get_email().is_none()
                            on:click=send_otp
                        >
                            {move || if EmailVerificationState::get_send_otp_loading() { "Sending..." } else { "Send Verification Code" }}
                        </button>
                    </div>
                </Show>

                // Error display
                <Show when=move || EmailVerificationState::get_verification_error().is_some()>
                    <div class="mt-4 p-3 bg-red-50 border border-red-200 rounded-lg">
                        <div class="flex items-center">
                            <Icon icon=icondata::AiExclamationCircleOutlined class="text-red-500 mr-2" />
                            <p class="text-red-600 text-sm">{move || EmailVerificationState::get_verification_error().unwrap_or_default()}</p>
                        </div>
                    </div>
                </Show>
            </div>
        </div>
    }
}

#[component]
pub fn PaymentProviderButtons() -> impl IntoView {
    let block_room_state: BlockRoomUIState = expect_context();
    let ui_search_ctx: UISearchCtx = expect_context();

    // Get pricing information
    let calculated_total = move || BlockRoomUIState::get_calculated_total_from_summary();

    // Payment loading state
    let (payment_loading, set_payment_loading) = create_signal(false);
    let (selected_provider, set_selected_provider) = create_signal::<Option<PaymentProvider>>(None);

    // Create payment action
    let create_payment_action = create_action(move |provider: &PaymentProvider| {
        let provider = provider.clone();
        async move {
            log!("Creating payment invoice with provider: {:?}", provider);
            set_payment_loading.set(true);
            set_selected_provider.set(Some(provider.clone()));

            // Get booking details
            let block_room_state: BlockRoomUIState = expect_context();
            let ui_search_ctx: UISearchCtx = expect_context();
            let hotel_info_ctx: HotelInfoCtx = expect_context();

            // Validate required email with debug logging
            let adults_list = block_room_state.adults.get_untracked();
            log!("Payment action - adults list: {:?}", adults_list);
            if let Some(first_adult) = adults_list.first() {
                log!(
                    "Payment action - first adult email: {:?}",
                    first_adult.email
                );
                log!(
                    "Payment action - first adult first_name: '{}'",
                    first_adult.first_name
                );
                log!(
                    "Payment action - first adult phone: {:?}",
                    first_adult.phone
                );
            }

            let Some(email) = adults_list.first().and_then(|adult| adult.email.clone()) else {
                log!("Payment creation failed - no primary adult email provided");
                BlockRoomUIState::batch_update_on_error(
                    Some("payment".to_string()),
                    Some("Email required for payment".to_string()),
                    Some("Primary adult email is required to create payment invoice".to_string()),
                );
                set_payment_loading.set(false);
                set_selected_provider.set(None);
                return None;
            };

            // Generate booking ID and order ID
            let Some((email_from_local_storage, app_ref_from_local_storage)) =
                BookingId::read_from_storage()
            else {
                log!("Payment creation failed - could not generate app reference");
                BlockRoomUIState::batch_update_on_error(
                    Some("payment".to_string()),
                    Some("App Reference generation failed".to_string()),
                    Some("Unable to generate app reference for payment".to_string()),
                );
                set_payment_loading.set(false);
                set_selected_provider.set(None);
                return None;
            };

            let booking_id = BookingId::new(email_from_local_storage, app_ref_from_local_storage);
            // Create order ID using the proper booking_id method
            let order_id = booking_id.to_order_id();

            // Get price information
            let price_amount = calculated_total();

            // Create domain request using proper URL helper functions
            let hotel_name = hotel_info_ctx.selected_hotel_name.get_untracked();
            log!("Payment action - hotel_name: '{}'", hotel_name);

            let consts_provider: crate::api::consts::PaymentProvider = provider.clone().into();
            let domain_request = create_domain_request(
                price_amount,
                "USD".to_string(),
                order_id,
                if hotel_name.is_empty() {
                    "Hotel Room Booking".to_string()
                } else {
                    hotel_name
                },
                email.clone(),
                get_ipn_callback_url(consts_provider.clone()),
                get_payments_url_v2("success", consts_provider.clone()),
                get_payments_url_v2("cancel", consts_provider.clone()),
                get_payments_url_v2("partial", consts_provider),
                false,
                false,
                provider,
            );

            // Call payment API via client-side API
            let client = ClientSideApiClient::new();
            match client.create_payment_invoice(domain_request).await {
                Some(response) => {
                    log!(
                        "Payment invoice created successfully: {}",
                        response.payment_url
                    );
                    // Redirect to payment URL
                    let window = web_sys::window().expect("no global `window` exists");
                    let location = window.location();
                    let _ = location.set_href(&response.payment_url);
                    Some(response)
                }
                None => {
                    log!("Payment invoice creation failed");
                    BlockRoomUIState::batch_update_on_error(
                        Some("payment".to_string()),
                        Some("Payment creation failed".to_string()),
                        Some("Failed to create payment invoice".to_string()),
                    );
                    None
                }
            }
        }
    });

    // Handle action completion
    create_effect(move |_| {
        if create_payment_action.value().get().is_some() {
            set_payment_loading.set(false);
            set_selected_provider.set(None);
        }
    });

    view! {
        <div class="space-y-3">
            // <!-- NowPayments Button -->
            <button
                class=move || format!(
                    "payment-button border-2 rounded-lg p-3 flex items-center cursor-pointer relative transition-all duration-200 w-full {}",
                    if selected_provider().map_or(false, |p| p == PaymentProvider::NowPayments) {
                        "border-blue-500 bg-blue-50"
                    } else {
                        "border-gray-300 hover:border-blue-400 hover:bg-gray-50"
                    }
                )
                disabled=payment_loading
                on:click=move |_| {
                    if !payment_loading() {
                        create_payment_action.dispatch(PaymentProvider::NowPayments);
                    }
                }
            >
                <div class="flex items-center justify-between w-full">
                    <div class="flex items-center">
                        <div class="w-8 h-8 rounded-full bg-gradient-to-r from-blue-500 to-purple-600 flex items-center justify-center mr-3">
                            <span class="text-white text-sm font-bold">"C"</span>
                        </div>
                        <div class="text-left">
                            <div class="font-semibold text-gray-900">"Crypto"</div>
                            <div class="text-sm text-gray-600">"Pay with crypto currencies"</div>
                        </div>
                    </div>
                    <Show when=move || selected_provider().map_or(false, |p| p == PaymentProvider::NowPayments) && payment_loading()>
                        <div class="animate-spin rounded-full h-5 w-5 border-b-2 border-blue-500"></div>
                    </Show>
                </div>
            </button>

            // <!-- Stripe Button -->
            <button
                class=move || format!(
                    "payment-button border-2 rounded-lg p-3 flex items-center cursor-pointer relative transition-all duration-200 w-full {}",
                    if selected_provider().map_or(false, |p| p == PaymentProvider::Stripe) {
                        "border-indigo-500 bg-indigo-50"
                    } else {
                        "border-gray-300 hover:border-indigo-400 hover:bg-gray-50"
                    }
                )
                disabled=payment_loading
                on:click=move |_| {
                    if !payment_loading() {
                        create_payment_action.dispatch(PaymentProvider::Stripe);
                    }
                }
            >
                <div class="flex items-center justify-between w-full">
                    <div class="flex items-center">
                        <div class="w-8 h-8 rounded-full bg-gradient-to-r from-indigo-500 to-purple-600 flex items-center justify-center mr-3">
                            <span class="text-white text-sm font-bold">"S"</span>
                        </div>
                        <div class="text-left">
                            <div class="font-semibold text-gray-900">"Stripe"</div>
                            <div class="text-sm text-gray-600">"Pay with credit/debit cards"</div>
                        </div>
                    </div>
                    <Show when=move || selected_provider().map_or(false, |p| p == PaymentProvider::Stripe) && payment_loading()>
                        <div class="animate-spin rounded-full h-5 w-5 border-b-2 border-indigo-500"></div>
                    </Show>
                </div>
            </button>

            // <!-- Loading overlay for general payment processing -->
            <Show when=payment_loading>
                <div class="text-center py-2">
                    <div class="text-sm text-gray-600">
                        {move || format!("Creating {} payment...",
                            selected_provider().map_or("payment".to_string(), |p| p.as_str().to_string())
                        )}
                    </div>
                </div>
            </Show>
        </div>
    }
}
