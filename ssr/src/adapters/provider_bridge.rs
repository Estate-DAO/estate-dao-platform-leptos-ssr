//! Bridge module for integrating hotel-providers with existing SSR adapters
//!
//! This module provides adapters that wrap the existing SSR `LiteApiAdapter`
//! to implement the traits from the `hotel-providers` crate.

use std::sync::Arc;

use hotel_providers::domain as new_domain;
use hotel_providers::ports::{
    HotelProviderPort as NewHotelProviderPort, PlaceProviderPort as NewPlaceProviderPort,
    ProviderError as NewProviderError, ProviderErrorKind, ProviderSteps,
    UISearchFilters as NewUISearchFilters,
};

use crate::adapters::LiteApiAdapter;
use crate::api::liteapi::LiteApiHTTPClient;
use crate::application_services::filter_types::UISearchFilters as OldUISearchFilters;
use crate::domain as old_domain;
use crate::ports::hotel_provider_port::ProviderError as OldProviderError;
use crate::ports::traits::{
    HotelProviderPort as OldHotelProviderPort, PlaceProviderPort as OldPlaceProviderPort,
};

/// Bridge adapter that wraps the existing LiteApiAdapter and implements
/// the new hotel-providers HotelProviderPort trait
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

// Type conversions between old SSR domain types and new hotel-providers domain types
// For now, we keep both type systems in sync and do simple conversions

impl From<OldUISearchFilters> for NewUISearchFilters {
    fn from(old: OldUISearchFilters) -> Self {
        NewUISearchFilters {
            min_star_rating: old.min_star_rating,
            max_price_per_night: old.max_price_per_night,
            min_price_per_night: old.min_price_per_night,
            amenities: old.amenities,
            property_types: old.property_types,
            popular_filters: old.popular_filters,
            hotel_name_search: old.hotel_name_search,
        }
    }
}

impl From<NewUISearchFilters> for OldUISearchFilters {
    fn from(new: NewUISearchFilters) -> Self {
        OldUISearchFilters {
            min_star_rating: new.min_star_rating,
            max_price_per_night: new.max_price_per_night,
            min_price_per_night: new.min_price_per_night,
            amenities: new.amenities,
            property_types: new.property_types,
            popular_filters: new.popular_filters,
            hotel_name_search: new.hotel_name_search,
        }
    }
}

