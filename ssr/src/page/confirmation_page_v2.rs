use leptos::*;
use leptos_icons::Icon;
use leptos_router::use_query_map;
use std::collections::HashMap;

use crate::api::client_side_api::{
    ClientSideApiClient, ConfirmationProcessRequest, ConfirmationProcessResponse,
};
use crate::component::{Divider, Navbar, NotificationData, NotificationListener, SpinnerGray};
use crate::log;
use crate::utils::app_reference::BookingId;
use crate::view_state_layer::{
    booking_context_state::BookingContextState,
    ui_confirmation_page_v2::{ConfirmationPageState, ConfirmationStep},
    GlobalStateForLeptos,
};

/// **Simplified Confirmation Page V2**
///
/// **Purpose**: Clean, maintainable confirmation page with integrated step tracking
/// **Key Features**:
/// - Single state source with integrated step progression
/// - Real-time SSE updates with step details
/// - Simplified UI components following block_room_v1.rs patterns
/// - Responsive design with error handling

#[component]
pub fn ConfirmationPageV2() -> impl IntoView {
    let confirmation_state: ConfirmationPageState = expect_context();
    let booking_context: BookingContextState = expect_context();
    let query_map = use_query_map();

    // Initialize state on component mount
    create_effect(move |_| {
        ConfirmationPageState::initialize();

        // Extract payment_id from query params - support both NowPayments and Stripe
        let np_payment_id =
            query_map.with(|params| params.get("NP_payment_id").map(|p| p.to_string()));
        let checkout_session_id =
            query_map.with(|params| params.get("session_id").map(|p| p.to_string()));
        let payment_id = np_payment_id.or(checkout_session_id);

        // Read booking data from localStorage
        let booking_id = BookingId::extract_booking_id_from_local_storage();
        let app_reference = BookingId::extract_app_reference_from_local_storage();

        // Initialize booking context state
        BookingContextState::initialize_with_data(booking_id.clone(), app_reference.clone());

        log!(
            "ConfirmationPageV2 - payment_id: {:?}, app_reference: {:?}",
            payment_id,
            app_reference
        );

        ConfirmationPageState::set_payment_id(payment_id.clone());
        ConfirmationPageState::set_app_reference(app_reference.clone());

        // Trigger backend booking workflow - support both flows
        let should_trigger_workflow = match (payment_id.clone(), app_reference.clone()) {
            (Some(_), Some(_)) => true, // Flow 1: Both payment_id and app_reference present
            (None, Some(_)) => true,    // Flow 2: Only app_reference present (new flow)
            _ => false,
        };

        if should_trigger_workflow {
            ConfirmationPageState::set_loading(true);

            // Set step message based on flow type
            let step_message = if payment_id.is_some() {
                "Processing payment confirmation...".to_string()
            } else {
                "Processing booking confirmation...".to_string()
            };

            ConfirmationPageState::advance_to_step(
                ConfirmationStep::PaymentConfirmation,
                step_message,
            );

            // Extract query parameters for API call
            let query_params = query_map.with(|params| {
                params
                    .0
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect::<HashMap<String, String>>()
            });

            // Spawn async task to call confirmation API
            spawn_local(async move {
                let api_client = ClientSideApiClient::new();
                let (order_id, email) = BookingContextState::get_order_details_untracked();
                let app_ref_value = app_reference.unwrap(); // Safe because we checked above
                let order_id = order_id.unwrap_or_else(|| app_ref_value.clone());

                let request = ConfirmationProcessRequest {
                    payment_id, // This is now optional - could be None for new flow
                    app_reference: Some(order_id),
                    email,
                    query_params,
                };

                log!(
                    "Triggering confirmation API - payment_id: {:?}, app_reference: {:?}, email: {:?}",
                    request.payment_id,
                    request.app_reference,
                    request.email
                );

                match api_client.process_confirmation(request).await {
                    Some(response) => {
                        // Check if we have booking data first
                        let has_booking_data = response.booking_data.is_some();

                        // Handle booking data from API response first
                        if let Some(booking_data) = response.booking_data {
                            log!("Received booking data from API response");
                            ConfirmationPageState::set_booking_details_from_json(Some(
                                booking_data,
                            ));

                            // If we have booking data, we can show the completion immediately
                            ConfirmationPageState::advance_to_step(
                                ConfirmationStep::Completed,
                                "Your booking is confirmed! Details loaded from backend."
                                    .to_string(),
                            );
                            ConfirmationPageState::set_loading(false);
                        } else {
                            log!("No booking data in API response - will wait for SSE updates");
                            ConfirmationPageState::add_step_detail(
                                "Waiting for real-time booking updates...".to_string(),
                            );
                        }

                        if response.success {
                            log!(
                                "Booking workflow triggered successfully: {}",
                                response.message
                            );
                            ConfirmationPageState::add_step_detail(
                                "Payment confirmed, processing booking...".to_string(),
                            );
                        } else {
                            log!("Booking workflow failed: {} - but booking data may still be available", response.message);

                            // Even if pipeline failed, we might have booking data from the pre-fetch
                            if !has_booking_data {
                                ConfirmationPageState::batch_update_on_error(response.message);
                            } else {
                                // We have booking data despite pipeline failure - show a warning but continue
                                ConfirmationPageState::add_step_detail(format!(
                                    "Pipeline issue: {} - but booking data retrieved",
                                    response.message
                                ));
                            }
                        }
                    }
                    None => {
                        let error_msg = "Failed to communicate with booking service";
                        log!("{}", error_msg);
                        ConfirmationPageState::batch_update_on_error(error_msg.to_string());
                    }
                }
            });
        } else {
            // Handle error cases
            match (payment_id.clone(), app_reference.clone()) {
                (Some(_), None) => {
                    ConfirmationPageState::batch_update_on_error(
                        "Payment ID found but booking reference missing. Please ensure you completed the payment process correctly.".to_string()
                    );
                }
                (None, None) => {
                    ConfirmationPageState::batch_update_on_error(
                        "Missing booking reference. Please ensure you have a valid booking reference to view confirmation details.".to_string()
                    );
                }
                _ => {
                    // This shouldn't happen based on our logic above, but handle gracefully
                    log!("Unexpected state in confirmation page initialization");
                }
            }
        }
    });

    view! {
        <section class="flex flex-col items-center min-h-screen w-full bg-gray-50">
            <div class="w-full">
                <Navbar />
            </div>

            // SSE Integration
            <NotificationListenerWrapper />

            <div class="flex flex-col items-center w-full max-w-4xl mx-auto px-3 sm:px-4 md:px-6 lg:px-8 pt-4 sm:pt-6 md:pt-8">

                // Integrated Progress Stepper
                <div class="w-full mb-8 sm:mb-12 md:mb-16">
                    <IntegratedProgressStepper />
                </div>

                // Main Content Area - Status-based rendering
                <Show
                    when=move || ConfirmationPageState::has_error().get()
                    fallback=move || view! {
                        <Show
                            when=move || ConfirmationPageState::is_workflow_complete().get()
                            fallback=LoadingView
                        >
                            <BookingConfirmationDisplay />
                        </Show>
                    }
                >
                    <ErrorView />
                </Show>
            </div>
        </section>
    }
}

