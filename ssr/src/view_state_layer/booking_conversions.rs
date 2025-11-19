use crate::domain::{
    BookingError, DomainAdultDetail, DomainBlockRoomRequest, DomainBookRoomRequest,
    DomainBookingContext, DomainBookingGuest, DomainBookingHolder, DomainChildDetail,
    DomainDestination, DomainHotelDetails, DomainHotelInfoCriteria, DomainHotelSearchCriteria,
    DomainPaymentInfo, DomainPaymentMethod, DomainRoomData, DomainRoomGuest,
    DomainRoomOccupancyForBooking, DomainSelectedDateRange, DomainSelectedRoomWithQuantity,
    DomainUserDetails, ServiceBookingData, ServiceBookingStatus,
};
use crate::utils::app_reference::BookingId;
use crate::view_state_layer::ui_block_room::{AdultDetail, ChildDetail, RoomSelectionSummary};
use crate::view_state_layer::ui_search_state::UISearchCtx;
use crate::view_state_layer::view_state::HotelInfoCtx;
use leptos::*;

/// Trait for converting UI state to Domain types
pub trait UIToDomain<T> {
    fn to_domain(&self) -> Result<T, BookingError>;
}

/// Trait for converting Domain types to UI state
pub trait DomainToUI<T> {
    fn to_ui(&self) -> Result<T, BookingError>;
}

/// Conversion utilities for booking-related data transformations
pub struct BookingConversions;

