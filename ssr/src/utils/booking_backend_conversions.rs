use crate::{
    canister::backend::{self, Booking, HotelRoomDetails},
    domain::{
        BookingError, DomainAdultDetail, DomainChildDetail, DomainDestination, DomainHotelDetails,
        DomainRoomData, DomainSelectedDateRange, DomainUserDetails, ServiceBookingData,
    },
    utils::app_reference::BookingId as AppReferenceBookingId,
};

/// Trait for converting Domain types to Backend types
pub trait DomainToBackend<T> {
    fn to_backend(&self) -> Result<T, BookingError>;
}

/// Trait for converting Backend types to Domain types
pub trait BackendToDomain<T> {
    fn to_domain(&self) -> Result<T, BookingError>;
}

/// Conversion utilities for Domain ↔ Backend transformations
pub struct BookingBackendConversions;

impl BookingBackendConversions {
    /// Convert domain booking data to backend Booking struct
    /// This is the main entry point for creating backend bookings
    pub fn create_backend_booking(
        destination: Option<DomainDestination>,
        date_range: DomainSelectedDateRange,
        room_details: Vec<DomainRoomData>,
        hotel_details: DomainHotelDetails,
        guests: DomainUserDetails,
        booking_id: AppReferenceBookingId,
        payment_amount: f64,
        payment_currency: String,
        block_room_id: Option<String>,
        hotel_token: Option<String>,
    ) -> Result<Booking, BookingError> {
        // Create hotel room details from domain parts
        let mut hotel_room_details = HotelRoomDetails::from_domain_parts(
            destination,
            date_range,
            room_details,
            hotel_details,
            payment_amount,
        );

        // Update with block room information if available
        if let Some(block_id) = block_room_id {
            hotel_room_details.hotel_details.block_room_id = block_id;
        }
        if let Some(token) = hotel_token {
            hotel_room_details.hotel_details.hotel_token = token;
        }

        // Convert BookingId to backend format
        let backend_booking_id: backend::BookingId = booking_id.into();

        // Create booking from domain parts
        let booking = Booking::from_domain_parts(
            hotel_room_details,
            guests,
            backend_booking_id,
            payment_amount,
            payment_currency,
        );

        Ok(booking)
    }

    /// Extract domain data from backend Booking
    pub fn extract_domain_data_from_booking(
        booking: &Booking,
    ) -> Result<
        (
            Option<DomainDestination>,
            DomainSelectedDateRange,
            DomainUserDetails,
        ),
        BookingError,
    > {
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

        Ok((destination, date_range, guests))
    }

    /// Convert ServiceBookingData to backend Booking
    /// This requires additional UI context data
    pub fn service_booking_to_backend(
        service_booking: ServiceBookingData,
        destination: Option<DomainDestination>,
        date_range: DomainSelectedDateRange,
        room_details: Vec<DomainRoomData>,
        hotel_details: DomainHotelDetails,
        guests: DomainUserDetails,
    ) -> Result<Booking, BookingError> {
        let booking_id =
            AppReferenceBookingId::new(service_booking.email, service_booking.app_reference);

        Self::create_backend_booking(
            destination,
            date_range,
            room_details,
            hotel_details,
            guests,
            booking_id,
            service_booking.payment_amount,
            service_booking.payment_currency,
            service_booking.block_room_id,
            None, // hotel_token can be added later if needed
        )
    }

    /// Extract ServiceBookingData from backend Booking
    pub fn backend_to_service_booking(
        booking: &Booking,
    ) -> Result<ServiceBookingData, BookingError> {
        // Extract booking ID information
        let booking_ref = booking.get_booking_ref();
        let email = booking.get_user_email();
        let payment_amount = booking.get_amount_paid();

        // Determine status based on booking state
        let status = if !booking
            .user_selected_hotel_room_details
            .hotel_details
            .block_room_id
            .is_empty()
        {
            crate::domain::ServiceBookingStatus::RoomBlocked
        } else {
            crate::domain::ServiceBookingStatus::Draft
        };

        Ok(ServiceBookingData {
            booking_id: booking_ref.clone(),
            email,
            app_reference: booking_ref,
            block_room_id: if booking
                .user_selected_hotel_room_details
                .hotel_details
                .block_room_id
                .is_empty()
            {
                None
            } else {
                Some(
                    booking
                        .user_selected_hotel_room_details
                        .hotel_details
                        .block_room_id
                        .clone(),
                )
            },
            payment_amount,
            payment_currency: "USD".to_string(), // Default currency
            status,
        })
    }

