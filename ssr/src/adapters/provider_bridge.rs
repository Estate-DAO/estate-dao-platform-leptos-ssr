//! Bridge module for integrating hotel-providers with existing SSR adapters
//!
//! This module provides adapters that wrap the existing SSR `LiteApiAdapter`
//! to implement the traits from the `hotel-providers` crate.
//!
//! With the shared `hotel-types` crate, both SSR and hotel-providers use the
//! same domain types - no conversion needed!

use hotel_providers::ports::{
    HotelProviderPort as ProviderHotelPort, PlaceProviderPort as ProviderPlacePort, ProviderError,
    ProviderErrorKind, ProviderSteps, UISearchFilters as ProviderUISearchFilters,
};

// Both crates use the same types from hotel-types
use hotel_types::{
    DomainBlockRoomRequest, DomainBlockRoomResponse, DomainBookRoomRequest, DomainBookRoomResponse,
    DomainGetBookingRequest, DomainGetBookingResponse, DomainHotelInfoCriteria,
    DomainHotelListAfterSearch, DomainHotelSearchCriteria, DomainHotelStaticDetails,
    DomainPlaceDetails, DomainPlaceDetailsPayload, DomainPlacesResponse, DomainPlacesSearchPayload,
    DomainPrice, DomainRoomOption,
};

use crate::adapters::LiteApiAdapter;
use crate::api::liteapi::LiteApiHTTPClient;
use crate::application_services::filter_types::UISearchFilters as SsrUISearchFilters;
use crate::ports::hotel_provider_port::ProviderError as SsrProviderError;
use crate::ports::traits::{HotelProviderPort as SsrHotelPort, PlaceProviderPort as SsrPlacePort};

/// Bridge adapter that wraps the existing LiteApiAdapter and implements
/// the hotel-providers traits for registry integration.
#[derive(Clone)]
pub struct LiteApiProviderBridge {
    inner: LiteApiAdapter,
}

impl LiteApiProviderBridge {
    /// Create a new bridge adapter wrapping the given LiteApiAdapter
    pub fn new(adapter: LiteApiAdapter) -> Self {
        Self { inner: adapter }
    }

    /// Create a new bridge adapter from an HTTP client
    pub fn from_client(client: LiteApiHTTPClient) -> Self {
        Self::new(LiteApiAdapter::new(client))
    }

    /// Get access to the underlying adapter
    pub fn inner(&self) -> &LiteApiAdapter {
        &self.inner
    }
}

// =============================================================================
// Type Conversions
// =============================================================================
// With hotel-types, domain types are shared - only filter/error types need conversion

impl From<SsrUISearchFilters> for ProviderUISearchFilters {
    fn from(ssr: SsrUISearchFilters) -> Self {
        ProviderUISearchFilters {
            min_star_rating: ssr.min_star_rating,
            max_price_per_night: ssr.max_price_per_night,
            min_price_per_night: ssr.min_price_per_night,
            amenities: ssr.amenities,
            property_types: ssr.property_types,
            popular_filters: ssr.popular_filters,
            hotel_name_search: ssr.hotel_name_search,
        }
    }
}

impl From<ProviderUISearchFilters> for SsrUISearchFilters {
    fn from(provider: ProviderUISearchFilters) -> Self {
        SsrUISearchFilters {
            min_star_rating: provider.min_star_rating,
            max_price_per_night: provider.max_price_per_night,
            min_price_per_night: provider.min_price_per_night,
            amenities: provider.amenities,
            property_types: provider.property_types,
            popular_filters: provider.popular_filters,
            hotel_name_search: provider.hotel_name_search,
        }
    }
}

