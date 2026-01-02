//! Hotel Provider Port - trait definition for hotel providers

use async_trait::async_trait;
use std::collections::HashMap;

use crate::ports::ProviderError;
use crate::{
    DomainBlockRoomRequest, DomainBlockRoomResponse, DomainBookRoomRequest, DomainBookRoomResponse,
    DomainGetBookingRequest, DomainGetBookingResponse, DomainHotelInfoCriteria,
    DomainHotelListAfterSearch, DomainHotelSearchCriteria, DomainHotelStaticDetails, DomainPrice,
    DomainRoomOption,
};

/// Filter criteria for hotel search - passed alongside search criteria
#[derive(Debug, Clone, Default)]
pub struct UISearchFilters {
    pub min_star_rating: Option<u8>,
    pub max_price_per_night: Option<f64>,
    pub min_price_per_night: Option<f64>,
    pub amenities: Option<Vec<String>>,
    pub property_types: Option<Vec<String>>,
    pub popular_filters: Option<Vec<String>>,
    pub hotel_name_search: Option<String>,
}

/// The main trait that all hotel providers must implement
#[async_trait]
pub trait HotelProviderPort: Send + Sync {
    /// Returns the name of the provider for logging and identification
    fn name(&self) -> &'static str;

    /// Returns whether the provider is currently healthy and available
    fn is_healthy(&self) -> bool {
        true // Default to healthy
    }

    /// Search for hotels in a given location
    async fn search_hotels(
        &self,
        criteria: DomainHotelSearchCriteria,
        ui_filters: UISearchFilters,
    ) -> Result<DomainHotelListAfterSearch, ProviderError>;

    /// Get static details for a hotel (name, description, amenities, photos, etc.)
    async fn get_hotel_static_details(
        &self,
        hotel_id: &str,
    ) -> Result<DomainHotelStaticDetails, ProviderError>;

    /// Get available room rates for a hotel
    async fn get_hotel_rates(
        &self,
        criteria: DomainHotelInfoCriteria,
    ) -> Result<Vec<DomainRoomOption>, ProviderError>;

    /// Get minimum rates for multiple hotels (lightweight endpoint for search results)
    async fn get_min_rates(
        &self,
        criteria: DomainHotelSearchCriteria,
        hotel_ids: Vec<String>,
    ) -> Result<HashMap<String, DomainPrice>, ProviderError>;

    /// Block/reserve a room before payment
    async fn block_room(
        &self,
        block_request: DomainBlockRoomRequest,
    ) -> Result<DomainBlockRoomResponse, ProviderError>;

    /// Book a room (finalize with payment)
    async fn book_room(
        &self,
        book_request: DomainBookRoomRequest,
    ) -> Result<DomainBookRoomResponse, ProviderError>;

    /// Get booking details
    async fn get_booking_details(
        &self,
        request: DomainGetBookingRequest,
    ) -> Result<DomainGetBookingResponse, ProviderError>;
}
