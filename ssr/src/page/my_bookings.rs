use crate::api::auth::auth_state::{AuthState, AuthStateSignal};
use crate::api::canister::user_my_bookings::user_get_my_bookings;
use crate::component::{yral_auth_provider::YralAuthProvider, Navbar};
use crate::log;
use crate::page::Wishlist;
use crate::view_state_layer::my_bookings_state::{BookingStatus, BookingTab, MyBookingItem};
use chrono::{DateTime, Utc};
use leptos::*;
use leptos_icons::Icon;
use std::rc::Rc;
async fn load_my_bookings() -> Result<Vec<MyBookingItem>, ServerFnError> {
    log!("[MyBookings] Loading bookings from API");

    let auth_state = AuthStateSignal::auth_state().get();
    let backend_bookings =
        user_get_my_bookings(auth_state.email.ok_or(ServerFnError::new("Unauthorized"))?).await?;
    log!(
        "[MyBookings] Retrieved {} bookings from backend",
        backend_bookings.len()
    );

    let total_bookings_count = backend_bookings.len();

    let complete_bookings: Vec<_> = backend_bookings
        .into_iter()
        .filter(|booking| {
            let has_payment = matches!(
                booking.payment_details.payment_status,
                crate::canister::backend::BackendPaymentStatus::Paid(_)
            );
            let has_booking_confirmation = booking.book_room_status.is_some();
            has_payment && has_booking_confirmation
        })
        .collect();

    let bookings: Vec<MyBookingItem> = complete_bookings
        .into_iter()
        .map(|booking| booking.into())
        .collect();

    log!(
        "[MyBookings] Returning {} complete bookings (filtered from {} total)",
        bookings.len(),
        total_bookings_count
    );
    Ok(bookings)
}

#[component]
pub fn MyBookingsPage() -> impl IntoView {
    log!("[MyBookings] MyBookingsPage started");

    view! {
        <div class="min-h-screen flex flex-col">
            <Navbar />
            <MyBookings />
        </div>
    }
}

#[component]
pub fn MyBookings() -> impl IntoView {
    view! {
        <div class="flex flex-col">
        {/* Sticky header */}
            <div class="sticky top-0 z-30 bg-white border-b border-gray-200">
                <div class="max-w-6xl mx-auto px-4 sm:px-6 lg:px-8 py-4">
                    <h1 class="text-2xl font-semibold text-gray-900">My Bookings</h1>
                </div>
                <div class="border-t border-gray-200">
                    <AuthGatedBookings />
                </div>
            </div>

            {/* Content */}
            <div class="flex-1 bg-gray-50">
                <BookingsLoader />
            </div>
        </div>
    }
}

#[component]
pub fn AuthGatedBookings() -> impl IntoView {
    view! {
        <Show when=move||AuthStateSignal::auth_state().get().email.is_none()>
            <BookingsLoginPrompt />
        </Show>
    }
}

