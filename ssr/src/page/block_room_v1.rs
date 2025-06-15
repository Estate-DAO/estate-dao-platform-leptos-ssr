use leptos::*;
use leptos_icons::Icon;
use leptos_router::use_navigate;

use crate::api::client_side_api::ClientSideApiClient;
use crate::app::AppRoutes;
use crate::component::{Divider, Navbar, SpinnerGray};
use crate::domain::{
    DomainAdultDetail, DomainBlockRoomRequest, DomainChildDetail, DomainDestination,
    DomainHotelDetails, DomainHotelInfoCriteria, DomainHotelSearchCriteria, DomainRoomData,
    DomainRoomGuest, DomainSelectedDateRange, DomainUserDetails,
};
use crate::log;
use crate::utils::{
    app_reference::{generate_app_reference, BookingId},
    BackendIntegrationHelper,
};
use crate::view_state_layer::hotel_details_state::PricingBookNowState;
use crate::view_state_layer::ui_block_room::{
    AdultDetail, BlockRoomUIState, ChildDetail, RoomSelectionSummary,
};
use crate::view_state_layer::ui_search_state::UISearchCtx;
use crate::view_state_layer::view_state::HotelInfoCtx;

#[component]
pub fn BlockRoomV1Page() -> impl IntoView {
    let block_room_state: BlockRoomUIState = expect_context();
    let ui_search_ctx: UISearchCtx = expect_context();
    let hotel_info_ctx: HotelInfoCtx = expect_context();
    let navigate = use_navigate();

    // Initialize form data on mount
    create_effect(move |_| {
        let adults_count = ui_search_ctx.guests.get().adults.get() as usize;
        let children_count = ui_search_ctx.guests.get().children.get() as usize;
        let children_ages = ui_search_ctx.guests.get().children_ages.clone();

        // Initialize adults and children
        BlockRoomUIState::create_adults(adults_count);
        BlockRoomUIState::create_children(children_count);

        // Set pricing data from PricingBookNowState
        let room_price = PricingBookNowState::total_room_price_for_all_user_selected_rooms();
        let date_range = ui_search_ctx.date_range.get();
        let num_nights = date_range.no_of_nights();

        BlockRoomUIState::set_room_price(room_price);
        BlockRoomUIState::set_num_nights(num_nights);
        let _total = BlockRoomUIState::calculate_total_price();

        log!(
            "BlockRoomV1Page initialized - adults: {}, children: {}, room_price: {}, nights: {}",
            adults_count,
            children_count,
            room_price,
            num_nights
        );
    });

    // Navigation handler
    let go_back_to_details = move |_: ev::MouseEvent| {
        let _ = navigate(AppRoutes::HotelDetails.to_string(), Default::default());
    };

    // Get reactive signals
    let room_price = move || block_room_state.room_price.get();
    let total_price = move || block_room_state.total_price.get();
    let num_nights = move || block_room_state.num_nights.get();
    let room_summary = move || block_room_state.room_selection_summary.get();

    // <!-- Calculate totals from room selections -->
    let calculated_total = move || {
        let summary = room_summary();
        let nights = num_nights();
        summary
            .iter()
            .map(|room| room.price_per_night * room.quantity as f64 * nights as f64)
            .sum::<f64>()
    };

    let rooms_total_per_night = move || {
        room_summary()
            .iter()
            .map(|room| room.price_per_night * room.quantity as f64)
            .sum::<f64>()
    };
    let num_rooms = move || ui_search_ctx.guests.get().rooms.get();
    let adult_count = move || ui_search_ctx.guests.get().adults.get();
    let child_count = move || ui_search_ctx.guests.get().children.get();

    // Hotel info signals
    let hotel_name = move || hotel_info_ctx.selected_hotel_name.get();
    let hotel_address = move || hotel_info_ctx.selected_hotel_location.get();
    let hotel_image = move || {
        let img = hotel_info_ctx.selected_hotel_image.get();
        if img.is_empty() {
            "/img/home.webp".to_string()
        } else {
            img
        }
    };

    // Date formatting
    let checkin_date = move || ui_search_ctx.date_range.get().dd_month_yyyy_start();
    let checkout_date = move || ui_search_ctx.date_range.get().dd_month_yyyy_end();
    let formatted_nights = move || ui_search_ctx.date_range.get().formatted_nights();

    view! {
        <section class="relative min-h-screen bg-gray-50">
            <Navbar />

            <div class="max-w-5xl mx-auto px-2 sm:px-6">
                <div class="flex items-center py-4 md:py-8">
                    <span class="inline-flex items-center cursor-pointer" on:click=go_back_to_details>
                        <Icon icon=icondata::AiArrowLeftOutlined class="text-black font-light" />
                    </span>
                    <h1 class="ml-2 sm:ml-4 text-2xl sm:text-3xl font-bold">"You're just one step away!"</h1>
                </div>
            </div>

            <div class="relative flex flex-col lg:flex-row min-h-[calc(100vh-5rem)] items-start justify-center p-2 sm:p-6 max-w-5xl mx-auto gap-4 md:gap-6">
                // Left side - Form content
                <div class="w-full lg:w-3/5 flex flex-col gap-8 order-1">
                    // Hotel information card
                    <div class="p-2 sm:p-6 bg-white rounded-2xl shadow w-full">
                        <div class="flex items-center gap-3 mb-2">
                            <img
                                src=hotel_image
                                alt=hotel_name
                                class="h-10 w-10 sm:h-12 sm:w-12 rounded-lg object-cover"
                            />
                            <div class="flex flex-col justify-center min-h-[2.5rem]">
                                <div class="font-bold text-base sm:text-lg min-h-[1.25rem]">
                                    {hotel_name}
                                </div>
                                <div class="text-gray-500 text-sm min-h-[1rem]">
                                    {hotel_address}
                                </div>
                            </div>
                        </div>

                        <hr class="my-3 border-gray-200" />

                        // Date and guest information
                        <div class="flex items-center justify-between mb-3">
                            <div class="flex flex-col items-start">
                                <span class="text-xs text-gray-400">Check-in</span>
                                <span class="font-semibold text-base">{checkin_date}</span>
                            </div>
                            <div class="flex flex-col items-center">
                                <span class="bg-gray-100 rounded-full px-3 py-1 text-xs font-semibold text-gray-700 mb-1">
                                    {formatted_nights}
                                </span>
                            </div>
                            <div class="flex flex-col items-end">
                                <span class="text-xs text-gray-400">Check-out</span>
                                <span class="font-semibold text-base">{checkout_date}</span>
                            </div>
                        </div>

                        <hr class="my-3 border-gray-200" />

                        <div class="flex items-center gap-2 mt-2">
                            <Icon icon=icondata::AiUserOutlined class="text-gray-400 text-lg" />
                            <span class="text-xs text-gray-400 font-semibold">Guests & Rooms</span>
                            <span class="font-bold text-sm ml-2 text-right min-w-0 break-words">
                                {move || format!(
                                    "{} Room{}{} {} Adult{}{} {} child{}",
                                    num_rooms(),
                                    if num_rooms() == 1 { "" } else { "s" },
                                    if num_rooms() > 0 { "," } else { "" },
                                    adult_count(),
                                    if adult_count() == 1 { "" } else { "s" },
                                    if child_count() > 0 { "," } else { "" },
                                    child_count(),
                                    if child_count() == 1 { "" } else { "ren" }
                                )}
                            </span>
                        </div>
                    </div>

                    // <!-- Selected rooms summary -->
                    <SelectedRoomsSummary />

                    // Mobile pricing summary
                    <EnhancedPricingDisplay mobile=true />

                    // Guest form will go here
                    <GuestForm />

                    // Terms and conditions
                    <TermsCheckbox />

                    // Mobile confirm button
                    <ConfirmButton mobile=true />
                </div>

                // Right side - Desktop pricing summary
                <div class="hidden lg:flex w-full lg:w-2/5 mb-8 lg:mb-0 rounded-2xl bg-white p-4 sm:p-8 shadow-xl flex-col items-stretch order-2 lg:sticky lg:top-28">
                    <EnhancedPricingDisplay mobile=false />

                    // Desktop confirm button
                    <ConfirmButton mobile=false />
                </div>
            </div>

            // Payment Modal
            <PaymentModal />
        </section>
    }
}

