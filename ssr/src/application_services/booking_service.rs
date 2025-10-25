use crate::api::client_side_api::ClientSideApiClient;
use crate::domain::{
    BookingError, BookingServiceConfig, DomainBlockRoomRequest, DomainBlockRoomResponse,
    DomainBookRoomRequest, DomainBookRoomResponse, ServiceBookingData, ServiceBookingStatus,
};
use crate::utils::app_reference::BookingId;
use crate::view_state_layer::booking_conversions::BookingConversions;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// Request structure for the integrated block room server function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegratedBlockRoomRequest {
    pub block_room_request: DomainBlockRoomRequest,
    pub booking_id: String,
    pub email: String,
    pub hotel_token: Option<String>,
}

/// Response structure for the integrated block room server function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegratedBlockRoomResponse {
    pub success: bool,
    pub message: String,
    pub block_room_response: Option<DomainBlockRoomResponse>,
    pub booking_id: String,
}

/// BookingService provides a clean interface for booking operations
/// This version focuses on UI-to-domain conversion and calling server functions
#[derive(Clone)]
pub struct BookingService {
    config: BookingServiceConfig,
}

impl BookingService {
    /// Create a new BookingService
    pub fn new() -> Self {
        Self {
            config: BookingServiceConfig::default(),
        }
    }

    /// Create a new BookingService with custom configuration
    pub fn with_config(config: BookingServiceConfig) -> Self {
        Self { config }
    }

    /// Create BookingService from UI context (legacy compatibility)
    pub fn from_ui_context<T: Clone>(_provider: T) -> Self {
        Self::new()
    }

    /// Block a room using the integrated server function
    /// This method converts UI state to domain objects and calls the server function
    pub async fn block_room_integrated(
        &self,
        booking_id: String,
        email: String,
        hotel_token: Option<String>,
    ) -> Result<IntegratedBlockRoomResponse, BookingError> {
        // Convert UI state to domain request
        let block_room_request = BookingConversions::ui_to_block_room_request()?;

        // Create integrated request
        let integrated_request = IntegratedBlockRoomRequest {
            block_room_request,
            booking_id,
            email,
            hotel_token,
        };

        // Call server function
        self.call_integrated_block_room_server_fn(integrated_request)
            .await
    }

    /// Call the integrated block room server function
    async fn call_integrated_block_room_server_fn(
        &self,
        request: IntegratedBlockRoomRequest,
    ) -> Result<IntegratedBlockRoomResponse, BookingError> {
        // Use ClientSideApiClient for consistent HTTP handling
        let client = ClientSideApiClient::new();

        match client.integrated_block_room(request).await {
            Some(response) => Ok(response),
            None => Err(BookingError::NetworkError(
                "Failed to call integrated block room server function".to_string(),
            )),
        }
    }

    /// Legacy block_room method for backward compatibility
    /// Now delegates to the integrated approach
    pub async fn block_room_with_backend_integration(
        &self,
        booking_id: String,
        email: String,
        hotel_token: Option<String>,
    ) -> Result<String, BookingError> {
        let response = self
            .block_room_integrated(booking_id.clone(), email, hotel_token)
            .await?;

        if response.success {
            crate::log!(
                "Successfully completed integrated block room for booking_id: {}",
                booking_id
            );
            Ok(booking_id)
        } else {
            Err(BookingError::BackendError(response.message))
        }
    }

    // /// Validation methods
    // fn validate_block_room_request(
    //     &self,
    //     request: &DomainBlockRoomRequest,
    // ) -> Result<(), BookingError> {
    //     if request.selected_rooms.is_empty() {
    //         return Err(BookingError::ValidationError(
    //             "At least one room must be selected".to_string(),
    //         ));
    //     }

    //     if request.user_details.adults.is_empty() {
    //         return Err(BookingError::ValidationError(
    //             "At least one adult guest is required".to_string(),
    //         ));
    //     }

    //     // Check that primary adult has required contact information
    //     if let Some(primary_adult) = request.user_details.adults.first() {
    //         if primary_adult.email.is_none() || primary_adult.phone.is_none() {
    //             return Err(BookingError::ValidationError(
    //                 "Primary adult must have email and phone".to_string(),
    //             ));
    //         }
    //     }

    //     Ok(())
    // }

    // fn validate_book_room_request(&self, request: &DomainBookRoomRequest) -> Result<(), BookingError> {
    //     if request.block_id.is_empty() {
    //         return Err(BookingError::ValidationError(
    //             "Block ID is required for booking".to_string(),
    //         ));
    //     }

    //     if request.holder.email.is_empty() {
    //         return Err(BookingError::ValidationError(
    //             "Booking holder email is required".to_string(),
    //         ));
    //     }

    //     if request.guests.is_empty() {
    //         return Err(BookingError::ValidationError(
    //             "At least one guest is required".to_string(),
    //         ));
    //     }

    //     Ok(())
    // }

    /// Convert UI state to ServiceBookingData
    pub fn create_service_booking_data(
        &self,
        booking_id: BookingId,
        block_room_id: Option<String>,
    ) -> Result<ServiceBookingData, BookingError> {
        BookingConversions::ui_to_service_booking_data(booking_id, block_room_id)
    }
}

impl Default for BookingService {
    fn default() -> Self {
        Self::new()
    }
}