    /// Update backend booking with block room information
    pub fn update_booking_with_block_info(
        booking: &mut Booking,
        block_room_id: String,
        hotel_token: String,
    ) {
        booking
            .user_selected_hotel_room_details
            .hotel_details
            .block_room_id = block_room_id;
        booking
            .user_selected_hotel_room_details
            .hotel_details
            .hotel_token = hotel_token;
    }

    /// Validate backend booking data
    pub fn validate_backend_booking(booking: &Booking) -> Result<(), BookingError> {
        if booking.get_user_email().is_empty() {
            return Err(BookingError::ValidationError(
                "User email is required".to_string(),
            ));
        }

        if booking.get_booking_ref().is_empty() {
            return Err(BookingError::ValidationError(
                "Booking reference is required".to_string(),
            ));
        }

        if booking.get_hotel_name().is_empty() {
            return Err(BookingError::ValidationError(
                "Hotel name is required".to_string(),
            ));
        }

        // if booking.get_amount_paid() <= 0.0 {
        //     return Err(BookingError::ValidationError(
        //         "Payment amount must be greater than 0".to_string(),
        //     ));
        // }

        Ok(())
    }

    /// Create a minimal backend booking for testing purposes
    pub fn create_minimal_backend_booking(
        email: String,
        app_reference: String,
        hotel_name: String,
        amount: f64,
    ) -> Result<Booking, BookingError> {
        let booking_id = AppReferenceBookingId::new(email, app_reference);

        // Create minimal domain data
        let destination = None;
        let date_range = DomainSelectedDateRange {
            start: (2024, 1, 1),
            end: (2024, 1, 2),
        };
        let room_details = vec![DomainRoomData {
            room_name: "Standard Room".to_string(),
            room_unique_id: "standard_room_1".to_string(),
            rate_key: "standard_rate".to_string(),
            offer_id: "offer_1".to_string(),
            mapped_room_id: 1,
        }];
        let hotel_details = DomainHotelDetails {
            checkin: "2024-01-01".to_string(),
            checkout: "2024-01-02".to_string(),
            hotel_name,
            hotel_code: "HOTEL_001".to_string(),
            star_rating: 4,
            description: "Test hotel".to_string(),
            hotel_facilities: vec![],
            address: "Test Address".to_string(),
            images: vec![],
            all_rooms: vec![],
            amenities: vec![],
            search_criteria: None,
            search_info: None,
        };
        let guests = DomainUserDetails {
            adults: vec![DomainAdultDetail {
                email: Some(booking_id.email.clone()),
                first_name: "Test".to_string(),
                last_name: Some("User".to_string()),
                phone: Some("+1234567890".to_string()),
            }],
            children: vec![],
        };

        Self::create_backend_booking(
            destination,
            date_range,
            room_details,
            hotel_details,
            guests,
            booking_id,
            amount,
            "USD".to_string(),
            None,
            None,
        )
    }
}

// Implement conversion traits for common types

impl DomainToBackend<backend::UserDetails> for DomainUserDetails {
    fn to_backend(&self) -> Result<backend::UserDetails, BookingError> {
        Ok(self.clone().into())
    }
}

impl BackendToDomain<DomainUserDetails> for backend::UserDetails {
    fn to_domain(&self) -> Result<DomainUserDetails, BookingError> {
        Ok(self.clone().into())
    }
}

impl DomainToBackend<backend::SelectedDateRange> for DomainSelectedDateRange {
    fn to_backend(&self) -> Result<backend::SelectedDateRange, BookingError> {
        Ok(self.clone().into())
    }
}

impl BackendToDomain<DomainSelectedDateRange> for backend::SelectedDateRange {
    fn to_domain(&self) -> Result<DomainSelectedDateRange, BookingError> {
        Ok(self.clone().into())
    }
}

// Helper functions for serialization

impl BookingBackendConversions {
    /// Serialize booking to JSON string for backend storage
    pub fn serialize_booking_for_backend(booking: &Booking) -> Result<String, BookingError> {
        serde_json::to_string(booking).map_err(|e| BookingError::SerializationError(e.to_string()))
    }

    /// Deserialize booking from JSON string from backend
    pub fn deserialize_booking_from_backend(json: &str) -> Result<Booking, BookingError> {
        serde_json::from_str(json).map_err(|e| BookingError::SerializationError(e.to_string()))
    }
}