// <!-- Phase 2.2: Enhanced Pricing Display Component -->
#[component]
pub fn EnhancedPricingDisplay(mobile: bool) -> impl IntoView {
    let block_room_state: BlockRoomUIState = expect_context();

    let room_summary = move || block_room_state.room_selection_summary.get();
    let num_nights = move || block_room_state.num_nights.get();

    // Calculate totals from room selections
    let rooms_total_per_night = move || {
        room_summary()
            .iter()
            .map(|room| room.price_per_night * room.quantity as f64)
            .sum::<f64>()
    };

    let calculated_total = move || {
        let summary = room_summary();
        let nights = num_nights();
        summary
            .iter()
            .map(|room| room.price_per_night * room.quantity as f64 * nights as f64)
            .sum::<f64>()
    };

    let container_class = if mobile {
        "lg:hidden mb-6 rounded-2xl bg-white p-4 sm:p-8 shadow-xl flex flex-col items-stretch"
    } else {
        "mb-4"
    };

    view! {
        <div class=container_class>
            <h2 class="mb-4 text-2xl font-bold flex items-end">
                <span class="text-3xl font-bold">{move || format!("${:.2}", rooms_total_per_night())}</span>
                <span class="ml-1 text-base font-normal text-gray-600">/night</span>
            </h2>

            <Divider class="my-4".into() />

            // <!-- Per-room breakdown -->
            <div class="price-breakdown space-y-3 mt-4">
                <Show when=move || !room_summary().is_empty()>
                    {move || room_summary().into_iter().map(|room| {
                        view! {
                            <div class="flex justify-between items-center text-sm">
                                <span class="text-gray-700 flex-1 min-w-0">
                                    <span class="truncate break-words whitespace-normal">{room.room_name.clone()}</span>
                                    <span class="text-xs text-gray-500 ml-1">
                                        "× " {room.quantity} " × " {num_nights()} " nights"
                                    </span>
                                </span>
                                <span class="font-semibold ml-2">
                                    ${format!("{:.2}", room.price_per_night * room.quantity as f64 * num_nights() as f64)}
                                </span>
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
                            {move || format!("${:.2}", calculated_total())}
                        </span>
                    </div>
                </Show>

                // <!-- Taxes and fees -->
                <div class="flex justify-between items-center text-base">
                    <span class="text-gray-700">Taxes and fees</span>
                    <span class="font-semibold">$0.00</span>
                </div>
            </div>

            <Divider class="my-4".into() />

            // <!-- Total -->
            <div class="flex justify-between items-center font-bold text-lg mb-2">
                <span>Total</span>
                <span class="text-2xl">{move || format!("${:.2}", calculated_total())}</span>
            </div>
        </div>
    }
}

// <!-- Phase 2.1: Room Selection Summary Component -->
#[component]
pub fn SelectedRoomsSummary() -> impl IntoView {
    let block_room_state: BlockRoomUIState = expect_context();

    let room_summary = move || block_room_state.room_selection_summary.get();
    let hotel_context = move || block_room_state.hotel_context.get();

    view! {
        <div class="bg-white rounded-2xl shadow p-4 sm:p-6 mb-6">
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
    view! {
        <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between border border-gray-200 rounded-lg p-3 sm:p-4 bg-gray-50">
            // <!-- Room details -->
            <div class="flex-1 min-w-0 mb-2 sm:mb-0">
                <div class="font-semibold text-base min-w-0 break-words whitespace-normal truncate">
                    {room.room_name.clone()}
                </div>
                <div class="text-sm text-gray-600 flex items-center gap-2 mt-1">
                    <span class="bg-blue-100 text-blue-800 px-2 py-1 rounded-full text-xs font-medium">
                        {format!("{} room{}", room.quantity, if room.quantity == 1 { "" } else { "s" })}
                    </span>
                    <span class="text-gray-500">"•"</span>
                    <span>${format!("{:.2}", room.price_per_night)} /night</span>
                </div>
            </div>

            // <!-- Price display -->
            <div class="flex flex-col items-start sm:items-end sm:text-right">
                <div class="text-lg font-bold">
                    ${format!("{:.2}", room.price_per_night * room.quantity as f64)}
                    <span class="text-sm font-normal text-gray-600 ml-1">/night</span>
                </div>
                <div class="text-xs text-gray-500">
                    {format!("${:.2} × {}", room.price_per_night, room.quantity)}
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn GuestForm() -> impl IntoView {
    let block_room_state: BlockRoomUIState = expect_context();
    let ui_search_ctx: UISearchCtx = expect_context();

    let adult_count = move || ui_search_ctx.guests.get().adults.get();
    let child_count = move || ui_search_ctx.guests.get().children.get();
    let children_ages = ui_search_ctx.guests.get().children_ages.clone();

    view! {
        <div class="guest-form mt-4 space-y-6">
            // Adults
            {(0..adult_count())
                .map(|i| {
                    view! {
                        <AdultFormSection index=i />
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
pub fn AdultFormSection(index: u32) -> impl IntoView {
    let update_adult = move |field: &str, value: String| {
        BlockRoomUIState::update_adult(index as usize, field, value);
        BlockRoomUIState::validate_form();
    };

    view! {
        <div class="person-details mb-2">
            <h3 class="font-semibold text-gray-700 text-sm sm:text-base mb-2">
                {if index == 0 {
                    String::from("Primary Adult")
                } else {
                    format!("Adult {}", index + 1)
                }}
            </h3>
            <div class="flex flex-col sm:flex-row gap-2 sm:gap-4">
                // <!-- Phase 4.3: Enhanced form validation with real-time feedback -->
                <input
                    type="text"
                    placeholder="First Name *"
                    class="w-full sm:w-1/2 rounded-md border border-gray-300 p-3 min-h-[44px] focus:border-blue-500 focus:ring-2 focus:ring-blue-200 transition-colors"
                    required=true
                    on:input=move |ev| {
                        let value = event_target_value(&ev);
                        update_adult("first_name", value.clone());

                        // Real-time validation feedback
                        if value.trim().is_empty() {
                            // Could set individual field validation error here
                        }
                    }
                    on:blur=move |_| {
                        // Validate on blur for better UX
                        BlockRoomUIState::validate_form();
                    }
                />
                <input
                    type="text"
                    placeholder="Last Name"
                    class="w-full sm:w-1/2 rounded-md border border-gray-300 p-3 min-h-[44px]"
                    on:input=move |ev| {
                        update_adult("last_name", event_target_value(&ev));
                    }
                />
            </div>

            // Primary adult gets email and phone fields
            {move || {
                if index == 0 {
                    view! {
                        <div class="flex flex-col sm:flex-row gap-2 sm:gap-4 mt-2">
                            // <!-- Phase 4.3: Enhanced email validation -->
                            <input
                                type="email"
                                placeholder="Email *"
                                class="w-full sm:w-1/2 rounded-md border border-gray-300 p-3 min-h-[44px] focus:border-blue-500 focus:ring-2 focus:ring-blue-200 transition-colors"
                                required=true
                                on:input=move |ev| {
                                    let value = event_target_value(&ev);
                                    update_adult("email", value.clone());

                                    // Real-time email validation feedback
                                    if !value.trim().is_empty() && !BlockRoomUIState::is_valid_email(&value) {
                                        // Email format invalid - could show validation message
                                    }
                                }
                                on:blur=move |_| {
                                    BlockRoomUIState::validate_form();
                                }
                            />
                            // <!-- Phase 4.3: Enhanced phone validation -->
                            <input
                                type="tel"
                                placeholder="Phone *"
                                class="w-full sm:w-1/2 rounded-md border border-gray-300 p-3 min-h-[44px] focus:border-blue-500 focus:ring-2 focus:ring-blue-200 transition-colors"
                                required=true
                                on:input=move |ev| {
                                    let value = event_target_value(&ev);
                                    update_adult("phone", value.clone());

                                    // Real-time phone validation feedback
                                    if !value.trim().is_empty() && !BlockRoomUIState::is_valid_phone(&value) {
                                        // Phone format invalid - could show validation message
                                    }
                                }
                                on:blur=move |_| {
                                    BlockRoomUIState::validate_form();
                                }
                            />
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

    let age_value = ui_search_ctx.guests.get().children_ages.get_value_at(index);

    view! {
        <div class="person-details mb-2">
            <h3 class="font-semibold text-gray-700 text-sm sm:text-base mb-2">
                {format!("Child {}", index + 1)}
            </h3>
            <div class="flex flex-col sm:flex-row gap-2 sm:gap-4">
                <input
                    type="text"
                    placeholder="First Name *"
                    class="w-full sm:w-2/5 rounded-md border border-gray-300 p-3"
                    required=true
                    on:input=move |ev| {
                        update_child("first_name", event_target_value(&ev));
                    }
                />
                <input
                    type="text"
                    placeholder="Last Name"
                    class="w-full sm:w-2/5 rounded-md border border-gray-300 p-3"
                    on:input=move |ev| {
                        update_child("last_name", event_target_value(&ev));
                    }
                />
                <select
                    class="w-full sm:w-1/5 rounded-md border border-gray-300 bg-white p-3"
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
    }
}

#[component]
pub fn TermsCheckbox() -> impl IntoView {
    view! {
        <div class="mt-2 flex items-start">
            <input
                type="checkbox"
                id="agree"
                class="mr-2 mt-1"
                on:change=move |ev| {
                    BlockRoomUIState::set_terms_accepted(event_target_checked(&ev));
                    BlockRoomUIState::validate_form();
                }
            />
            <label for="agree" class="text-xs sm:text-sm text-gray-600">
                "Property once booked cannot be cancelled. Confirm the details before making payment."
            </label>
        </div>
    }
}

#[component]
pub fn ConfirmButton(mobile: bool) -> impl IntoView {
    let block_room_state: BlockRoomUIState = expect_context();
    let is_form_valid = move || block_room_state.form_valid.get();

    let open_modal = move |_| {
        if is_form_valid() {
            BlockRoomUIState::set_show_payment_modal(true);
        }
    };

    // <!-- Phase 4.3: Enhanced button styling with validation feedback -->
    let button_class = if mobile {
        "mt-6 w-full rounded-full py-3 text-white text-base sm:text-lg font-bold shadow-lg block lg:hidden min-h-[44px] transition-all duration-200"
    } else {
        "mt-6 w-full rounded-full py-3 text-white text-base sm:text-lg font-bold shadow-lg hidden lg:block min-h-[44px] transition-all duration-200"
    };

    let button_style = move || {
        if is_form_valid() {
            "bg-blue-600 hover:bg-blue-700 hover:shadow-xl transform hover:scale-105"
        } else {
            "bg-gray-300 cursor-not-allowed"
        }
    };

    view! {
        <button
            class=move || format!("{} {}", button_class, button_style())
            disabled=move || !is_form_valid()
            on:click=open_modal
        >
            // <!-- Phase 4.3: Dynamic button text with validation feedback -->
            {move || {
                if is_form_valid() {
                    "Confirm & Book"
                } else {
                    "Complete Required Fields"
                }
            }}
        </button>
    }
}

#[component]
pub fn PaymentModal() -> impl IntoView {
    let block_room_state: BlockRoomUIState = expect_context();
    let ui_search_ctx: UISearchCtx = expect_context();
    let hotel_info_ctx: HotelInfoCtx = expect_context();

    let show_modal = move || block_room_state.show_payment_modal.get();
    let is_loading = move || block_room_state.loading.get();
    let block_room_called = move || block_room_state.block_room_called.get();

    let room_price = move || block_room_state.room_price.get();
    let total_price = move || block_room_state.total_price.get();
    let num_nights = move || block_room_state.num_nights.get();

    // Block room action
    let block_room_action = create_action(move |_: &()| {
        let client = ClientSideApiClient::new();

        async move {
            BlockRoomUIState::set_loading(true);
            BlockRoomUIState::set_error(None);

            // Build block room request
            let request = build_block_room_request().await;
            match request {
                Some(req) => {
                    log!("Calling block_room API with request: {:?}", req);
                    match client.block_room(req).await {
                        Some(response) => {
                            log!("Block room successful: {:?}", response);
                            BlockRoomUIState::set_block_room_id(Some(response.block_id.clone()));
                            BlockRoomUIState::set_block_room_called(true);

                            // Save to backend after successful block room
                            if let Err(e) = save_booking_to_backend(&response.block_id).await {
                                log!("Failed to save booking to backend: {}", e);
                                BlockRoomUIState::set_error(Some(format!(
                                    "Room blocked but booking save failed: {}",
                                    e
                                )));
                            }
                        }
                        None => {
                            log!("Block room failed");
                            // <!-- Phase 3.2: Enhanced error handling with specific error types -->
                            BlockRoomUIState::increment_retry_count();
                            BlockRoomUIState::set_api_error(
                                Some("server".to_string()),
                                Some("Unable to reserve your room. Please try again.".to_string()),
                                Some("Block room API returned None response".to_string()),
                            );
                        }
                    }
                }
                None => {
                    // <!-- Phase 3.2: Enhanced error handling for request building failure -->
                    BlockRoomUIState::set_api_error(
                        Some("validation".to_string()),
                        Some("Invalid booking information. Please check your details.".to_string()),
                        Some(
                            "Failed to build block room request - missing required data"
                                .to_string(),
                        ),
                    );
                }
            }

            BlockRoomUIState::set_loading(false);
        }
    });

    // <!-- Phase 3.3: Availability checking action before block room -->
    // let availability_check_action = create_action(move |_: &()| async move {
    //     BlockRoomUIState::set_availability_checking(true);

    //     // Simulate availability check (in real implementation, call availability API)
    //     // Simple delay simulation without external dependencies

    //     // Simulate availability result - in real implementation, parse API response
    //     let available = true; // For now, always available (replace with actual API call)

    //     if available {
    //         BlockRoomUIState::set_room_availability_status(Some("available".to_string()));
    //     } else {
    //         BlockRoomUIState::set_room_availability_status(Some("unavailable".to_string()));
    //         BlockRoomUIState::set_api_error(
    //             Some("room_unavailable".to_string()),
    //             Some("Sorry, this room is no longer available. Please select a different room.".to_string()),
    //             Some("Room availability check returned unavailable".to_string())
    //         );
    //     }

    //     BlockRoomUIState::set_availability_checking(false);
    // });

    // Trigger availability check and then block room when modal opens
    // create_effect(move |_| {
    //     if show_modal() && !block_room_called() && !is_loading() {
    //         // First check availability, then proceed to block room
    //         let availability_status = BlockRoomUIState::get_room_availability_status();

    //         if availability_status.is_none() && !BlockRoomUIState::is_availability_checking() {
    //             // Start availability check
    //             availability_check_action.dispatch(());
    //         } else if availability_status == Some("available".to_string()) {
    //             // Room is available, proceed with block room
    //             block_room_action.dispatch(());
    //         }
    //     }
    // });

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
                    // <Show
                    //     when=move || {!is_loading()
                    //         // && !BlockRoomUIState::is_availability_checking()
                    //     }
                    //     fallback=move || view! {
                    //         <EnhancedLoadingState />
                    //     }
                    // >
                        // <!-- Phase 4.2: Enhanced error display -->
                        <Show when=move || block_room_state.error.get().is_some()>
                            <EnhancedErrorDisplay />
                        </Show>

                        <Show when=move || block_room_called() && block_room_state.error.get().is_none()>
                            <h2 class="text-xl font-bold text-center mb-6">Payment</h2>
                            <div class="flex flex-col gap-2 mb-6">
                                <div class="flex justify-between items-end">
                                    <span class="text-lg font-bold">{move || format!("${:.2}", room_price())}</span>
                                    <span class="ml-1 text-base font-normal text-gray-600">/night</span>
                                </div>
                                <div class="flex justify-between items-center text-base">
                                    <span class="text-gray-700">
                                        {move || format!("${:.2} x {} nights", room_price(), num_nights())}
                                    </span>
                                    <span class="font-semibold">
                                        {move || format!("${:.2}", room_price() * num_nights() as f64)}
                                    </span>
                                </div>
                                <Divider class="my-2".into() />
                                <div class="flex justify-between items-center font-bold text-lg mb-2">
                                    <span>Total</span>
                                    <span class="text-2xl">{move || format!("${:.2}", total_price())}</span>
                                </div>
                            </div>

                            <div class="font-bold">
                                <label>"Pay with"</label>
                                <div class="flex flex-col w-full mt-4 space-y-2">
                                    <button class="payment-button border-2 rounded-lg p-3 flex items-center cursor-pointer relative border-gray-500">
                                        <span class="px-2 py-2">"We'll enable payments soon."</span>
                                    </button>
                                </div>
                            </div>
                        </Show>
                    // </Show>
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

    let error_message = move || block_room_state.error.get().unwrap_or_default();
    let error_type = move || block_room_state.api_error_type.get();
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
            // Reset error state and try again
            BlockRoomUIState::set_error(None);
            BlockRoomUIState::set_api_error(None, None, None);
            // BlockRoomUIState::set_room_availability_status(None);
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

// Helper function to build block room request
async fn build_block_room_request() -> Option<DomainBlockRoomRequest> {
    let block_room_state: BlockRoomUIState = expect_context();
    let ui_search_ctx: UISearchCtx = expect_context();
    let hotel_info_ctx: HotelInfoCtx = expect_context();

    // Get form data
    let adults = block_room_state.adults.get_untracked();
    let children = block_room_state.children.get_untracked();

    // Convert to domain types
    let domain_adults: Vec<DomainAdultDetail> = adults
        .into_iter()
        .map(|adult| DomainAdultDetail {
            email: adult.email,
            first_name: adult.first_name,
            last_name: adult.last_name,
            phone: adult.phone,
        })
        .collect();

    let domain_children: Vec<DomainChildDetail> = children
        .into_iter()
        .filter_map(|child| {
            child.age.map(|age| DomainChildDetail {
                age,
                first_name: child.first_name,
                last_name: child.last_name,
            })
        })
        .collect();

    let user_details = DomainUserDetails {
        adults: domain_adults,
        children: domain_children,
    };

    // Build hotel info criteria
    let destination = ui_search_ctx.destination.get_untracked()?;
    let date_range = ui_search_ctx.date_range.get_untracked();
    let guests = ui_search_ctx.guests.get_untracked();

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
        guest_nationality: "US".to_string(),
    };

    let hotel_code = hotel_info_ctx.hotel_code.get_untracked();
    let hotel_info_criteria = DomainHotelInfoCriteria {
        token: hotel_code.clone(),
        hotel_ids: vec![hotel_code],
        search_criteria,
    };

    // <!-- Phase 3.1: Use real room selection data from BlockRoomUIState -->
    let selected_rooms_data = block_room_state.selected_rooms.get_untracked();

    // Use the first selected room for the API call
    // TODO: Enhance API to support multiple room types in future
    let selected_room = if let Some((_, (_, room_data))) = selected_rooms_data.iter().next() {
        room_data.clone()
    } else {
        // Fallback to basic room data if no selection available
        DomainRoomData {
            room_name: "Default Room".to_string(),
            room_unique_id: "default_room".to_string(),
            rate_key: "default_rate".to_string(),
            offer_id: "".to_string(),
        }
    };

    let total_guests = guests.adults.get_untracked() + guests.children.get_untracked();

    Some(DomainBlockRoomRequest {
        hotel_info_criteria,
        user_details,
        selected_room,
        total_guests,
        special_requests: None,
    })
}

// Helper function to save booking to backend after successful block room
async fn save_booking_to_backend(block_room_id: &str) -> Result<(), String> {
    use crate::api::canister::add_booking::add_booking_backend;

    let block_room_state: BlockRoomUIState = expect_context();
    let ui_search_ctx: UISearchCtx = expect_context();
    let hotel_info_ctx: HotelInfoCtx = expect_context();

    // Generate app reference and booking ID
    let email = block_room_state
        .adults
        .get_untracked()
        .first()
        .and_then(|adult| adult.email.clone())
        .ok_or("Primary adult email is required")?;

    let app_reference_signal = generate_app_reference(email.clone());
    let booking_id = app_reference_signal
        .get_untracked()
        .ok_or("Failed to generate app reference")?;

    // Get form data and convert to domain types
    let adults = block_room_state.adults.get_untracked();
    let children = block_room_state.children.get_untracked();

    let domain_adults: Vec<DomainAdultDetail> = adults
        .into_iter()
        .map(|adult| DomainAdultDetail {
            email: adult.email,
            first_name: adult.first_name,
            last_name: adult.last_name,
            phone: adult.phone,
        })
        .collect();

    let domain_children: Vec<DomainChildDetail> = children
        .into_iter()
        .filter_map(|child| {
            child.age.map(|age| DomainChildDetail {
                age,
                first_name: child.first_name,
                last_name: child.last_name,
            })
        })
        .collect();

    let user_details = DomainUserDetails {
        adults: domain_adults,
        children: domain_children,
    };

    // Build destination and date range
    let destination = ui_search_ctx
        .destination
        .get_untracked()
        .map(|dest| DomainDestination {
            city_id: dest.city_id.parse().unwrap_or(0),
            city_name: dest.city,
            country_code: dest.country_code,
            country_name: dest.country_name,
        });

    let date_range = {
        let dr = ui_search_ctx.date_range.get_untracked();
        DomainSelectedDateRange {
            start: dr.start,
            end: dr.end,
        }
    };

    // Build hotel details
    let hotel_details = DomainHotelDetails {
        checkin: date_range.start.0.to_string()
            + "-"
            + &date_range.start.1.to_string()
            + "-"
            + &date_range.start.2.to_string(),
        checkout: date_range.end.0.to_string()
            + "-"
            + &date_range.end.1.to_string()
            + "-"
            + &date_range.end.2.to_string(),
        hotel_name: hotel_info_ctx.selected_hotel_name.get_untracked(),
        hotel_code: hotel_info_ctx.hotel_code.get_untracked(),
        star_rating: 4,                               // Default, should come from API
        description: "Hotel description".to_string(), // Should come from API
        hotel_facilities: vec![],                     // Should come from API
        address: hotel_info_ctx.selected_hotel_location.get_untracked(),
        images: vec![hotel_info_ctx.selected_hotel_image.get_untracked()],
        first_room_details: crate::domain::DomainFirstRoomDetails {
            price: crate::domain::DomainDetailedPrice {
                published_price: block_room_state.total_price.get_untracked(),
                published_price_rounded_off: block_room_state.total_price.get_untracked(),
                offered_price: block_room_state.total_price.get_untracked(),
                offered_price_rounded_off: block_room_state.total_price.get_untracked(),
                room_price: block_room_state.total_price.get_untracked(),
                tax: 0.0,
                extra_guest_charge: 0.0,
                child_charge: 0.0,
                other_charges: 0.0,
                currency_code: "USD".to_string(),
            },
            room_data: crate::domain::DomainRoomData {
                room_name: "Selected Room".to_string(),
                room_unique_id: block_room_id.to_string(),
                rate_key: "".to_string(),
                offer_id: "".to_string(),
            },
        },
        amenities: vec![], // Should come from API
    };

    // Get room details
    let selected_rooms_data = block_room_state.selected_rooms.get_untracked();
    let room_details: Vec<DomainRoomData> = selected_rooms_data
        .into_iter()
        .map(|(_, (_, room_data))| room_data)
        .collect();

    let payment_amount = block_room_state.total_price.get_untracked();
    let payment_currency = "USD".to_string();

    // Create backend booking using the integration helper
    let backend_booking = BackendIntegrationHelper::create_backend_booking(
        destination,
        date_range,
        room_details,
        hotel_details,
        user_details,
        booking_id.clone().into(),
        payment_amount,
        payment_currency,
    );

    // Update the backend booking with block room ID
    let mut backend_booking = backend_booking;
    backend_booking
        .user_selected_hotel_room_details
        .hotel_details
        .block_room_id = block_room_id.to_string();
    backend_booking
        .user_selected_hotel_room_details
        .hotel_details
        .hotel_token = hotel_info_ctx.hotel_code.get_untracked();

    // Serialize and save to backend
    let booking_json = serde_json::to_string(&backend_booking)
        .map_err(|e| format!("Failed to serialize booking: {}", e))?;

    add_booking_backend(email, booking_json)
        .await
        .map_err(|e| format!("Failed to save booking to backend: {}", e))?;

    log!(
        "Successfully saved booking to backend with ID: {}",
        booking_id.app_reference
    );
    Ok(())
}