#[component]
pub fn BookingsLoginPrompt() -> impl IntoView {
    view! {
        <div class="max-w-md mx-auto mt-12">
            <div class="bg-white rounded-2xl shadow-md p-8 text-center">
                <Icon icon=icondata::AiUserOutlined class="text-gray-400 text-5xl mx-auto mb-3" />
                <h2 class="text-xl font-semibold text-gray-900 mb-1">Login Required</h2>
                <p class="text-gray-600 mb-5">Please login to view your booking history</p>
                <div class="flex justify-center">
                    <YralAuthProvider />
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn BookingsLoader() -> impl IntoView {
    let bookings_resource = create_resource(
        || (),
        |_| async move {
            log!("[MyBookings] Fetching bookings...");
            load_my_bookings().await
        },
    );

    view! {
        <Suspense fallback=move || view! {
            <div class="flex justify-center items-center py-12">
                <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
                <span class="ml-3 text-gray-600">Loading bookings...</span>
            </div>
        }>
            {move || match bookings_resource.get() {
                Some(Ok(bookings)) => view! { <BookingsContent bookings=bookings /> }.into_view(),
                Some(Err(e)) => view! {
                    <div class="text-center py-12 text-red-600">Error loading bookings: {e.to_string()}</div>
                }.into_view(),
                None => view! { <></> }.into_view(),
            }}
        </Suspense>
    }
}

#[component]
fn BookingsTabs() -> impl IntoView {
    let current_tab = RwSignal::new(BookingTab::Completed);
    view! {
        <div class="flex justify-start sm:justify-start border-b border-gray-200 max-w-6xl mx-auto px-4 sm:px-6 lg:px-8">
            <TabButtonLocal tab=BookingTab::Upcoming label="Upcoming" current_tab=current_tab />
            <TabButtonLocal tab=BookingTab::Completed label="Completed" current_tab=current_tab />
            <TabButtonLocal tab=BookingTab::Cancelled label="Cancelled" current_tab=current_tab />
        </div>
    }
}

#[component]
fn TabButtonLocal(
    tab: BookingTab,
    label: &'static str,
    current_tab: RwSignal<BookingTab>,
) -> impl IntoView {
    let on_click = move |_| current_tab.set(tab);
    view! {
        <button
            on:click=on_click
            class=move || format!(
                "relative py-3 px-4 text-sm font-medium transition-colors {}",
                if current_tab.get() == tab {
                    "text-blue-600 after:absolute after:bottom-0 after:left-0 after:w-full after:h-0.5 after:bg-blue-600"
                } else {
                    "text-gray-500 hover:text-gray-800"
                }
            )
        >
            {label}
        </button>
    }
}

#[component]
fn BookingsContent(bookings: Vec<MyBookingItem>) -> impl IntoView {
    // Track current active tab
    let current_tab = RwSignal::new(BookingTab::Upcoming);

    // Derive filtered list reactively
    let filtered_bookings = Signal::derive({
        let bookings = bookings.clone();
        move || {
            let active_tab = current_tab.get();
            let mut filtered = bookings
                .iter()
                .filter(|b| match active_tab {
                    BookingTab::Upcoming => b.status == BookingStatus::Upcoming,
                    BookingTab::Completed => b.status == BookingStatus::Completed,
                    BookingTab::Cancelled => b.status == BookingStatus::Cancelled,
                })
                .cloned()
                .collect::<Vec<_>>();

            // Sort upcoming bookings by check-in date (soonest first)
            if active_tab == BookingTab::Upcoming {
                filtered.sort_by_key(|b| b.check_in_date);
            }

            filtered
        }
    });

    // Helper: count bookings per tab
    let get_tab_count = {
        let bookings = bookings.clone();
        move |tab: BookingTab| {
            bookings
                .iter()
                .filter(|b| match tab {
                    BookingTab::Upcoming => b.status == BookingStatus::Upcoming,
                    BookingTab::Completed => b.status == BookingStatus::Completed,
                    BookingTab::Cancelled => b.status == BookingStatus::Cancelled,
                })
                .count()
        }
    };

    view! {
        <div class="max-w-6xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
            // Tabs
            <div class="flex border-b border-gray-200 mb-6">
                <TabButtonLocal
                    tab=BookingTab::Upcoming
                    label="Upcoming"
                    current_tab=current_tab
                />
                <span class="ml-1 text-xs text-gray-500 self-center">
                    {get_tab_count(BookingTab::Upcoming)}
                </span>

                <TabButtonLocal
                    tab=BookingTab::Completed
                    label="Completed"
                    current_tab=current_tab
                />
                <span class="ml-1 text-xs text-gray-500 self-center">
                    {get_tab_count(BookingTab::Completed)}
                </span>

                <TabButtonLocal
                    tab=BookingTab::Cancelled
                    label="Cancelled"
                    current_tab=current_tab
                />
                <span class="ml-1 text-xs text-gray-500 self-center">
                    {get_tab_count(BookingTab::Cancelled)}
                </span>
            </div>

            // Booking list
            {move || {
                let data = filtered_bookings.get();
                if data.is_empty() {
                    view! { <EmptyBookingsState /> }.into_view()
                } else {
                    view! {
                        <div class="space-y-6">
                            <For
                                each=move || data.clone()
                                key=|b| b.booking_id.clone()
                                children=|b| view! { <BookingCard booking=b /> }
                            />
                        </div>
                    }.into_view()
                }
            }}
        </div>
    }
}

#[component]
fn BookingCard(booking: MyBookingItem) -> impl IntoView {
    let format_date = |date: DateTime<Utc>| date.format("%d %B %Y").to_string();
    let status_color = match booking.status {
        BookingStatus::Upcoming => "text-blue-600",
        BookingStatus::Completed => "text-green-600",
        BookingStatus::Cancelled => "text-red-600",
    };

    let hotel_code = booking.hotel_code.clone();

    view! {
        <div class="bg-white rounded-xl shadow-sm border border-gray-100 hover:shadow-md transition-shadow duration-200">
            <div class="flex flex-col md:flex-row">
                {/* Hotel image + Wishlist */}
                <div class="relative w-full h-40 md:h-auto md:basis-[28%] md:flex-shrink-0">
                    <img
                        class="w-full h-full object-cover md:rounded-l-xl rounded-t-xl md:rounded-t-none"
                        src=booking.hotel_image_url.clone()
                        alt=booking.hotel_name.clone()
                    />

                    {/* Top-right container for Wishlist + Test badge */}
                    <div class="absolute top-2 right-2 flex items-center space-x-2">
                        <Wishlist hotel_code />

                        <Show when=move || booking.is_test>
                            <div class="bg-yellow-500 text-white text-xs font-semibold px-2 py-0.5 rounded-md shadow">
                                "Test"
                            </div>
                        </Show>
                    </div>
                </div>

                {/* Booking details */}
                <div class="flex-1 p-5 flex flex-col justify-between">
                    <div>
                        <h3 class="text-lg font-semibold text-gray-900">{booking.hotel_name.clone()}</h3>
                        <p class="text-sm text-gray-600 mb-3">{booking.hotel_location.clone()}</p>

                        <p class="text-sm text-gray-500 mb-2">
                            Booking ID: <span class="font-medium text-gray-700">{booking.booking_id.clone()}</span>
                        </p>

                        <div class="flex items-center text-sm text-gray-600 mb-2">
                            <svg class="w-4 h-4 mr-1 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                    d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
                            </svg>
                            {format!("{} - {}", format_date(booking.check_in_date), format_date(booking.check_out_date))}
                        </div>

                        <div class="flex flex-wrap items-center gap-3 text-sm text-gray-600">
                            <div class="flex items-center">
                                <svg class="w-4 h-4 mr-1 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                        d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
                                </svg>
                                <span>{format!("{} adult", booking.adults)}</span>
                            </div>
                            <div class="flex items-center">
                                <svg class="w-4 h-4 mr-1 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                        d="M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4" />
                                </svg>
                                <span>{format!("{} room", booking.rooms)}</span>
                            </div>
                        </div>
                    </div>

                    <div class="mt-3">
                        <span class=format!("text-sm font-medium {}", status_color)>
                            {booking.status.to_string()}
                        </span>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn EmptyBookingsState() -> impl IntoView {
    view! {
        <div class="text-center py-16">
            <img src="/img/no-booking-found.svg" alt="No bookings" class="mx-auto w-40 mb-4" />
            <h3 class="text-lg font-semibold text-gray-900">No bookings yet</h3>
            <p class="text-gray-500 mb-6">When you book a hotel, it will appear here.</p>
            <a href="/" class="inline-flex items-center px-4 py-2 text-sm font-medium text-white bg-blue-600 rounded-md hover:bg-blue-700 transition">
                Start Booking
            </a>
        </div>
    }
}
