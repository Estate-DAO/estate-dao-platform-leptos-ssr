//! Bridge module for integrating hotel-providers with existing SSR code
//!
//! This module provides the `LiteApiProviderBridge` which wraps `LiteApiDriver`
//! directly and implements the registry traits from `hotel-types::ports`.

use hotel_providers::liteapi::LiteApiDriver;
use hotel_types::ports::{HotelProviderPort, PlaceProviderPort, ProviderError, UISearchFilters};

// Domain types from hotel-types
use hotel_types::{
    DomainBlockRoomRequest, DomainBlockRoomResponse, DomainBookRoomRequest, DomainBookRoomResponse,
    DomainGetBookingRequest, DomainGetBookingResponse, DomainHotelInfoCriteria,
    DomainHotelListAfterSearch, DomainHotelSearchCriteria, DomainHotelStaticDetails,
    DomainPlaceDetails, DomainPlaceDetailsPayload, DomainPlacesResponse, DomainPlacesSearchPayload,
    DomainPrice, DomainRoomOption,
};

/// Bridge adapter that wraps `LiteApiDriver` and implements the port traits.
#[derive(Clone)]
pub struct LiteApiProviderBridge {
    driver: LiteApiDriver,
}

impl LiteApiProviderBridge {
    /// Create a new bridge wrapping the given driver
    pub fn new(driver: LiteApiDriver) -> Self {
        Self { driver }
    }

    /// Get access to the underlying driver
    pub fn driver(&self) -> &LiteApiDriver {
        &self.driver
    }
}

// Type conversions are handled in filter_types.rs

// =============================================================================
// HotelProviderPort Implementation - Delegates to Driver
// =============================================================================

#[async_trait::async_trait]
impl HotelProviderPort for LiteApiProviderBridge {
    fn name(&self) -> &'static str {
        "LiteAPI"
    }

    fn is_healthy(&self) -> bool {
        true
    }

    async fn search_hotels(
        &self,
        criteria: DomainHotelSearchCriteria,
        ui_filters: UISearchFilters,
    ) -> Result<DomainHotelListAfterSearch, ProviderError> {
        self.driver.search_hotels(criteria, ui_filters).await
    }

    async fn get_hotel_static_details(
        &self,
        hotel_id: &str,
    ) -> Result<DomainHotelStaticDetails, ProviderError> {
        self.driver.get_hotel_static_details(hotel_id).await
    }

    async fn get_hotel_rates(
        &self,
        criteria: DomainHotelInfoCriteria,
    ) -> Result<Vec<DomainRoomOption>, ProviderError> {
        self.driver.get_hotel_rates(criteria).await
    }

    async fn get_min_rates(
        &self,
        criteria: DomainHotelSearchCriteria,
        hotel_ids: Vec<String>,
    ) -> Result<std::collections::HashMap<String, DomainPrice>, ProviderError> {
        self.driver.get_min_rates(criteria, hotel_ids).await
    }

    async fn block_room(
        &self,
        request: DomainBlockRoomRequest,
    ) -> Result<DomainBlockRoomResponse, ProviderError> {
        self.driver.block_room(request).await
    }

    async fn book_room(
        &self,
        request: DomainBookRoomRequest,
    ) -> Result<DomainBookRoomResponse, ProviderError> {
        self.driver.book_room(request).await
    }

    async fn get_booking_details(
        &self,
        request: DomainGetBookingRequest,
    ) -> Result<DomainGetBookingResponse, ProviderError> {
        self.driver.get_booking_details(request).await
    }
}

// =============================================================================
// PlaceProviderPort Implementation - Delegates to Driver
// =============================================================================

#[async_trait::async_trait]
impl PlaceProviderPort for LiteApiProviderBridge {
    fn name(&self) -> &'static str {
        "LiteAPI"
    }

    fn is_healthy(&self) -> bool {
        true
    }

    async fn search_places(
        &self,
        criteria: DomainPlacesSearchPayload,
    ) -> Result<DomainPlacesResponse, ProviderError> {
        self.driver.search_places(criteria).await
    }

    async fn get_single_place_details(
        &self,
        payload: DomainPlaceDetailsPayload,
    ) -> Result<DomainPlaceDetails, ProviderError> {
        self.driver.get_single_place_details(payload).await
    }
}
