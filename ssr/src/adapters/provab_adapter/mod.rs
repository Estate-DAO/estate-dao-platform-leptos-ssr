use crate::api::api_client::ApiClient;
use crate::api::provab::Provab;
pub mod impl_hotel_provider_trait;

use std::sync::Arc;

use crate::api::provab::a01_hotel_info::{
    FirstRoomDetails as ProvabFirstRoomDetails, HotelDetailsLevel1 as ProvabHotelDetailsLevel1,
    HotelDetailsLevel2 as ProvabHotelDetailsLevel2, HotelInfoResult as ProvabHotelInfoResult,
    Price as ProvabDetailedPrice, RoomData as ProvabRoomData,
};
use futures::future::{BoxFuture, FutureExt};

use crate::api::provab::a00_search::{
    HotelResult as ProvabHotelResult, HotelSearchRequest as ProvabHotelSearchRequest,
    HotelSearchResponse as ProvabHotelSearchResponse, HotelSearchResult as ProvabHotelSearchResult,
    Price as ProvabPrice, RoomGuest as ProvabRoomGuest, Search as ProvabSearch,
};
use crate::api::provab::a01_hotel_info::{
    HotelInfoRequest as ProvabHotelInfoRequest, HotelInfoResponse as ProvabHotelInfoResponse,
};
use crate::api::provab::a03_block_room::{
    BlockRoomRequest as ProvabBlockRoomRequest, BlockRoomResponse as ProvabBlockRoomResponse,
    HotelRoomDetail as ProvabHotelRoomDetail,
};
use crate::api::ApiError;
use crate::application_services::filter_types::UISearchFilters;
use crate::domain::{
    DomainBlockRoomRequest, DomainBlockRoomResponse, DomainBlockedRoom, DomainDetailedPrice,
    DomainFirstRoomDetails, DomainHotelAfterSearch, DomainHotelDetails, DomainHotelInfoCriteria,
    DomainHotelListAfterSearch, DomainHotelSearchCriteria, DomainPrice, DomainRoomData,
    DomainRoomOccupancy, DomainRoomOption,
};
use crate::ports::hotel_provider_port::{ProviderError, ProviderErrorDetails, ProviderSteps};
use crate::ports::ProviderNames;
use crate::utils::date::date_tuple_to_dd_mm_yyyy;
use async_trait::async_trait;

