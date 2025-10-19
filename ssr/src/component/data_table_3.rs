use crate::{
    api::canister::get_all_bookings::get_all_bookings_backend,
    canister::backend::{self, BookingSummary},
};
use leptos::prelude::*;

#[derive(Clone, Default, Debug)]
pub struct DataTableCtx {
    // Filters
    pub booking_id_filter: RwSignal<String>,
    pub destination_filter: RwSignal<String>,
    pub hotel_name_filter: RwSignal<String>,
    pub booking_dates_filter: RwSignal<String>,
    pub user_email_filter: RwSignal<String>,
    pub booking_status_filter: RwSignal<String>,
    pub payment_status_filter: RwSignal<String>,
    pub payment_id_filter: RwSignal<String>,

    // Sorting
    pub sort_column: RwSignal<Option<String>>,
    pub sort_direction: RwSignal<SortDirection>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SortDirection {
    Asc,
    Desc,
}

impl Default for SortDirection {
    fn default() -> Self {
        SortDirection::Asc
    }
}

impl DataTableCtx {
    // Generic filter setter to reduce duplication
    fn set_filter(filter_signal: RwSignal<String>, value: String) {
        filter_signal.set(value);
    }

    // Filter setters
    pub fn set_booking_id_filter(value: String) {
        let this: Self = expect_context();
        Self::set_filter(this.booking_id_filter, value);
    }

    pub fn set_destination_filter(value: String) {
        let this: Self = expect_context();
        Self::set_filter(this.destination_filter, value);
    }

    pub fn set_hotel_name_filter(value: String) {
        let this: Self = expect_context();
        Self::set_filter(this.hotel_name_filter, value);
    }

    pub fn set_booking_dates_filter(value: String) {
        let this: Self = expect_context();
        Self::set_filter(this.booking_dates_filter, value);
    }

    pub fn set_user_email_filter(value: String) {
        let this: Self = expect_context();
        Self::set_filter(this.user_email_filter, value);
    }

    pub fn set_booking_status_filter(value: String) {
        let this: Self = expect_context();
        Self::set_filter(this.booking_status_filter, value);
    }

    pub fn set_payment_status_filter(value: String) {
        let this: Self = expect_context();
        Self::set_filter(this.payment_status_filter, value);
    }

    pub fn set_payment_id_filter(value: String) {
        let this: Self = expect_context();
        Self::set_filter(this.payment_id_filter, value);
    }

    // Sorting methods
    pub fn set_sort_column(column: String) {
        let this: Self = expect_context();
        let current_column = this.sort_column.get_untracked();

        if current_column == Some(column.clone()) {
            // Toggle direction if same column
            let current_direction = this.sort_direction.get_untracked();
            this.sort_direction.set(match current_direction {
                SortDirection::Asc => SortDirection::Desc,
                SortDirection::Desc => SortDirection::Asc,
            });
        } else {
            // New column, default to ascending
            this.sort_column.set(Some(column));
            this.sort_direction.set(SortDirection::Asc);
        }
    }

    pub fn clear_filters() {
        let this: Self = expect_context();
        this.booking_id_filter.set(String::new());
        this.destination_filter.set(String::new());
        this.hotel_name_filter.set(String::new());
        this.booking_dates_filter.set(String::new());
        this.user_email_filter.set(String::new());
        this.booking_status_filter.set(String::new());
        this.payment_status_filter.set(String::new());
        this.payment_id_filter.set(String::new());
    }

