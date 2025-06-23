cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {
        pub mod impl_hotel_provider_trait;
    }
}

use std::sync::Arc;

use crate::api::api_client::ApiClient;
use crate::api::liteapi::{
    liteapi_hotel_details, liteapi_hotel_rates, liteapi_hotel_search, liteapi_prebook,
    LiteApiError, LiteApiHTTPClient, LiteApiHotelImage, LiteApiHotelRatesRequest,
    LiteApiHotelRatesResponse, LiteApiHotelResult, LiteApiHotelSearchRequest,
    LiteApiHotelSearchResponse, LiteApiOccupancy, LiteApiPrebookRequest, LiteApiPrebookResponse,
    LiteApiSingleHotelDetailData, LiteApiSingleHotelDetailRequest,
    LiteApiSingleHotelDetailResponse,
};
use crate::utils;
use futures::future::{BoxFuture, FutureExt};

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
                room_price: 0.0, // Will be populated by get_hotel_rates in search_hotels
                currency_code: liteapi_hotel.currency,
            },
            hotel_picture: liteapi_hotel.main_photo,
            result_token: hotel_id,
        }
    }

    // Map search results with pricing from rates API
    async fn map_liteapi_search_to_domain_with_pricing(
        &self,
        liteapi_response: LiteApiHotelSearchResponse,
        search_criteria: &DomainHotelSearchCriteria,
    ) -> Result<DomainHotelListAfterSearch, ProviderError> {
        let mut domain_results = Self::map_liteapi_search_to_domain(liteapi_response.clone());

        // (todo): review hotel_search - Extract hotel IDs (max 50 as per plan)
        let hotel_ids: Vec<String> = liteapi_response
            .data
            .iter()
            .take(50)
            .map(|hotel| hotel.id.clone())
            .collect();

        if hotel_ids.is_empty() {
            return Ok(domain_results);
        }

        // Create hotel info criteria for rates call
        let hotel_info_criteria = DomainHotelInfoCriteria {
            // todo: liteapi does not need this token
            token: String::new(),
            hotel_ids: hotel_ids.clone(),
            search_criteria: search_criteria.clone(),
        };

        // Call get_hotel_rates to get pricing
        let rates_request = Self::map_domain_info_to_liteapi_rates(&hotel_info_criteria)?;

        let rates_response: LiteApiHotelRatesResponse = self
            .client
            .send(rates_request)
            .await
            .map_err(|e: ApiError| {
                // Log but don't fail the entire search if rates fail
                ProviderError::from_api_error(e, ProviderNames::LiteApi, ProviderSteps::HotelSearch)
            })?;

        // Log the rates response structure

        if let Some(error) = &rates_response.error {}

        if let Some(data) = &rates_response.data {}

        // Check if rates response indicates no availability
        if rates_response.is_no_availability() {
            return Ok(DomainHotelListAfterSearch {
                hotel_results: vec![],
            });
        }

        // Check if it's an error response
        if rates_response.is_error_response() {
            // Continue processing but without pricing data
        }

        // Merge pricing data into search results
        Self::merge_pricing_into_search_results(&mut domain_results, rates_response.clone());

        // Filter out hotels with zero pricing
        Self::filter_hotels_with_valid_pricing(&mut domain_results);

        Ok(domain_results)
    }

    // Helper method to merge pricing data into search results
    fn merge_pricing_into_search_results(
        domain_results: &mut DomainHotelListAfterSearch,
        rates_response: LiteApiHotelRatesResponse,
    ) {
        // Create a map of hotel_id -> price for quick lookup
        let mut hotel_prices = std::collections::HashMap::new();

        // Only process if we have data (not an error response)
        if let Some(data) = rates_response.data {
            for hotel_data in data {
                if let Some(room_type) = hotel_data.room_types.first() {
                    if let Some(rate) = room_type.rates.first() {
                        if let Some(amount) = rate.retail_rate.suggested_selling_price.first() {
                            hotel_prices.insert(
                                hotel_data.hotel_id.clone(),
                                DomainPrice {
                                    room_price: amount.amount,
                                    currency_code: amount.currency.clone(),
                                },
                            );
                        } else {
                        }
                    } else {
                    }
                } else {
                }
            }
        } else {
        }

        // Update search results with pricing

        for hotel in &mut domain_results.hotel_results {
            if let Some(price) = hotel_prices.get(&hotel.hotel_code) {
                hotel.price = price.clone();
            } else {
            }
        }
    }

    // Filter out hotels with zero pricing from search results
    fn filter_hotels_with_valid_pricing(domain_results: &mut DomainHotelListAfterSearch) {
        let original_count = domain_results.hotel_results.len();
        let hotels_without_pricing = domain_results
            .hotel_results
            .iter()
            .filter(|hotel| hotel.price.room_price <= 0.0)
            .count();

        domain_results
            .hotel_results
            .retain(|hotel| hotel.price.room_price > 0.0);

        let final_count = domain_results.hotel_results.len();

        if hotels_without_pricing > 0 {
            crate::log!(
                "LiteAPI search filtering: Found {} hotels total, {} without pricing ({}%), {} with valid pricing retained",
                original_count,
                hotels_without_pricing,
                if original_count > 0 { (hotels_without_pricing * 100) / original_count } else { 0 },
                final_count
            );
        } else if original_count > 0 {
            crate::log!(
                "LiteAPI search filtering: All {} hotels had valid pricing",
                original_count
            );
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

        // Get hotel IDs - use provided hotel_ids or fall back to token
        let hotel_ids = if !domain_criteria.hotel_ids.is_empty() {
            domain_criteria.hotel_ids.clone()
        } else if !domain_criteria.token.is_empty() {
            // For LiteAPI, the token is the hotel ID from search results
            vec![domain_criteria.token.clone()]
        } else {
            return Err(ProviderError(Arc::new(ProviderErrorDetails {
                provider_name: ProviderNames::LiteApi,
                api_error: ApiError::Other(
                    "No hotel ID available. Either hotel_ids or token must be provided."
                        .to_string(),
                ),
                error_step: ProviderSteps::HotelDetails,
            })));
        };

        Ok(LiteApiHotelRatesRequest {
            hotel_ids,
            occupancies,
            currency: "USD".to_string(), // Could be configurable
            guest_nationality: search_criteria.guest_nationality.clone(),
            checkin,
            checkout,
        })
    }

    // Map facility ID to human-readable name
    fn map_facility_id_to_name(facility_id: i32) -> String {
        match facility_id {
            1 => "Swimming Pool".to_string(),
            2 => "Fitness Center".to_string(),
            3 => "Spa & Wellness".to_string(),
            4 => "Restaurant".to_string(),
            5 => "Bar".to_string(),
            6 => "Free WiFi".to_string(),
            7 => "Parking".to_string(),
            8 => "Business Center".to_string(),
            9 => "Concierge".to_string(),
            10 => "Room Service".to_string(),
            11 => "Laundry Service".to_string(),
            12 => "Air Conditioning".to_string(),
            13 => "Pet Friendly".to_string(),
            14 => "Elevator".to_string(),
            15 => "Non-Smoking Rooms".to_string(),
            16 => "Airport Shuttle".to_string(),
            17 => "Meeting Rooms".to_string(),
            18 => "Childcare".to_string(),
            19 => "Breakfast".to_string(),
            20 => "24-Hour Front Desk".to_string(),
            _ => format!("Facility {}", facility_id),
        }
    }

    // #[tracing::instrument]
    // Map LiteAPI rates response and hotel details to domain hotel details
    fn map_liteapi_rates_and_details_to_domain(
        liteapi_rates_response: LiteApiHotelRatesResponse,
        hotel_details: Option<LiteApiSingleHotelDetailData>,
        search_criteria: &DomainHotelSearchCriteria,
        hotel_info: Option<&LiteApiHotelResult>,
    ) -> Result<DomainHotelDetails, ProviderError> {
        // Check if we have any data
        if liteapi_rates_response
            .data
            .as_deref()
            .map_or(true, |d| d.is_empty())
        {
            return Err(ProviderError(Arc::new(ProviderErrorDetails {
                provider_name: ProviderNames::LiteApi,
                api_error: ApiError::Other(
                    "Empty response from LiteAPI rates endpoint. Hotel may not be available for the selected dates.".to_string()
                ),
                error_step: ProviderSteps::HotelDetails,
            })));
        }

        // Get first hotel data - because we show this on hotel details page, we only request the data for one hotel
        let hotel_data = liteapi_rates_response
            .get_first_hotel_data()
            .ok_or_else(|| {
                ProviderError(Arc::new(ProviderErrorDetails {
                    provider_name: ProviderNames::LiteApi,
                    api_error: ApiError::Other("No hotel data found in rates response".to_string()),
                    error_step: ProviderSteps::HotelDetails,
                }))
            })?;

        // Extract all rooms and rates from LiteAPI response
        let mut all_rooms = Vec::new();

        // Iterate through all room types and their rates
        if let Some(data) = &liteapi_rates_response.data {
            for hotel_data_item in data {
                for room_type in &hotel_data_item.room_types {
                    for rate in &room_type.rates {
                        // Extract price from rate
                        let room_price = rate
                            .retail_rate
                            .suggested_selling_price // we want to use suggested selling price from liteapi
                            .first()
                            .map(|amount| amount.amount)
                            .unwrap_or(0.0);

                        let currency_code = rate
                            .retail_rate
                            .suggested_selling_price
                            .first()
                            .map(|amount| amount.currency.clone())
                            .unwrap_or_else(|| "USD".to_string());

                        // todo: liteapi maybe get meal plan (future)
                        let meal_plan = None;

                        let occupancy_info = Some(crate::domain::DomainRoomOccupancy {
                            max_occupancy: Some(rate.max_occupancy),
                            adult_count: Some(rate.adult_count),
                            child_count: Some(rate.child_count),
                        });

                        // Create room option
                        let room_option = DomainRoomOption {
                            price: crate::domain::DomainDetailedPrice {
                                published_price: room_price,
                                published_price_rounded_off: room_price,
                                offered_price: room_price,
                                offered_price_rounded_off: room_price,
                                room_price,
                                // todo: get tax from liteapi currently deafult values
                                tax: 0.0,
                                extra_guest_charge: 0.0,
                                child_charge: 0.0,
                                other_charges: 0.0,
                                currency_code: currency_code.clone(),
                            },
                            room_data: crate::domain::DomainRoomData {
                                room_name: rate.name.clone(),
                                room_unique_id: room_type.room_type_id.clone(),
                                rate_key: rate.rate_id.clone(),
                                offer_id: room_type.offer_id.clone(),
                            },
                            meal_plan,
                            occupancy_info,
                        };

                        all_rooms.push(room_option);
                    }
                }
            }
        }

        // Ensure we have at least one room
        if all_rooms.is_empty() {
            return Err(ProviderError(Arc::new(ProviderErrorDetails {
                provider_name: ProviderNames::LiteApi,
                api_error: ApiError::Other(
                    "No room types or rates available for this hotel. It may be fully booked for the selected dates.".to_string()
                ),
                error_step: ProviderSteps::HotelDetails,
            })));
        }

        // Extract hotel information - prioritize hotel details response, fallback to search results
        let (hotel_name, star_rating, description, address, images, hotel_facilities) =
            if let Some(details) = hotel_details {
                crate::log!("Using LiteAPI hotel details data - Name: {}, Description length: {}, Address length: {}, Images count: {}, Facilities count: {}", 
                    details.name, details.hotel_description.len(), details.address.len(), details.hotel_images.len(), details.hotel_facilities.len());
                // Use comprehensive hotel details data
                let mut images = Vec::new();

                // Add main_photo if available
                if !details.main_photo.is_empty() {
                    images.push(details.main_photo.clone());
                }

                // Add thumbnail if available and different from main_photo
                if let Some(thumbnail) = &details.thumbnail {
                    if !thumbnail.is_empty() && thumbnail != &details.main_photo {
                        images.push(thumbnail.clone());
                    }
                }

                // Add all hotel images from the hotelImages array
                for hotel_image in &details.hotel_images {
                    if !hotel_image.url.is_empty() && !images.contains(&hotel_image.url) {
                        images.push(hotel_image.url.clone());
                    }
                    // Also add HD version if different
                    if !hotel_image.url_hd.is_empty()
                        && hotel_image.url_hd != hotel_image.url
                        && !images.contains(&hotel_image.url_hd)
                    {
                        images.push(hotel_image.url_hd.clone());
                    }
                }

                (
                    details.name.clone(),
                    details.star_rating,
                    details.hotel_description.clone(),
                    details.address.clone(),
                    images,
                    details.hotel_facilities.clone(),
                )
            } else if let Some(info) = hotel_info {
                crate::log!("Using LiteAPI search results data as fallback - Name: {}, Description length: {}, Address length: {}, Facilities count: {}", 
                    info.name, info.hotel_description.len(), info.address.len(), info.facility_ids.len());
                // Fallback to search results data
                let mut images = Vec::new();

                // Add main_photo if available
                if !info.main_photo.is_empty() {
                    images.push(info.main_photo.clone());
                }

                // Add thumbnail if available and different from main_photo
                if let Some(thumbnail) = &info.thumbnail {
                    if !thumbnail.is_empty() && thumbnail != &info.main_photo {
                        images.push(thumbnail.clone());
                    }
                }

                // Map facility IDs to string descriptions
                let hotel_facilities = info
                    .facility_ids
                    .iter()
                    .map(|id| Self::map_facility_id_to_name(*id))
                    .collect();

                (
                    info.name.clone(),
                    info.stars,
                    info.hotel_description.clone(),
                    info.address.clone(),
                    images,
                    hotel_facilities,
                )
            } else {
                crate::log!("Using enhanced defaults with basic hotel info");
                // Enhanced fallback with basic hotel information
                let hotel_name = format!("Hotel in {}", search_criteria.destination_city_name);
                let description = format!("A quality hotel located in {}, {}. This property offers comfortable accommodations and excellent service for your stay.", 
                    search_criteria.destination_city_name, search_criteria.destination_country_name);
                let address = format!(
                    "{}, {}",
                    search_criteria.destination_city_name, search_criteria.destination_country_name
                );

                // Add some default images - these could be placeholder images
                let images = vec![
                    "https://images.unsplash.com/photo-1566073771259-6a8506099945?w=800"
                        .to_string(),
                    "https://images.unsplash.com/photo-1564501049412-61c2a3083791?w=800"
                        .to_string(),
                ];

                // Add common hotel facilities
                let hotel_facilities = vec![
                    "Free WiFi".to_string(),
                    "Air Conditioning".to_string(),
                    "24-Hour Front Desk".to_string(),
                    "Room Service".to_string(),
                ];

                (
                    hotel_name,
                    3, // Default 3-star rating
                    description,
                    address,
                    images,
                    hotel_facilities,
                )
            };

        // Build domain hotel details
        let domain_hotel_details = DomainHotelDetails {
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
            hotel_name: hotel_name.clone(),
            hotel_code: hotel_data.hotel_id.clone(),
            star_rating,
            description: description.clone(),
            hotel_facilities: hotel_facilities.clone(),
            address: address.clone(),
            images: images.clone(),
            all_rooms,
            amenities: vec![],
        };

        // crate::log!("Final domain hotel details - Name: {}, Description length: {}, Address length: {}, Images count: {}, Facilities count: {}",
        //     domain_hotel_details.hotel_name, domain_hotel_details.description.len(), domain_hotel_details.address.len(),
        //     domain_hotel_details.images.len(), domain_hotel_details.hotel_facilities.len());

        Ok(domain_hotel_details)
    }

    // Legacy method for backward compatibility
    fn map_liteapi_rates_to_domain_details(
        liteapi_response: LiteApiHotelRatesResponse,
        search_criteria: &DomainHotelSearchCriteria,
        hotel_info: Option<&LiteApiHotelResult>,
    ) -> Result<DomainHotelDetails, ProviderError> {
        Self::map_liteapi_rates_and_details_to_domain(
            liteapi_response,
            None, // No hotel details data
            search_criteria,
            hotel_info,
        )
    }

    // Map domain block room request to LiteAPI prebook request
    fn map_domain_block_to_liteapi_prebook(
        domain_request: &DomainBlockRoomRequest,
    ) -> Result<LiteApiPrebookRequest, ProviderError> {
        // Extract offer ID from selected room
        let offer_id = &domain_request.selected_room.offer_id;

        if offer_id.is_empty() {
            return Err(ProviderError(Arc::new(ProviderErrorDetails {
                provider_name: ProviderNames::LiteApi,
                api_error: ApiError::Other(
                    "Offer ID is required for LiteAPI prebook. Ensure hotel details were fetched correctly.".to_string(),
                ),
                error_step: ProviderSteps::HotelBlockRoom,
            })));
        }

        Ok(LiteApiPrebookRequest {
            offer_id: offer_id.clone(),
            use_payment_sdk: false, // Always false for our use case
        })
    }

    // Map LiteAPI prebook response to domain block room response
    fn map_liteapi_prebook_to_domain_block(
        liteapi_response: LiteApiPrebookResponse,
        original_request: &DomainBlockRoomRequest,
    ) -> DomainBlockRoomResponse {
        // Serialize provider data before extracting data to avoid borrow issues
        let provider_data_json = serde_json::to_string(&liteapi_response).unwrap_or_default();

        // At this point, we know data exists because error handling was done in the caller
        let data = liteapi_response
            .data
            .expect("Data should exist after error handling");

        // Extract price information from prebook response
        let total_price = data.price;
        let suggested_selling_price = data.suggested_selling_price;
        let currency = data.currency.clone();

        // Get room details from first room type and rate if available
        let (room_name, cancellation_policy, meal_plan) =
            if let Some(room_type) = data.room_types.first() {
                if let Some(rate) = room_type.rates.first() {
                    (
                        rate.name.clone(),
                        Some(rate.cancellation_policies.refundable_tag.clone()),
                        Some(format!("{} - {}", rate.board_type, rate.board_name)),
                    )
                } else {
                    (original_request.selected_room.room_name.clone(), None, None)
                }
            } else {
                (original_request.selected_room.room_name.clone(), None, None)
            };

        // Create detailed price from prebook response
        let detailed_price = DomainDetailedPrice {
            published_price: suggested_selling_price,
            published_price_rounded_off: suggested_selling_price.round(),
            offered_price: total_price,
            offered_price_rounded_off: total_price.round(),
            room_price: total_price,
            tax: 0.0, // Could extract from taxes_and_fees if needed
            extra_guest_charge: 0.0,
            child_charge: 0.0,
            other_charges: 0.0,
            currency_code: currency.clone(),
        };

        // Create a blocked room entry from the prebook response
        let blocked_room = DomainBlockedRoom {
            room_code: original_request.selected_room.room_unique_id.clone(),
            room_name,
            room_type_code: Some(original_request.selected_room.room_unique_id.clone()),
            price: detailed_price.clone(),
            cancellation_policy,
            meal_plan,
        };

        DomainBlockRoomResponse {
            block_id: data.prebook_id.clone(),
            is_price_changed: data.price_difference_percent != 0.0,
            is_cancellation_policy_changed: data.cancellation_changed,
            blocked_rooms: vec![blocked_room],
            total_price: detailed_price,
            provider_data: Some(provider_data_json),
        }
    }

    // --- Validation Functions ---

    // Validate block room request before processing
    fn validate_block_room_request(request: &DomainBlockRoomRequest) -> Result<(), ProviderError> {
        // Validate guest details match room occupancy
        let total_adults = request.user_details.adults.len() as u32;
        let total_children = request.user_details.children.len() as u32;
        let total_guests_from_details = total_adults + total_children;

        if total_guests_from_details != request.total_guests {
            return Err(ProviderError(Arc::new(ProviderErrorDetails {
                provider_name: ProviderNames::LiteApi,
                api_error: ApiError::Other(format!(
                    "[LITEAPI VALIDATION ERROR] Guest count mismatch. Expected {} guests but got {} in details",
                    request.total_guests, total_guests_from_details
                )),
                error_step: ProviderSteps::HotelBlockRoom,
            })));
        }

        // Validate essential fields are not empty
        if request.selected_room.room_unique_id.is_empty() {
            return Err(ProviderError(Arc::new(ProviderErrorDetails {
                provider_name: ProviderNames::LiteApi,
                api_error: ApiError::Other(
                    "[LITEAPI VALIDATION ERROR] Room unique ID cannot be empty".to_string(),
                ),
                error_step: ProviderSteps::HotelBlockRoom,
            })));
        }

        // Validate at least one adult is present
        if request.user_details.adults.is_empty() {
            return Err(ProviderError(Arc::new(ProviderErrorDetails {
                provider_name: ProviderNames::LiteApi,
                api_error: ApiError::Other(
                    "[LITEAPI VALIDATION ERROR] At least one adult guest is required".to_string(),
                ),
                error_step: ProviderSteps::HotelBlockRoom,
            })));
        }

        // Validate adult details
        for (i, adult) in request.user_details.adults.iter().enumerate() {
            if adult.first_name.trim().is_empty() {
                return Err(ProviderError(Arc::new(ProviderErrorDetails {
                    provider_name: ProviderNames::LiteApi,
                    api_error: ApiError::Other(format!(
                        "[LITEAPI VALIDATION ERROR] Adult {} first name cannot be empty",
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
                    provider_name: ProviderNames::LiteApi,
                    api_error: ApiError::Other(format!(
                        "[LITEAPI VALIDATION ERROR] Child age {} exceeds maximum allowed age of 17",
                        child.age
                    )),
                    error_step: ProviderSteps::HotelBlockRoom,
                })));
            }
        }

        Ok(())
    }

    // <!-- BOOKING MAPPING FUNCTIONS -->

    /// Validates the domain book room request
    /// Apply email and phone fallback strategy for guests
    ///
    /// Strategy:
    /// - Use guest's own email/phone if available
    /// - Otherwise, use the email/phone from the holder (first guest with contact info)
    /// - Keep original guest name and occupancy info
    pub fn apply_guest_contact_fallback(
        guests: &[crate::domain::DomainBookingGuest],
        holder: &crate::domain::DomainBookingHolder,
    ) -> Vec<crate::domain::DomainBookingGuest> {
        crate::log!(
            "Applying guest contact fallback strategy. Holder email: '{}', phone: '{}'",
            holder.email,
            holder.phone
        );

        guests
            .iter()
            .map(|guest| {
                // Use guest's own contact info if available, otherwise fallback to holder's
                let email = if guest.email.is_empty() {
                    crate::log!(
                        "Guest {} {} has empty email, using holder email: '{}'",
                        guest.first_name,
                        guest.last_name,
                        holder.email
                    );
                    holder.email.clone()
                } else {
                    crate::log!(
                        "Guest {} {} has email '{}', using guest's own email",
                        guest.first_name,
                        guest.last_name,
                        guest.email
                    );
                    guest.email.clone()
                };

                let phone = if guest.phone.is_empty() {
                    crate::log!(
                        "Guest {} {} has empty phone, using holder phone: '{}'",
                        guest.first_name,
                        guest.last_name,
                        holder.phone
                    );
                    holder.phone.clone()
                } else {
                    crate::log!(
                        "Guest {} {} has phone '{}', using guest's own phone",
                        guest.first_name,
                        guest.last_name,
                        guest.phone
                    );
                    guest.phone.clone()
                };

                crate::domain::DomainBookingGuest {
                    occupancy_number: guest.occupancy_number,
                    first_name: guest.first_name.clone(),
                    last_name: guest.last_name.clone(),
                    email,
                    phone,
                    remarks: guest.remarks.clone(),
                }
            })
            .collect()
    }

    pub fn validate_book_room_request(
        request: &crate::domain::DomainBookRoomRequest,
    ) -> Result<(), ProviderError> {
        use crate::api::ApiError;
        use crate::ports::hotel_provider_port::{ProviderErrorDetails, ProviderSteps};
        use crate::ports::ProviderNames;
        use std::sync::Arc;

        // Validate block ID is present
        // this is called "prebookId" in the request

        if request.block_id.is_empty() {
            return Err(ProviderError(Arc::new(ProviderErrorDetails {
                provider_name: ProviderNames::LiteApi,
                api_error: ApiError::Other(
                    "[LITEAPI VALIDATION ERROR] Block ID cannot be empty".to_string(),
                ),
                error_step: ProviderSteps::HotelBookRoom,
            })));
        }

        // Validate holder information
        if request.holder.first_name.trim().is_empty() || request.holder.last_name.trim().is_empty()
        {
            return Err(ProviderError(Arc::new(ProviderErrorDetails {
                provider_name: ProviderNames::LiteApi,
                api_error: ApiError::Other(
                    "[LITEAPI VALIDATION ERROR] Holder first name and last name are required"
                        .to_string(),
                ),
                error_step: ProviderSteps::HotelBookRoom,
            })));
        }

        if request.holder.email.trim().is_empty() {
            return Err(ProviderError(Arc::new(ProviderErrorDetails {
                provider_name: ProviderNames::LiteApi,
                api_error: ApiError::Other(
                    "[LITEAPI VALIDATION ERROR] Holder email is required".to_string(),
                ),
                error_step: ProviderSteps::HotelBookRoom,
            })));
        }

        if request.holder.phone.trim().is_empty() {
            return Err(ProviderError(Arc::new(ProviderErrorDetails {
                provider_name: ProviderNames::LiteApi,
                api_error: ApiError::Other(
                    "[LITEAPI VALIDATION ERROR] Holder phone is required".to_string(),
                ),
                error_step: ProviderSteps::HotelBookRoom,
            })));
        }

        // Validate at least one guest
        if request.guests.is_empty() {
            return Err(ProviderError(Arc::new(ProviderErrorDetails {
                provider_name: ProviderNames::LiteApi,
                api_error: ApiError::Other(
                    "[LITEAPI VALIDATION ERROR] At least one guest is required".to_string(),
                ),
                error_step: ProviderSteps::HotelBookRoom,
            })));
        }

        // LITEAPI SPECIFIC VALIDATION: Exactly one primary contact per room requirement
        let number_of_rooms = request.booking_context.number_of_rooms;

        // Validate exactly one guest per room (primary contact/room manager model)
        if (request.guests.len() as u32) != number_of_rooms {
            return Err(ProviderError(Arc::new(ProviderErrorDetails {
                provider_name: ProviderNames::LiteApi,
                api_error: ApiError::Other(format!(
                    "[LITEAPI VALIDATION ERROR] LiteAPI requires exactly one primary contact per room. Expected {} guests for {} rooms, but got {}. Each guest represents the primary contact/room manager for their assigned room.",
                    number_of_rooms, number_of_rooms, request.guests.len()
                )),
                error_step: ProviderSteps::HotelBookRoom,
            })));
        }

        // Validate occupancy numbers are sequential starting from 1
        let mut occupancy_numbers: Vec<u32> =
            request.guests.iter().map(|g| g.occupancy_number).collect();
        occupancy_numbers.sort();

        for (i, &occupancy_num) in occupancy_numbers.iter().enumerate() {
            let expected = (i + 1) as u32;
            if occupancy_num != expected {
                return Err(ProviderError(Arc::new(ProviderErrorDetails {
                    provider_name: ProviderNames::LiteApi,
                    api_error: ApiError::Other(format!(
                        "[LITEAPI VALIDATION ERROR] LiteAPI requires occupancy numbers to be sequential starting from 1. Expected {}, but found {}",
                        expected, occupancy_num
                    )),
                    error_step: ProviderSteps::HotelBookRoom,
                })));
            }
        }

        // Validate no duplicate occupancy numbers
        let unique_occupancy_count = occupancy_numbers
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len();
        if unique_occupancy_count != occupancy_numbers.len() {
            return Err(ProviderError(Arc::new(ProviderErrorDetails {
                provider_name: ProviderNames::LiteApi,
                api_error: ApiError::Other(
                    "[LITEAPI VALIDATION ERROR] LiteAPI requires unique occupancy numbers for each guest".to_string(),
                ),
                error_step: ProviderSteps::HotelBookRoom,
            })));
        }

        // Validate guest details (existing validation)
        for (i, guest) in request.guests.iter().enumerate() {
            if guest.first_name.trim().is_empty() || guest.last_name.trim().is_empty() {
                return Err(ProviderError(Arc::new(ProviderErrorDetails {
                    provider_name: ProviderNames::LiteApi,
                    api_error: ApiError::Other(format!(
                        "[LITEAPI VALIDATION ERROR] Guest {} first name and last name are required",
                        i + 1
                    )),
                    error_step: ProviderSteps::HotelBookRoom,
                })));
            }

            if guest.email.trim().is_empty() {
                return Err(ProviderError(Arc::new(ProviderErrorDetails {
                    provider_name: ProviderNames::LiteApi,
                    api_error: ApiError::Other(format!(
                        "[LITEAPI VALIDATION ERROR] Guest {} email is required",
                        i + 1
                    )),
                    error_step: ProviderSteps::HotelBookRoom,
                })));
            }

            if guest.occupancy_number == 0 {
                return Err(ProviderError(Arc::new(ProviderErrorDetails {
                    provider_name: ProviderNames::LiteApi,
                    api_error: ApiError::Other(format!(
                        "[LITEAPI VALIDATION ERROR] Guest {} occupancy number must be greater than 0",
                        i + 1
                    )),
                    error_step: ProviderSteps::HotelBookRoom,
                })));
            }

            // Validate occupancy number does not exceed number of rooms
            // if guest.occupancy_number > number_of_rooms {
            //     return Err(ProviderError(Arc::new(ProviderErrorDetails {
            //         provider_name: ProviderNames::LiteApi,
            //         api_error: ApiError::Other(format!(
            //             "Guest {} occupancy number {} exceeds the number of rooms being booked ({})",
            //             i + 1, guest.occupancy_number, number_of_rooms
            //         )),
            //         error_step: ProviderSteps::HotelBookRoom,
            //     })));
            // }
        }

        // Validate room occupancies consistency (if provided)
        if !request.booking_context.room_occupancies.is_empty() {
            if request.booking_context.room_occupancies.len() as u32 != number_of_rooms {
                return Err(ProviderError(Arc::new(ProviderErrorDetails {
                    provider_name: ProviderNames::LiteApi,
                    api_error: ApiError::Other(format!(
                        "[LITEAPI VALIDATION ERROR] Room occupancies count ({}) does not match number of rooms ({})",
                        request.booking_context.room_occupancies.len(),
                        number_of_rooms
                    )),
                    error_step: ProviderSteps::HotelBookRoom,
                })));
            }

            // Validate that each room in occupancies has a corresponding guest
            for occupancy in &request.booking_context.room_occupancies {
                let guest_exists = request
                    .guests
                    .iter()
                    .any(|g| g.occupancy_number == occupancy.room_number);
                if !guest_exists {
                    return Err(ProviderError(Arc::new(ProviderErrorDetails {
                        provider_name: ProviderNames::LiteApi,
                        api_error: ApiError::Other(format!(
                            "[LITEAPI VALIDATION ERROR] No guest provided for room {} as required by LiteAPI",
                            occupancy.room_number
                        )),
                        error_step: ProviderSteps::HotelBookRoom,
                    })));
                }
            }
        }

        Ok(())
    }

    /// Maps domain book room request to LiteAPI book request
    pub fn map_domain_book_to_liteapi_book(
        domain_request: &crate::domain::DomainBookRoomRequest,
    ) -> Result<crate::api::liteapi::LiteApiBookRequest, ProviderError> {
        use crate::api::liteapi::l03_book::LiteApiPaymentAddress;
        use crate::api::liteapi::{
            LiteApiBookGuest, LiteApiBookHolder, LiteApiBookMetadata, LiteApiBookRequest,
            LiteApiGuestPayment, LiteApiPayment,
        };
        use crate::domain::DomainPaymentMethod;

        // Map holder
        let holder = LiteApiBookHolder {
            first_name: domain_request.holder.first_name.clone(),
            last_name: domain_request.holder.last_name.clone(),
            email: domain_request.holder.email.clone(),
            phone: domain_request.holder.phone.clone(),
        };

        // Map payment method
        let payment = LiteApiPayment {
            method: match domain_request.payment.method {
                DomainPaymentMethod::AccCreditCard => "ACC_CREDIT_CARD".to_string(),
                DomainPaymentMethod::Wallet => "WALLET".to_string(),
            },
        };

        // Map guests (fallback should already be applied before this function is called)
        let guests: Vec<LiteApiBookGuest> = domain_request
            .guests
            .iter()
            .map(|guest| LiteApiBookGuest {
                occupancy_number: guest.occupancy_number,
                first_name: guest.first_name.clone(),
                last_name: guest.last_name.clone(),
                email: guest.email.clone(),
                phone: guest.phone.clone(),
                remarks: guest.remarks.clone(),
            })
            .collect();

        // Map metadata if present
        // let metadata = domain_request.metadata.as_ref().map(|meta| LiteApiBookMetadata {
        //     ip: meta.ip.clone(),
        //     country: meta.country.clone(),
        //     language: meta.language.clone(),
        //     platform: meta.platform.clone(),
        //     device_id: meta.device_id.clone(),
        //     user_agent: meta.user_agent.clone(),
        //     utm_medium: meta.utm_medium.clone(),
        //     utm_source: meta.utm_source.clone(),
        //     utm_campaign: meta.utm_campaign.clone(),
        // });
        let metadata = None;

        // Map guest payment if present
        let guest_payment = domain_request
            .guest_payment
            .as_ref()
            .map(|gp| LiteApiGuestPayment {
                address: LiteApiPaymentAddress {
                    city: gp.address.city.clone(),
                    address: gp.address.address.clone(),
                    country: gp.address.country.clone(),
                    postal_code: gp.address.postal_code.clone(),
                },
                method: gp.method.clone(),
                phone: gp.phone.clone(),
                payee_first_name: gp.payee_first_name.clone(),
                payee_last_name: gp.payee_last_name.clone(),
                last_4_digits: gp.last_4_digits.clone(),
            });

        Ok(LiteApiBookRequest {
            holder,
            metadata,
            guest_payment,
            payment,
            prebook_id: domain_request.block_id.clone(),
            guests,
            client_reference: domain_request.client_reference.clone(),
        })
    }

    /// Maps LiteAPI book response to domain book room response
    pub fn map_liteapi_book_to_domain_book(
        liteapi_response: crate::api::liteapi::LiteApiBookResponse,
        _original_request: &crate::domain::DomainBookRoomRequest,
    ) -> crate::domain::DomainBookRoomResponse {
        use crate::domain::{
            DomainBookRoomResponse, DomainBookedHotel, DomainBookedPrice, DomainBookedRetailRate,
            DomainBookedRoom, DomainBookedRoomRate, DomainBookingHolder, DomainBookingStatus,
            DomainCancelPolicyInfo, DomainCancellationPolicies, DomainRoomTypeInfo,
        };

        let data = liteapi_response.data;

        // Map booking status
        let status = match data.status.as_str() {
            "CONFIRMED" => DomainBookingStatus::Confirmed,
            "PENDING" => DomainBookingStatus::Pending,
            "FAILED" => DomainBookingStatus::Failed,
            "CANCELLED" => DomainBookingStatus::Cancelled,
            _ => DomainBookingStatus::Pending, // Default to pending for unknown statuses
        };

        // Map hotel
        let hotel = DomainBookedHotel {
            hotel_id: data.hotel.hotel_id,
            name: data.hotel.name,
        };

        // Map booked rooms
        let booked_rooms: Vec<DomainBookedRoom> = data
            .booked_rooms
            .into_iter()
            .map(|room| DomainBookedRoom {
                room_type: DomainRoomTypeInfo {
                    name: room.room_type.name,
                },
                board_type: room.board_type,
                board_name: room.board_name,
                adults: room.adults,
                children: room.children,
                rate: DomainBookedRoomRate {
                    retail_rate: DomainBookedRetailRate {
                        total: DomainBookedPrice {
                            amount: room.rate.retail_rate.total.amount,
                            currency: room.rate.retail_rate.total.currency,
                        },
                    },
                },
                first_name: room.first_name,
                last_name: room.last_name,
                mapped_room_id: room.mapped_room_id,
            })
            .collect();

        // Map holder
        let holder = DomainBookingHolder {
            first_name: data.holder.first_name,
            last_name: data.holder.last_name,
            email: data.holder.email,
            phone: data.holder.phone,
        };

        // Map cancellation policies
        let cancellation_policies = DomainCancellationPolicies {
            cancel_policy_infos: data
                .cancellation_policies
                .cancel_policy_infos
                .into_iter()
                .map(|policy| DomainCancelPolicyInfo {
                    cancel_time: policy.cancel_time,
                    amount: policy.amount,
                    policy_type: policy.policy_type,
                    timezone: policy.timezone,
                    currency: policy.currency,
                })
                .collect(),
            hotel_remarks: data.cancellation_policies.hotel_remarks,
            refundable_tag: data.cancellation_policies.refundable_tag,
        };

        DomainBookRoomResponse {
            booking_id: data.booking_id,
            client_reference: data.client_reference,
            supplier_booking_id: data.supplier_booking_id,
            supplier_booking_name: data.supplier_booking_name,
            supplier: data.supplier,
            supplier_id: data.supplier_id,
            status,
            hotel_confirmation_code: data.hotel_confirmation_code,
            checkin: data.checkin,
            checkout: data.checkout,
            hotel,
            booked_rooms,
            holder,
            created_at: data.created_at,
            cancellation_policies,
            price: data.price,
            commission: data.commission,
            currency: data.currency,
            special_remarks: data.special_remarks,
            optional_fees: data.optional_fees,
            mandatory_fees: data.mandatory_fees,
            know_before_you_go: data.know_before_you_go,
            remarks: data.remarks,
            guest_id: data.guest_id,
        }
    }
}
