cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {
        pub mod impl_hotel_provider_trait;
    }
}

use std::sync::Arc;

use crate::api::api_client::ApiClient;
use crate::api::liteapi::{
    liteapi_hotel_rates, liteapi_hotel_search, LiteApiHTTPClient, LiteApiHotelRatesRequest,
    LiteApiHotelRatesResponse, LiteApiHotelResult, LiteApiHotelSearchRequest,
    LiteApiHotelSearchResponse, LiteApiOccupancy,
};
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

#[derive(Clone)]
pub struct LiteApiAdapter {
    client: LiteApiHTTPClient,
}

impl LiteApiAdapter {
    pub fn new(client: LiteApiHTTPClient) -> Self {
        Self { client }
    }

    // --- Mapping functions ---
    fn map_domain_search_to_liteapi(
        domain_criteria: &DomainHotelSearchCriteria,
        ui_filters: UISearchFilters,
    ) -> LiteApiHotelSearchRequest {
        LiteApiHotelSearchRequest {
            country_code: domain_criteria.destination_country_code.clone(),
            city_name: domain_criteria.destination_city_name.clone(), // Assuming this field exists
            offset: 0,
            limit: 50,
        }
    }

    fn map_liteapi_search_to_domain(
        liteapi_response: LiteApiHotelSearchResponse,
    ) -> DomainHotelListAfterSearch {
        DomainHotelListAfterSearch {
            hotel_results: liteapi_response
                .data
                .into_iter()
                .map(|hotel| Self::map_liteapi_hotel_to_domain(hotel))
                .collect(),
        }
    }

    fn map_liteapi_hotel_to_domain(liteapi_hotel: LiteApiHotelResult) -> DomainHotelAfterSearch {
        let hotel_id = liteapi_hotel.id.clone();
        DomainHotelAfterSearch {
            hotel_code: hotel_id.clone(),
            hotel_name: liteapi_hotel.name,
            hotel_category: format!("{} Star", liteapi_hotel.stars),
            star_rating: liteapi_hotel.stars as u8,
            price: DomainPrice {
                room_price: 0.0, // LiteAPI doesn't provide price in search
                currency_code: liteapi_hotel.currency,
            },
            hotel_picture: liteapi_hotel.main_photo,
            result_token: hotel_id,
        }
    }

    // Convert domain search criteria to LiteAPI hotel rates request
    fn map_domain_info_to_liteapi_rates(
        domain_criteria: &DomainHotelInfoCriteria,
    ) -> Result<LiteApiHotelRatesRequest, ProviderError> {
        let search_criteria = &domain_criteria.search_criteria;

        // Convert room guests to occupancies
        let occupancies: Vec<LiteApiOccupancy> = search_criteria
            .room_guests
            .iter()
            .map(|room_guest| {
                let children = if room_guest.no_of_children > 0 {
                    room_guest.children_ages.as_ref().map(|ages| {
                        ages.iter()
                            .filter_map(|age_str| age_str.parse::<u32>().ok())
                            .collect()
                    })
                } else {
                    None
                };

                LiteApiOccupancy {
                    adults: room_guest.no_of_adults,
                    children,
                }
            })
            .collect();

        // Format dates as YYYY-MM-DD
        let checkin = utils::date::date_tuple_to_yyyy_mm_dd(search_criteria.check_in_date.clone());
        let checkout =
            utils::date::date_tuple_to_yyyy_mm_dd(search_criteria.check_out_date.clone());

        // Require hotel_ids to be provided - don't fall back to token
        if domain_criteria.hotel_ids.is_empty() {
            return Err(ProviderError(Arc::new(ProviderErrorDetails {
                provider_name: ProviderNames::LiteApi,
                api_error: ApiError::Other(
                    "hotel_ids is required for LiteAPI hotel rates request.".to_string(),
                ),
                error_step: ProviderSteps::HotelDetails,
            })));
        }

        let hotel_ids = domain_criteria.hotel_ids.clone();

        Ok(LiteApiHotelRatesRequest {
            hotel_ids,
            occupancies,
            currency: "USD".to_string(), // Could be configurable
            guest_nationality: search_criteria.guest_nationality.clone(),
            checkin,
            checkout,
        })
    }

    // Map LiteAPI rates response to domain hotel details
    fn map_liteapi_rates_to_domain_details(
        liteapi_response: LiteApiHotelRatesResponse,
        search_criteria: &DomainHotelSearchCriteria,
    ) -> Result<DomainHotelDetails, ProviderError> {
        // Get first hotel data
        let hotel_data = liteapi_response.get_first_hotel_data().ok_or_else(|| {
            ProviderError(Arc::new(ProviderErrorDetails {
                provider_name: ProviderNames::LiteApi,
                api_error: ApiError::Other("No hotel data found in rates response".to_string()),
                error_step: ProviderSteps::HotelDetails,
            }))
        })?;

        // Get first room type and rate
        let room_type = liteapi_response.get_first_room_type().ok_or_else(|| {
            ProviderError(Arc::new(ProviderErrorDetails {
                provider_name: ProviderNames::LiteApi,
                api_error: ApiError::Other("No room types found in rates response".to_string()),
                error_step: ProviderSteps::HotelDetails,
            }))
        })?;

        let rate = liteapi_response.get_first_rate().ok_or_else(|| {
            ProviderError(Arc::new(ProviderErrorDetails {
                provider_name: ProviderNames::LiteApi,
                api_error: ApiError::Other("No rates found in rates response".to_string()),
                error_step: ProviderSteps::HotelDetails,
            }))
        })?;

        // Extract price from first rate
        let room_price = rate
            .retail_rate
            .total
            .first()
            .map(|amount| amount.amount)
            .unwrap_or(0.0);

        let currency_code = rate
            .retail_rate
            .total
            .first()
            .map(|amount| amount.currency.clone())
            .unwrap_or_else(|| "USD".to_string());

        // Build domain hotel details
        Ok(DomainHotelDetails {
            checkin: format!(
                "{}-{:02}-{:02}",
                search_criteria.check_in_date.0,
                search_criteria.check_in_date.1,
                search_criteria.check_in_date.2
            ),
            checkout: format!(
                "{}-{:02}-{:02}",
                search_criteria.check_out_date.0,
                search_criteria.check_out_date.1,
                search_criteria.check_out_date.2
            ),
            hotel_name: search_criteria.destination_city_name.clone(), // Would need to get from search results
            hotel_code: hotel_data.hotel_id.clone(),
            star_rating: 0,             // Would need to get from search results
            description: String::new(), // LiteAPI doesn't provide in rates endpoint
            hotel_facilities: vec![],
            address: String::new(), // Would need to get from search results
            images: vec![],         // Would need to get from search results
            first_room_details: DomainFirstRoomDetails {
                price: DomainDetailedPrice {
                    published_price: room_price,
                    published_price_rounded_off: room_price,
                    offered_price: room_price,
                    offered_price_rounded_off: room_price,
                    room_price,
                    tax: 0.0,
                    extra_guest_charge: 0.0,
                    child_charge: 0.0,
                    other_charges: 0.0,
                    currency_code: currency_code.clone(),
                },
                room_data: DomainRoomData {
                    room_name: rate.name.clone(),
                    room_unique_id: room_type.room_type_id.clone(),
                    rate_key: rate.rate_id.clone(),
                    offer_id: room_type.offer_id.clone(),
                },
            },
            amenities: vec![],
        })
    }
}
