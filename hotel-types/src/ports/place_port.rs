//! Place Provider Port - trait definition for place/location providers

use async_trait::async_trait;

use crate::ports::ProviderError;
use crate::{
    DomainPlaceDetails, DomainPlaceDetailsPayload, DomainPlacesResponse, DomainPlacesSearchPayload,
};

/// The trait that all place/location providers must implement
#[async_trait]
pub trait PlaceProviderPort: Send + Sync {
    /// Returns the name of the provider for logging and identification
    fn name(&self) -> &'static str;

    /// Returns whether the provider is currently healthy and available
    fn is_healthy(&self) -> bool {
        true // Default to healthy
    }

    /// Search for places/locations by text query
    async fn search_places(
        &self,
        criteria: DomainPlacesSearchPayload,
    ) -> Result<DomainPlacesResponse, ProviderError>;

    /// Get details for a specific place
    async fn get_single_place_details(
        &self,
        payload: DomainPlaceDetailsPayload,
    ) -> Result<DomainPlaceDetails, ProviderError>;
}
