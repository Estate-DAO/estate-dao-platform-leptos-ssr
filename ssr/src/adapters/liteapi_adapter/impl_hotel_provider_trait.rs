use std::sync::Arc;

use crate::adapters::LiteApiAdapter;
use crate::api::api_client::ApiClient;
use crate::api::liteapi::{
    liteapi_book_room, liteapi_get_booking_details, liteapi_hotel_details, liteapi_hotel_rates,
    liteapi_hotel_search, liteapi_prebook, LiteApiBookRequest, LiteApiBookResponse,
    LiteApiGetBookingRequest, LiteApiGetBookingResponse, LiteApiHTTPClient,
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
    DomainDetailedPrice, DomainFirstRoomDetails, DomainGetBookingRequest, DomainGetBookingResponse,
    DomainHotelAfterSearch, DomainHotelDetails, DomainHotelInfoCriteria,
    DomainHotelListAfterSearch, DomainHotelSearchCriteria, DomainHotelStaticDetails, DomainPrice,
    DomainRoomData, DomainRoomOption,
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

        crate::log!(
            "LiteAPI search_hotels: received response with {} hotels",
            liteapi_response.data.len()
        );

        // Use the new method that includes pricing from rates API
        self.map_liteapi_search_to_domain_with_pricing(liteapi_response, &criteria)
            .await
    }

    #[tracing::instrument(skip(self))]
    async fn get_hotel_static_details(
        &self,
        hotel_id: &str,
    ) -> Result<DomainHotelStaticDetails, ProviderError> {
        let hotel_details_request =
            crate::api::liteapi::l04_one_hotel_detail::LiteApiSingleHotelDetailRequest {
                hotel_id: hotel_id.to_string(),
            };

        let hotel_details_response: LiteApiSingleHotelDetailResponse = self
            .client
            .send(hotel_details_request)
            .await
            .map_err(|e: ApiError| {
                ProviderError::from_api_error(
                    e,
                    ProviderNames::LiteApi,
                    ProviderSteps::HotelDetails,
                )
            })?;

        Ok(Self::map_liteapi_to_domain_static_details(
            hotel_details_response.data,
        ))
    }

    #[tracing::instrument(skip(self))]
    async fn get_hotel_rates(
        &self,
        criteria: DomainHotelInfoCriteria,
    ) -> Result<Vec<DomainRoomOption>, ProviderError> {
        let rates_request = Self::map_domain_info_to_liteapi_rates(&criteria)?;
        let rates_response: LiteApiHotelRatesResponse = self
            .client
            .send(rates_request)
            .await
            .map_err(|e: ApiError| {
                ProviderError::from_api_error(e, ProviderNames::LiteApi, ProviderSteps::HotelRate)
            })?;

        let hotel_rate_info = rates_response
            .data
            .and_then(|d| d.into_iter().next())
            .ok_or_else(|| {
                ProviderError::from_api_error(
                    ApiError::Other(
                        "Hotel may not be available for the selected dates.".to_string(),
                    ),
                    ProviderNames::LiteApi,
                    ProviderSteps::HotelRate,
                )
            })?;

        let all_rooms = hotel_rate_info
            .room_types
            .into_iter()
            .flat_map(|rt| {
                let offer_retail_rate = rt.offer_retail_rate.clone();
                rt.rates.into_iter().map(move |r| {
                    (
                        rt.room_type_id.clone(),
                        rt.offer_id.clone(),
                        offer_retail_rate.clone(),
                        r,
                    )
                })
            })
            .map(|(room_type_id, offer_id, offer_retail_rate, rate)| {
                Self::map_liteapi_room_to_domain(
                    rate,
                    room_type_id,
                    offer_id,
                    Some(offer_retail_rate),
                )
            })
            .collect();

        Ok(all_rooms)
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
    async fn get_booking_details(
        &self,
        request: DomainGetBookingRequest,
    ) -> Result<DomainGetBookingResponse, ProviderError> {
        // Convert domain request to LiteAPI request
        let liteapi_request = Self::map_domain_get_booking_to_liteapi(&request)?;

        crate::log!(
            "LiteAPI get_booking_details: calling API with request: {:#?}",
            liteapi_request
        );

        // Call LiteAPI get booking details endpoint
        let liteapi_response: LiteApiGetBookingResponse = self
            .client
            .send(liteapi_request)
            .await
            .map_err(|e: ApiError| {
            ProviderError::from_api_error(
                e,
                ProviderNames::LiteApi,
                ProviderSteps::GetBookingDetails,
            )
        })?;

        crate::log!(
            "LiteAPI get_booking_details: received response with {} bookings",
            liteapi_response.data.len()
        );

        // Map LiteAPI response to domain response
        Self::map_liteapi_get_booking_to_domain(liteapi_response).map_err(|e| {
            crate::log!("LiteAPI get booking details mapping error: {:?}", e);
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
