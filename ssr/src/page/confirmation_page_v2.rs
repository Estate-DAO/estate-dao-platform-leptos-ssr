use leptos::*;
use leptos_icons::Icon;
use leptos_router::use_query_map;
use std::collections::HashMap;

use crate::api::client_side_api::{ClientSideApiClient, ConfirmationProcessRequest};
use crate::component::yral_auth_provider::YralAuthProvider;
use crate::component::{
    CurrencySelectorModal, Divider, Navbar, NotificationData, NotificationListener, SpinnerGray,
};
use crate::log;
use crate::view_state_layer::{
    cookie_booking_context_state::CookieBookingContextState,
    ui_confirmation_page_v2::{ConfirmationPageState, ConfirmationStep},
};

const MSITE_CARD_CLASS: &str = "rounded-xl border border-gray-200 bg-white shadow-sm";
const MSITE_SECTION_CLASS: &str =
    "w-full rounded-xl border border-gray-200 bg-white p-4 shadow-sm sm:p-6";
const MSITE_SUBSECTION_CLASS: &str = "rounded-lg border border-gray-200 bg-slate-50 p-3.5";
const MSITE_LABEL_CLASS: &str =
    "text-[11px] font-semibold uppercase tracking-[0.14em] text-slate-500";

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

        // Initialize cookie-based booking context state (loads from cookies automatically)
        CookieBookingContextState::initialize();

        // Debug: Log domain information for troubleshooting
        #[cfg(feature = "hydrate")]
        {
            if let Some(window) = leptos_use::use_window().as_ref() {
                let location = window.location();
                if let (Ok(hostname), Ok(href)) = (location.hostname(), location.href()) {
                    log!(
                        "ConfirmationPageV2 - Domain debug: hostname={}, full_url={}",
                        hostname,
                        href
                    );
                }
            }
        }

        log!("ConfirmationPageV2 - payment_id: {:?}", payment_id);

        ConfirmationPageState::set_payment_id(payment_id.clone());

        // We'll get app_reference from the cookie-loaded context once it's ready
        // This is handled in a separate effect below
    });

    // Separate effect to handle workflow trigger once cookie data is loaded
    create_effect(move |_| {
        // Cookie data is loaded synchronously, so we can check immediately
        let app_reference = CookieBookingContextState::get_app_reference().get();
        let payment_id = ConfirmationPageState::get_payment_id().get();

        // Set app_reference in confirmation state
        ConfirmationPageState::set_app_reference(app_reference.clone());

        log!(
            "Cookie data check - app_reference: {:?}, payment_id: {:?}",
            app_reference,
            payment_id
        );

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
                let (order_id, email) = CookieBookingContextState::get_order_details_untracked();
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
        <section class="relative min-h-screen bg-slate-50">
            <div class="hidden lg:block">
                <Navbar />
            </div>

            <nav class="lg:hidden sticky top-0 z-[1001] bg-white/95 supports-[backdrop-filter]:bg-white/90 backdrop-blur border-b border-gray-100 h-14 flex items-center justify-between px-4">
                <a href="/" class="flex items-center">
                    <img src="/img/nofeebooking.webp" alt="NoFeeBooking" class="h-8 w-auto" />
                </a>

                <div class="flex items-center gap-2">
                    <CurrencySelectorModal />
                    <YralAuthProvider />
                </div>
            </nav>

            <div class="mx-auto flex w-full max-w-4xl flex-col gap-4 px-4 pb-10 pt-3 sm:px-6 sm:pt-5 lg:px-8 lg:pt-6">
                <div class=format!("{MSITE_CARD_CLASS} px-4 py-3.5 sm:px-5 sm:py-4")>
                    <div class="space-y-0.5">
                        <p class=MSITE_LABEL_CLASS>"Confirmation"</p>
                        <h1 class="text-lg font-semibold text-slate-900 sm:text-xl">
                            "Booking Confirmation"
                        </h1>
                        <p class="text-sm text-slate-600">
                            "Keep this page open while we finalize your reservation details."
                        </p>
                    </div>
                </div>

                // SSE Integration
                <NotificationListenerWrapper />

                // Integrated Progress Stepper
                <div class="w-full">
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
                let order_id = CookieBookingContextState::get_order_id_untracked().unwrap_or_default();
                let email = CookieBookingContextState::get_email_untracked().unwrap_or_default();

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
                        <div class="rounded-lg border border-amber-200 bg-amber-50 px-3 py-2.5 text-sm text-amber-800">
                            "Missing order details for real-time updates"
                        </div>
                    }.into_view()
                }
            } else {
                view! {
                    <div class="rounded-lg border border-amber-200 bg-amber-50 px-3 py-2.5 text-sm text-amber-800">
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
        <div class=MSITE_SECTION_CLASS>
            <div class="space-y-4">
                <div class="space-y-1">
                    <p class=MSITE_LABEL_CLASS>"Confirmation Progress"</p>
                    <p class="text-sm text-slate-600">
                        "Track each step as your booking is confirmed."
                    </p>
                </div>

                // Step indicators
                <div class="flex flex-row items-center justify-center w-full overflow-x-auto pb-1">
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
                                let is_completed = completed_steps.contains(&step)
                                    || (step == ConfirmationStep::Completed
                                        && current_step == ConfirmationStep::Completed);

                                let circle_classes = if is_completed {
                                    "flex h-7 w-7 min-w-[1.75rem] items-center justify-center rounded-full bg-green-500 text-white transition-colors sm:h-8 sm:w-8"
                                } else if is_current {
                                    "flex h-7 w-7 min-w-[1.75rem] items-center justify-center rounded-full bg-blue-600 text-white transition-colors animate-pulse sm:h-8 sm:w-8"
                                } else {
                                    "flex h-7 w-7 min-w-[1.75rem] items-center justify-center rounded-full bg-slate-300 text-slate-700 transition-colors sm:h-8 sm:w-8"
                                };

                                let line_color = if is_completed || (is_current && index > 0) {
                                    "bg-green-500"
                                } else if is_current {
                                    "bg-blue-600"
                                } else {
                                    "bg-slate-300"
                                };

                                view! {
                                    <div class="flex items-start shrink-0">
                                        <div class="flex flex-col items-center">
                                            <div class=circle_classes>
                                                {if is_completed {
                                                    view! {
                                                        <Icon icon=icondata::AiCheckOutlined class="h-3 w-3 sm:h-4 sm:w-4" />
                                                    }.into_view()
                                                } else {
                                                    view! {
                                                        <span class="text-xs font-medium sm:text-sm">{(index + 1).to_string()}</span>
                                                    }.into_view()
                                                }}
                                            </div>
                                            <span class="mt-2 max-w-[84px] break-words text-center text-[11px] font-medium text-slate-600 sm:max-w-[100px]">
                                                {label}
                                            </span>
                                        </div>
                                        {if index < 3 {
                                            view! {
                                                <div class=format!("mt-3 mx-1 h-px w-10 transition-colors sm:mt-4 sm:mx-2 sm:w-14 md:w-20 {}", line_color) />
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
                <div class="text-center">
                    <p class="text-sm font-medium text-slate-700 sm:text-base">
                        {move || ConfirmationPageState::get_current_step_message().get()}
                    </p>

                    // Step details (debug mode or detailed progress)
                    <Show when=move || cfg!(feature = "debug_display")>
                        <div class="mt-2 text-xs text-slate-500">
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
        </div>
    }
}

/// **Loading View Component**
#[component]
fn LoadingView() -> impl IntoView {
    view! {
        <div class=MSITE_SECTION_CLASS>
            <div class="flex flex-col items-center justify-center text-center">
                <div class="flex justify-center items-center">
                    <SpinnerGray />
                </div>
                <p class="mt-4 text-sm text-slate-600 sm:text-base">
                    {move || ConfirmationPageState::get_current_step_message().get()}
                </p>
            </div>

            // Warning message
            <div class="rounded-lg border border-amber-200 bg-amber-50 px-3 py-2.5 text-center text-sm text-amber-800">
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
        <div class=MSITE_SECTION_CLASS>

            // Error header
            <div class="space-y-2 text-center">
                <Icon icon=icondata::AiCloseCircleOutlined class="w-8 h-8 sm:w-10 sm:h-10 mx-auto mb-2 text-red-500" />
                <h2 class="text-lg font-semibold text-red-600 sm:text-xl">
                    "Booking Processing Error"
                </h2>
            </div>

            <Divider />

            // Error message
            <div class="space-y-2">
                <h3 class="text-sm font-semibold text-slate-800">
                    "What happened?"
                </h3>
                <p class="rounded-lg border border-red-200 bg-red-50 p-3.5 text-sm text-red-800 sm:text-base">
                    {move || ConfirmationPageState::get_error().get().unwrap_or_else(|| "An unknown error occurred".to_string())}
                </p>
            </div>

            // Support information
            <div class="space-y-2 border-t border-gray-200 pt-4">
                <h3 class="text-sm font-semibold text-slate-800">
                    "Need Help?"
                </h3>
                <div class="rounded-lg border border-green-200 bg-green-50 p-3.5">
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
        <div class=MSITE_SECTION_CLASS>

            // Success header - dynamic based on booking status
            {move || {
                match ConfirmationPageState::get_display_info().get() {
                    Some(display_info) => {
                        if display_info.is_confirmed {
                            view! {
                                <div class="space-y-2 text-center">
                                    <Icon icon=icondata::AiCheckCircleOutlined class="w-8 h-8 sm:w-10 sm:h-10 mx-auto mb-2" />
                                    <div class="text-lg font-semibold text-green-600 sm:text-xl">
                                        "Your Booking has been confirmed!"
                                    </div>
                                </div>
                            }.into_view()
                        } else {
                            view! {
                                <div class="space-y-2 text-center">
                                    <Icon icon=icondata::AiClockCircleOutlined class="w-8 h-8 sm:w-10 sm:h-10 mx-auto mb-2" />
                                    <div class="text-lg font-semibold text-blue-600 sm:text-xl">
                                        "Your Booking is being processed"
                                    </div>
                                    <p class="text-sm font-normal text-slate-600">
                                        {format!("Status: {}", display_info.booking_status_message)}
                                    </p>
                                </div>
                            }.into_view()
                        }
                    }
                    None => {
                        view! {
                            <div class="space-y-2 text-center">
                                <Icon icon=icondata::AiClockCircleOutlined class="w-8 h-8 sm:w-10 sm:h-10 mx-auto mb-2" />
                                <div class="text-lg font-semibold text-blue-600 sm:text-xl">
                                    "Loading booking details..."
                                </div>
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
                            <div class="space-y-4">

                                // Hotel information
                                <div class=MSITE_SUBSECTION_CLASS>
                                    <p class=MSITE_LABEL_CLASS>"Hotel"</p>
                                    <h2 class="mt-1.5 text-left text-base font-semibold text-slate-900 sm:text-lg">
                                        {display_info.hotel_name}
                                    </h2>
                                    <p class="mt-1 text-left text-sm text-slate-600">
                                        {display_info.hotel_location}
                                    </p>
                                </div>

                                // Reference ID (app reference)
                                <div class="grid gap-3 sm:grid-cols-2">
                                    <div class=MSITE_SUBSECTION_CLASS>
                                        <p class=MSITE_LABEL_CLASS>"Reference ID"</p>
                                        <p class="mt-1.5 break-all font-mono text-sm text-slate-900 sm:text-base">
                                            {display_info.booking_reference}
                                        </p>
                                    </div>

                                    // Booking ID (from provider - only show if available)
                                    {if !display_info.booking_ref_no.is_empty() {
                                        view! {
                                            <div class=MSITE_SUBSECTION_CLASS>
                                                <p class=MSITE_LABEL_CLASS>"Booking ID"</p>
                                                <p class="mt-1.5 break-all font-mono text-sm text-slate-900 sm:text-base">
                                                    {display_info.booking_ref_no.clone()}
                                                </p>
                                            </div>
                                        }.into_view()
                                    } else {
                                        view! { <div /> }.into_view()
                                    }}

                                    // Confirmation Number (from provider - only show if available)
                                    {if !display_info.confirmation_no.is_empty() {
                                        view! {
                                            <div class=MSITE_SUBSECTION_CLASS>
                                                <p class=MSITE_LABEL_CLASS>"Confirmation Number"</p>
                                                <p class="mt-1.5 break-all font-mono text-sm text-slate-900 sm:text-base">
                                                    {display_info.confirmation_no}
                                                </p>
                                            </div>
                                        }.into_view()
                                    } else {
                                        view! { <div /> }.into_view()
                                    }}
                                </div>

                                // Check-in and Check-out dates
                                <div class="grid gap-3 sm:grid-cols-2">
                                    <div class=MSITE_SUBSECTION_CLASS>
                                        <div class="flex items-center gap-2">
                                            <Icon icon=icondata::FaCalendarSolid class="h-3 w-3 text-slate-500" />
                                            <span class=MSITE_LABEL_CLASS>"Check-in"</span>
                                        </div>
                                        <p class="mt-1.5 text-sm font-medium text-slate-900 sm:text-base">
                                            {display_info.check_in_date_formatted}
                                        </p>
                                    </div>

                                    <div class=MSITE_SUBSECTION_CLASS>
                                        <div class="flex items-center gap-2">
                                            <Icon icon=icondata::FaCalendarSolid class="h-3 w-3 text-slate-500" />
                                            <span class=MSITE_LABEL_CLASS>"Check-out"</span>
                                        </div>
                                        <p class="mt-1.5 text-sm font-medium text-slate-900 sm:text-base">
                                            {display_info.check_out_date_formatted}
                                        </p>
                                    </div>
                                </div>

                                // Guests & Rooms
                                <div class="grid gap-3 sm:grid-cols-2">
                                    <div class=MSITE_SUBSECTION_CLASS>
                                        <p class=MSITE_LABEL_CLASS>"Stay Length"</p>
                                        <p class="mt-1.5 text-sm font-medium text-slate-900 sm:text-base">
                                            {format!("{} Night{}", display_info.number_of_nights, if display_info.number_of_nights > 1 { "s" } else { "" })}
                                        </p>
                                    </div>

                                    <div class=MSITE_SUBSECTION_CLASS>
                                        <div class="flex items-center gap-2">
                                            <Icon icon=icondata::FaUsersSolid class="h-3 w-3 text-slate-500" />
                                            <span class=MSITE_LABEL_CLASS>"Guests & Rooms"</span>
                                        </div>
                                        <p class="mt-1.5 text-sm font-medium text-slate-900 sm:text-base">
                                            {format!(
                                                "{} Room{}, {} Adult{} • {} children",
                                                display_info.number_of_rooms,
                                                if display_info.number_of_rooms == 1 { "" } else { "s" },
                                                display_info.number_of_adults,
                                                if display_info.number_of_adults == 1 { "" } else { "s" },
                                                display_info.number_of_children
                                            )}
                                        </p>
                                    </div>
                                </div>

                                // Guest Information
                                <div class=MSITE_SUBSECTION_CLASS>
                                    <p class=MSITE_LABEL_CLASS>"Guest Information"</p>
                                    <div class="mt-1.5 space-y-1">
                                        <p class="text-sm font-medium text-slate-900 sm:text-base">
                                            {format!("{} (Primary Guest)", display_info.user_name)}
                                        </p>
                                        <p class="text-sm text-slate-600 sm:text-base">
                                            {display_info.user_email}
                                        </p>
                                        <p class="text-sm text-slate-600 sm:text-base">
                                            {display_info.user_phone}
                                        </p>
                                    </div>
                                </div>
                            </div>
                        }.into_view()
                    }
                    None => {
                        view! {
                            <div class="flex flex-col items-center justify-center text-center text-slate-500">
                                <div class="flex justify-center items-center">
                                    <SpinnerGray />
                                </div>
                                <p class="mt-2 text-sm sm:text-base">"Loading booking details..."</p>
                            </div>
                        }.into_view()
                    }
                }
            }}

            // Footer message
            <div class="border-t border-gray-200 pt-3 text-center text-sm font-medium text-slate-600">
                "Please take a screenshot for your reference"
            </div>
        </div>
    }
}
