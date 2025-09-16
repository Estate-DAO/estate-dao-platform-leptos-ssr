use std::sync::Arc;

use crate::adapters::LiteApiAdapter;
use crate::api::api_client::ApiClient;
use crate::api::liteapi::{
    liteapi_book_room, liteapi_get_booking_details, liteapi_hotel_details, liteapi_hotel_rates,
    liteapi_hotel_search, liteapi_prebook, LiteApiBookRequest, LiteApiBookResponse,
    LiteApiGetBookingRequest, LiteApiGetBookingResponse, LiteApiGetPlaceResponse,
    LiteApiGetPlacesResponse, LiteApiHTTPClient, LiteApiHotelRatesRequest,
    LiteApiHotelRatesResponse, LiteApiHotelResult, LiteApiHotelSearchRequest,
    LiteApiHotelSearchResponse, LiteApiOccupancy, LiteApiPrebookRequest, LiteApiPrebookResponse,
    LiteApiSingleHotelDetailRequest, LiteApiSingleHotelDetailResponse,
};
use crate::ports::traits::PlaceProviderPort;
use crate::utils;
use futures::future::{BoxFuture, FutureExt};

use crate::api::ApiError;
use crate::application_services::filter_types::UISearchFilters;
use crate::domain::{
    DomainBlockRoomRequest, DomainBlockRoomResponse, DomainBookRoomRequest, DomainBookRoomResponse,
    DomainDetailedPrice, DomainFirstRoomDetails, DomainGetBookingRequest, DomainGetBookingResponse,
    DomainHotelAfterSearch, DomainHotelDetails, DomainHotelInfoCriteria,
    DomainHotelListAfterSearch, DomainHotelSearchCriteria, DomainPlaceDetails,
    DomainPlaceDetailsPayload, DomainPlacesResponse, DomainPlacesSearchPayload, DomainPrice,
    DomainRoomData,
};
use crate::ports::hotel_provider_port::{ProviderError, ProviderErrorDetails, ProviderSteps};
use crate::ports::ProviderNames;
use crate::utils::date::date_tuple_to_dd_mm_yyyy;
use async_trait::async_trait;

#[async_trait]
impl PlaceProviderPort for LiteApiAdapter {
    #[tracing::instrument(skip(self))]
    async fn search_places(
        &self,
        criteria: DomainPlacesSearchPayload,
    ) -> Result<DomainPlacesResponse, ProviderError> {
        let liteapi_request = Self::map_places_domain_to_liteapi(criteria);
        crate::log!(
            "LiteAPI search_places: calling search API with request: {:#?}",
            liteapi_request
        );

        let liteapi_response: LiteApiGetPlacesResponse = self
            .client
            .send(liteapi_request)
            .await
            .map_err(|e: ApiError| {
                ProviderError::from_api_error(e, ProviderNames::LiteApi, ProviderSteps::PlaceSearch)
            })?;

        crate::log!(
            "LiteAPI search_places: received response with {} places",
            liteapi_response.data.len()
        );

        // Use the new method that includes pricing from rates API
        Ok(Self::map_liteapi_places_response_to_domain(
            liteapi_response,
        ))
    }

    #[tracing::instrument(skip(self))]
    async fn get_single_place_details(
        &self,
        criteria: DomainPlaceDetailsPayload,
    ) -> Result<DomainPlaceDetails, ProviderError> {
        let liteapi_request = Self::map_places_id_domain_to_liteapi(criteria);
        crate::log!(
            "LiteAPI place_details: calling search API with request: {:#?}",
            liteapi_request
        );

        let liteapi_response: LiteApiGetPlaceResponse = self
            .client
            .send(liteapi_request)
            .await
            .map_err(|e: ApiError| {
                ProviderError::from_api_error(
                    e,
                    ProviderNames::LiteApi,
                    ProviderSteps::PlaceDetails,
                )
            })?;

        crate::log!(
            "LiteAPI place_details: received response with {:?} places",
            liteapi_response.data
        );

        // Use the new method that includes pricing from rates API
        Ok(Self::map_liteapi_place_details_response_to_domain(
            liteapi_response,
        ))
    }
}

// <!-- Future methods to be implemented -->
// async fn get_room_options(&self, hotel_id: String, token: String) -> Result<DomainRoomOptions, ProviderError> {
//     let liteapi_request = /* create request */;
//     let liteapi_response = self.client.send(liteapi_request).await.map_err(ProviderError::from)?;
//     Ok(/* map response */)
// }

// async fn book_room(&self, book_request: DomainBookRoomRequest) -> Result<DomainBookRoomResponse, ProviderError> {
//     let liteapi_request = /* map to liteapi book request */;
//     let liteapi_response = self.client.send(liteapi_request).await.map_err(ProviderError::from)?;
//     Ok(/* map response */)
// }
// }