impl BookingConversions {
    /// Convert UI state to DomainBlockRoomRequest
    /// This function extracts data from Leptos contexts and converts to domain format
    pub fn ui_to_block_room_request() -> Result<DomainBlockRoomRequest, BookingError> {
        // Get required contexts
        let block_room_state =
            crate::view_state_layer::ui_block_room::BlockRoomUIState::from_leptos_context();
        let ui_search_ctx: UISearchCtx = expect_context();
        let hotel_info_ctx: HotelInfoCtx = expect_context();

        // let block_room_id = block_room_state.block_room_id.get_untracked();

        // Get form data
        let adults: Vec<AdultDetail> = block_room_state.adults.get_untracked();
        let total_adults = adults.len();
        let children = block_room_state.children.get_untracked();

        // Convert UI adults to domain adults
        let domain_adults: Vec<DomainAdultDetail> = adults
            .into_iter()
            .map(|adult| DomainAdultDetail {
                email: adult.email,
                first_name: adult.first_name,
                last_name: adult.last_name,
                phone: adult.phone,
            })
            .collect();

        // Convert UI children to domain children
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
        // let place_details = ui_search_ctx.place_details.get_untracked().ok_or_else(|| {
        //     BookingError::ValidationError("Place details is required".to_string())
        // })?;
        // let place = ui_search_ctx
        //     .place
        //     .get_untracked()
        //     .ok_or_else(|| BookingError::ValidationError("Place is required".to_string()))?;
        let date_range = ui_search_ctx.date_range.get_untracked();
        let guests = &ui_search_ctx.guests;
        let rooms_count = guests.rooms.get_untracked();
        let effective_adult_count = std::cmp::max(total_adults as u32, rooms_count);
        let children_count = guests.children.get_untracked();

        let room_guests = vec![DomainRoomGuest {
            no_of_adults: effective_adult_count,
            no_of_children: children_count,
            children_ages: if children_count > 0 {
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
            // destination_city_id: pla.city_id.parse().unwrap_or(0),
            // destination_city_name: destination.city.clone(),
            // destination_country_code: destination.country_code.clone(),
            // destination_country_name: destination.country_name.clone(),
            // destination_latitude: Some(place_details.location.latitude),
            // destination_longitude: Some(place_details.location.longitude),
            place_id: String::new(),
            // place_id: place.place_id.clone(),
            check_in_date: (date_range.start.0, date_range.start.1, date_range.start.2),
            check_out_date: (date_range.end.0, date_range.end.1, date_range.end.2),
            no_of_nights: date_range.no_of_nights(),
            no_of_rooms: guests.rooms.get_untracked(),
            room_guests,
            guest_nationality: "US".to_string(),
            pagination: None, // No pagination for booking conversions
                              // ..Default::default()
        };

        let hotel_code = hotel_info_ctx.hotel_code.get_untracked();
        let hotel_info_criteria = DomainHotelInfoCriteria {
            token: hotel_code.clone(), // Use hotel_code as token for LiteAPI (not block_room_id)
            hotel_ids: vec![hotel_code],
            search_criteria,
        };

        // Get room selection data
        let room_selection_summary = block_room_state.room_selection_summary.get_untracked();

        // Build selected rooms with quantities
        let selected_rooms: Vec<DomainSelectedRoomWithQuantity> = room_selection_summary
            .into_iter()
            .map(|room_summary| DomainSelectedRoomWithQuantity {
                room_data: room_summary.room_data.clone(),
                quantity: room_summary.quantity,
                price_per_night: room_summary.price_per_night,
            })
            .collect();

        // Backward compatibility: Use first room for providers that don't support multiple rooms
        let selected_room = if let Some(first_room) = selected_rooms.first() {
            first_room.room_data.clone()
        } else {
            return Err(BookingError::ValidationError(
                "At least one room must be selected".to_string(),
            ));
        };

        Ok(DomainBlockRoomRequest {
            hotel_info_criteria,
            user_details,
            selected_rooms,
            selected_room,
            total_guests: effective_adult_count + children_count,
            special_requests: None,
        })
    }

    /// Convert UI state to DomainBookRoomRequest
    /// This requires a block_id from a previous block_room call
    pub fn ui_to_book_room_request(
        block_id: String,
    ) -> Result<DomainBookRoomRequest, BookingError> {
        let block_room_state =
            crate::view_state_layer::ui_block_room::BlockRoomUIState::from_leptos_context();
        let ui_search_ctx: UISearchCtx = expect_context();

        let adults = block_room_state.adults.get_untracked();

        // Get primary adult as booking holder
        let primary_adult = adults.first().ok_or_else(|| {
            BookingError::ValidationError(
                "[UI VALIDATION ERROR] Primary adult is required".to_string(),
            )
        })?;

        let holder = DomainBookingHolder {
            first_name: primary_adult.first_name.clone(),
            last_name: primary_adult.last_name.clone().ok_or_else(|| {
                BookingError::ValidationError(
                    "[UI VALIDATION ERROR] Primary adult last name is required".to_string(),
                )
            })?,
            email: primary_adult.email.clone().ok_or_else(|| {
                BookingError::ValidationError(
                    "[UI VALIDATION ERROR] Primary adult email is required".to_string(),
                )
            })?,
            phone: primary_adult.phone.clone().ok_or_else(|| {
                BookingError::ValidationError(
                    "[UI VALIDATION ERROR] Primary adult phone is required".to_string(),
                )
            })?,
        };

        // Convert adults to guests (one PRIMARY CONTACT per room)
        // LiteAPI Rule: Need exactly one guest per room as the primary contact/room manager
        let guests_ctx = &ui_search_ctx.guests;
        let number_of_rooms = guests_ctx.rooms.get_untracked() as usize;
        let effective_adult_count =
            std::cmp::max(adults.len() as u32, guests_ctx.rooms.get_untracked());

        // Validation: Must have at least one adult per room
        if adults.len() < number_of_rooms {
            return Err(BookingError::ValidationError(format!(
                "[UI VALIDATION ERROR] Need at least {} adults for {} rooms, but only {} provided. Each room requires a primary contact.",
                number_of_rooms, number_of_rooms, adults.len()
            )));
        }

        // Take only the first N adults as primary contacts (ignore extras)
        // This follows LiteAPI rule: one primary contact per room, not per person
        let guests: Vec<DomainBookingGuest> = adults
            .into_iter()
            .take(number_of_rooms)  // 🔑 KEY FIX: Limit to room count, not adult count
            .enumerate()
            .map(|(index, adult)| -> Result<DomainBookingGuest, BookingError> {
                Ok(DomainBookingGuest {
                    occupancy_number: (index + 1) as u32,  // Room number (1, 2, 3...)
                    first_name: adult.first_name,
                    last_name: adult.last_name.ok_or_else(|| {
                        BookingError::ValidationError(format!(
                            "[UI VALIDATION ERROR] Primary contact for room {} must have a last name", index + 1
                        ))
                    })?,
                    email: adult.email.unwrap_or_default(),  // Will be handled by fallback
                    phone: adult.phone.unwrap_or_default(),  // Will be handled by fallback
                    remarks: None,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        // Build booking context
        let room_occupancies = vec![DomainRoomOccupancyForBooking {
            room_number: 1,
            adults: effective_adult_count,
            children: guests_ctx.children.get_untracked(),
            children_ages: guests_ctx
                .children_ages
                .get_untracked()
                .into_iter()
                .map(|age| age as u8)
                .collect(),
        }];

        let booking_context = DomainBookingContext {
            number_of_rooms: guests_ctx.rooms.get_untracked(),
            room_occupancies,
            total_guests: effective_adult_count + guests_ctx.children.get_untracked(),
            original_search_criteria: None, // Can be filled if needed
        };

        let payment = DomainPaymentInfo {
            method: DomainPaymentMethod::default(), // Default to wallet for crypto payments
        };

        Ok(DomainBookRoomRequest {
            block_id,
            holder,
            guests,
            payment,
            guest_payment: None,
            special_requests: None,
            booking_context,
            client_reference: None, // Can be set from BookingId if needed
        })
    }

    /// Convert UI state to ServiceBookingData
    /// This creates service-level booking data from UI state
    pub fn ui_to_service_booking_data(
        booking_id: BookingId,
        block_room_id: Option<String>,
    ) -> Result<ServiceBookingData, BookingError> {
        let block_room_state =
            crate::view_state_layer::ui_block_room::BlockRoomUIState::from_leptos_context();

        let payment_amount = block_room_state.total_price.get_untracked();

        Ok(ServiceBookingData {
            booking_id: booking_id.app_reference.clone(),
            email: booking_id.email.clone(),
            app_reference: booking_id.app_reference,
            block_room_id: block_room_id.clone(),
            payment_amount,
            payment_currency: "USD".to_string(),
            status: if block_room_id.is_some() {
                ServiceBookingStatus::RoomBlocked
            } else {
                ServiceBookingStatus::Draft
            },
        })
    }

    /// Validate UI form data before conversion
    pub fn validate_ui_form_data() -> Result<(), BookingError> {
        let block_room_state =
            crate::view_state_layer::ui_block_room::BlockRoomUIState::from_leptos_context();

        // Use existing validation logic
        if !block_room_state.form_valid.get_untracked() {
            return Err(BookingError::ValidationError(
                "Form validation failed. Please check all required fields.".to_string(),
            ));
        }

        Ok(())
    }
}

// Implement conversion traits for common UI types

impl UIToDomain<DomainAdultDetail> for AdultDetail {
    fn to_domain(&self) -> Result<DomainAdultDetail, BookingError> {
        Ok(DomainAdultDetail {
            email: self.email.clone(),
            first_name: self.first_name.clone(),
            last_name: self.last_name.clone(),
            phone: self.phone.clone(),
        })
    }
}

impl UIToDomain<DomainChildDetail> for ChildDetail {
    fn to_domain(&self) -> Result<DomainChildDetail, BookingError> {
        let age = self
            .age
            .ok_or_else(|| BookingError::ValidationError("Child age is required".to_string()))?;

        Ok(DomainChildDetail {
            age,
            first_name: self.first_name.clone(),
            last_name: self.last_name.clone(),
        })
    }
}

impl DomainToUI<AdultDetail> for DomainAdultDetail {
    fn to_ui(&self) -> Result<AdultDetail, BookingError> {
        Ok(AdultDetail {
            first_name: self.first_name.clone(),
            last_name: self.last_name.clone(),
            email: self.email.clone(),
            phone: self.phone.clone(),
        })
    }
}

impl DomainToUI<ChildDetail> for DomainChildDetail {
    fn to_ui(&self) -> Result<ChildDetail, BookingError> {
        Ok(ChildDetail {
            first_name: self.first_name.clone(),
            last_name: self.last_name.clone(),
            age: Some(self.age),
        })
    }
}
