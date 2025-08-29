use crate::{
    canister::backend::{self, Booking, BookingId, HotelRoomDetails},
    domain::{
        DomainAdultDetail, DomainChildDetail, DomainDestination, DomainDetailedPrice,
        DomainHotelDetails, DomainRoomData, DomainSelectedDateRange, DomainUserDetails,
    },
    utils::app_reference::BookingId as AppReferenceBookingId,
};

/// Helper functions to convert domain models to backend models and vice versa
/// These functions provide a clean API for the application to interact with the backend

pub struct BackendIntegrationHelper;

impl BackendIntegrationHelper {
    /// Convert domain booking data to backend Booking struct
    pub fn create_backend_booking(
        destination: Option<DomainDestination>,
        date_range: DomainSelectedDateRange,
        room_details: Vec<DomainRoomData>,
        hotel_details: DomainHotelDetails,
        guests: DomainUserDetails,
        booking_id: AppReferenceBookingId,
        payment_amount: f64,
        payment_currency: String,
    ) -> Booking {
        let hotel_room_details = HotelRoomDetails::from_domain_parts(
            destination,
            date_range,
            room_details,
            hotel_details,
            payment_amount,
        );

        let backend_booking_id: BookingId = booking_id.into();

        Booking::from_domain_parts(
            hotel_room_details,
            guests,
            backend_booking_id,
            payment_amount,
            payment_currency,
        )
    }

    /// Extract domain data from backend Booking
    pub fn extract_domain_data_from_booking(
        booking: &Booking,
    ) -> (
        Option<DomainDestination>,
        DomainSelectedDateRange,
        DomainUserDetails,
    ) {
        let destination = booking
            .user_selected_hotel_room_details
            .destination
            .clone()
            .map(|d| d.into());

        let date_range = booking
            .user_selected_hotel_room_details
            .date_range
            .clone()
            .into();
        let guests = booking.guests.clone().into();

        (destination, date_range, guests)
    }

    /// Get booking summary information for display
    pub fn get_booking_display_info(booking: &Booking) -> BookingDisplayInfo {
        BookingDisplayInfo {
            booking_reference: booking.get_booking_ref(),
            booking_ref_no: booking.get_booking_ref_no(),
            confirmation_no: booking.get_confirmation_no(),
            is_confirmed: booking.is_booking_confirmed(),
            booking_status_message: booking.get_booking_status_message(),
            hotel_name: booking.get_hotel_name(),
            hotel_location: booking.get_hotel_location(),
            check_in_date: booking.get_check_in_date(),
            check_out_date: booking.get_check_out_date(),
            check_in_date_formatted: booking.get_check_in_date_formatted(),
            check_out_date_formatted: booking.get_check_out_date_formatted(),
            number_of_nights: booking.get_number_of_nights(),
            user_email: booking.get_user_email(),
            user_name: booking.get_user_name(),
            user_phone: booking.get_user_phone(),
            number_of_adults: booking.get_number_of_adults(),
            number_of_children: booking.get_number_of_children(),
            amount_paid: booking.get_amount_paid(),
        }
    }

    /// Update hotel room details with block room ID and token
    pub fn update_hotel_room_details_with_block_info(
        hotel_room_details: &mut HotelRoomDetails,
        block_room_id: String,
        hotel_token: String,
    ) {
        hotel_room_details.hotel_details.block_room_id = block_room_id;
        hotel_room_details.hotel_details.hotel_token = hotel_token;
    }
}

/// Simplified booking information for display purposes
#[derive(Debug, Clone)]
pub struct BookingDisplayInfo {
    pub booking_reference: String,
    pub booking_ref_no: String,
    pub confirmation_no: String,
    pub is_confirmed: bool,
    pub booking_status_message: String,
    pub hotel_name: String,
    pub hotel_location: String,
    pub check_in_date: String,
    pub check_out_date: String,
    pub check_in_date_formatted: String,
    pub check_out_date_formatted: String,
    pub number_of_nights: u32,
    pub user_email: String,
    pub user_name: String,
    pub user_phone: String,
    pub number_of_adults: usize,
    pub number_of_children: usize,
    pub amount_paid: f64,
}