    // Filter and sort bookings
    pub fn filter_and_sort_bookings(bookings: Vec<BookingSummary>) -> Vec<BookingSummary> {
        let this: Self = expect_context();

        let booking_id_filter = this.booking_id_filter.get();
        let destination_filter = this.destination_filter.get();
        let hotel_name_filter = this.hotel_name_filter.get();
        let booking_dates_filter = this.booking_dates_filter.get();
        let user_email_filter = this.user_email_filter.get();
        let booking_status_filter = this.booking_status_filter.get();
        let payment_status_filter = this.payment_status_filter.get();
        let payment_id_filter = this.payment_id_filter.get();
        let sort_column = this.sort_column.get();
        let sort_direction = this.sort_direction.get();

        let mut filtered_bookings: Vec<BookingSummary> = bookings
            .into_iter()
            .filter(|booking| {
                let booking_id_str = format!("{:?}", booking.booking_id);

                (booking_id_filter.is_empty()
                    || booking_id_str
                        .to_lowercase()
                        .contains(&booking_id_filter.to_lowercase()))
                    && (destination_filter.is_empty()
                        || booking
                            .destination
                            .to_lowercase()
                            .contains(&destination_filter.to_lowercase()))
                    && (hotel_name_filter.is_empty()
                        || booking
                            .hotel_name
                            .to_lowercase()
                            .contains(&hotel_name_filter.to_lowercase()))
                    && (booking_dates_filter.is_empty()
                        || booking
                            .booking_dates
                            .to_lowercase()
                            .contains(&booking_dates_filter.to_lowercase()))
                    && (user_email_filter.is_empty()
                        || booking
                            .user_email
                            .to_lowercase()
                            .contains(&user_email_filter.to_lowercase()))
                    && (booking_status_filter.is_empty()
                        || booking
                            .booking_status
                            .to_lowercase()
                            .contains(&booking_status_filter.to_lowercase()))
                    && (payment_status_filter.is_empty()
                        || booking
                            .payment_status
                            .to_lowercase()
                            .contains(&payment_status_filter.to_lowercase()))
                    && (payment_id_filter.is_empty()
                        || booking
                            .payment_id
                            .to_lowercase()
                            .contains(&payment_id_filter.to_lowercase()))
            })
            .collect();

        // Sort bookings
        if let Some(column) = sort_column {
            filtered_bookings.sort_by(|a, b| {
                let comparison = match column.as_str() {
                    "booking_id" => {
                        let a_id = format!("{:?}", a.booking_id);
                        let b_id = format!("{:?}", b.booking_id);
                        a_id.cmp(&b_id)
                    }
                    "destination" => a.destination.cmp(&b.destination),
                    "hotel_name" => a.hotel_name.cmp(&b.hotel_name),
                    "booking_dates" => a.booking_dates.cmp(&b.booking_dates),
                    "user_email" => a.user_email.cmp(&b.user_email),
                    "nights" => a.nights.cmp(&b.nights),
                    "booking_status" => a.booking_status.cmp(&b.booking_status),
                    "payment_status" => a.payment_status.cmp(&b.payment_status),
                    "payment_id" => a.payment_id.cmp(&b.payment_id),
                    _ => std::cmp::Ordering::Equal,
                };

                match sort_direction {
                    SortDirection::Asc => comparison,
                    SortDirection::Desc => comparison.reverse(),
                }
            });
        }

        filtered_bookings
    }
}

// Helper component to reduce duplication in sort indicators
#[component]
fn SortIndicator(column: &'static str, label: &'static str) -> impl IntoView {
    let ctx: DataTableCtx = expect_context();

    view! {
        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider cursor-pointer hover:bg-gray-100"
            on:click=move |_| DataTableCtx::set_sort_column(column.to_string())>
            <div class="flex items-center space-x-1">
                <span>{label}</span>
                {move || {
                    let sort_column = ctx.sort_column.get();
                    let sort_direction = ctx.sort_direction.get();
                    if sort_column == Some(column.to_string()) {
                        match sort_direction {
                            SortDirection::Asc => view! { <span>"↑"</span> }.into_any(),
                            SortDirection::Desc => view! { <span>"↓"</span> }.into_any(),
                        }
                    } else {
                        view! { <span></span> }.into_any()
                    }
                }}
            </div>
        </th>
    }
}

// Helper component for filter inputs
#[component]
fn FilterInput(
    placeholder: &'static str,
    value_signal: RwSignal<String>,
    on_input: impl Fn(String) + 'static + Clone,
) -> impl IntoView {
    let column_name = placeholder.replace("Filter ", "").replace("...", "");

    view! {
        <div class="relative group">
            <input
                type="text"
                placeholder=placeholder
                class="px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                prop:value=move || value_signal.get()
                on:input=move |ev| on_input(event_target_value(&ev))
            />
            <div class="absolute left-1/2 -translate-x-1/2 -top-2 transform -translate-y-full
                        px-2 py-1 bg-gray-800 text-white text-sm rounded-md
                        opacity-0 group-hover:opacity-100 transition-opacity duration-200
                        pointer-events-none whitespace-nowrap">
                {column_name}
            </div>
        </div>
    }
}

// Helper component for status badges
#[component]
fn StatusBadge(status: String, status_type: &'static str) -> impl IntoView {
    let status_clone = status.clone();

    view! {
        <span class={move || {
            let base_classes = "inline-flex px-2 py-1 text-xs font-semibold rounded-full";
            match status_type {
                "booking" => match status.as_str() {
                    "CONFIRMED" => format!("{} bg-green-100 text-green-800", base_classes),
                    "BookOnHold" => format!("{} bg-yellow-100 text-yellow-800", base_classes),
                    "BookFailed" => format!("{} bg-red-100 text-red-800", base_classes),
                    _ => format!("{} bg-gray-100 text-gray-800", base_classes),
                },
                "payment" => match status.as_str() {
                    "Paid" => format!("{} bg-green-100 text-green-800", base_classes),
                    "Pending" => format!("{} bg-yellow-100 text-yellow-800", base_classes),
                    "Failed" => format!("{} bg-red-100 text-red-800", base_classes),
                    _ => format!("{} bg-gray-100 text-gray-800", base_classes),
                },
                _ => format!("{} bg-gray-100 text-gray-800", base_classes),
            }
        }}>
            {status_clone}
        </span>
    }
}

#[component]
pub fn DataTableV3() -> impl IntoView {
    let ctx: DataTableCtx = expect_context();

    // Load the data from server function get_all_bookings
    let all_user_bookings = Resource::new(
        move || (),
        move |_| async move { get_all_bookings_backend().await.unwrap_or(vec![]) },
    );

    // Use create_local_resource that combines data fetching with filtering/sorting
    let filtered_sorted_bookings = Resource::new(
        move || {
            (
                ctx.booking_id_filter.get(),
                ctx.destination_filter.get(),
                ctx.hotel_name_filter.get(),
                ctx.booking_dates_filter.get(),
                ctx.user_email_filter.get(),
                ctx.booking_status_filter.get(),
                ctx.payment_status_filter.get(),
                ctx.payment_id_filter.get(),
                ctx.sort_column.get(),
                ctx.sort_direction.get(),
            )
        },
        move |filters| async move {
            let bookings = all_user_bookings.get().unwrap_or(vec![]);
            DataTableCtx::filter_and_sort_bookings(bookings)
        },
    );

    // this create effect method also works.
    // // Create a signal that filters and sorts the bookings
    // let filtered_sorted_bookings = RwSignal::new(vec![]);

    // Effect::new(move |_| {
    //     all_user_bookings.get().map(|bookings| {
    //         filtered_sorted_bookings.set(DataTableCtx::filter_and_sort_bookings(bookings));
    //     });
    // });

    // let filtered_sorted_bookings = LocalResource::new(move || all_user_bookings.get(), move |user_bookings_from_api| async move {
    //     DataTableCtx::filter_and_sort_bookings(user_bookings_from_api.unwrap_or(vec![]))
    // });

    view! {
        <div class="p-4 bg-white shadow rounded-lg">
            // Header with count and clear filters button
            <div class="flex justify-between items-center mb-4">
                <Suspense fallback=move || view! { <h1 class="text-2xl font-bold">"Loading bookings..."</h1> }>
                    {move || {
                        let bookings_len = all_user_bookings.get().map(|bookings| bookings.len()).unwrap_or(0);
                        view! {
                            <h1 class="text-2xl font-bold">
                                {format!("Bookings: {}", bookings_len)}
                            </h1>
                        }
                    }}
                </Suspense>

                <button
                    class="px-4 py-2 bg-gray-500 text-white rounded hover:bg-gray-600 transition-colors"
                    on:click=move |_| DataTableCtx::clear_filters()
                >
                    "Clear Filters"
                </button>
            </div>

            // Filter inputs
            <div class="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-8 gap-6 mb-6">
                <FilterInput
                    placeholder="Filter Booking ID..."
                    value_signal=ctx.booking_id_filter
                    on_input=DataTableCtx::set_booking_id_filter
                />

                <FilterInput
                    placeholder="Filter Destination..."
                    value_signal=ctx.destination_filter
                    on_input=DataTableCtx::set_destination_filter
                />

                <FilterInput
                    placeholder="Filter Hotel Name..."
                    value_signal=ctx.hotel_name_filter
                    on_input=DataTableCtx::set_hotel_name_filter
                />

                <FilterInput
                    placeholder="Filter Booking Dates..."
                    value_signal=ctx.booking_dates_filter
                    on_input=DataTableCtx::set_booking_dates_filter
                />

                <FilterInput
                    placeholder="Filter User Email..."
                    value_signal=ctx.user_email_filter
                    on_input=DataTableCtx::set_user_email_filter
                />

                <FilterInput
                    placeholder="Filter Booking Status..."
                    value_signal=ctx.booking_status_filter
                    on_input=DataTableCtx::set_booking_status_filter
                />

                <FilterInput
                    placeholder="Filter Payment Status..."
                    value_signal=ctx.payment_status_filter
                    on_input=DataTableCtx::set_payment_status_filter
                />

                <FilterInput
                    placeholder="Filter Payment ID..."
                    value_signal=ctx.payment_id_filter
                    on_input=DataTableCtx::set_payment_id_filter
                />
            </div>

            <div class="overflow-x-auto">
                <table class="min-w-full divide-y divide-gray-200">
                    <thead class="bg-gray-50">
                        <tr>
                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"#"</th>
                            <SortIndicator column="booking_id" label="Booking ID" />
                            <SortIndicator column="destination" label="Destination" />
                            <SortIndicator column="hotel_name" label="Hotel Name" />
                            <SortIndicator column="booking_dates" label="Booking Dates" />
                            <SortIndicator column="user_email" label="User Email" />
                            <SortIndicator column="nights" label="Nights" />
                            <SortIndicator column="booking_status" label="Booking Status" />
                            <SortIndicator column="payment_status" label="Payment Status" />
                            <SortIndicator column="payment_id" label="Payment ID" />
                        </tr>
                    </thead>
                    <tbody class="bg-white divide-y divide-gray-200">
                        <Suspense fallback=move || view! {
                            <tr>
                                <td colspan="10" class="px-6 py-4 text-center text-sm text-gray-500">"Loading bookings..."</td>
                            </tr>
                        }>
                            {move || {
                                // let bookings = filtered_sorted_bookings.get();
                                filtered_sorted_bookings.get().map(|bookings| {
                                    if bookings.is_empty() {
                                    view! {
                                        <tr>
                                            <td colspan="10" class="px-6 py-4 text-center text-sm text-gray-500">"No bookings found matching the current filters."</td>
                                        </tr>
                                    }.into_any()
                                } else {
                                    bookings.into_iter().enumerate().map(|(i, booking)| {
                                        let booking_id_display = format!("{:?}", booking.booking_id);

                                        view! {
                                            <tr>
                                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                                    {i + 1}
                                                </td>
                                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                                    {booking_id_display}
                                                </td>
                                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                                    {booking.destination.to_string()}
                                                </td>
                                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                                    {booking.hotel_name.to_string()}
                                                </td>
                                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                                    {booking.booking_dates.to_string()}
                                                </td>
                                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                                    {booking.user_email.to_string()}
                                                </td>
                                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                                    {booking.nights}
                                                </td>
                                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                                    <StatusBadge status=booking.booking_status.clone() status_type="booking" />
                                                </td>
                                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                                    <StatusBadge status=booking.payment_status.clone() status_type="payment" />
                                                </td>
                                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                                    {booking.payment_id.to_string()}
                                                </td>
                                            </tr>
                                        }
                                    }).collect::<Vec<_>>().into_any()
                                }
                            })
                            }}
                        </Suspense>
                    </tbody>
                </table>
            </div>
        </div>
    }
}
