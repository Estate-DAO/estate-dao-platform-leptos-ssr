//! Mock Provider Adapter - for testing and development
//!
//! This module provides mock implementations of the provider ports
//! for testing fallback behavior and development purposes.

use async_trait::async_trait;
use std::collections::HashMap;

use crate::domain::*;
use crate::ports::{
    HotelProviderPort, PlaceProviderPort, ProviderError, ProviderSteps, UISearchFilters,
};

/// Mock hotel provider for testing
#[derive(Clone, Default)]
pub struct MockHotelProvider {
    name: &'static str,
    healthy: bool,
    should_fail: bool,
}

impl MockHotelProvider {
    pub fn new() -> Self {
        Self {
            name: "MockHotelProvider",
            healthy: true,
            should_fail: false,
        }
    }

    pub fn with_name(mut self, name: &'static str) -> Self {
        self.name = name;
        self
    }

    pub fn unhealthy(mut self) -> Self {
        self.healthy = false;
        self
    }

    pub fn failing(mut self) -> Self {
        self.should_fail = true;
        self
    }
}

#[async_trait]
impl HotelProviderPort for MockHotelProvider {
    fn name(&self) -> &'static str {
        self.name
    }

    fn is_healthy(&self) -> bool {
        self.healthy
    }

    async fn search_hotels(
        &self,
        _criteria: DomainHotelSearchCriteria,
        _ui_filters: UISearchFilters,
    ) -> Result<DomainHotelListAfterSearch, ProviderError> {
        if self.should_fail {
            return Err(ProviderError::service_unavailable(
                self.name,
                ProviderSteps::HotelSearch,
                "Mock provider configured to fail",
            ));
        }

        Ok(DomainHotelListAfterSearch {
            hotel_results: vec![DomainHotelAfterSearch {
                hotel_code: "MOCK001".to_string(),
                hotel_name: "Mock Hotel".to_string(),
                hotel_category: "Hotel".to_string(),
                star_rating: 4,
                price: Some(DomainPrice {
                    room_price: 100.0,
                    currency_code: "USD".to_string(),
                }),
                hotel_picture: "https://example.com/mock.jpg".to_string(),
                amenities: vec!["WiFi".to_string(), "Pool".to_string()],
                property_type: Some("Hotel".to_string()),
                result_token: "mock_token".to_string(),
                hotel_address: Some("123 Mock Street".to_string()),
                distance_from_center_km: Some(1.5),
            }],
            pagination: None,
        })
    }

    async fn get_hotel_static_details(
        &self,
        hotel_id: &str,
    ) -> Result<DomainHotelStaticDetails, ProviderError> {
        if self.should_fail {
            return Err(ProviderError::service_unavailable(
                self.name,
                ProviderSteps::HotelDetails,
                "Mock provider configured to fail",
            ));
        }

        Ok(DomainHotelStaticDetails {
            hotel_name: format!("Mock Hotel {}", hotel_id),
            hotel_code: hotel_id.to_string(),
            star_rating: 4,
            rating: Some(4.5),
            review_count: Some(100),
            categories: vec![],
            description: "A mock hotel for testing".to_string(),
            hotel_facilities: vec!["WiFi".to_string()],
            address: "123 Mock Street".to_string(),
            images: vec![],
            amenities: vec!["WiFi".to_string()],
            rooms: vec![],
            location: None,
            checkin_checkout_times: None,
            policies: vec![],
        })
    }

    async fn get_hotel_rates(
        &self,
        _criteria: DomainHotelInfoCriteria,
    ) -> Result<Vec<DomainRoomOption>, ProviderError> {
        if self.should_fail {
            return Err(ProviderError::service_unavailable(
                self.name,
                ProviderSteps::HotelRate,
                "Mock provider configured to fail",
            ));
        }

        Ok(vec![])
    }

    async fn get_min_rates(
        &self,
        _criteria: DomainHotelSearchCriteria,
        hotel_ids: Vec<String>,
    ) -> Result<HashMap<String, DomainPrice>, ProviderError> {
        if self.should_fail {
            return Err(ProviderError::service_unavailable(
                self.name,
                ProviderSteps::HotelRate,
                "Mock provider configured to fail",
            ));
        }

        let mut rates = HashMap::new();
        for id in hotel_ids {
            rates.insert(
                id,
                DomainPrice {
                    room_price: 99.0,
                    currency_code: "USD".to_string(),
                },
            );
        }
        Ok(rates)
    }

    async fn block_room(
        &self,
        _block_request: DomainBlockRoomRequest,
    ) -> Result<DomainBlockRoomResponse, ProviderError> {
        if self.should_fail {
            return Err(ProviderError::service_unavailable(
                self.name,
                ProviderSteps::HotelBlockRoom,
                "Mock provider configured to fail",
            ));
        }

        Err(ProviderError::other(
            self.name,
            ProviderSteps::HotelBlockRoom,
            "Mock block room not implemented",
        ))
    }

    async fn book_room(
        &self,
        _book_request: DomainBookRoomRequest,
    ) -> Result<DomainBookRoomResponse, ProviderError> {
        Err(ProviderError::other(
            self.name,
            ProviderSteps::HotelBookRoom,
            "Mock book room not implemented",
        ))
    }

    async fn get_booking_details(
        &self,
        _request: DomainGetBookingRequest,
    ) -> Result<DomainGetBookingResponse, ProviderError> {
        Err(ProviderError::other(
            self.name,
            ProviderSteps::GetBookingDetails,
            "Mock get booking details not implemented",
        ))
    }
}

/// Mock place provider for testing
#[derive(Clone, Default)]
pub struct MockPlaceProvider {
    name: &'static str,
    healthy: bool,
    should_fail: bool,
}

impl MockPlaceProvider {
    pub fn new() -> Self {
        Self {
            name: "MockPlaceProvider",
            healthy: true,
            should_fail: false,
        }
    }

    pub fn failing(mut self) -> Self {
        self.should_fail = true;
        self
    }
}

#[async_trait]
impl PlaceProviderPort for MockPlaceProvider {
    fn name(&self) -> &'static str {
        self.name
    }

    fn is_healthy(&self) -> bool {
        self.healthy
    }

    async fn search_places(
        &self,
        criteria: DomainPlacesSearchPayload,
    ) -> Result<DomainPlacesResponse, ProviderError> {
        if self.should_fail {
            return Err(ProviderError::service_unavailable(
                self.name,
                ProviderSteps::PlaceSearch,
                "Mock provider configured to fail",
            ));
        }

        Ok(DomainPlacesResponse {
            data: vec![DomainPlace {
                place_id: "mock_place_id".to_string(),
                display_name: format!("Mock Place for '{}'", criteria.text_query),
                formatted_address: "123 Mock Street, Mock City".to_string(),
            }],
        })
    }

    async fn get_single_place_details(
        &self,
        _payload: DomainPlaceDetailsPayload,
    ) -> Result<DomainPlaceDetails, ProviderError> {
        if self.should_fail {
            return Err(ProviderError::service_unavailable(
                self.name,
                ProviderSteps::PlaceDetails,
                "Mock provider configured to fail",
            ));
        }

        Ok(DomainPlaceDetails::default())
    }
}
