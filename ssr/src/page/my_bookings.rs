use crate::api::auth::auth_state::{AuthState, AuthStateSignal};
use crate::api::canister::user_my_bookings::user_get_my_bookings;
use crate::component::Navbar;
use crate::log;
use crate::view_state_layer::my_bookings_state::{
    BookingStatus, BookingTab, MyBookingItem, MyBookingsState,
};
use chrono::{DateTime, Utc};
use leptos::SignalGet;
use leptos::*;
use leptos_router::*;
use std::rc::Rc;

async fn load_my_bookings() -> Result<Vec<MyBookingItem>, ServerFnError> {
    log!("[MyBookings] Loading bookings from API");

    let auth_state_signal: AuthStateSignal = expect_context();
    let auth_state = auth_state_signal.get();
    // Call actual canister API to get bookings
    let backend_bookings = user_get_my_bookings(auth_state).await?;
    log!(
        "[MyBookings] Retrieved {} bookings from backend",
        backend_bookings.len()
    );

    // Convert backend Booking objects to MyBookingItem
    let bookings: Vec<MyBookingItem> = backend_bookings
        .into_iter()
        .map(|booking| booking.into())
        .collect();

    log!(
        "[MyBookings] Returning {} converted bookings",
        bookings.len()
    );
    Ok(bookings)
}

#[component]
pub fn MyBookingsPage() -> impl IntoView {
    log!("[MyBookings] MyBookingsPage component started");

    let auth_state_signal: AuthStateSignal = expect_context();
    // Create resource for loading bookings data that waits for canister store to be ready
    let bookings_resource = create_resource(
        move || {
            let auth = auth_state_signal.get();

            // Check if canister store is ready
            let canister_store_ready = auth.new_cans_setter.get().is_some();
            let user_identity_ready = auth.user_identity.get().is_some();
            log!(
                "[MyBookings] Resource signal - canister_store_ready: {}, user_identity_ready: {}",
                canister_store_ready,
                user_identity_ready
            );
            (canister_store_ready, user_identity_ready)
        },
        move |(canister_ready, identity_ready)| async move {
            log!(
                "[MyBookings] Resource triggered - canister_ready: {}, identity_ready: {}",
                canister_ready,
                identity_ready
            );

            // Wait for both canister store and user identity to be ready
            if !canister_ready || !identity_ready {
                log!("[MyBookings] Resource waiting - auth not fully ready yet");
                return Err(ServerFnError::new("Auth state not ready yet"));
            }

            // let auth_state = auth_state_signal.get();
            log!("[MyBookings] Resource loading bookings");
            load_my_bookings().await
        },
    );

    log!("[MyBookings] Starting to render view");

    view! {
        <div class="min-h-screen bg-gray-50">
            // <!-- Navbar -->
            <Navbar />

            // <!-- Header section with hero background -->
            <div class="relative bg-gradient-to-r from-blue-600 to-blue-800 text-white">
                <div class="absolute inset-0 bg-black opacity-20"></div>
                <div class="relative max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-16">
                    <h1 class="text-4xl font-bold mb-2">My Bookings</h1>
                </div>
            </div>

            <div class="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 pt-6">
                // <!-- Tab navigation -->
                <div class="bg-white rounded-lg shadow-lg mb-6">

                    // <!-- Content area -->
                    <div class="p-6">
                        <Suspense fallback=move || view! {
                            <div class="flex justify-center items-center py-12">
                                <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
                                <span class="ml-3 text-gray-600">Loading bookings...</span>
                            </div>
                        }>
                            {move || {
                                match bookings_resource.get() {
                                    Some(Ok(bookings)) => {
                                        log!("[MyBookings] Resource loaded {} bookings", bookings.len());
                                        view! { <BookingsContent bookings=bookings /> }.into_view()
                                    }
                                    Some(Err(error)) => {
                                        log!("[MyBookings] Resource error: {}", error);
                                        view! {
                                            <div class="text-center py-8">
                                                <p class="text-red-600 mb-2">Failed to load bookings</p>
                                                <p class="text-gray-500 text-sm">{error.to_string()}</p>
                                            </div>
                                        }.into_view()
                                    }
                                    None => view! { <>"Load None"</> }.into_view()
                                }
                            }}
                        </Suspense>
                    </div>
                </div>
            </div>
        </div>
    }.into_view()
}

