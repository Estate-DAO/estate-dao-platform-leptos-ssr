use crate::app::AppRoutes;
use crate::component::{Divider, Navbar};
use crate::utils::app_reference::BookingId;
use crate::view_state_layer::{
    booking_id_state::BookingIdState, cookie_booking_context_state::CookieBookingContextState,
    ui_block_room::BlockRoomUIState,
};
use leptos::*;
use leptos_icons::Icon;
use leptos_router::{use_navigate, use_query_map};

#[component]
pub fn PaymentCancelledPage() -> impl IntoView {
    let query_map = use_query_map();
    let navigate = use_navigate();

    // Initialize cookie-based booking context state
    CookieBookingContextState::initialize();

    // Check if we have booking context to determine button behavior
    let has_booking_context = Signal::derive(move || {
        // Check if we have booking ID in storage or context
        BookingIdState::get_current_booking_id().is_some()
            || CookieBookingContextState::get_booking_id_untracked().is_some()
    });

    // Get session_id from query parameters (for Stripe)
    let session_id = Signal::derive(move || {
        query_map.with(|params| params.get("session_id").map(|id| id.clone()))
    });

    // Navigation handlers
    let navigate_clone = navigate.clone();
    let handle_back_to_booking = move |_| {
        if has_booking_context.get_untracked() {
            // Navigate back to block room page if we have booking context
            navigate_clone(&AppRoutes::BlockRoom.to_string(), Default::default());
        } else {
            // If no booking context, go to home page to start over
            navigate_clone(&AppRoutes::Root.to_string(), Default::default());
        }
    };

    let handle_try_payment_again = move |_| {
        if has_booking_context.get_untracked() {
            // Navigate to block room with retry_payment parameter
            navigate(
                &format!("{}?retry_payment=true", AppRoutes::BlockRoom.to_string()),
                Default::default(),
            );
        } else {
            // No booking context, start fresh booking flow
            navigate(&AppRoutes::Root.to_string(), Default::default());
        }
    };

    view! {
        <section class="flex flex-col items-center min-h-screen w-full bg-gray-50">
            <div class="w-full">
                <Navbar />
            </div>

            <div class="w-full max-w-full sm:max-w-[450px] md:max-w-[500px] lg:max-w-[600px]
                        border border-red-100 md:border-red-200
                        rounded-xl md:rounded-2xl
                        p-3 sm:p-4 md:p-8 lg:p-10
                        bg-white shadow-md md:shadow-lg
                        space-y-4 sm:space-y-6 md:space-y-8
                        mx-auto mt-4 md:mt-10">

                // Header
                <div class="text-center">
                    <Icon icon=icondata::AiStopOutlined class="w-8 h-8 sm:w-10 sm:h-10 mx-auto mb-2 text-red-500" />
                    <h2 class="text-lg sm:text-xl md:text-2xl font-semibold text-red-600 mb-2">
                        "Payment Was Not Completed"
                    </h2>
                    <p class="text-sm sm:text-base text-gray-600">
                        "It looks like you canceled the payment process or it did not finish."
                    </p>
                </div>

                <Divider />

                // Session ID info (for debugging/support)
                {move || {
                    if let Some(id) = session_id.get() {
                        view! {
                            <div class="text-xs text-gray-500 bg-gray-50 p-2 rounded border">
                                "Session ID: " {id}
                            </div>
                        }.into_view()
                    } else {
                        view! { <div></div> }.into_view()
                    }
                }}

                // Info section
                <div class="space-y-3">
                    <h3 class="font-semibold text-gray-800">
                        "No worries!"
                    </h3>
                    <p class="text-sm sm:text-base text-gray-700">
                        "Your booking has not been confirmed, and you haven't been charged."
                        " You can retry payment or go back to edit your booking details."
                    </p>
                </div>

                // Action buttons
                <div class="flex flex-col sm:flex-row gap-3 pt-4">
                    <button
                        on:click=handle_back_to_booking
                        class="w-full text-center py-2 px-4 bg-gray-200 hover:bg-gray-300
                               rounded-lg text-gray-700 text-sm md:text-base font-medium
                               transition-colors duration-200">
                        {move || {
                            if has_booking_context.get() {
                                "Back to Booking"
                            } else {
                                "Start New Booking"
                            }
                        }}
                    </button>

                    <button
                        on:click=handle_try_payment_again
                        class="w-full text-center py-2 px-4 bg-blue-600 hover:bg-blue-700
                               rounded-lg text-white text-sm md:text-base font-medium
                               transition-colors duration-200">
                        {move || {
                            if has_booking_context.get() {
                                "Try Payment Again"
                            } else {
                                "Start New Booking"
                            }
                        }}
                    </button>
                </div>

                // Footer help text
                <div class="text-center text-xs md:text-sm text-gray-600 pt-2 border-t border-gray-200">
                    "If you believe this is a mistake, feel free to contact support."
                </div>
            </div>
        </section>
    }
}
