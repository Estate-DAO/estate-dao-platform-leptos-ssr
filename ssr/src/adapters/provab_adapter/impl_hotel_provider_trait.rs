use crate::adapters::ProvabAdapter;
use crate::api::api_client::ApiClient;
use crate::api::provab::a01_hotel_info::{
    FirstRoomDetails as ProvabFirstRoomDetails, HotelDetailsLevel1 as ProvabHotelDetailsLevel1,
    HotelDetailsLevel2 as ProvabHotelDetailsLevel2, HotelInfoResult as ProvabHotelInfoResult,
    Price as ProvabDetailedPrice, RoomData as ProvabRoomData,
};
use crate::ports::traits::HotelProviderPort;
use futures::future::{BoxFuture, FutureExt};

use crate::api::provab::{
    BlockRoomRequest as ProvabBlockRoomRequest,
    BlockRoomResponse as ProvabBlockRoomResponse,
    HotelInfoRequest as ProvabHotelInfoRequest,
    HotelInfoResponse as ProvabHotelInfoResponse,
    HotelResult as ProvabHotelResult,
    HotelSearchRequest as ProvabHotelSearchRequest,
    HotelSearchResponse as ProvabHotelSearchResponse,
    HotelSearchResult as ProvabHotelSearchResult,
    Price as ProvabPrice,
    Provab, // Your existing Provab client
    RoomGuest as ProvabRoomGuest,
    Search as ProvabSearch,
};
use crate::api::ApiError;
use crate::application_services::filter_types::UISearchFilters;
use crate::domain::{
    DomainBlockRoomRequest, DomainBlockRoomResponse, DomainBookRoomRequest, DomainBookRoomResponse,
    DomainDetailedPrice, DomainFirstRoomDetails, DomainHotelAfterSearch, DomainHotelDetails,
    DomainHotelInfoCriteria, DomainHotelListAfterSearch, DomainHotelSearchCriteria, DomainPrice,
    DomainRoomData,
};
use crate::ports::hotel_provider_port::{ProviderError, ProviderErrorDetails, ProviderSteps};
use crate::ports::ProviderNames;
use crate::utils::date::date_tuple_to_dd_mm_yyyy;
use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
impl HotelProviderPort for ProvabAdapter {
    async fn search_hotels(
        // async fn search_hotels(
        &self,
        criteria: DomainHotelSearchCriteria,
        ui_filters: UISearchFilters,
    ) -> Result<DomainHotelListAfterSearch, ProviderError> {
        // ) -> LocalBoxFuture<'_, Result<DomainHotelListAfterSearch, ProviderError>> {
        // async move {
        let provab_request = Self::map_domain_search_to_provab(&criteria, ui_filters.clone());
        let provab_response: ProvabHotelSearchResponse = self
            .client
            .send(provab_request)
            .await
            .map_err(|e: ApiError| {
                ProviderError::from_api_error(e, ProviderNames::Provab, ProviderSteps::HotelSearch)
            })?;
        Ok(Self::map_provab_search_to_domain(provab_response))
        // }
        // .boxed_local()
    }

    async fn get_single_hotel_details(
        &self,
        criteria: DomainHotelInfoCriteria,
    ) -> Result<DomainHotelDetails, ProviderError> {
        // ) -> LocalBoxFuture<'_, Result<DomainHotelDetails, ProviderError>> {
        // async move {
        let provab_request = Self::map_domain_hotel_info_to_provab(&criteria);
        let provab_response: ProvabHotelInfoResponse = self
            .client
            .send(provab_request)
            .await
            .map_err(|e: ApiError| {
                ProviderError::from_api_error(e, ProviderNames::Provab, ProviderSteps::HotelDetails)
            })?;
        Self::map_provab_hotel_info_to_domain(provab_response)
    }
    // .boxed_local()

    async fn block_room(
        &self,
        block_request: DomainBlockRoomRequest,
    ) -> Result<DomainBlockRoomResponse, ProviderError> {
        // Validate request before processing
        Self::validate_block_room_request(&block_request)?;

        // Map domain request to Provab format
        let provab_request = Self::map_domain_block_to_provab(&block_request);

        // Call Provab block room endpoint
        let provab_response: ProvabBlockRoomResponse = self
            .client
            .send(provab_request)
            .await
            .map_err(|e: ApiError| {
                ProviderError::from_api_error(
                    e,
                    ProviderNames::Provab,
                    ProviderSteps::HotelBlockRoom,
                )
            })?;

        // Map response to domain block room response
        Self::map_provab_block_to_domain(provab_response, &block_request)
    }

    async fn book_room(
        &self,
        _book_request: DomainBookRoomRequest,
    ) -> Result<DomainBookRoomResponse, ProviderError> {
        // TODO: Implement Provab book room functionality
        // For now, return a not implemented error
        Err(ProviderError(Arc::new(ProviderErrorDetails {
            provider_name: ProviderNames::Provab,
            api_error: ApiError::Other(
                "Book room not yet implemented for Provab provider".to_string(),
            ),
            error_step: ProviderSteps::HotelBookRoom,
        })))
    }

    async fn get_hotel_rates(
        &self,
        criteria: DomainHotelInfoCriteria,
    ) -> Result<DomainHotelDetails, ProviderError> {
        // For Provab, hotel rates are equivalent to hotel details
        // since Provab provides pricing information in their hotel info response
        self.get_single_hotel_details(criteria).await
    }
}

// <!-- Future methods to be implemented -->
// async fn get_room_options(&self, hotel_id: String, token: String) -> Result<DomainRoomOptions, ProviderError> {
//     let provab_request = /* create request */;
//     let provab_response = self.client.send(provab_request).await.map_err(ProviderError::from)?;
//     Ok(/* map response */)
// }

// async fn book_room(&self, book_request: DomainBookRoomRequest) -> Result<DomainBookRoomResponse, ProviderError> {
//     let provab_request = /* map to provab book request */;
//     let provab_response = self.client.send(provab_request).await.map_err(ProviderError::from)?;
//     Ok(/* map response */)
// }
// }
