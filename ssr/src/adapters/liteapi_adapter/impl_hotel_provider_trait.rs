use std::sync::Arc;

use crate::adapters::LiteApiAdapter;
use crate::api::api_client::ApiClient;
use crate::api::liteapi::{
    liteapi_book_room, liteapi_hotel_details, liteapi_hotel_rates, liteapi_hotel_search,
    liteapi_prebook, LiteApiBookRequest, LiteApiBookResponse, LiteApiHTTPClient,
    LiteApiHotelRatesRequest, LiteApiHotelRatesResponse, LiteApiHotelResult,
    LiteApiHotelSearchRequest, LiteApiHotelSearchResponse, LiteApiOccupancy, LiteApiPrebookRequest,
    LiteApiPrebookResponse, LiteApiSingleHotelDetailRequest, LiteApiSingleHotelDetailResponse,
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
    #[tracing::instrument(skip(self, ui_filters))]
    async fn search_hotels(
        &self,
        criteria: DomainHotelSearchCriteria,
        ui_filters: UISearchFilters,
    ) -> Result<DomainHotelListAfterSearch, ProviderError> {
        let liteapi_request = Self::map_domain_search_to_liteapi(&criteria, ui_filters.clone());
        crate::log!(
            "LiteAPI search_hotels: calling search API with request: {:#?}",
            liteapi_request
        );

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

    #[tracing::instrument(skip(self))]
    async fn get_single_hotel_details(
        &self,
        criteria: DomainHotelInfoCriteria,
    ) -> Result<DomainHotelDetails, ProviderError> {
        // Get hotel ID from criteria
        let hotel_id = if !criteria.hotel_ids.is_empty() {
            let id = criteria.hotel_ids[0].clone();
            crate::log!("LiteAPI get_hotel_details called with hotel_id: {}", id);
            id
        } else {
            return Err(ProviderError::from_api_error(
                ApiError::Other("Hotel IDs cannot be empty".to_string()),
                ProviderNames::LiteApi,
                ProviderSteps::HotelDetails,
            ));
        };

        // Create requests for both APIs
        let rates_request = Self::map_domain_info_to_liteapi_rates(&criteria)?;
        let hotel_details_request =
            crate::api::liteapi::l04_one_hotel_detail::LiteApiSingleHotelDetailRequest {
                hotel_id: hotel_id.clone(),
            };

        crate::log!(
            "LiteAPI get_hotel_details: calling both hotel details API and rates API for hotel_id: {}",
            hotel_id
        );

        // Call both APIs concurrently and handle the result
        let (hotel_details_data, rates_response) = match tokio::try_join!(
            self.client.send(hotel_details_request),
            self.client.send(rates_request)
        ) {
            Ok((mut hotel_details_response, rates_response)) => {
                // Populate main_photo from other image fields if it's empty
                hotel_details_response.data.populate_main_photo_if_empty();
                // Check if hotel has room data - if not, skip this hotel
                if hotel_details_response.data.rooms.is_empty() {
                    crate::log!("Hotel {} has no room data, skipping hotel", hotel_id);
                    return Err(ProviderError(Arc::new(ProviderErrorDetails {
                        provider_name: ProviderNames::LiteApi,
                        api_error: ApiError::Other(format!(
                            "Hotel {} has no room data available and should be skipped",
                            hotel_id
                        )),
                        error_step: ProviderSteps::HotelDetails,
                    })));
                }

                // Check if hotel details are empty (name, description, address, etc.)
                if Self::is_hotel_details_empty(&hotel_details_response.data) {
                    crate::log!("Hotel {} has empty details, skipping hotel", hotel_id);
                    return Err(ProviderError(Arc::new(ProviderErrorDetails {
                        provider_name: ProviderNames::LiteApi,
                        api_error: ApiError::Other(format!(
                            "Hotel {} has empty details and should be skipped",
                            hotel_id
                        )),
                        error_step: ProviderSteps::HotelDetails,
                    })));
                }

                crate::log!(
                    "✅ Successfully retrieved both hotel details and rates for hotel_id: {}",
                    hotel_id
                );
                crate::log!(
                    "📊 Hotel has {} rooms, main_photo: '{}', {} facilities, description length: {}",
                    hotel_details_response.data.rooms.len(),
                    if hotel_details_response.data.main_photo.is_empty() { "EMPTY" } else { "Available" },
                    hotel_details_response.data.hotel_facilities.len(),
                    hotel_details_response.data.hotel_description.len()
                );
                (Some(hotel_details_response.data), rates_response)
            }
            Err(e) => {
                // Log detailed failure analysis
                Self::log_api_failure_details(&hotel_id, &e);

                // Try rates API alone as fallback
                let rates_request_fallback = Self::map_domain_info_to_liteapi_rates(&criteria)?;
                let rates_response =
                    self.client
                        .send(rates_request_fallback)
                        .await
                        .map_err(|e: ApiError| {
                            ProviderError::from_api_error(
                                e,
                                ProviderNames::LiteApi,
                                ProviderSteps::HotelDetails,
                            )
                        })?;

                crate::log!(
                    "✅ Successfully retrieved rates as fallback for hotel_id: {} - Hotel details will use basic info",
                    hotel_id
                );
                crate::log!(
                    "ℹ️  Impact: No detailed hotel information (images, description, facilities) available for this hotel"
                );
                (None, rates_response)
            }
        };

        // Map response to domain hotel details using both hotel details and rates data
        Self::map_liteapi_rates_and_details_to_domain(
            rates_response,
            hotel_details_data,
            &criteria.search_criteria,
            None, // No search info available in this context
        )
        .map_err(|e| {
            // Log the error for debugging
            crate::log!("LiteAPI mapping error: {:?}", e);
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

        // Check for room unavailability before mapping
        if liteapi_response.is_no_availability() {
            return Err(ProviderError::from_api_error(
                ApiError::RoomUnavailable(
                    liteapi_response
                        .get_error_message()
                        .unwrap_or("Room is no longer available")
                        .to_string(),
                ),
                ProviderNames::LiteApi,
                ProviderSteps::HotelBlockRoom,
            ));
        }

        // Check for other errors
        if liteapi_response.is_error_response() {
            return Err(ProviderError::from_api_error(
                ApiError::ResponseError(
                    liteapi_response
                        .get_error_message()
                        .unwrap_or("Unknown error from provider")
                        .to_string(),
                ),
                ProviderNames::LiteApi,
                ProviderSteps::HotelBlockRoom,
            ));
        }

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
        // Apply guest contact fallback strategy BEFORE validation
        let guests_with_fallback = LiteApiAdapter::apply_guest_contact_fallback(
            &book_request.guests,
            &book_request.holder,
        );
        let book_request_with_fallback = DomainBookRoomRequest {
            guests: guests_with_fallback,
            ..book_request
        };

        // Validate request after applying fallback
        Self::validate_book_room_request(&book_request_with_fallback)?;

        // Map domain request to LiteAPI book request
        let liteapi_request = Self::map_domain_book_to_liteapi_book(&book_request_with_fallback)?;

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
            &book_request_with_fallback,
        ))
    }

    // #[tracing::instrument(skip(self))]
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
