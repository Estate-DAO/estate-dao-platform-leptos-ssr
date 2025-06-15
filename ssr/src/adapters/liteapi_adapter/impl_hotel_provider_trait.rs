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

        // Use the new method that includes pricing from rates API
        self.map_liteapi_search_to_domain_with_pricing(liteapi_response, &criteria)
            .await
    }

    async fn get_hotel_details(
        &self,
        criteria: DomainHotelInfoCriteria,
    ) -> Result<DomainHotelDetails, ProviderError> {
        // First, get hotel information from search to get detailed hotel info
        let hotel_search_request = LiteApiHotelSearchRequest {
            country_code: criteria.search_criteria.destination_country_code.clone(),
            city_name: criteria.search_criteria.destination_city_name.clone(),
            offset: 0,
            limit: 50,
        };

        let hotel_search_response: LiteApiHotelSearchResponse = self
            .client
            .send(hotel_search_request)
            .await
            .map_err(|e: ApiError| {
                ProviderError::from_api_error(
                    e,
                    ProviderNames::LiteApi,
                    ProviderSteps::HotelDetails,
                )
            })?;

        // Find the specific hotel in search results
        let hotel_id = if !criteria.hotel_ids.is_empty() {
            criteria.hotel_ids[0].clone()
        } else {
            criteria.token.clone()
        };

        let hotel_info = hotel_search_response
            .data
            .iter()
            .find(|h| h.id == hotel_id)
            .ok_or_else(|| {
                ProviderError(Arc::new(ProviderErrorDetails {
                    provider_name: ProviderNames::LiteApi,
                    api_error: ApiError::Other(format!(
                        "Hotel with ID {} not found in search results",
                        hotel_id
                    )),
                    error_step: ProviderSteps::HotelDetails,
                }))
            })?;

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

        // Map response to domain hotel details using hotel info from search
        Self::map_liteapi_rates_to_domain_details(
            liteapi_response,
            &criteria.search_criteria,
            Some(hotel_info),
        )
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

    async fn get_hotel_rates(
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
        Self::map_liteapi_rates_to_domain_details(liteapi_response, &criteria.search_criteria, None)
            .map_err(|e| {
                // Log the error for debugging
                crate::log!("LiteAPI rates mapping error: {:?}", e);
                e
            })
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
