use crate::api::ApiError;
use crate::application_services::filter_types::{
    DomainSortDirection, DomainSortField, UISearchFilters, UISortOptions,
};
use crate::domain::{
    DomainBlockRoomRequest, DomainBlockRoomResponse, DomainBookRoomRequest, DomainBookRoomResponse,
    DomainGetBookingRequest, DomainGetBookingResponse, DomainPlaceDetails,
    DomainPlaceDetailsPayload, DomainPlacesResponse, DomainPlacesSearchPayload,
};
use crate::ports::hotel_provider_port::ProviderError;
use crate::ports::traits::PlaceProviderPort;
use std::cmp::Ordering;
use std::sync::Arc;

// #[derive(Clone)]
// pub struct PlaceService {
//     provider: Arc<dyn  PlaceProviderPort+ Send+ Sync>,
// }

#[derive(Clone)]
pub struct PlaceService<T: PlaceProviderPort + Clone> {
    provider: T,
}

impl<T: PlaceProviderPort + Clone> PlaceService<T> {
    pub fn init(provider: T) -> Self {
        // todo (provab) is the default adaptyer for now. we will test this later.
        // let provab_client = Provab::default();
        // let provab_adapter  = ProvabAdapter::new(provab_client);
        PlaceService::new(provider)
    }

    pub fn new(provider: T) -> Self {
        Self { provider }
    }

    pub async fn search_places_with_filters(
        &self,
        payload: DomainPlacesSearchPayload,
    ) -> Result<DomainPlacesResponse, ProviderError> {
        self.provider
            .search_places(payload)
            // .await
            .await
    }

    // <!-- Keep the original method for backward compatibility -->
    pub async fn get_single_place_details(
        &self,
        payload: DomainPlaceDetailsPayload,
    ) -> Result<DomainPlaceDetails, ProviderError> {
        // <!-- Use the new method with empty filters -->
        self.provider.get_single_place_details(payload).await
    }
}