/// **SSE Integration Wrapper**
#[component]
fn NotificationListenerWrapper() -> impl IntoView {
    view! {
        {move || {
            let app_reference = ConfirmationPageState::get_app_reference().get();
            let payment_id = ConfirmationPageState::get_payment_id().get();

            // SSE should work if we have app_reference (regardless of payment_id)
            if let Some(_app_ref) = app_reference {
                let order_id = BookingContextState::get_order_id_untracked().unwrap_or_default();
                let email = BookingContextState::get_email_untracked().unwrap_or_default();

                if !order_id.is_empty() && !email.is_empty() {
                    log!("NotificationListener setup - order_id: {}, email: {}, payment_id: {:?}", order_id, email, payment_id);

                    // Choose event type based on whether we have payment_id
                    let event_type = if payment_id.is_some() {
                        "nowpayments".to_string()
                    } else {
                        "booking_confirmation".to_string() // Different event type for non-payment flow
                    };

                    view! {
                        <NotificationListener
                            order_id={order_id.clone()}
                            email={email.clone()}
                            event_type={event_type}
                            on_notification={Box::new(move |notification: NotificationData| {
                                log!("Received notification: {:#?}", notification);
                                ConfirmationPageState::update_from_sse_notification(&notification);
                            })}
                        />
                    }.into_view()
                } else {
                    view! {
                        <div class="text-sm text-yellow-600 mb-4">
                            "Missing order details for real-time updates"
                        </div>
                    }.into_view()
                }
            } else {
                view! {
                    <div class="text-sm text-yellow-600 mb-4">
                        {if payment_id.is_some() {
                            "Payment ID found but no booking reference - manual confirmation only"
                        } else {
                            "No booking reference found - unable to track confirmation status"
                        }}
                    </div>
                }.into_view()
            }
        }}
    }
}