fn convert_old_provider_error_to_new(old: OldProviderError) -> NewProviderError {
    let step = match old.0.error_step {
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

    NewProviderError::other(format!("{:?}", old.0.provider_name), step, old.to_string())
}

// Note: These conversion functions would need to be implemented fully in a production system.
// For now, we rely on serde_json roundtrip for type-compatible structures.

fn convert_search_criteria_new_to_old(
    new: new_domain::DomainHotelSearchCriteria,
) -> old_domain::DomainHotelSearchCriteria {
    // Serde roundtrip for compatible structures
    let json = serde_json::to_string(&new).expect("Serialization should work");
    serde_json::from_str(&json).expect("Deserialization should work for compatible types")
}

fn convert_search_result_old_to_new(
    old: old_domain::DomainHotelListAfterSearch,
) -> new_domain::DomainHotelListAfterSearch {
    let json = serde_json::to_string(&old).expect("Serialization should work");
    serde_json::from_str(&json).expect("Deserialization should work for compatible types")
}

fn convert_static_details_old_to_new(
    old: old_domain::DomainHotelStaticDetails,
) -> new_domain::DomainHotelStaticDetails {
    let json = serde_json::to_string(&old).expect("Serialization should work");
    serde_json::from_str(&json).expect("Deserialization should work for compatible types")
}

fn convert_info_criteria_new_to_old(
    new: new_domain::DomainHotelInfoCriteria,
) -> old_domain::DomainHotelInfoCriteria {
    let json = serde_json::to_string(&new).expect("Serialization should work");
    serde_json::from_str(&json).expect("Deserialization should work for compatible types")
}

fn convert_room_options_old_to_new(
    old: Vec<old_domain::DomainRoomOption>,
) -> Vec<new_domain::DomainRoomOption> {
    let json = serde_json::to_string(&old).expect("Serialization should work");
    serde_json::from_str(&json).expect("Deserialization should work for compatible types")
}

fn convert_block_request_new_to_old(
    new: new_domain::DomainBlockRoomRequest,
) -> old_domain::DomainBlockRoomRequest {
    let json = serde_json::to_string(&new).expect("Serialization should work");
    serde_json::from_str(&json).expect("Deserialization should work for compatible types")
}

fn convert_block_response_old_to_new(
    old: old_domain::DomainBlockRoomResponse,
) -> new_domain::DomainBlockRoomResponse {
    let json = serde_json::to_string(&old).expect("Serialization should work");
    serde_json::from_str(&json).expect("Deserialization should work for compatible types")
}

fn convert_book_request_new_to_old(
    new: new_domain::DomainBookRoomRequest,
) -> old_domain::DomainBookRoomRequest {
    let json = serde_json::to_string(&new).expect("Serialization should work");
    serde_json::from_str(&json).expect("Deserialization should work for compatible types")
}

fn convert_book_response_old_to_new(
    old: old_domain::DomainBookRoomResponse,
) -> new_domain::DomainBookRoomResponse {
    let json = serde_json::to_string(&old).expect("Serialization should work");
    serde_json::from_str(&json).expect("Deserialization should work for compatible types")
}

fn convert_get_booking_request_new_to_old(
    new: new_domain::DomainGetBookingRequest,
) -> old_domain::DomainGetBookingRequest {
    let json = serde_json::to_string(&new).expect("Serialization should work");
    serde_json::from_str(&json).expect("Deserialization should work for compatible types")
}

fn convert_get_booking_response_old_to_new(
    old: old_domain::DomainGetBookingResponse,
) -> new_domain::DomainGetBookingResponse {
    let json = serde_json::to_string(&old).expect("Serialization should work");
    serde_json::from_str(&json).expect("Deserialization should work for compatible types")
}

fn convert_price_map(
    old: std::collections::HashMap<String, old_domain::DomainPrice>,
) -> std::collections::HashMap<String, new_domain::DomainPrice> {
    old.into_iter()
        .map(|(k, v)| {
            (
                k,
                new_domain::DomainPrice {
                    room_price: v.room_price,
                    currency_code: v.currency_code,
                },
            )
        })
        .collect()
}

#[async_trait::async_trait]
impl NewHotelProviderPort for LiteApiProviderBridge {
    fn name(&self) -> &'static str {
        "LiteAPI"
    }

    fn is_healthy(&self) -> bool {
        true // LiteAPI doesn't expose health status currently
    }

    async fn search_hotels(
        &self,
        criteria: new_domain::DomainHotelSearchCriteria,
        ui_filters: NewUISearchFilters,
    ) -> Result<new_domain::DomainHotelListAfterSearch, NewProviderError> {
        let old_criteria = convert_search_criteria_new_to_old(criteria);
        let old_filters: OldUISearchFilters = ui_filters.into();

        self.inner
            .search_hotels(old_criteria, old_filters)
            .await
            .map(convert_search_result_old_to_new)
            .map_err(convert_old_provider_error_to_new)
    }

    async fn get_hotel_static_details(
        &self,
        hotel_id: &str,
    ) -> Result<new_domain::DomainHotelStaticDetails, NewProviderError> {
        self.inner
            .get_hotel_static_details(hotel_id)
            .await
            .map(convert_static_details_old_to_new)
            .map_err(convert_old_provider_error_to_new)
    }

    async fn get_hotel_rates(
        &self,
        criteria: new_domain::DomainHotelInfoCriteria,
    ) -> Result<Vec<new_domain::DomainRoomOption>, NewProviderError> {
        let old_criteria = convert_info_criteria_new_to_old(criteria);

        self.inner
            .get_hotel_rates(old_criteria)
            .await
            .map(convert_room_options_old_to_new)
            .map_err(convert_old_provider_error_to_new)
    }

    async fn get_min_rates(
        &self,
        criteria: new_domain::DomainHotelSearchCriteria,
        hotel_ids: Vec<String>,
    ) -> Result<std::collections::HashMap<String, new_domain::DomainPrice>, NewProviderError> {
        let old_criteria = convert_search_criteria_new_to_old(criteria);

        self.inner
            .get_min_rates(old_criteria, hotel_ids)
            .await
            .map(convert_price_map)
            .map_err(convert_old_provider_error_to_new)
    }

    async fn block_room(
        &self,
        block_request: new_domain::DomainBlockRoomRequest,
    ) -> Result<new_domain::DomainBlockRoomResponse, NewProviderError> {
        let old_request = convert_block_request_new_to_old(block_request);

        self.inner
            .block_room(old_request)
            .await
            .map(convert_block_response_old_to_new)
            .map_err(convert_old_provider_error_to_new)
    }

    async fn book_room(
        &self,
        book_request: new_domain::DomainBookRoomRequest,
    ) -> Result<new_domain::DomainBookRoomResponse, NewProviderError> {
        let old_request = convert_book_request_new_to_old(book_request);

        self.inner
            .book_room(old_request)
            .await
            .map(convert_book_response_old_to_new)
            .map_err(convert_old_provider_error_to_new)
    }

    async fn get_booking_details(
        &self,
        request: new_domain::DomainGetBookingRequest,
    ) -> Result<new_domain::DomainGetBookingResponse, NewProviderError> {
        let old_request = convert_get_booking_request_new_to_old(request);

        self.inner
            .get_booking_details(old_request)
            .await
            .map(convert_get_booking_response_old_to_new)
            .map_err(convert_old_provider_error_to_new)
    }
}

