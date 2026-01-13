//! Composite Provider - wraps multiple providers with fallback logic

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn};

use crate::domain::*;
use crate::ports::{HotelProviderPort, PlaceProviderPort, ProviderError, UISearchFilters};

/// Strategy for handling provider fallback
#[derive(Debug, Clone, Default)]
pub enum FallbackStrategy {
    /// Try providers in order, use first success
    #[default]
    Sequential,
    /// Only fallback on specific error kinds that are retryable
    OnRetryableError,
    /// Never fallback - use only the primary provider
    NeverFallback,
}

/// Composite hotel provider that tries multiple providers with fallback
pub struct CompositeHotelProvider {
    providers: Vec<Arc<dyn HotelProviderPort>>,
    fallback_strategy: FallbackStrategy,
}

impl CompositeHotelProvider {
    /// Create a new composite provider with the given providers
    pub fn new(providers: Vec<Arc<dyn HotelProviderPort>>) -> Self {
        Self {
            providers,
            fallback_strategy: FallbackStrategy::default(),
        }
    }

    /// Create with a specific fallback strategy
    pub fn with_strategy(
        providers: Vec<Arc<dyn HotelProviderPort>>,
        strategy: FallbackStrategy,
    ) -> Self {
        Self {
            providers,
            fallback_strategy: strategy,
        }
    }

    /// Check if we should fallback based on the error and strategy
    fn should_fallback(&self, error: &ProviderError) -> bool {
        match &self.fallback_strategy {
            FallbackStrategy::Sequential => true,
            FallbackStrategy::OnRetryableError => error.should_fallback(),
            FallbackStrategy::NeverFallback => false,
        }
    }

    /// Get healthy providers
    fn healthy_providers(&self) -> Vec<Arc<dyn HotelProviderPort>> {
        self.providers
            .iter()
            .filter(|p| p.is_healthy())
            .cloned()
            .collect()
    }
}

#[async_trait]
impl HotelProviderPort for CompositeHotelProvider {
    fn name(&self) -> &'static str {
        "CompositeHotelProvider"
    }

    fn is_healthy(&self) -> bool {
        self.providers.iter().any(|p| p.is_healthy())
    }

    async fn search_hotels(
        &self,
        criteria: DomainHotelSearchCriteria,
        ui_filters: UISearchFilters,
    ) -> Result<DomainHotelListAfterSearch, ProviderError> {
        let providers = self.healthy_providers();
        let mut last_error: Option<ProviderError> = None;

        for provider in providers {
            info!(
                provider = provider.name(),
                "Trying provider for hotel search"
            );

            match provider
                .search_hotels(criteria.clone(), ui_filters.clone())
                .await
            {
                Ok(result) => {
                    info!(
                        provider = provider.name(),
                        count = result.hotel_results.len(),
                        "Search successful"
                    );
                    return Ok(result);
                }
                Err(error) => {
                    warn!(provider = provider.name(), error = %error, "Provider search failed");
                    if !self.should_fallback(&error) {
                        return Err(error);
                    }
                    last_error = Some(error);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            ProviderError::service_unavailable(
                "CompositeHotelProvider",
                crate::ports::ProviderSteps::HotelSearch,
                "No healthy providers available",
            )
        }))
    }

    async fn get_hotel_static_details(
        &self,
        hotel_id: &str,
    ) -> Result<DomainHotelStaticDetails, ProviderError> {
        let providers = self.healthy_providers();
        let mut last_error: Option<ProviderError> = None;

        for provider in providers {
            match provider.get_hotel_static_details(hotel_id).await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    if !self.should_fallback(&error) {
                        return Err(error);
                    }
                    last_error = Some(error);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            ProviderError::service_unavailable(
                "CompositeHotelProvider",
                crate::ports::ProviderSteps::HotelDetails,
                "No healthy providers available",
            )
        }))
    }

    async fn get_hotel_rates(
        &self,
        criteria: DomainHotelInfoCriteria,
    ) -> Result<DomainGroupedRoomRates, ProviderError> {
        let providers = self.healthy_providers();
        let mut last_error: Option<ProviderError> = None;

        for provider in providers {
            match provider.get_hotel_rates(criteria.clone()).await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    if !self.should_fallback(&error) {
                        return Err(error);
                    }
                    last_error = Some(error);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            ProviderError::service_unavailable(
                "CompositeHotelProvider",
                crate::ports::ProviderSteps::HotelRate,
                "No healthy providers available",
            )
        }))
    }

    async fn get_min_rates(
        &self,
        criteria: DomainHotelSearchCriteria,
        hotel_ids: Vec<String>,
    ) -> Result<HashMap<String, DomainPrice>, ProviderError> {
        let providers = self.healthy_providers();
        let mut last_error: Option<ProviderError> = None;

        for provider in providers {
            match provider
                .get_min_rates(criteria.clone(), hotel_ids.clone())
                .await
            {
                Ok(result) => return Ok(result),
                Err(error) => {
                    if !self.should_fallback(&error) {
                        return Err(error);
                    }
                    last_error = Some(error);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            ProviderError::service_unavailable(
                "CompositeHotelProvider",
                crate::ports::ProviderSteps::HotelRate,
                "No healthy providers available",
            )
        }))
    }

    async fn block_room(
        &self,
        block_request: DomainBlockRoomRequest,
    ) -> Result<DomainBlockRoomResponse, ProviderError> {
        let providers = self.healthy_providers();
        let mut last_error: Option<ProviderError> = None;

        for provider in providers {
            match provider.block_room(block_request.clone()).await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    if !self.should_fallback(&error) {
                        return Err(error);
                    }
                    last_error = Some(error);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            ProviderError::service_unavailable(
                "CompositeHotelProvider",
                crate::ports::ProviderSteps::HotelBlockRoom,
                "No healthy providers available",
            )
        }))
    }

    async fn book_room(
        &self,
        book_request: DomainBookRoomRequest,
    ) -> Result<DomainBookRoomResponse, ProviderError> {
        // For booking, we typically don't want to fallback
        // The user has already selected a specific room from a specific provider
        let providers = self.healthy_providers();

        if let Some(provider) = providers.first() {
            provider.book_room(book_request).await
        } else {
            Err(ProviderError::service_unavailable(
                "CompositeHotelProvider",
                crate::ports::ProviderSteps::HotelBookRoom,
                "No healthy providers available",
            ))
        }
    }

    async fn get_booking_details(
        &self,
        request: DomainGetBookingRequest,
    ) -> Result<DomainGetBookingResponse, ProviderError> {
        let providers = self.healthy_providers();
        let mut last_error: Option<ProviderError> = None;

        for provider in providers {
            match provider.get_booking_details(request.clone()).await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    if !self.should_fallback(&error) {
                        return Err(error);
                    }
                    last_error = Some(error);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            ProviderError::service_unavailable(
                "CompositeHotelProvider",
                crate::ports::ProviderSteps::GetBookingDetails,
                "No healthy providers available",
            )
        }))
    }
}