#[derive(Clone)]
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
        ui_filters: UISearchFilters, // <!-- New parameter for UI filters -->
    ) -> ProvabHotelSearchRequest {
        // <!-- Start with core criteria mapping -->
        let provab_request = ProvabHotelSearchRequest {
            check_in_date: date_tuple_to_dd_mm_yyyy(domain_criteria.check_in_date.clone()),
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
    ) -> DomainHotelListAfterSearch {
        match provab_response.search {
            Some(search) => DomainHotelListAfterSearch {
                hotel_results: search
                    .hotel_search_result
                    .hotel_results
                    .into_iter()
                    .map(|hotel| Self::map_provab_hotel_to_domain(hotel))
                    .collect(),
                pagination: None, // Provab adapter does not support pagination yet
            },
            None => DomainHotelListAfterSearch {
                hotel_results: vec![],
                pagination: None, // No results, no pagination
            },
        }
    }

    fn map_provab_hotel_to_domain(provab_hotel: ProvabHotelResult) -> DomainHotelAfterSearch {
        DomainHotelAfterSearch {
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
            all_rooms: vec![Self::map_provab_first_room_details_to_room_option(
                provab_details.first_room_details,
            )],
            amenities: provab_details.amenities,
        }
    }

    fn map_provab_hotel_info_to_domain(
        provab_response: ProvabHotelInfoResponse,
    ) -> Result<DomainHotelDetails, ProviderError> {
        // Check if the response status indicates success (typically 200 or 1)
        if provab_response.status != 200 && provab_response.status != 1 {
            return Err(ProviderError(Arc::new(ProviderErrorDetails {
                provider_name: ProviderNames::Provab,
                api_error: ApiError::ResponseNotOK(provab_response.message.clone()),
                error_step: ProviderSteps::HotelDetails,
            })));
        }

        match provab_response.hotel_details {
            Some(details) => Ok(Self::map_provab_hotel_details_level2_to_domain(
                details.hotel_info_result.hotel_details,
            )),
            None => Err(ProviderError(Arc::new(ProviderErrorDetails {
                provider_name: ProviderNames::Provab,
                api_error: ApiError::Other("No hotel details found in response".to_string()),
                error_step: ProviderSteps::HotelDetails,
            }))),
        }
    }

    fn map_provab_first_room_details_to_room_option(
        provab_room: ProvabFirstRoomDetails,
    ) -> DomainRoomOption {
        DomainRoomOption {
            price: Self::map_provab_detailed_price_to_domain(provab_room.price),
            room_data: Self::map_provab_room_data_to_domain(
                provab_room.room_data,
                provab_room.room_name,
            ),
            meal_plan: None, // Provab doesn't provide meal plan info in first room details
            occupancy_info: None, // Provab doesn't provide occupancy info in first room details
        }
    }

    // Keep the old function for backward compatibility if needed elsewhere
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
            offer_id: String::new(), // Provab doesn't have offer_id, use empty string
        }
    }

    // --- Block Room Mapping Functions ---

    // Map domain block room request to Provab format
    fn map_domain_block_to_provab(
        domain_request: &DomainBlockRoomRequest,
    ) -> ProvabBlockRoomRequest {
        ProvabBlockRoomRequest {
            token: domain_request.hotel_info_criteria.token.clone(),
            room_unique_id: vec![domain_request.selected_room.room_unique_id.clone()],
        }
    }

    // Map Provab block room response to domain format
    fn map_provab_block_to_domain(
        provab_response: ProvabBlockRoomResponse,
        original_request: &DomainBlockRoomRequest,
    ) -> Result<DomainBlockRoomResponse, ProviderError> {
        match provab_response {
            ProvabBlockRoomResponse::Success(success) => {
                // Map each blocked room from Provab response
                let blocked_rooms: Vec<DomainBlockedRoom> = success
                    .get_block_room_hotel_details()
                    .into_iter()
                    .map(|room| DomainBlockedRoom {
                        room_code: room.room_code.clone(),
                        room_name: room.room_type_name.clone(),
                        room_type_code: room.room_type_code.clone(),
                        price: DomainDetailedPrice {
                            published_price: 0.0, // Not provided by Provab in block response
                            published_price_rounded_off: 0.0,
                            offered_price: room.get_offered_price(),
                            offered_price_rounded_off: room.get_offered_price().round(),
                            room_price: room.get_offered_price(),
                            tax: 0.0, // Not provided in block response
                            extra_guest_charge: 0.0,
                            child_charge: 0.0,
                            other_charges: 0.0,
                            currency_code: "USD".to_string(), // Default currency since it's private
                        },
                        cancellation_policy: None, // Would need to extract if available
                        meal_plan: None,           // Would need to extract if available
                    })
                    .collect();

                // Calculate total price
                let total_price_amount = blocked_rooms
                    .iter()
                    .map(|room| room.price.offered_price)
                    .sum();

                let currency = "USD".to_string(); // Use default since currency_code is private

                Ok(DomainBlockRoomResponse {
                    block_id: success.get_block_room_id().unwrap_or_default(),
                    is_price_changed: success.block_room.block_room_result.is_price_changed,
                    is_cancellation_policy_changed: success
                        .block_room
                        .block_room_result
                        .is_cancellation_policy_changed,
                    blocked_rooms,
                    total_price: DomainDetailedPrice {
                        published_price: total_price_amount,
                        published_price_rounded_off: total_price_amount.round(),
                        offered_price: total_price_amount,
                        offered_price_rounded_off: total_price_amount.round(),
                        room_price: total_price_amount,
                        tax: 0.0,
                        extra_guest_charge: 0.0,
                        child_charge: 0.0,
                        other_charges: 0.0,
                        currency_code: currency,
                    },
                    provider_data: Some(serde_json::to_string(&success).unwrap_or_default()),
                })
            }
            ProvabBlockRoomResponse::Failure(failure) => {
                Err(ProviderError(Arc::new(ProviderErrorDetails {
                    provider_name: ProviderNames::Provab,
                    api_error: ApiError::Other(
                        failure
                            .message
                            .unwrap_or_else(|| "Block room failed".to_string()),
                    ),
                    error_step: ProviderSteps::HotelBlockRoom,
                })))
            }
        }
    }

    // --- Validation Functions ---

    // Validate block room request before processing
    fn validate_block_room_request(request: &DomainBlockRoomRequest) -> Result<(), ProviderError> {
        // Validate token is not empty (Provab specific)
        if request.hotel_info_criteria.token.is_empty() {
            return Err(ProviderError(Arc::new(ProviderErrorDetails {
                provider_name: ProviderNames::Provab,
                api_error: ApiError::Other(
                    "Result token cannot be empty for Provab block room".to_string(),
                ),
                error_step: ProviderSteps::HotelBlockRoom,
            })));
        }

        // Validate guest details match room occupancy
        let total_adults = request.user_details.adults.len() as u32;
        let total_children = request.user_details.children.len() as u32;
        let total_guests_from_details = total_adults + total_children;

        if total_guests_from_details != request.total_guests {
            return Err(ProviderError(Arc::new(ProviderErrorDetails {
                provider_name: ProviderNames::Provab,
                api_error: ApiError::Other(format!(
                    "Guest count mismatch. Expected {} guests but got {} in details",
                    request.total_guests, total_guests_from_details
                )),
                error_step: ProviderSteps::HotelBlockRoom,
            })));
        }

        // Validate essential fields are not empty
        if request.selected_room.room_unique_id.is_empty() {
            return Err(ProviderError(Arc::new(ProviderErrorDetails {
                provider_name: ProviderNames::Provab,
                api_error: ApiError::Other("Room unique ID cannot be empty".to_string()),
                error_step: ProviderSteps::HotelBlockRoom,
            })));
        }

        // Validate at least one adult is present
        if request.user_details.adults.is_empty() {
            return Err(ProviderError(Arc::new(ProviderErrorDetails {
                provider_name: ProviderNames::Provab,
                api_error: ApiError::Other("At least one adult guest is required".to_string()),
                error_step: ProviderSteps::HotelBlockRoom,
            })));
        }

        // Validate adult details
        for (i, adult) in request.user_details.adults.iter().enumerate() {
            if adult.first_name.trim().is_empty() {
                return Err(ProviderError(Arc::new(ProviderErrorDetails {
                    provider_name: ProviderNames::Provab,
                    api_error: ApiError::Other(format!(
                        "Adult {} first name cannot be empty",
                        i + 1
                    )),
                    error_step: ProviderSteps::HotelBlockRoom,
                })));
            }
        }

        // Validate children ages
        for child in &request.user_details.children {
            if child.age > 17 {
                return Err(ProviderError(Arc::new(ProviderErrorDetails {
                    provider_name: ProviderNames::Provab,
                    api_error: ApiError::Other(format!(
                        "Child age {} exceeds maximum allowed age of 17",
                        child.age
                    )),
                    error_step: ProviderSteps::HotelBlockRoom,
                })));
            }
        }

        Ok(())
    }
}
