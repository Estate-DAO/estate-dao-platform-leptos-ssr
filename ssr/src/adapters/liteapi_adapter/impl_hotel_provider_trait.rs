use std::sync::Arc;

use crate::adapters::LiteApiAdapter;
use crate::api::api_client::ApiClient;
use crate::api::liteapi::{
    liteapi_book_room, liteapi_hotel_rates, liteapi_hotel_search, liteapi_prebook,
    LiteApiBookRequest, LiteApiBookResponse, LiteApiHTTPClient, LiteApiHotelRatesRequest,
    LiteApiHotelRatesResponse, LiteApiHotelResult, LiteApiHotelSearchRequest,
    LiteApiHotelSearchResponse, LiteApiOccupancy, LiteApiPrebookRequest, LiteApiPrebookResponse,
};
use crate::ports::traits::HotelProviderPort;
use crate::utils;
use futures::future::{BoxFuture, FutureExt};

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

#[async_trait]
impl HotelProviderPort for LiteApiAdapter {
    async fn search_hotels(
        &self,
        criteria: DomainHotelSearchCriteria,
        ui_filters: UISearchFilters,
    ) -> Result<DomainHotelListAfterSearch, ProviderError> {
        let liteapi_request = Self::map_domain_search_to_liteapi(&criteria, ui_filters.clone());
        let liteapi_response: LiteApiHotelSearchResponse = self
            .client
            .send(liteapi_request)
            .await
            .map_err(|e: ApiError| {
                ProviderError::from_api_error(e, ProviderNames::LiteApi, ProviderSteps::HotelSearch)
            })?;
        Ok(Self::map_liteapi_search_to_domain(liteapi_response))
    }

    async fn get_hotel_details(
        &self,
        criteria: DomainHotelInfoCriteria,
    ) -> Result<DomainHotelDetails, ProviderError> {
        // Convert domain criteria to LiteAPI rates request
        let liteapi_request = Self::map_domain_info_to_liteapi_rates(&criteria)?;

        // Call LiteAPI rates endpoint
        let liteapi_response: LiteApiHotelRatesResponse = self
            .client
            .send(liteapi_request)
            .await
            .map_err(|e: ApiError| {
            ProviderError::from_api_error(e, ProviderNames::LiteApi, ProviderSteps::HotelDetails)
        })?;

        // Map response to domain hotel details
        Self::map_liteapi_rates_to_domain_details(liteapi_response, &criteria.search_criteria)
            .map_err(|e| {
                // Log the error for debugging
                crate::log!("LiteAPI rates mapping error: {:?}", e);
                e
            })
    }

    async fn block_room(
        &self,
        block_request: DomainBlockRoomRequest,
    ) -> Result<DomainBlockRoomResponse, ProviderError> {
        // Validate request before processing
        Self::validate_block_room_request(&block_request)?;

        // Map domain request to LiteAPI prebook request
        let liteapi_request = Self::map_domain_block_to_liteapi_prebook(&block_request)?;

        // Call LiteAPI prebook endpoint
        let liteapi_response: LiteApiPrebookResponse = self
            .client
            .send(liteapi_request)
            .await
            .map_err(|e: ApiError| {
                ProviderError::from_api_error(
                    e,
                    ProviderNames::LiteApi,
                    ProviderSteps::HotelBlockRoom,
                )
            })?;

        // Map response to domain block room response
        Ok(Self::map_liteapi_prebook_to_domain_block(
            liteapi_response,
            &block_request,
        ))
    }

    async fn book_room(
        &self,
        book_request: DomainBookRoomRequest,
    ) -> Result<DomainBookRoomResponse, ProviderError> {
        // Validate request before processing
        Self::validate_book_room_request(&book_request)?;

        // Map domain request to LiteAPI book request
        let liteapi_request = Self::map_domain_book_to_liteapi_book(&book_request)?;

        // Call LiteAPI book endpoint
        let liteapi_response: LiteApiBookResponse = self
            .client
            .send(liteapi_request)
            .await
            .map_err(|e: ApiError| {
                ProviderError::from_api_error(
                    e,
                    ProviderNames::LiteApi,
                    ProviderSteps::HotelBookRoom,
                )
            })?;

        // Map response to domain book room response
        Ok(Self::map_liteapi_book_to_domain_book(
            liteapi_response,
            &book_request,
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