/// Composite place provider that tries multiple providers with fallback
pub struct CompositePlaceProvider {
    providers: Vec<Arc<dyn PlaceProviderPort>>,
    fallback_strategy: FallbackStrategy,
}

impl CompositePlaceProvider {
    /// Create a new composite provider with the given providers
    pub fn new(providers: Vec<Arc<dyn PlaceProviderPort>>) -> Self {
        Self {
            providers,
            fallback_strategy: FallbackStrategy::default(),
        }
    }

    /// Check if we should fallback based on the error and strategy
    fn should_fallback(&self, error: &ProviderError) -> bool {
        match &self.fallback_strategy {
            FallbackStrategy::Sequential => true,
            FallbackStrategy::OnRetryableError => error.should_fallback(),
            FallbackStrategy::NeverFallback => false,
        }
    }

    /// Get healthy providers
    fn healthy_providers(&self) -> Vec<Arc<dyn PlaceProviderPort>> {
        self.providers
            .iter()
            .filter(|p| p.is_healthy())
            .cloned()
            .collect()
    }
}

#[async_trait]
impl PlaceProviderPort for CompositePlaceProvider {
    fn name(&self) -> &'static str {
        "CompositePlaceProvider"
    }

    fn is_healthy(&self) -> bool {
        self.providers.iter().any(|p| p.is_healthy())
    }

    async fn search_places(
        &self,
        criteria: DomainPlacesSearchPayload,
    ) -> Result<DomainPlacesResponse, ProviderError> {
        let providers = self.healthy_providers();
        let mut last_error: Option<ProviderError> = None;

        for provider in providers {
            match provider.search_places(criteria.clone()).await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    if !self.should_fallback(&error) {
                        return Err(error);
                    }
                    last_error = Some(error);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            ProviderError::service_unavailable(
                "CompositePlaceProvider",
                crate::ports::ProviderSteps::PlaceSearch,
                "No healthy providers available",
            )
        }))
    }

    async fn get_single_place_details(
        &self,
        payload: DomainPlaceDetailsPayload,
    ) -> Result<DomainPlaceDetails, ProviderError> {
        let providers = self.healthy_providers();
        let mut last_error: Option<ProviderError> = None;

        for provider in providers {
            match provider.get_single_place_details(payload.clone()).await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    if !self.should_fallback(&error) {
                        return Err(error);
                    }
                    last_error = Some(error);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            ProviderError::service_unavailable(
                "CompositePlaceProvider",
                crate::ports::ProviderSteps::PlaceDetails,
                "No healthy providers available",
            )
        }))
    }
}
