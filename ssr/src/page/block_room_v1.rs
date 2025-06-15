use leptos::*;
use leptos_icons::Icon;
use leptos_router::use_navigate;

use crate::api::client_side_api::ClientSideApiClient;
use crate::app::AppRoutes;
use crate::component::{Divider, Navbar, SpinnerGray};
use crate::domain::{
    DomainAdultDetail, DomainBlockRoomRequest, DomainChildDetail, DomainHotelInfoCriteria,
    DomainHotelSearchCriteria, DomainRoomData, DomainRoomGuest, DomainUserDetails,
};
use crate::log;
use crate::view_state_layer::hotel_details_state::PricingBookNowState;
use crate::view_state_layer::ui_block_room::{AdultDetail, BlockRoomUIState, ChildDetail};
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
                <div class="flex items-center py-8">
                    <span class="inline-flex items-center cursor-pointer" on:click=go_back_to_details>
                        <Icon icon=icondata::AiArrowLeftOutlined class="text-black font-light" />
                    </span>
                    <h1 class="ml-2 sm:ml-4 text-2xl sm:text-3xl font-bold">"You're just one step away!"</h1>
                </div>
            </div>

            <div class="relative flex flex-col lg:flex-row min-h-[calc(100vh-5rem)] items-start justify-center p-2 sm:p-6 max-w-5xl mx-auto gap-6">
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
                            <span class="font-bold text-sm ml-2 text-right">
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

                    // Mobile pricing summary
                    <div class="lg:hidden mb-6 rounded-2xl bg-white p-4 sm:p-8 shadow-xl flex flex-col items-stretch">
                        <h2 class="mb-4 text-2xl font-bold flex items-end">
                            <span class="text-3xl font-bold">{move || format!("${:.2}", room_price())}</span>
                            <span class="ml-1 text-base font-normal text-gray-600">/night</span>
                        </h2>
                        <Divider class="my-4".into() />
                        <div class="price-breakdown space-y-4 mt-4">
                            <div class="flex justify-between items-center text-base">
                                <span class="text-gray-700">
                                    {move || format!("${:.2} x {} nights", room_price(), num_nights())}
                                </span>
                                <span class="font-semibold">
                                    {move || format!("${:.2}", room_price() * num_nights() as f64)}
                                </span>
                            </div>
                            <div class="flex justify-between items-center text-base">
                                <span class="text-gray-700">Taxes and fees</span>
                                <span class="font-semibold">$0.00</span>
                            </div>
                        </div>
                        <Divider class="my-4".into() />
                        <div class="flex justify-between items-center font-bold text-lg mb-2">
                            <span>Total</span>
                            <span class="text-2xl">{move || format!("${:.2}", total_price())}</span>
                        </div>
                    </div>

                    // Guest form will go here
                    <GuestForm />

                    // Terms and conditions
                    <TermsCheckbox />

                    // Mobile confirm button
                    <ConfirmButton mobile=true />
                </div>

                // Right side - Desktop pricing summary
                <div class="hidden lg:flex w-full lg:w-2/5 mb-8 lg:mb-0 rounded-2xl bg-white p-4 sm:p-8 shadow-xl flex-col items-stretch order-2 lg:sticky lg:top-28">
                    <h2 class="mb-4 text-2xl font-bold flex items-end">
                        <span class="text-3xl font-bold">{move || format!("${:.2}", room_price())}</span>
                        <span class="ml-1 text-base font-normal text-gray-600">/night</span>
                    </h2>
                    <Divider class="my-4".into() />
                    <div class="price-breakdown space-y-4 mt-4">
                        <div class="flex justify-between items-center text-base">
                            <span class="text-gray-700">
                                {move || format!("${:.2} x {} nights", room_price(), num_nights())}
                            </span>
                            <span class="font-semibold">
                                {move || format!("${:.2}", room_price() * num_nights() as f64)}
                            </span>
                        </div>
                        <div class="flex justify-between items-center text-base">
                            <span class="text-gray-700">Taxes and fees</span>
                            <span class="font-semibold">$0.00</span>
                        </div>
                    </div>
                    <Divider class="my-4".into() />
                    <div class="flex justify-between items-center font-bold text-lg mb-2">
                        <span>Total</span>
                        <span class="text-2xl">{move || format!("${:.2}", total_price())}</span>
                    </div>

                    // Desktop confirm button
                    <ConfirmButton mobile=false />
                </div>
            </div>

            // Payment Modal
            <PaymentModal />
        </section>
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
                <input
                    type="text"
                    placeholder="First Name *"
                    class="w-full sm:w-1/2 rounded-md border border-gray-300 p-3"
                    required=true
                    on:input=move |ev| {
                        update_adult("first_name", event_target_value(&ev));
                    }
                />
                <input
                    type="text"
                    placeholder="Last Name"
                    class="w-full sm:w-1/2 rounded-md border border-gray-300 p-3"
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
                            <input
                                type="email"
                                placeholder="Email *"
                                class="w-full sm:w-1/2 rounded-md border border-gray-300 p-3"
                                required=true
                                on:input=move |ev| {
                                    update_adult("email", event_target_value(&ev));
                                }
                            />
                            <input
                                type="tel"
                                placeholder="Phone *"
                                class="w-full sm:w-1/2 rounded-md border border-gray-300 p-3"
                                required=true
                                on:input=move |ev| {
                                    update_adult("phone", event_target_value(&ev));
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

    let button_class = if mobile {
        "mt-6 w-full rounded-full bg-blue-600 py-3 text-white hover:bg-blue-700 disabled:bg-gray-300 text-base sm:text-lg font-bold shadow-lg block lg:hidden"
    } else {
        "mt-6 w-full rounded-full bg-blue-600 py-3 text-white hover:bg-blue-700 disabled:bg-gray-300 text-base sm:text-lg font-bold shadow-lg hidden lg:block"
    };

    view! {
        <button
            class=button_class
            disabled=move || !is_form_valid()
            on:click=open_modal
        >
            Confirm & Book
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
                            BlockRoomUIState::set_block_room_id(Some(response.block_id));
                            BlockRoomUIState::set_block_room_called(true);
                        }
                        None => {
                            log!("Block room failed");
                            BlockRoomUIState::set_error(Some("Failed to block room".to_string()));
                        }
                    }
                }
                None => {
                    BlockRoomUIState::set_error(Some(
                        "Failed to build block room request".to_string(),
                    ));
                }
            }

            BlockRoomUIState::set_loading(false);
        }
    });

    // Trigger block room when modal opens
    create_effect(move |_| {
        if show_modal() && !block_room_called() && !is_loading() {
            block_room_action.dispatch(());
        }
    });

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

                    <Show
                        when=move || !is_loading()
                        fallback=move || view! {
                            <div class="flex justify-center items-center h-32">
                                <SpinnerGray />
                                <span class="ml-2">Checking room availability...</span>
                            </div>
                        }
                    >
                        <Show when=move || block_room_called()>
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
                    </Show>
                </div>
            </div>
        </Show>
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

    // For now, create a basic selected room - this should come from PricingBookNowState
    let selected_room = DomainRoomData {
        room_name: "Selected Room".to_string(),
        room_unique_id: "default_room".to_string(),
        rate_key: "default_rate".to_string(),
        offer_id: "".to_string(),
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