#[component]
fn TabButtonLocal(
    tab: BookingTab,
    label: &'static str,
    count: usize,
    current_tab: RwSignal<BookingTab>,
) -> impl IntoView {
    let on_click = move |_| {
        current_tab.set(tab);
    };

    view! {
        <button
            class=move || format!(
                "flex-1 px-6 py-4 text-sm font-medium border-b-2 transition-colors {}",
                if current_tab.get() == tab {
                    "border-blue-600 text-blue-600 bg-blue-50"
                } else {
                    "border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300"
                }
            )
            on:click=on_click
        >
            <span class="flex items-center justify-center gap-2">
                {label}
                <span class=move || format!(
                    "px-2 py-1 text-xs rounded-full {}",
                    if current_tab.get() == tab {
                        "bg-blue-600 text-white"
                    } else {
                        "bg-gray-200 text-gray-600"
                    }
                )>
                    {count}
                </span>
            </span>
        </button>
    }
}

#[component]
fn BookingsContent(bookings: Vec<MyBookingItem>) -> impl IntoView {
    log!(
        "[MyBookings] BookingsContent component started with {} bookings",
        bookings.len()
    );

    // Create state for managing current tab
    let current_tab = RwSignal::new(BookingTab::Upcoming);

    // Wrap bookings in Rc to allow sharing between closures
    let bookings_rc = Rc::new(bookings);
    let bookings_for_filter = bookings_rc.clone();
    let bookings_for_count = bookings_rc.clone();

    // Filter bookings by current tab
    let filtered_bookings = Signal::derive(move || {
        let active_tab = current_tab.get();
        bookings_for_filter
            .iter()
            .filter(|booking| match active_tab {
                BookingTab::Upcoming => booking.status == BookingStatus::Upcoming,
                BookingTab::Completed => booking.status == BookingStatus::Completed,
                BookingTab::Cancelled => booking.status == BookingStatus::Cancelled,
            })
            .cloned()
            .collect::<Vec<_>>()
    });

    // Get tab counts
    let get_tab_count = move |tab: BookingTab| {
        bookings_for_count
            .iter()
            .filter(|booking| match tab {
                BookingTab::Upcoming => booking.status == BookingStatus::Upcoming,
                BookingTab::Completed => booking.status == BookingStatus::Completed,
                BookingTab::Cancelled => booking.status == BookingStatus::Cancelled,
            })
            .count()
    };

    view! {
        // <!-- Tab navigation -->
        <div class="flex border-b border-gray-200 mb-6">
            <TabButtonLocal
                tab=BookingTab::Upcoming
                label="Upcoming"
                count=get_tab_count(BookingTab::Upcoming)
                current_tab=current_tab
            />
            <TabButtonLocal
                tab=BookingTab::Completed
                label="Completed"
                count=get_tab_count(BookingTab::Completed)
                current_tab=current_tab
            />
            <TabButtonLocal
                tab=BookingTab::Cancelled
                label="Cancelled"
                count=get_tab_count(BookingTab::Cancelled)
                current_tab=current_tab
            />
        </div>

        // <!-- Bookings content -->
        <Suspense fallback=move || view! {
            <div class="flex justify-center items-center py-12">
                <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
                <span class="ml-3 text-gray-600">Loading bookings...</span>
            </div>
        }>
            {move || {
            let filtered = filtered_bookings.get();
            if filtered.is_empty() {
                view! { <EmptyBookingsState /> }.into_view()
            } else {
                view! {
                    <div class="space-y-4">
                        <For
                            each=move || filtered_bookings.get()
                            key=|booking| booking.booking_id.clone()
                            children=|booking| view! { <BookingCard booking=booking /> }
                        />
                    </div>
                }.into_view()
            }
        }}
    </Suspense>
    }
}