// Place provider bridge
fn convert_places_search_payload(
    new: new_domain::DomainPlacesSearchPayload,
) -> old_domain::DomainPlacesSearchPayload {
    old_domain::DomainPlacesSearchPayload {
        text_query: new.text_query,
    }
}

fn convert_places_response(
    old: old_domain::DomainPlacesResponse,
) -> new_domain::DomainPlacesResponse {
    new_domain::DomainPlacesResponse {
        data: old
            .data
            .into_iter()
            .map(|p| new_domain::DomainPlace {
                place_id: p.place_id,
                display_name: p.display_name,
                formatted_address: p.formatted_address,
            })
            .collect(),
    }
}

fn convert_place_details_payload(
    new: new_domain::DomainPlaceDetailsPayload,
) -> old_domain::DomainPlaceDetailsPayload {
    old_domain::DomainPlaceDetailsPayload {
        place_id: new.place_id,
    }
}

fn convert_place_details(old: old_domain::DomainPlaceDetails) -> new_domain::DomainPlaceDetails {
    let json = serde_json::to_string(&old).expect("Serialization should work");
    serde_json::from_str(&json).expect("Deserialization should work for compatible types")
}

#[async_trait::async_trait]
impl NewPlaceProviderPort for LiteApiProviderBridge {
    fn name(&self) -> &'static str {
        "LiteAPI"
    }

    fn is_healthy(&self) -> bool {
        true
    }

    async fn search_places(
        &self,
        criteria: new_domain::DomainPlacesSearchPayload,
    ) -> Result<new_domain::DomainPlacesResponse, NewProviderError> {
        let old_criteria = convert_places_search_payload(criteria);

        self.inner
            .search_places(old_criteria)
            .await
            .map(convert_places_response)
            .map_err(convert_old_provider_error_to_new)
    }

    async fn get_single_place_details(
        &self,
        payload: new_domain::DomainPlaceDetailsPayload,
    ) -> Result<new_domain::DomainPlaceDetails, NewProviderError> {
        let old_payload = convert_place_details_payload(payload);

        self.inner
            .get_single_place_details(old_payload)
            .await
            .map(convert_place_details)
            .map_err(convert_old_provider_error_to_new)
    }
}