/// **Integrated Progress Stepper Component**
#[component]
fn IntegratedProgressStepper() -> impl IntoView {
    view! {
        <div class="flex flex-col items-center justify-center w-full py-4 sm:py-6 md:py-8">

            // Step indicators
            <div class="flex flex-row items-center justify-center w-full overflow-x-auto px-2 pb-2">
                {move || {
                    let current_step = ConfirmationPageState::get_current_step().get();
                    let completed_steps = ConfirmationPageState::get_completed_steps().get();

                    let steps = vec![
                        (ConfirmationStep::PaymentConfirmation, "Payment"),
                        (ConfirmationStep::BookingProcessing, "Booking"),
                        (ConfirmationStep::EmailSending, "Email"),
                        (ConfirmationStep::Completed, "Complete"),
                    ];

                    steps
                        .into_iter()
                        .enumerate()
                        .map(|(index, (step, label))| {
                            let is_current = current_step == step;
                            let is_completed = completed_steps.contains(&step) ||
                                (step == ConfirmationStep::Completed && current_step == ConfirmationStep::Completed);

                            let circle_classes = if is_completed {
                                "min-w-[2rem] w-6 h-6 sm:w-8 sm:h-8 rounded-full flex items-center justify-center font-medium transition-colors bg-green-500 text-white"
                            } else if is_current {
                                "min-w-[2rem] w-6 h-6 sm:w-8 sm:h-8 rounded-full flex items-center justify-center font-medium transition-colors bg-blue-500 text-white animate-pulse"
                            } else {
                                "min-w-[2rem] w-6 h-6 sm:w-8 sm:h-8 rounded-full flex items-center justify-center font-medium transition-colors bg-gray-300 text-black"
                            };

                            let line_color = if is_completed || (is_current && index > 0) {
                                "bg-green-500"
                            } else if is_current {
                                "bg-blue-500"
                            } else {
                                "bg-gray-300"
                            };

                            view! {
                                <div class="flex items-start shrink-0">
                                    <div class="flex flex-col items-center">
                                        <div class=circle_classes>
                                            {if is_completed {
                                                view! {
                                                    <Icon icon=icondata::AiCheckOutlined class="w-3 h-3 sm:w-4 sm:h-4" />
                                                }.into_view()
                                            } else {
                                                view! {
                                                    <span class="text-xs sm:text-sm">{(index + 1).to_string()}</span>
                                                }.into_view()
                                            }}
                                        </div>
                                        <span class="mt-2 sm:mt-3 md:mt-4 text-[10px] sm:text-xs text-gray-600 text-center break-words max-w-[80px] sm:max-w-[100px] md:max-w-[120px]">
                                            {label}
                                        </span>
                                    </div>
                                    {if index < 3 {
                                        view! {
                                            <div class=format!("h-[1px] w-12 sm:w-16 md:w-24 lg:w-40 transition-colors mt-3 sm:mt-4 mx-1 sm:mx-2 {}", line_color) />
                                        }.into_view()
                                    } else {
                                        view! { <div /> }.into_view()
                                    }}
                                </div>
                            }
                        })
                        .collect::<Vec<_>>()
                }}
            </div>

            // Current step message
            <div class="mt-4 text-center">
                <p class="text-sm sm:text-base text-gray-700 font-medium">
                    {move || ConfirmationPageState::get_current_step_message().get()}
                </p>

                // Step details (debug mode or detailed progress)
                <Show when=move || cfg!(feature = "debug_display")>
                    <div class="mt-2 text-xs text-gray-500">
                        {move || {
                            let step_details = ConfirmationPageState::get_step_progress().get().step_details;
                            if !step_details.is_empty() {
                                view! {
                                    <div class="space-y-1">
                                        {step_details.into_iter().map(|detail| {
                                            view! { <div>{detail}</div> }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }.into_view()
                            } else {
                                view! { <div /> }.into_view()
                            }
                        }}
                    </div>
                </Show>
            </div>
        </div>
    }
}

/// **Loading View Component**
#[component]
fn LoadingView() -> impl IntoView {
    view! {
        <div class="w-full max-w-full sm:max-w-[450px] md:max-w-[500px] lg:max-w-[600px] border border-blue-100 md:border-blue-200 rounded-xl md:rounded-2xl p-3 sm:p-4 md:p-8 lg:p-10 bg-white shadow-md md:shadow-lg space-y-4 sm:space-y-6 md:space-y-8 mx-auto mt-4 md:mt-10">
            <div class="flex flex-col items-center justify-center text-center">
                <div class="flex justify-center items-center">
                    <SpinnerGray />
                </div>
            <p class="mt-4 text-gray-600">
                    {move || ConfirmationPageState::get_current_step_message().get()}
                </p>
            </div>

            // Warning message
            <div class="text-center text-red-500 text-sm border-t border-b py-4 my-8">
                <p>"Do not close this tab until your payment is fully processed"</p>
                <p>"to avoid issues with your booking."</p>
            </div>
        </div>
    }
}

/// **Error View Component**
#[component]
fn ErrorView() -> impl IntoView {
    view! {
        <div class="w-full max-w-full sm:max-w-[450px] md:max-w-[500px] lg:max-w-[600px] border border-red-100 md:border-red-200 rounded-xl md:rounded-2xl p-3 sm:p-4 md:p-8 lg:p-10 bg-white shadow-md md:shadow-lg space-y-4 sm:space-y-6 md:space-y-8 mx-auto mt-4 md:mt-10">

            // Error header
            <div class="text-center">
                <Icon icon=icondata::AiCloseCircleOutlined class="w-8 h-8 sm:w-10 sm:h-10 mx-auto mb-2 text-red-500" />
                <h2 class="text-lg sm:text-xl md:text-2xl font-semibold text-red-600 mb-2">
                    "Booking Processing Error"
                </h2>
            </div>

            <Divider />

            // Error message
            <div class="space-y-2">
                <h3 class="font-semibold text-gray-800">
                    "What happened?"
                </h3>
                <p class="text-sm sm:text-base text-gray-700 bg-red-50 p-3 rounded-lg border border-red-100">
                    {move || "An unknown error occurred".to_string()}
                </p>
            </div>

            // Support information
            <div class="space-y-2 border-t border-gray-200 pt-4">
                <h3 class="font-semibold text-gray-800">
                    "Need Help?"
                </h3>
                <div class="bg-green-50 p-3 rounded-lg border border-green-200">
                    <p class="text-sm text-green-800 mb-1 font-medium">
                        "Our support team is here to help"
                    </p>
                    <p class="text-xs text-green-700">
                        "Please contact support with your payment reference and this error message for quick assistance."
                    </p>
                </div>
            </div>
        </div>
    }
}

/// **Booking Confirmation Display Component**
#[component]
fn BookingConfirmationDisplay() -> impl IntoView {
    view! {
        <div class="w-full max-w-full sm:max-w-[450px] md:max-w-[500px] lg:max-w-[600px] border border-blue-100 md:border-blue-200 rounded-xl md:rounded-2xl p-3 sm:p-4 md:p-8 lg:p-10 bg-white shadow-md md:shadow-lg space-y-4 sm:space-y-6 md:space-y-8 mx-auto mt-4 md:mt-10">

            // Success header - dynamic based on booking status
            {move || {
                match ConfirmationPageState::get_display_info().get() {
                    Some(display_info) => {
                        if display_info.is_confirmed {
                            view! {
                                <div class="text-center text-lg sm:text-xl md:text-2xl font-semibold text-green-600">
                                    <Icon icon=icondata::AiCheckCircleOutlined class="w-8 h-8 sm:w-10 sm:h-10 mx-auto mb-2" />
                                    "Your Booking has been confirmed!"
                                </div>
                            }.into_view()
                        } else {
                            view! {
                                <div class="text-center text-lg sm:text-xl md:text-2xl font-semibold text-blue-600">
                                    <Icon icon=icondata::AiClockCircleOutlined class="w-8 h-8 sm:w-10 sm:h-10 mx-auto mb-2" />
                                    "Your Booking is being processed"
                                    <p class="text-sm text-gray-600 mt-2 font-normal">
                                        {format!("Status: {}", display_info.booking_status_message)}
                                    </p>
                                </div>
                            }.into_view()
                        }
                    }
                    None => {
                        view! {
                            <div class="text-center text-lg sm:text-xl md:text-2xl font-semibold text-blue-600">
                                <Icon icon=icondata::AiClockCircleOutlined class="w-8 h-8 sm:w-10 sm:h-10 mx-auto mb-2" />
                                "Loading booking details..."
                            </div>
                        }.into_view()
                    }
                }
            }}

            <Divider />

            // Booking details display
            {move || {
                match ConfirmationPageState::get_display_info().get() {
                    Some(display_info) => {
                        view! {
                            <div class="space-y-4 sm:space-y-6">

                                // Hotel information
                                <div class="space-y-1">
                                    <h2 class="text-left text-base sm:text-lg md:text-xl font-semibold">
                                        {display_info.hotel_name}
                                    </h2>
                                    <p class="text-left text-gray-600 text-xs md:text-sm lg:text-base">
                                        {display_info.hotel_location}
                                    </p>
                                </div>

                                // Reference ID (app reference)
                                <div class="space-y-1 sm:space-y-1.5 md:space-y-2">
                                    <p class="text-left text-gray-600 text-xs md:text-sm lg:text-base">"Reference ID"</p>
                                    <p class="font-mono text-xs sm:text-sm md:text-base lg:text-lg break-all">
                                        {display_info.booking_reference}
                                    </p>
                                </div>

                                // Booking ID (from provider - only show if available)
                                {if !display_info.booking_ref_no.is_empty() {
                                    view! {
                                        <div class="space-y-1 sm:space-y-1.5 md:space-y-2">
                                            <p class="text-left text-gray-600 text-xs md:text-sm lg:text-base">"Booking ID"</p>
                                            <p class="font-mono text-xs sm:text-sm md:text-base lg:text-lg break-all">
                                                {display_info.booking_ref_no.clone()}
                                            </p>
                                        </div>
                                    }.into_view()
                                } else {
                                    view! { <div></div> }.into_view()
                                }}

                                // Confirmation Number (from provider - only show if available)
                                {if !display_info.confirmation_no.is_empty() {
                                    view! {
                                        <div class="space-y-1 sm:space-y-1.5 md:space-y-2">
                                            <p class="text-left text-gray-600 text-xs md:text-sm lg:text-base">"Confirmation Number"</p>
                                            <p class="font-mono text-xs sm:text-sm md:text-base lg:text-lg break-all">
                                                {display_info.confirmation_no}
                                            </p>
                                        </div>
                                    }.into_view()
                                } else {
                                    view! { <div></div> }.into_view()
                                }}

                                // Check-in and Check-out dates
                                <div class="flex flex-col md:flex-row md:justify-between md:items-center space-y-2 md:space-y-0 md:space-x-4">
                                    <div class="flex-1">
                                        <div class="flex items-center space-x-2">
                                            <Icon icon=icondata::FaCalendarSolid class="w-3 h-3 text-gray-500" />
                                            <span class="text-gray-600 text-xs md:text-sm">"Check-in"</span>
                                        </div>
                                        <p class="text-sm md:text-base font-medium mt-1">
                                            {display_info.check_in_date_formatted}
                                        </p>
                                    </div>

                                    <div class="flex items-center justify-center text-gray-400 text-xs md:text-sm">
                                        {format!("{} Night{}", display_info.number_of_nights, if display_info.number_of_nights > 1 { "s" } else { "" })}
                                    </div>

                                    <div class="flex-1">
                                        <div class="flex items-center space-x-2">
                                            <Icon icon=icondata::FaCalendarSolid class="w-3 h-3 text-gray-500" />
                                            <span class="text-gray-600 text-xs md:text-sm">"Check-out"</span>
                                        </div>
                                        <p class="text-sm md:text-base font-medium mt-1">
                                            {display_info.check_out_date_formatted}
                                        </p>
                                    </div>
                                </div>

                                // Guests & Rooms
                                <div class="space-y-1 sm:space-y-1.5 md:space-y-2">
                                    <div class="flex items-center space-x-2">
                                        <Icon icon=icondata::FaUsersSolid class="w-3 h-3 text-gray-500" />
                                        <span class="text-gray-600 text-xs md:text-sm">"Guests & Rooms"</span>
                                    </div>
                                    <p class="text-sm md:text-base font-medium">
                                        {format!("1 Room, {} Adult{} • {} children",
                                            display_info.number_of_adults,
                                            if display_info.number_of_adults > 1 { "s" } else { "" },
                                            display_info.number_of_children
                                        )}
                                    </p>
                                </div>

                                // Guest Information
                                <div class="space-y-1 sm:space-y-1.5 md:space-y-2">
                                    <h3 class="font-semibold mb-1 sm:mb-2 md:mb-3 text-xs sm:text-sm md:text-base lg:text-lg">
                                        "Guest Information"
                                    </h3>
                                    <div class="space-y-0.5 sm:space-y-1">
                                        <p class="text-[10px] sm:text-xs md:text-sm lg:text-base font-medium">
                                            {format!("{} (Primary Guest)", display_info.user_name)}
                                        </p>
                                        <p class="text-[10px] sm:text-xs md:text-sm lg:text-base text-gray-600">
                                            {display_info.user_email}
                                        </p>
                                        <p class="text-[10px] sm:text-xs md:text-sm lg:text-base text-gray-600">
                                            {display_info.user_phone}
                                        </p>
                                    </div>
                                </div>
                            </div>
                        }.into_view()
                    }
                    None => {
                        view! {
                            <div class="flex flex-col items-center justify-center text-center text-gray-500">
                                <div class="flex justify-center items-center">
                                    <SpinnerGray />
                                </div>
                                <p class="mt-2">"Loading booking details..."</p>
                            </div>
                        }.into_view()
                    }
                }
            }}

            // Footer message
            <div class="text-center text-[10px] sm:text-xs md:text-sm lg:text-base font-medium text-gray-600 pt-2 border-t border-gray-200">
                "Please take a screenshot for your reference"
            </div>

            // Support email note
            <div class="text-center text-[10px] sm:text-xs md:text-sm text-gray-500 pt-2">
                "Please check the spam folder for support email"
            </div>
        </div>
    }
}