#[component]
fn BookingCard(booking: MyBookingItem) -> impl IntoView {
    let format_date = |date: DateTime<Utc>| date.format("%d %b %Y").to_string();

    let status_color = match booking.status {
        BookingStatus::Upcoming => "text-green-600 bg-green-50",
        BookingStatus::Completed => "text-blue-600 bg-blue-50",
        BookingStatus::Cancelled => "text-red-600 bg-red-50",
    };

    view! {
        <div class="bg-white border border-gray-200 rounded-lg overflow-hidden hover:shadow-md transition-shadow">
            <div class="flex flex-col md:flex-row">
                // <!-- Hotel Image -->
                <div class="md:w-48 h-48 md:h-auto flex-shrink-0">
                    <img
                        src=&booking.hotel_image_url
                        alt=format!("{} image", &booking.hotel_name)
                        class="w-full h-full object-cover"
                        loading="lazy"
                    />
                </div>

                // <!-- Booking Details -->
                <div class="flex-1 p-6">
                    <div class="flex flex-col md:flex-row md:items-start md:justify-between">
                        <div class="flex-1 mb-4 md:mb-0 md:mr-6">
                            <h3 class="text-xl font-semibold text-gray-900 mb-1">
                                {&booking.hotel_name}
                            </h3>
                            <p class="text-gray-600 mb-3">{&booking.hotel_location}</p>

                            <div class="flex items-center gap-4 text-sm text-gray-600 mb-3">
                                <div class="flex items-center gap-1">
                                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
                                    </svg>
                                    <span>{format_date(booking.check_in_date)} - {format_date(booking.check_out_date)}</span>
                                </div>
                                <div class="flex items-center gap-1">
                                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
                                    </svg>
                                    <span>{booking.adults} adult</span>
                                </div>
                                <div class="flex items-center gap-1">
                                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4" />
                                    </svg>
                                    <span>{booking.rooms} room</span>
                                </div>
                            </div>

                            <p class="text-sm text-gray-500 mb-2">
                                Booking ID: <span class="font-mono">{&booking.booking_id}</span>
                            </p>
                        </div>

                        <div class="flex flex-col items-end gap-3">
                            <span class=format!("px-3 py-1 rounded-full text-sm font-medium {}", status_color)>
                                {booking.status.to_string()}
                            </span>

                            <div class="flex items-center gap-4">
                                <button class="text-blue-600 hover:text-blue-800 font-medium text-sm flex items-center gap-1">
                                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                                    </svg>
                                    View Booking
                                </button>
                                <button class="text-blue-600 hover:text-blue-800 font-medium text-sm flex items-center gap-1">
                                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8.684 13.342C8.886 12.938 9 12.482 9 12c0-.482-.114-.938-.316-1.342m0 2.684a3 3 0 110-2.684m0 2.684l6.632 3.316m-6.632-6l6.632-3.316m0 0a3 3 0 105.367-2.684 3 3 0 00-5.367 2.684zm0 9.316a3 3 0 105.367 2.684 3 3 0 00-5.367-2.684z" />
                                    </svg>
                                    Share
                                </button>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn EmptyBookingsState() -> impl IntoView {
    view! {
        <div class="text-center py-12">
            <div class="mb-6">
                <img
                    src="/img/no-booking-found.svg"
                    alt="No bookings found"
                    class="mx-auto h-48 w-48"
                />
            </div>
            <h3 class="text-lg font-medium text-gray-900 mb-2">No bookings yet</h3>
            <p class="text-gray-500 mb-6">When you book a hotel, it will appear here.</p>
            <a
                href="/"
                class="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-blue-600 hover:bg-blue-700 transition-colors"
            >
                Start Booking
            </a>
        </div>
    }
}
