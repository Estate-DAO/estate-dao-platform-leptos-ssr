use crate::api::provab::a01_hotel_info::{
    FirstRoomDetails as ProvabFirstRoomDetails, HotelDetailsLevel1 as ProvabHotelDetailsLevel1,
    HotelDetailsLevel2 as ProvabHotelDetailsLevel2, HotelInfoResult as ProvabHotelInfoResult,
    Price as ProvabDetailedPrice, RoomData as ProvabRoomData,
};
use crate::api::provab::{
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
use crate::application_services::filter_types::UISearchFilters;
use crate::domain::{
    DomainDetailedPrice, DomainFirstRoomDetails, DomainHotelDetails, DomainHotelInfoCriteria,
    DomainHotelResult, DomainHotelSearchCriteria, DomainHotelSearchResponse,
    DomainHotelSearchResult, DomainPrice, DomainRoomData,
};
use crate::ports::hotel_provider_port::{HotelProviderPort, ProviderError};
use async_trait::async_trait;
// use crate::api::ApiClient; // If your Provab client uses this

pub struct ProvabAdapter {
    client: Provab, // Your existing Provab HTTP client
}

impl ProvabAdapter {
    pub fn new(client: Provab) -> Self {
        Self { client }
    }

    // --- Mapping functions ---
    fn map_domain_search_to_provab(
        domain_criteria: &DomainHotelSearchCriteria,
        ui_filters: &UISearchFilters, // <!-- New parameter for UI filters -->
    ) -> ProvabHotelSearchRequest {
        // <!-- Start with core criteria mapping -->
        let mut provab_request = ProvabHotelSearchRequest {
            check_in_date: domain_criteria.check_in_date.clone(),
            no_of_nights: domain_criteria.no_of_nights,
            country_code: domain_criteria.destination_country_code.clone(),
            city_id: domain_criteria.destination_city_id,
            guest_nationality: domain_criteria.guest_nationality.clone(),
            no_of_rooms: domain_criteria.no_of_rooms,
            room_guests: domain_criteria
                .room_guests
                .iter()
                .map(|guest| ProvabRoomGuest {
                    no_of_adults: guest.no_of_adults,
                    no_of_child: guest.no_of_children,
                    child_age: guest.children_ages.clone(),
                })
                .collect(),
        };

        // <!-- Try to apply UI filters if Provab supports them -->
        // <!-- Note: Current Provab API structure doesn't seem to support these directly -->
        // <!-- So they will be handled in the HotelService post-fetch -->

        // <!-- If Provab API supported star rating filter, we would do: -->
        // if let Some(min_rating) = ui_filters.min_star_rating {
        //     provab_request.min_star_rating = Some(min_rating);
        // }

        // <!-- If Provab API supported price range, we would do: -->
        // if let Some(max_price) = ui_filters.max_price_per_night {
        //     provab_request.max_price = Some(max_price);
        // }

        provab_request
    }

    fn map_provab_search_to_domain(
        provab_response: ProvabHotelSearchResponse,
    ) -> DomainHotelSearchResponse {
        DomainHotelSearchResponse {
            status: provab_response.status,
            message: provab_response.message,
            search: provab_response
                .search
                .map(|search| DomainHotelSearchResult {
                    hotel_results: search
                        .hotel_search_result
                        .hotel_results
                        .into_iter()
                        .map(|hotel| Self::map_provab_hotel_to_domain(hotel))
                        .collect(),
                }),
        }
    }

    fn map_provab_hotel_to_domain(provab_hotel: ProvabHotelResult) -> DomainHotelResult {
        DomainHotelResult {
            hotel_code: provab_hotel.hotel_code,
            hotel_name: provab_hotel.hotel_name,
            hotel_category: provab_hotel.hotel_category,
            star_rating: provab_hotel.star_rating,
            price: DomainPrice {
                room_price: provab_hotel.price.room_price,
                currency_code: provab_hotel.price.currency_code,
            },
            hotel_picture: provab_hotel.hotel_picture,
            result_token: provab_hotel.result_token,
        }
    }

    fn map_domain_hotel_info_to_provab(
        domain_criteria: &DomainHotelInfoCriteria,
    ) -> ProvabHotelInfoRequest {
        ProvabHotelInfoRequest {
            token: domain_criteria.token.clone(),
        }
    }

    fn map_provab_hotel_details_level2_to_domain(
        provab_details: ProvabHotelDetailsLevel2,
    ) -> DomainHotelDetails {
        DomainHotelDetails {
            checkin: provab_details.checkin,
            checkout: provab_details.checkout,
            hotel_name: provab_details.hotel_name,
            hotel_code: provab_details.hotel_code,
            star_rating: provab_details.star_rating,
            description: provab_details.description,
            hotel_facilities: provab_details.hotel_facilities,
            address: provab_details.address,
            images: provab_details.images,
            first_room_details: Self::map_provab_first_room_details_to_domain(
                provab_details.first_room_details,
            ),
            amenities: provab_details.amenities,
        }
    }

    fn map_provab_hotel_info_to_domain(
        provab_response: ProvabHotelInfoResponse,
    ) -> Result<DomainHotelDetails, ProviderError> {
        // Check if the response status indicates success (typically 200 or 1)
        if provab_response.status != 200 && provab_response.status != 1 {
            return Err(ProviderError(format!(
                "Hotel info request failed: {}",
                provab_response.message
            )));
        }

        match provab_response.hotel_details {
            Some(details) => Ok(Self::map_provab_hotel_details_level2_to_domain(
                details.hotel_info_result.hotel_details,
            )),
            None => Err(ProviderError(
                "No hotel details found in response".to_string(),
            )),
        }
    }

    fn map_provab_first_room_details_to_domain(
        provab_room: ProvabFirstRoomDetails,
    ) -> DomainFirstRoomDetails {
        DomainFirstRoomDetails {
            price: Self::map_provab_detailed_price_to_domain(provab_room.price),
            room_data: Self::map_provab_room_data_to_domain(
                provab_room.room_data,
                provab_room.room_name,
            ),
        }
    }

    fn map_provab_detailed_price_to_domain(
        provab_price: ProvabDetailedPrice,
    ) -> DomainDetailedPrice {
        DomainDetailedPrice {
            published_price: provab_price.published_price,
            published_price_rounded_off: provab_price.published_price_rounded_off,
            offered_price: provab_price.offered_price,
            offered_price_rounded_off: provab_price.offered_price_rounded_off,
            room_price: provab_price.room_price,
            tax: provab_price.tax,
            extra_guest_charge: provab_price.extra_guest_charge,
            child_charge: provab_price.child_charge,
            other_charges: provab_price.other_charges,
            currency_code: provab_price.currency_code,
        }
    }

    fn map_provab_room_data_to_domain(
        provab_room_data: ProvabRoomData,
        room_name: String,
    ) -> DomainRoomData {
        DomainRoomData {
            room_name,
            room_unique_id: provab_room_data.room_unique_id,
            rate_key: provab_room_data.rate_key,
        }
    }
}

#[async_trait]
impl HotelProviderPort for ProvabAdapter {
    async fn search_hotels(
        &self,
        criteria: DomainHotelSearchCriteria,
        ui_filters: &UISearchFilters,
    ) -> Result<DomainHotelSearchResponse, ProviderError> {
        let provab_request = Self::map_domain_search_to_provab(&criteria, ui_filters);
        let provab_response = self
            .client
            .send(provab_request)
            .await
            .map_err(|e| ProviderError(format!("Provab search failed: {}", e)))?;
        Ok(Self::map_provab_search_to_domain(provab_response))
    }

    async fn get_hotel_details(
        &self,
        criteria: DomainHotelInfoCriteria,
    ) -> Result<DomainHotelDetails, ProviderError> {
        let provab_request = Self::map_domain_hotel_info_to_provab(&criteria);
        let provab_response = self
            .client
            .send(provab_request)
            .await
            .map_err(|e| ProviderError(format!("Provab hotel info failed: {}", e)))?;
        Self::map_provab_hotel_info_to_domain(provab_response)
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
}
