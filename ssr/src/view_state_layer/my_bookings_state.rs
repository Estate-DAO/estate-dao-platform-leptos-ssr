use crate::canister::backend;
use crate::log;
use crate::view_state_layer::GlobalStateForLeptos;
use chrono::{DateTime, Utc};
use leptos::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BookingStatus {
    Upcoming,
    Completed,
    Cancelled,
}

impl std::fmt::Display for BookingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BookingStatus::Upcoming => write!(f, "Upcoming"),
            BookingStatus::Completed => write!(f, "Completed"),
            BookingStatus::Cancelled => write!(f, "Cancelled"),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum BookingTab {
    #[default]
    Upcoming,
    Completed,
    Cancelled,
}

impl From<backend::Booking> for MyBookingItem {
    fn from(value: backend::Booking) -> Self {
        // Extract booking reference for ID
        let booking_id = format!(
            "{}-{}",
            value.booking_id.app_reference, value.booking_id.email
        );

        // Extract hotel details
        let hotel_details = &value.user_selected_hotel_room_details.hotel_details;
        let hotel_name = hotel_details.hotel_name.clone();
        let hotel_code = hotel_details.hotel_code.clone();
        let hotel_location = hotel_details.hotel_location.clone();
        let hotel_image_url = hotel_details.hotel_image.clone();

        // Extract date range and convert to DateTime<Utc>
        let date_range = &value.user_selected_hotel_room_details.date_range;
        let check_in_date = DateTime::from_timestamp(
            chrono::NaiveDate::from_ymd_opt(
                date_range.start.0 as i32,
                date_range.start.1,
                date_range.start.2,
            )
            .unwrap_or_default()
            .and_hms_opt(0, 0, 0)
            .unwrap_or_default()
            .and_utc()
            .timestamp(),
            0,
        )
        .unwrap_or_default();

        let check_out_date = DateTime::from_timestamp(
            chrono::NaiveDate::from_ymd_opt(
                date_range.end.0 as i32,
                date_range.end.1,
                date_range.end.2,
            )
            .unwrap_or_default()
            .and_hms_opt(0, 0, 0)
            .unwrap_or_default()
            .and_utc()
            .timestamp(),
            0,
        )
        .unwrap_or_default();

        // Extract guest counts
        let adults = value.guests.adults.len() as u32;
        let rooms = value.user_selected_hotel_room_details.room_details.len() as u32;

        // Determine booking status based on booking confirmation and dates
        let status = match &value.book_room_status {
            Some(response) => match response.commit_booking.resolved_booking_status {
                backend::ResolvedBookingStatus::BookingConfirmed => {
                    // For confirmed bookings, check if they're upcoming or completed based on check-out date
                    let now = Utc::now();
                    if check_out_date > now {
                        BookingStatus::Upcoming
                    } else {
                        BookingStatus::Completed
                    }
                }
                backend::ResolvedBookingStatus::BookingOnHold => BookingStatus::Upcoming,
                backend::ResolvedBookingStatus::BookingFailed => BookingStatus::Cancelled,
                backend::ResolvedBookingStatus::BookingCancelled => BookingStatus::Cancelled,
                backend::ResolvedBookingStatus::Unknown => BookingStatus::Upcoming,
            },
            None => BookingStatus::Upcoming,
        };

        // Extract payment details
        let total_amount = Some(
            value
                .user_selected_hotel_room_details
                .requested_payment_amount,
        );
        let currency = Some(
            value
                .payment_details
                .payment_api_response
                .pay_currency
                .clone(),
        );

        // Detect if this is a test booking by checking:
        // 1. Confirmation number is "test"
        // 2. Payment ID starts with "cs_test_"
        let is_test = match &value.book_room_status {
            Some(book_status) => book_status.commit_booking.confirmation_no == "test",
            None => false,
        } || value
            .payment_details
            .payment_api_response
            .payment_id_v2
            .starts_with("cs_test_");

        if is_test {
            log!(
                "[MyBookingItem] Detected test booking - app_reference: {}, confirmation_no: {:?}, payment_id: {}",
                value.booking_id.app_reference,
                value.book_room_status.as_ref().map(|s| &s.commit_booking.confirmation_no),
                value.payment_details.payment_api_response.payment_id_v2
            );
        }

        Self {
            booking_id,
            hotel_name,
            hotel_code,
            hotel_location,
            hotel_image_url,
            check_in_date,
            check_out_date,
            adults,
            rooms,
            status,
            total_amount,
            currency,
            is_test,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyBookingItem {
    pub booking_id: String,
    pub hotel_name: String,
    pub hotel_location: String,
    pub hotel_code: String,
    pub hotel_image_url: String,
    pub check_in_date: DateTime<Utc>,
    pub check_out_date: DateTime<Utc>,
    pub adults: u32,
    pub rooms: u32,
    pub status: BookingStatus,
    pub total_amount: Option<f64>,
    pub currency: Option<String>,
    pub is_test: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct MyBookingsState {
    pub bookings: RwSignal<Vec<MyBookingItem>>,
    pub current_tab: RwSignal<BookingTab>,
    pub is_loading: RwSignal<bool>,
    pub load_error: RwSignal<Option<String>>,
    pub filtered_bookings: Signal<Vec<MyBookingItem>>,
}

impl GlobalStateForLeptos for MyBookingsState {}

impl MyBookingsState {
    pub fn new() -> Self {
        let bookings = RwSignal::new(Vec::<MyBookingItem>::new());
        let current_tab = RwSignal::new(BookingTab::Upcoming);
        let is_loading = RwSignal::new(false);
        let load_error = RwSignal::new(None);

        let filtered_bookings = Signal::derive(move || {
            let all_bookings = bookings.get();
            let active_tab = current_tab.get();

            all_bookings
                .into_iter()
                .filter(|booking| match active_tab {
                    BookingTab::Upcoming => booking.status == BookingStatus::Upcoming,
                    BookingTab::Completed => booking.status == BookingStatus::Completed,
                    BookingTab::Cancelled => booking.status == BookingStatus::Cancelled,
                })
                .collect()
        });

        Self {
            bookings,
            current_tab,
            is_loading,
            load_error,
            filtered_bookings,
        }
    }

    pub fn from_leptos_context() -> Self {
        log!("[MyBookingsState] Getting state from leptos context");
        let state = Self::get();
        log!("[MyBookingsState] State retrieved from context");
        state
    }

    pub fn set_tab(&self, tab: BookingTab) {
        self.current_tab.set(tab);
    }

    pub fn set_bookings(&self, bookings: Vec<MyBookingItem>) {
        log!("[MyBookingsState] Setting {} bookings", bookings.len());
        self.bookings.set(bookings);
        log!("[MyBookingsState] Bookings set successfully");
    }

    pub fn set_loading(&self, is_loading: bool) {
        log!("[MyBookingsState] Setting loading state to: {}", is_loading);
        self.is_loading.set(is_loading);
    }

    pub fn set_error(&self, error: Option<String>) {
        self.load_error.set(error);
    }

    pub fn get_bookings_for_current_tab(&self) -> Vec<MyBookingItem> {
        self.filtered_bookings.get()
    }

    pub fn get_tab_count(&self, tab: BookingTab) -> usize {
        let all_bookings = self.bookings.get();
        all_bookings
            .iter()
            .filter(|booking| match tab {
                BookingTab::Upcoming => booking.status == BookingStatus::Upcoming,
                BookingTab::Completed => booking.status == BookingStatus::Completed,
                BookingTab::Cancelled => booking.status == BookingStatus::Cancelled,
            })
            .count()
    }

    // Create dummy data for development/testing
    // pub fn create_dummy_bookings() -> Vec<MyBookingItem> {
    //     log!("[MyBookingsState] Creating dummy bookings");
    //     use chrono::Duration;

    //     let now = Utc::now();

    //     let dummy_data = vec![
    //         MyBookingItem {
    //             booking_id: "FROWD3".to_string(),
    //             hotel_name: "Lakeside Motel Warefront".to_string(),
    //             hotel_location: "South Goa, India".to_string(),
    //             hotel_image_url: "/img/hotel-lakeside.jpg".to_string(),
    //             check_in_date: now + Duration::days(30),
    //             check_out_date: now + Duration::days(32),
    //             adults: 1,
    //             rooms: 1,
    //             status: BookingStatus::Upcoming,
    //             total_amount: Some(150.0),
    //             currency: Some("USD".to_string()),
    //         },
    //         MyBookingItem {
    //             booking_id: "FROWD2".to_string(),
    //             hotel_name: "Lakeside Motel Warefront".to_string(),
    //             hotel_location: "South Goa, India".to_string(),
    //             hotel_image_url: "/img/hotel-lakeside.jpg".to_string(),
    //             check_in_date: now - Duration::days(30),
    //             check_out_date: now - Duration::days(28),
    //             adults: 1,
    //             rooms: 1,
    //             status: BookingStatus::Completed,
    //             total_amount: Some(150.0),
    //             currency: Some("USD".to_string()),
    //         },
    //         MyBookingItem {
    //             booking_id: "FROWD1".to_string(),
    //             hotel_name: "Lakeside Motel Warefront".to_string(),
    //             hotel_location: "South Goa, India".to_string(),
    //             hotel_image_url: "/img/hotel-lakeside.jpg".to_string(),
    //             check_in_date: now - Duration::days(10),
    //             check_out_date: now - Duration::days(8),
    //             adults: 1,
    //             rooms: 1,
    //             status: BookingStatus::Cancelled,
    //             total_amount: Some(150.0),
    //             currency: Some("USD".to_string()),
    //         },
    //     ];

    //     log!("[MyBookingsState] Created {} dummy bookings", dummy_data.len());
    //     dummy_data
    // }
}