fn convert_ssr_error_to_provider(ssr_err: SsrProviderError) -> ProviderError {
    let step = match ssr_err.0.error_step {
        crate::ports::hotel_provider_port::ProviderSteps::PlaceSearch => ProviderSteps::PlaceSearch,
        crate::ports::hotel_provider_port::ProviderSteps::PlaceDetails => {
            ProviderSteps::PlaceDetails
        }
        crate::ports::hotel_provider_port::ProviderSteps::HotelSearch => ProviderSteps::HotelSearch,
        crate::ports::hotel_provider_port::ProviderSteps::HotelDetails => {
            ProviderSteps::HotelDetails
        }
        crate::ports::hotel_provider_port::ProviderSteps::HotelRate => ProviderSteps::HotelRate,
        crate::ports::hotel_provider_port::ProviderSteps::HotelBlockRoom => {
            ProviderSteps::HotelBlockRoom
        }
        crate::ports::hotel_provider_port::ProviderSteps::HotelBookRoom => {
            ProviderSteps::HotelBookRoom
        }
        crate::ports::hotel_provider_port::ProviderSteps::GetBookingDetails => {
            ProviderSteps::GetBookingDetails
        }
    };

    ProviderError::other(
        format!("{:?}", ssr_err.0.provider_name),
        step,
        ssr_err.to_string(),
    )
}

// =============================================================================
// HotelProviderPort Implementation
// =============================================================================
// All domain types are now from hotel-types - no conversions needed!

#[async_trait::async_trait]
impl ProviderHotelPort for LiteApiProviderBridge {
    fn name(&self) -> &'static str {
        "LiteAPI"
    }

    fn is_healthy(&self) -> bool {
        true // LiteAPI doesn't expose health status currently
    }

    async fn search_hotels(
        &self,
        criteria: DomainHotelSearchCriteria,
        ui_filters: ProviderUISearchFilters,
    ) -> Result<DomainHotelListAfterSearch, ProviderError> {
        self.inner
            .search_hotels(criteria, ui_filters.into())
            .await
            .map_err(convert_ssr_error_to_provider)
    }

    async fn get_hotel_static_details(
        &self,
        hotel_id: &str,
    ) -> Result<DomainHotelStaticDetails, ProviderError> {
        self.inner
            .get_hotel_static_details(hotel_id)
            .await
            .map_err(convert_ssr_error_to_provider)
    }

    async fn get_hotel_rates(
        &self,
        criteria: DomainHotelInfoCriteria,
    ) -> Result<Vec<DomainRoomOption>, ProviderError> {
        self.inner
            .get_hotel_rates(criteria)
            .await
            .map_err(convert_ssr_error_to_provider)
    }

    async fn get_min_rates(
        &self,
        criteria: DomainHotelSearchCriteria,
        hotel_ids: Vec<String>,
    ) -> Result<std::collections::HashMap<String, DomainPrice>, ProviderError> {
        self.inner
            .get_min_rates(criteria, hotel_ids)
            .await
            .map_err(convert_ssr_error_to_provider)
    }

    async fn block_room(
        &self,
        request: DomainBlockRoomRequest,
    ) -> Result<DomainBlockRoomResponse, ProviderError> {
        self.inner
            .block_room(request)
            .await
            .map_err(convert_ssr_error_to_provider)
    }

    async fn book_room(
        &self,
        request: DomainBookRoomRequest,
    ) -> Result<DomainBookRoomResponse, ProviderError> {
        self.inner
            .book_room(request)
            .await
            .map_err(convert_ssr_error_to_provider)
    }

    async fn get_booking_details(
        &self,
        request: DomainGetBookingRequest,
    ) -> Result<DomainGetBookingResponse, ProviderError> {
        self.inner
            .get_booking_details(request)
            .await
            .map_err(convert_ssr_error_to_provider)
    }
}

// =============================================================================
// PlaceProviderPort Implementation
// =============================================================================

#[async_trait::async_trait]
impl ProviderPlacePort for LiteApiProviderBridge {
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
        self.inner
            .search_places(criteria)
            .await
            .map_err(convert_ssr_error_to_provider)
    }

    async fn get_single_place_details(
        &self,
        payload: DomainPlaceDetailsPayload,
    ) -> Result<DomainPlaceDetails, ProviderError> {
        self.inner
            .get_single_place_details(payload)
            .await
            .map_err(convert_ssr_error_to_provider)
    }
}
