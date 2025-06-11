use std::sync::Arc;

use crate::adapters::LiteApiAdapter;
use crate::api::api_client::ApiClient;
use crate::api::liteapi::{
    liteapi_hotel_rates, liteapi_hotel_search, LiteApiHTTPClient, LiteApiHotelRatesRequest,
    LiteApiHotelRatesResponse, LiteApiHotelResult, LiteApiHotelSearchRequest,
    LiteApiHotelSearchResponse, LiteApiOccupancy,
};
use crate::ports::traits::HotelProviderPort;
use crate::utils;
use futures::future::{BoxFuture, FutureExt};

use crate::api::ApiError;
use crate::application_services::filter_types::UISearchFilters;
use crate::domain::{
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
    }
}

// <!-- Future methods to be implemented -->
// async fn get_room_options(&self, hotel_id: String, token: String) -> Result<DomainRoomOptions, ProviderError> {
//     let provab_request = /* create request */;
//     let provab_response = self.client.send(provab_request).await.map_err(ProviderError::from)?;
//     Ok(/* map response */)
// }

// async fn block_room(&self, block_request: DomainBlockRoomRequest) -> Result<DomainBlockRoomResponse, ProviderError> {
//     let provab_request = /* map to provab block request */;
//     let provab_response = self.client.send(provab_request).await.map_err(ProviderError::from)?;
//     Ok(/* map response */)
// }

// async fn book_room(&self, book_request: DomainBookRoomRequest) -> Result<DomainBookRoomResponse, ProviderError> {
//     let provab_request = /* map to provab book request */;
//     let provab_response = self.client.send(provab_request).await.map_err(ProviderError::from)?;
//     Ok(/* map response */)
// }
// }
