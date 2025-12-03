cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {
        pub mod impl_hotel_provider_trait;
        pub mod impl_place_provider_trait;
    }
}

use std::sync::Arc;
mod map_facility_id;

use crate::api::api_client::ApiClient;
use crate::api::consts::PAGINATION_LIMIT;
use crate::api::liteapi::l04_one_hotel_detail::LiteApiRoom;
use crate::api::liteapi::{
    liteapi_hotel_details, liteapi_hotel_rates, liteapi_hotel_search, liteapi_prebook,
    LiteApiError, LiteApiGetBookingRequest, LiteApiGetBookingResponse, LiteApiGetPlaceRequest,
    LiteApiGetPlaceResponse, LiteApiGetPlacesRequest, LiteApiGetPlacesResponse, LiteApiHTTPClient,
    LiteApiHotelImage, LiteApiHotelRatesRequest, LiteApiHotelRatesResponse, LiteApiHotelResult,
    LiteApiHotelSearchRequest, LiteApiHotelSearchResponse, LiteApiOccupancy, LiteApiPrebookRequest,
    LiteApiPrebookResponse, LiteApiSingleHotelDetailData, LiteApiSingleHotelDetailRequest,
    LiteApiSingleHotelDetailResponse,
};
use crate::utils;
use futures::future::{BoxFuture, FutureExt};
#[cfg(feature = "debug_log")]
use tracing::instrument;

use crate::api::ApiError;
use crate::application_services::filter_types::UISearchFilters;
use crate::domain::*;
use crate::ports::hotel_provider_port::{ProviderError, ProviderErrorDetails, ProviderSteps};
use crate::ports::traits::PlaceProviderPort;
use crate::ports::ProviderNames;
use crate::utils::date::date_tuple_to_dd_mm_yyyy;
use async_trait::async_trait;
use tracing::{error, info, warn};

#[derive(Clone)]
pub struct LiteApiAdapter {
    client: LiteApiHTTPClient,
}

impl LiteApiAdapter {
    pub fn new(client: LiteApiHTTPClient) -> Self {
        Self { client }
    }

    // --- Helper functions ---
    fn is_hotel_details_empty(hotel_details: &LiteApiSingleHotelDetailData) -> bool {
        // Check if essential hotel details are empty or meaningless
        let name_empty = hotel_details.name.trim().is_empty();
        let description_empty = hotel_details.hotel_description.trim().is_empty();
        let address_empty = hotel_details.address.trim().is_empty();

        // A hotel has no images only if ALL image fields are empty (using &&, not ||)
        let no_images = hotel_details.main_photo.trim().is_empty()
            && hotel_details.hotel_images.is_empty()
            && hotel_details
                .thumbnail
                .as_ref()
                .is_none_or(|t| t.trim().is_empty());

        // <!-- Commented out image-based filtering -->
        // let is_empty = name_empty || (description_empty && address_empty && no_images);
        let is_empty = name_empty || (description_empty && address_empty);

        // Add comprehensive logging to understand why hotels are considered empty
        if is_empty {
            warn!(
                hotel_id = %hotel_details.id,
                name_empty = %name_empty,
                description_empty = %description_empty,
                address_empty = %address_empty,
                no_images = %no_images,
                main_photo_empty = %hotel_details.main_photo.trim().is_empty(),
                hotel_images_count = %hotel_details.hotel_images.len(),
                thumbnail_empty = %hotel_details.thumbnail.as_ref().is_none_or(|t| t.trim().is_empty()),
                "Hotel details considered empty"
            );
        } else {
            info!(
                hotel_id = %hotel_details.id,
                name_empty = %name_empty,
                description_empty = %description_empty,
                address_empty = %address_empty,
                no_images = %no_images,
                main_photo_empty = %hotel_details.main_photo.trim().is_empty(),
                hotel_images_count = %hotel_details.hotel_images.len(),
                thumbnail_empty = %hotel_details.thumbnail.as_ref().is_none_or(|t| t.trim().is_empty()),
                "Hotel details validation passed"
            );
        }

        is_empty
    }

    fn log_api_failure_details(hotel_id: &str, error: &ApiError) {
        // Enhanced logging to understand which API failed and why
        crate::log!(
            "❌ LiteAPI parallel call failure for hotel_id: {} - Detailed analysis:",
            hotel_id
        );

        // Try to determine which API specifically failed
        crate::log!("🔍 Error details: {:?}", error);

        // Check if this is a deserialization error (like missing main_photo)
        if let ApiError::JsonParseFailed(ref json_error) = error {
            crate::log!(
                "📝 JSON Deserialization Error - This likely means the hotel details API response is missing expected fields"
            );
            crate::log!("🔧 JSON Error Details: {}", json_error);

            if json_error.contains("main_photo") {
                crate::log!(
                    "📸 Specific Issue: main_photo field is missing from hotel details response"
                );
            }
            if json_error.contains("missing field") {
                crate::log!("🏷️  Missing Field Error: The API response structure doesn't match our expected schema");
            }
        } else {
            crate::log!("🌐 Non-JSON Error: This could be a network issue, HTTP error, or other API problem");
        }

        crate::log!(
            "🔄 Attempting fallback to rates-only API for hotel_id: {}",
            hotel_id
        );
    }

    fn log_pricing_filter_results(
        original_count: usize,
        hotels_without_pricing_ids: &[String],
        final_count: usize,
        rates_response: &LiteApiHotelRatesResponse,
    ) {
        let hotels_without_pricing_count = hotels_without_pricing_ids.len();

        if hotels_without_pricing_count > 0 {
            crate::log!(
                "LiteAPI search filtering: Found {} hotels total, {} without pricing ({}%), {} with valid pricing retained",
                original_count,
                hotels_without_pricing_count,
                if original_count > 0 { (hotels_without_pricing_count * 100) / original_count } else { 0 },
                final_count
            );

            // Log the specific hotel IDs that were filtered out
            crate::log!(
                "🚫 Hotels without valid pricing (filtered out): count: {},  [{}]",
                hotels_without_pricing_count,
                hotels_without_pricing_ids.join(", ")
            );

            // // Log the raw API response for debugging
            // match serde_json::to_string_pretty(rates_response) {
            //     Ok(api_response_json) => {
            //         crate::log!(
            //             "📋 LiteAPI Rates Response (Raw) for filtered hotels: {}",
            //             api_response_json
            //         );
            //     }
            //     Err(e) => {
            //         crate::log!(
            //             "⚠️ Failed to serialize LiteAPI rates response for logging: {}",
            //             e
            //         );
            //         // Fallback to debug format
            //         crate::log!("📋 LiteAPI Rates Response (Debug): {:?}", rates_response);
            //     }
            // }
        } else if original_count > 0 {
            crate::log!(
                "LiteAPI search filtering: All {} hotels had valid pricing",
                original_count
            );
        }
    }

    // --- Pagination Helper Functions ---
    fn calculate_offset_limit(
        pagination: &Option<crate::domain::DomainPaginationParams>,
    ) -> (i32, i32) {
        match pagination {
            Some(params) => {
                let page = params.page.unwrap_or(1).max(1);
                let page_size = params
                    .page_size
                    .unwrap_or(PAGINATION_LIMIT as u32)
                    .clamp(1, PAGINATION_LIMIT as u32);
                let offset = (page - 1) * page_size;
                (offset as i32, page_size as i32)
            }
            None => (0, PAGINATION_LIMIT), // Default: first page, 50 results
        }
    }

    // fn create_pagination_meta(
    //     page: u32,
    //     page_size: u32,
    //     returned_count: usize,
    // ) -> crate::domain::DomainPaginationMeta {
    //     crate::domain::DomainPaginationMeta {
    //         page,
    //         page_size,
    //         total_results: None, // LiteAPI doesn't provide total count
    //         has_next_page: returned_count as u32 == page_size, // Assume more if full page
    //         has_previous_page: page > 1,
    //     }
    // }

    fn create_pagination_meta_with_original_count(
        page: u32,
        page_size: u32,
        returned_count: usize,
        original_count_from_api: usize,
        requested_limit: usize,
        total: i32,
    ) -> crate::domain::DomainPaginationMeta {
        // Better logic: has_next_page based on original API response count
        // If LiteAPI returned exactly what we requested, there might be more
        let has_next_page = original_count_from_api >= requested_limit;

        crate::domain::DomainPaginationMeta {
            page,
            page_size,
            total_results: Some(total), // LiteAPI doesn't provide total count
            has_next_page,
            has_previous_page: page > 1,
        }
    }

    // --- Mapping functions ---

    fn map_places_domain_to_liteapi(payload: DomainPlacesSearchPayload) -> LiteApiGetPlacesRequest {
        LiteApiGetPlacesRequest {
            text_query: payload.text_query,
        }
    }

    fn map_places_id_domain_to_liteapi(
        payload: DomainPlaceDetailsPayload,
    ) -> LiteApiGetPlaceRequest {
        LiteApiGetPlaceRequest {
            place_id: payload.place_id,
        }
    }

    fn map_liteapi_places_response_to_domain(
        payload: LiteApiGetPlacesResponse,
    ) -> DomainPlacesResponse {
        DomainPlacesResponse {
            data: payload
                .data
                .into_iter()
                .map(|place| DomainPlace {
                    place_id: place.place_id,
                    display_name: place.display_name,
                    formatted_address: place.formatted_address,
                })
                .collect(),
        }
    }

    fn map_liteapi_place_details_response_to_domain(
        payload: LiteApiGetPlaceResponse,
    ) -> DomainPlaceDetails {
        DomainPlaceDetails {
            data: DomainPlaceData {
                address_components: payload
                    .data
                    .address_components
                    .into_iter()
                    .map(|ac| DomainAddressComponent {
                        language_code: ac.language_code,
                        long_text: ac.long_text,
                        short_text: ac.short_text,
                        types: ac.types,
                    })
                    .collect(),
                location: DomainLocation {
                    latitude: payload.data.location.latitude,
                    longitude: payload.data.location.longitude,
                },
                viewport: DomainViewport {
                    high: DomainHigh {
                        latitude: payload.data.viewport.high.latitude,
                        longitude: payload.data.viewport.high.longitude,
                    },
                    low: DomainLow {
                        latitude: payload.data.viewport.low.latitude,
                        longitude: payload.data.viewport.low.longitude,
                    },
                },
            },
        }
    }

    fn map_domain_search_to_liteapi(
        domain_criteria: &DomainHotelSearchCriteria,
        ui_filters: UISearchFilters,
    ) -> LiteApiHotelSearchRequest {
        let (offset, limit) = Self::calculate_offset_limit(&domain_criteria.pagination);

        // Debug logging for pagination parameters
        crate::log!(
            "🔍 LiteAPI Pagination Debug: pagination_params={:?}, calculated offset={}, limit={}",
            domain_criteria.pagination,
            offset,
            limit
        );

        LiteApiHotelSearchRequest {
            place_id: domain_criteria.place_id.clone(),
            distance: 100000,
            // ai_search: domain_criteria.destination_city_name.clone(),
            // country_code: domain_criteria.destination_country_code.clone(),
            // city_name: domain_criteria.destination_city_name.clone(), // Assuming this field exists
            offset,
            limit,
            // destination_latitude: domain_criteria.destination_latitude,
            // destination_longitude: domain_criteria.destination_longitude,
            // // todo(hotel_search): default search radius is 10km in liteapi for now.
            // // not sure if to put this in domain_search_criteria
            // radius: Some(100000),
        }
    }

    fn map_liteapi_search_to_domain(
        liteapi_response: LiteApiHotelSearchResponse,
        center_coords: Option<(f64, f64)>,
    ) -> DomainHotelListAfterSearch {
        let original_hotels = liteapi_response.data;
        let original_count = original_hotels.len();

        // Filter out hotels with empty details
        let filtered_out_hotels: Vec<LiteApiHotelResult> = original_hotels
            .iter()
            .filter(|hotel| Self::is_search_hotel_details_empty(hotel))
            .cloned()
            .collect();

        let valid_hotels: Vec<LiteApiHotelResult> = original_hotels
            .into_iter()
            .filter(|hotel| !Self::is_search_hotel_details_empty(hotel))
            .collect();

        let final_count = valid_hotels.len();

        // Log filtering results with detailed reasons
        Self::log_search_hotel_filter_results_simple(
            original_count,
            &filtered_out_hotels,
            final_count,
        );

        DomainHotelListAfterSearch {
            hotel_results: valid_hotels
                .into_iter()
                .map(|hotel| Self::map_liteapi_hotel_to_domain(hotel, center_coords))
                .collect(),
            pagination: None, // Will be set by calling function if needed
        }
    }

    fn map_liteapi_hotel_to_domain(
        liteapi_hotel: LiteApiHotelResult,
        center_coords: Option<(f64, f64)>,
    ) -> DomainHotelAfterSearch {
        let hotel_id = liteapi_hotel.id.clone();
        let property_type = liteapi_hotel
            .hotel_type_id
            .map(|id| Self::map_hotel_type_id_to_label(id));

        // Map facility IDs to human-readable amenities
        let amenities: Vec<String> = liteapi_hotel
            .facility_ids
            .iter()
            .map(|&id| Self::map_facility_id_to_name(id))
            .collect();

        // Calculate distance from center if both coordinates are available
        let distance_from_center_km =
            if let (Some((center_lat, center_lon)), hotel_lat, hotel_lon) = (
                center_coords,
                liteapi_hotel.latitude,
                liteapi_hotel.longitude,
            ) {
                Some(Self::calculate_distance_km(
                    center_lat, center_lon, hotel_lat, hotel_lon,
                ))
            } else {
                None
            };

        DomainHotelAfterSearch {
            hotel_code: hotel_id.clone(),
            hotel_name: liteapi_hotel.name,
            hotel_address: Some(liteapi_hotel.address),
            hotel_category: format!("{} Star", liteapi_hotel.stars),
            star_rating: liteapi_hotel.stars as u8,
            price: Some(DomainPrice {
                room_price: 0.0, // Will be populated by get_hotel_rates in search_hotels
                currency_code: liteapi_hotel.currency,
            }),
            hotel_picture: liteapi_hotel.main_photo,
            amenities,
            property_type,
            result_token: hotel_id,
            distance_from_center_km,
        }
    }

    // Calculate distance between two coordinates using Haversine formula
    fn calculate_distance_km(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
        const EARTH_RADIUS_KM: f64 = 6371.0;

        let dlat = (lat2 - lat1).to_radians();
        let dlon = (lon2 - lon1).to_radians();
        let lat1_rad = lat1.to_radians();
        let lat2_rad = lat2.to_radians();

        let a = (dlat / 2.0).sin().powi(2)
            + lat1_rad.cos() * lat2_rad.cos() * (dlon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        EARTH_RADIUS_KM * c
    }

    // Map search results with pricing from rates API
    #[cfg_attr(feature = "debug_log", instrument(skip(self, liteapi_response)))]
    async fn map_liteapi_search_to_domain_with_pricing(
        &self,
        liteapi_response: LiteApiHotelSearchResponse,
        search_criteria: &DomainHotelSearchCriteria,
    ) -> Result<DomainHotelListAfterSearch, ProviderError> {
        // Track original count before any filtering
        let original_count_from_liteapi = liteapi_response.data.len();
        let total_properties = liteapi_response.total;

        // Get center coordinates from place details
        let center_coords = if !search_criteria.place_id.is_empty() {
            // Get place details to extract center coordinates
            match self
                .get_single_place_details(DomainPlaceDetailsPayload {
                    place_id: search_criteria.place_id.clone(),
                })
                .await
            {
                Ok(place_details) => Some((
                    place_details.data.location.latitude,
                    place_details.data.location.longitude,
                )),
                Err(e) => {
                    crate::log!(
                        "Failed to get place details for distance calculation: {:?}",
                        e
                    );
                    None
                }
            }
        } else {
            None
        };

        let mut domain_results =
            Self::map_liteapi_search_to_domain(liteapi_response.clone(), center_coords);

        // (todo): review hotel_search - Extract hotel IDs
        let hotel_ids: Vec<String> = liteapi_response
            .data
            .iter()
            // .take(100)
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

        crate::log!("rates_request: {:#?}", rates_request);

        let rates_response: LiteApiHotelRatesResponse = self
            .client
            .send(rates_request)
            .await
            .map_err(|e: ApiError| {
                // Log but don't fail the entire search if rates fail
                ProviderError::from_api_error(e, ProviderNames::LiteApi, ProviderSteps::HotelSearch)
            })?;

        // Log the rates response structure
        if let Some(error) = &rates_response.error {
            error!(?error, "Error in liteapi rates response");
        }

        if let Some(data) = &rates_response.data {
            info!(?data, "Got liteapi rates response data");
        }

        // Check if rates response indicates no availability
        if rates_response.is_no_availability() {
            return Ok(DomainHotelListAfterSearch {
                hotel_results: vec![],
                pagination: None,
            });
        }

        // Check if it's an error response
        if rates_response.is_error_response() {
            // Continue processing but without pricing data
        }

        // Merge pricing data into search results
        Self::merge_pricing_into_search_results(&mut domain_results, rates_response.clone());

        // Filter out hotels with zero pricing
        Self::filter_hotels_with_valid_pricing(&mut domain_results, &rates_response);

        // Add pagination metadata if pagination params are provided
        if let Some(ref pagination_params) = search_criteria.pagination {
            let page = pagination_params.page.unwrap_or(1);
            let page_size = pagination_params
                .page_size
                .unwrap_or(PAGINATION_LIMIT as u32);
            let returned_count = domain_results.hotel_results.len();

            // Check if we got the full requested amount from LiteAPI (before filtering)
            let requested_limit = page_size as usize;

            let pagination_meta = Self::create_pagination_meta_with_original_count(
                page,
                page_size,
                returned_count,
                original_count_from_liteapi,
                requested_limit,
                total_properties,
            );

            // Debug logging for pagination metadata
            crate::log!(
                "📊 Pagination Metadata: page={}, page_size={}, returned_count={}, original_count={}, has_next={}, has_previous={}",
                pagination_meta.page, pagination_meta.page_size, returned_count, original_count_from_liteapi, pagination_meta.has_next_page, pagination_meta.has_previous_page
            );

            domain_results.pagination = Some(pagination_meta);
        } else {
            crate::log!("⚠️ No pagination params provided, pagination metadata will be None");
        }

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
                            warn!(hotel_id = %hotel_data.hotel_id, "Hotel has no suggested selling price in rate");
                        }
                    } else {
                        warn!(hotel_id = %hotel_data.hotel_id, "Hotel has no rates in room type");
                    }
                } else {
                    warn!(hotel_id = %hotel_data.hotel_id, "Hotel has no room types available");
                }
            }
        } else {
            warn!("No data in liteapi rates response - all hotels will have no pricing");
        }

        // Update search results with pricing

        for hotel in &mut domain_results.hotel_results {
            if let Some(price) = hotel_prices.get(&hotel.hotel_code) {
                hotel.price = Some(price.clone());
            } else {
                warn!(hotel_code = %hotel.hotel_code, "No pricing data found for hotel in rates response");
            }
        }
    }

    // Filter out hotels with zero pricing from search results
    fn filter_hotels_with_valid_pricing(
        domain_results: &mut DomainHotelListAfterSearch,
        rates_response: &LiteApiHotelRatesResponse,
    ) {
        let original_count = domain_results.hotel_results.len();

        // Collect hotel IDs without valid pricing for logging
        let hotels_without_pricing_ids: Vec<String> = domain_results
            .hotel_results
            .iter()
            .filter(|hotel| {
                hotel
                    .price
                    .as_ref()
                    .map(|f| f.room_price <= 0.0)
                    .unwrap_or(false)
            })
            .map(|hotel| hotel.hotel_code.clone())
            .collect();

        // domain_results
        //     .hotel_results
        //     .retain(|hotel| hotel.price.room_price > 0.0);

        let final_count = domain_results.hotel_results.len();

        // Use separate logging function with API response
        Self::log_pricing_filter_results(
            original_count,
            &hotels_without_pricing_ids,
            final_count,
            rates_response,
        );
    }

    fn is_search_hotel_details_empty(hotel: &LiteApiHotelResult) -> bool {
        // Check if essential hotel details are empty or meaningless in search results
        let name_empty = hotel.name.trim().is_empty();
        let description_empty = hotel.hotel_description.trim().is_empty();
        let address_empty = hotel.address.trim().is_empty();
        let no_main_photo = hotel.main_photo.trim().is_empty();
        let no_thumbnail = hotel.thumbnail.as_ref().is_none_or(|t| t.trim().is_empty());
        let no_images = no_main_photo && no_thumbnail;

        // Consider hotel details empty if:
        // 1. Name is empty (critical field)
        // <!-- Commented out image-based filtering -->
        // 2. Description is empty AND no main photo (key visual content missing)
        // 3. All essential fields are empty (name, description, address, images)
        // name_empty
        //     || (description_empty && no_main_photo)
        //     || (description_empty && address_empty && no_images)
        // name_empty || (description_empty && address_empty)
        // todo - for now, remove the guardrails around missing hotel details
        false
    }

    fn log_search_hotel_filter_results_simple(
        original_count: usize,
        filtered_out_hotels: &[LiteApiHotelResult],
        final_count: usize,
    ) {
        let filtered_count = filtered_out_hotels.len();

        if filtered_count > 0 {
            crate::log!(
                "LiteAPI search hotel details filtering: Found {} hotels total, {} with empty details ({}%), {} with valid details retained",
                original_count,
                filtered_count,
                if original_count > 0 { (filtered_count * 100) / original_count } else { 0 },
                final_count
            );

            // Log detailed reasons for each filtered hotel
            for hotel in filtered_out_hotels {
                let mut reasons = Vec::new();

                if hotel.name.trim().is_empty() {
                    reasons.push("empty name");
                }
                if hotel.hotel_description.trim().is_empty() {
                    reasons.push("empty description");
                }
                if hotel.address.trim().is_empty() {
                    reasons.push("empty address");
                }
                if hotel.main_photo.trim().is_empty() {
                    reasons.push("empty main_photo");
                }
                if hotel.thumbnail.as_ref().is_none_or(|t| t.trim().is_empty()) {
                    reasons.push("empty thumbnail");
                }

                crate::log!(
                    "🚫 Hotel filtered out: {} (ID: {}) - Reasons: [{}]",
                    hotel.name,
                    hotel.id,
                    reasons.join(", ")
                );
            }
        } else if original_count > 0 {
            crate::log!(
                "LiteAPI search hotel details filtering: All {} hotels had valid details",
                original_count
            );
        }
    }

    // Convert domain search criteria to LiteAPI hotel rates request
    fn map_domain_info_to_liteapi_rates(
        domain_criteria: &DomainHotelInfoCriteria,
    ) -> Result<LiteApiHotelRatesRequest, ProviderError> {
        let search_criteria = &domain_criteria.search_criteria;

        // Normalize occupancies so LiteAPI gets one entry per room
        let room_count = if search_criteria.no_of_rooms == 0 {
            let derived = search_criteria.room_guests.len() as u32;
            if derived == 0 {
                1
            } else {
                derived
            }
        } else {
            search_criteria.no_of_rooms
        };

        let map_guest_to_occupancy = |room_guest: &DomainRoomGuest| -> LiteApiOccupancy {
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
        };

        let occupancies: Vec<LiteApiOccupancy> =
            if search_criteria.room_guests.len() as u32 == room_count {
                // Already have one guest group per room
                search_criteria
                    .room_guests
                    .iter()
                    .map(map_guest_to_occupancy)
                    .collect()
            } else {
                // Spread total guests evenly across rooms
                let total_adults: u32 = search_criteria
                    .room_guests
                    .iter()
                    .map(|rg| rg.no_of_adults)
                    .sum();
                let total_children: u32 = search_criteria
                    .room_guests
                    .iter()
                    .map(|rg| rg.no_of_children)
                    .sum();
                let children_ages: Vec<u32> = search_criteria
                    .room_guests
                    .iter()
                    .filter_map(|rg| rg.children_ages.as_ref())
                    .flat_map(|ages| ages.iter())
                    .filter_map(|age_str| age_str.parse::<u32>().ok())
                    .collect();

                let base_adults = total_adults / room_count;
                let mut extra_adults = total_adults % room_count;
                let base_children = total_children / room_count;
                let mut extra_children = total_children % room_count;
                let mut age_iter = children_ages.into_iter();

                let mut distributed = Vec::with_capacity(room_count as usize);
                for _ in 0..room_count {
                    let adults = base_adults
                        + if extra_adults > 0 {
                            extra_adults -= 1;
                            1
                        } else {
                            0
                        };
                    let children_for_room = base_children
                        + if extra_children > 0 {
                            extra_children -= 1;
                            1
                        } else {
                            0
                        };

                    let children = if children_for_room > 0 {
                        let kids: Vec<u32> =
                            age_iter.by_ref().take(children_for_room as usize).collect();
                        Some(kids)
                    } else {
                        None
                    };

                    distributed.push(LiteApiOccupancy { adults, children });
                }

                distributed
            };

        // Format dates as YYYY-MM-DD
        let checkin = utils::date::date_tuple_to_yyyy_mm_dd(search_criteria.check_in_date);
        let checkout = utils::date::date_tuple_to_yyyy_mm_dd(search_criteria.check_out_date);

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
                error_step: ProviderSteps::HotelRate,
            })));
        };

        Ok(LiteApiHotelRatesRequest {
            room_mapping: true,
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
        crate::adapters::liteapi_adapter::map_facility_id::map_facility_id_to_name(facility_id)
        // match facility_id {
        //     1 => "Swimming Pool".to_string(),
        //     2 => "Fitness Center".to_string(),
        //     3 => "Spa & Wellness".to_string(),
        //     4 => "Restaurant".to_string(),
        //     5 => "Bar".to_string(),
        //     6 => "Free WiFi".to_string(),
        //     7 => "Parking".to_string(),
        //     8 => "Business Center".to_string(),
        //     9 => "Concierge".to_string(),
        //     10 => "Room Service".to_string(),
        //     11 => "Laundry Service".to_string(),
        //     12 => "Air Conditioning".to_string(),
        //     13 => "Pet Friendly".to_string(),
        //     14 => "Elevator".to_string(),
        //     15 => "Non-Smoking Rooms".to_string(),
        //     16 => "Airport Shuttle".to_string(),
        //     17 => "Meeting Rooms".to_string(),
        //     18 => "Childcare".to_string(),
        //     19 => "Breakfast".to_string(),
        //     20 => "24-Hour Front Desk".to_string(),
        //     _ => format!("Facility {}", facility_id),
        // }
    }

    // Map hotel type ID to a human-readable label.
    // If we don't have a concrete mapping, fall back to a generic label.
    fn map_hotel_type_id_to_label(type_id: i32) -> String {
        match type_id {
            0 => "Not Available".to_string(),
            201 => "Apartments".to_string(),
            203 => "Hostels".to_string(),
            204 => "Hotels".to_string(),
            205 => "Motels".to_string(),
            206 => "Resorts".to_string(),
            207 => "Residences".to_string(),
            208 => "Bed and breakfasts".to_string(),
            209 => "Ryokans".to_string(),
            210 => "Farm stays".to_string(),
            212 => "Holiday parks".to_string(),
            213 => "Villas".to_string(),
            214 => "Campsites".to_string(),
            215 => "Boats".to_string(),
            216 => "Guest houses".to_string(),
            218 => "Inns".to_string(),
            219 => "Aparthotels".to_string(),
            220 => "Holiday homes".to_string(),
            221 => "Lodges".to_string(),
            222 => "Homestays".to_string(),
            223 => "Country houses".to_string(),
            224 => "Luxury tents".to_string(),
            225 => "Capsule hotels".to_string(),
            226 => "Love hotels".to_string(),
            227 => "Riads".to_string(),
            228 => "Chalets".to_string(),
            229 => "Condos".to_string(),
            230 => "Cottages".to_string(),
            231 => "Economy hotels".to_string(),
            232 => "Gites".to_string(),
            233 => "Health resorts".to_string(),
            234 => "Cruises".to_string(),
            235 => "Student accommodation".to_string(),
            243 => "Tree house property".to_string(),
            247 => "Pension".to_string(),
            250 => "Private vacation home".to_string(),
            251 => "Pousada".to_string(),
            252 => "Country house".to_string(),
            254 => "Campsite".to_string(),
            257 => "Cabin".to_string(),
            258 => "Holiday park".to_string(),
            262 => "Affittacamere".to_string(),
            264 => "Hostel/Backpacker accommodation".to_string(),
            265 => "Houseboat".to_string(),
            268 => "Ranch".to_string(),
            271 => "Agritourism property".to_string(),
            272 => "Mobile home".to_string(),
            273 => "Safari/Tentalow".to_string(),
            274 => "All-inclusive property".to_string(),
            276 => "Castle".to_string(),
            277 => "Property".to_string(),
            278 => "Palace".to_string(),
            _ => format!("Type {}", type_id),
        }
    }

    fn map_liteapi_to_domain_static_details(
        details: LiteApiSingleHotelDetailData,
    ) -> DomainHotelStaticDetails {
        let sentiment_categories = details
            .sentiment_analysis
            .as_ref()
            .map(|s| s.categories.clone())
            .unwrap_or_default();
        let mut images = Vec::new();
        if !details.main_photo.is_empty() {
            images.push(details.main_photo.clone());
        }
        if let Some(thumbnail) = &details.thumbnail {
            if !thumbnail.is_empty() && thumbnail != &details.main_photo {
                images.push(thumbnail.clone());
            }
        }
        for hotel_image in &details.hotel_images {
            if !hotel_image.url.is_empty() && !images.contains(&hotel_image.url) {
                images.push(hotel_image.url.clone());
            }
            if !hotel_image.url_hd.is_empty()
                && hotel_image.url_hd != hotel_image.url
                && !images.contains(&hotel_image.url_hd)
            {
                images.push(hotel_image.url_hd.clone());
            }
        }

        let location = details.location;

        DomainHotelStaticDetails {
            hotel_code: details.id,
            hotel_name: details.name,
            star_rating: details.star_rating,
            rating: if details.rating > 0.0 {
                Some(details.rating)
            } else {
                None
            },
            review_count: if details.review_count > 0 {
                Some(details.review_count as u32)
            } else {
                None
            },
            categories: if !sentiment_categories.is_empty() {
                sentiment_categories
            } else {
                details.categories
            }
            .into_iter()
            .map(|c| DomainReviewCategory {
                name: c.name,
                rating: c.rating as f32,
                description: if c.description.trim().is_empty() {
                    None
                } else {
                    Some(c.description)
                },
            })
            .collect(),
            description: details.hotel_description,
            address: details.address,
            images,
            hotel_facilities: details.hotel_facilities,
            amenities: vec![], // This can be populated if needed from other sources
            rooms: Self::map_room_details(details.rooms),
            location: location.map(|location| DomainLocation {
                latitude: location.latitude,
                longitude: location.longitude,
            }),
            checkin_checkout_times: Some(DomainCheckinCheckoutTimes {
                checkin: details.checkin_checkout_times.checkin,
                checkout: details.checkin_checkout_times.checkout,
            }),
            policies: details
                .policies
                .into_iter()
                .map(|p| DomainPolicy {
                    policy_type: if p.policy_type.is_empty() {
                        None
                    } else {
                        Some(p.policy_type)
                    },
                    name: p.name,
                    description: p.description,
                })
                .collect(),
        }
    }

    fn map_room_details(rooms: Option<Vec<LiteApiRoom>>) -> Vec<DomainStaticRoom> {
        rooms
            .unwrap_or_default()
            .into_iter()
            .map(|room| {
                let amenities = room.room_amenities.into_iter().map(|a| a.name).collect();
                let photos = room
                    .photos
                    .into_iter()
                    .filter_map(|photo| {
                        if !photo.hd_url.trim().is_empty() {
                            Some(photo.hd_url)
                        } else if !photo.url.trim().is_empty() {
                            Some(photo.url)
                        } else {
                            None
                        }
                    })
                    .collect();
                let bed_types = room
                    .bed_types
                    .into_iter()
                    .map(|bed| {
                        let mut label = format!("{} {}", bed.quantity, bed.bed_type);
                        if !bed.bed_size.trim().is_empty() {
                            label.push_str(&format!(" ({})", bed.bed_size));
                        }
                        label
                    })
                    .collect();

                DomainStaticRoom {
                    room_id: room.id.to_string(),
                    room_name: room.room_name,
                    description: room.description,
                    room_size_square: room.room_size_square,
                    room_size_unit: room.room_size_unit,
                    max_adults: Some(room.max_adults as u32),
                    max_children: Some(room.max_children as u32),
                    max_occupancy: Some(room.max_occupancy as u32),
                    amenities,
                    photos,
                    bed_types,
                }
            })
            .collect()
    }

    #[tracing::instrument]
    // Map LiteAPI rates response and hotel details to domain hotel details
    fn map_liteapi_rates_and_details_to_domain(
        liteapi_rates_response: LiteApiHotelRatesResponse,
        hotel_details: Option<LiteApiSingleHotelDetailData>,
        search_criteria: &DomainHotelSearchCriteria,
        hotel_info: Option<&LiteApiHotelResult>,
    ) -> Result<DomainHotelDetails, ProviderError> {
        // log all the details about why hotel details are considered empty
        crate::log!("liteapi_rates_response: {:?}, hotel_details: {:?}, search_criteria: {:?}, hotel_info: {:?}", liteapi_rates_response, hotel_details, search_criteria, hotel_info);

        if liteapi_rates_response
            .data
            .as_deref()
            .is_none_or(|d| d.is_empty())
        {
            return Err(ProviderError(Arc::new(ProviderErrorDetails {
                provider_name: ProviderNames::LiteApi,
                api_error: ApiError::Other(
                    "Hotel may not be available for the selected dates.".to_string(),
                ),
                error_step: ProviderSteps::HotelRate,
            })));
        }

        // Check if hotel details are empty when provided
        if let Some(ref details) = hotel_details {
            if Self::is_hotel_details_empty(details) {
                crate::log!("Hotel details are empty, filtering out this hotel");
                return Err(ProviderError(Arc::new(ProviderErrorDetails {
                    provider_name: ProviderNames::LiteApi,
                    api_error: ApiError::Other(
                        "Hotel has empty or insufficient details and should be filtered out"
                            .to_string(),
                    ),
                    error_step: ProviderSteps::HotelDetails,
                })));
            }
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
        let mut all_rooms: Vec<DomainRoomOption> = Vec::new();

        if let Some(data) = &liteapi_rates_response.data {
            for hotel_data_item in data {
                for room_type in &hotel_data_item.room_types {
                    for rate in &room_type.rates {
                        let room_option = Self::map_liteapi_room_to_domain(
                            rate.clone(),
                            room_type.room_type_id.clone(),
                            room_type.offer_id.clone(),
                        );

                        all_rooms.push(room_option);
                    }
                }
            }
        }

        // Sort rooms: unique names first, then repeated names at the end
        let mut name_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        for room in &all_rooms {
            *name_counts
                .entry(room.room_data.room_name.clone())
                .or_insert(0) += 1;
        }

        all_rooms.sort_by(|a, b| {
            let a_count = name_counts.get(&a.room_data.room_name).unwrap_or(&0);
            let b_count = name_counts.get(&b.room_data.room_name).unwrap_or(&0);

            // First sort by uniqueness (unique names first)
            match (a_count == &1, b_count == &1) {
                (true, false) => std::cmp::Ordering::Less, // a is unique, b is not
                (false, true) => std::cmp::Ordering::Greater, // b is unique, a is not
                _ => {
                    // If both are unique or both are repeated, sort by name for consistency
                    a.room_data.room_name.cmp(&b.room_data.room_name)
                }
            }
        });

        // Filter rooms based on image availability if hotel details are available and have room data
        // if let Some(details) = &hotel_details {
        //     if !details.rooms.is_empty() {
        //         let rooms_with_images: std::collections::HashSet<String> = details
        //             .rooms
        //             .iter()
        //             // todo (room filtering): filter rooms based on image availability
        //             // .filter(|room| {
        //             //     !room.photos.is_empty()
        //             //         && room
        //             //             .photos
        //             //             .iter()
        //             //             .any(|photo| !photo.url.is_empty() || !photo.hd_url.is_empty())
        //             // })
        //             .map(|room| room.room_name.clone())
        //             .collect();

        //         if !rooms_with_images.is_empty() {
        //             let original_room_count = all_rooms.len();
        //             all_rooms.retain(|room_option| {
        //                 rooms_with_images.contains(&room_option.room_data.room_name)
        //             });
        //             let filtered_room_count = all_rooms.len();

        //             if filtered_room_count < original_room_count {
        //                 crate::log!(
        //                     "LiteAPI room filtering: Filtered out {} rooms without images, {} rooms retained",
        //                     original_room_count - filtered_room_count,
        //                     filtered_room_count
        //                 );
        //             }
        //         } else {
        //             crate::log!("LiteAPI room filtering: Hotel details available but no rooms have images, keeping all rooms from rates API");
        //         }
        //     } else {
        //         crate::log!("LiteAPI room filtering: Hotel details available but rooms array is empty, keeping all rooms from rates API");
        //     }
        // }

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
                let hotel_name = "Hotel".to_string();
                let description = "A quality hotel located. This property offers comfortable accommodations and excellent service for your stay.".to_string();
                let address = String::new();

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
            rating: None,
            review_count: None,
            categories: Vec::new(),
            description: description.clone(),
            hotel_facilities: hotel_facilities.clone(),
            address: address.clone(),
            images: images.clone(),
            all_rooms,
            amenities: vec![],
            search_info: None,
            search_criteria: Some(search_criteria.clone()),
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

        // Flatten all rates returned in prebook so we can reflect multiple rooms of the same offer
        let mut flattened_rates: Vec<crate::api::liteapi::l02_prebook::LiteApiPrebookRate> =
            Vec::new();
        for room_type in &data.room_types {
            flattened_rates.extend(room_type.rates.clone());
        }

        // Build blocked rooms from rates, defaulting to selected_room info if missing
        let currency = data.currency.clone();
        let mut blocked_rooms = Vec::new();
        for rate in &flattened_rates {
            let room_price_amount = rate
                .retail_rate
                .as_ref()
                .and_then(|rr| rr.total.first().map(|a| a.amount))
                .unwrap_or(data.price);
            let room_currency = rate
                .retail_rate
                .as_ref()
                .and_then(|rr| rr.total.first().map(|a| a.currency.clone()))
                .unwrap_or_else(|| currency.clone());

            let detailed_price = DomainDetailedPrice {
                published_price: room_price_amount,
                published_price_rounded_off: room_price_amount.round(),
                offered_price: room_price_amount,
                offered_price_rounded_off: room_price_amount.round(),
                room_price: room_price_amount,
                tax: 0.0,
                extra_guest_charge: 0.0,
                child_charge: 0.0,
                other_charges: 0.0,
                currency_code: room_currency,
            };

            let room_name = if !rate.name.is_empty() {
                rate.name.clone()
            } else {
                original_request.selected_room.room_name.clone()
            };

            blocked_rooms.push(DomainBlockedRoom {
                room_code: original_request.selected_room.room_unique_id.clone(),
                room_name,
                room_type_code: Some(original_request.selected_room.room_unique_id.clone()),
                price: detailed_price,
                cancellation_policy: Some(rate.cancellation_policies.refundable_tag.clone()),
                meal_plan: Some(format!("{} - {}", rate.board_type, rate.board_name)),
            });
        }

        // If nothing came back, fall back to a single placeholder room
        if blocked_rooms.is_empty() {
            let fallback_price = DomainDetailedPrice {
                published_price: data.suggested_selling_price,
                published_price_rounded_off: data.suggested_selling_price.round(),
                offered_price: data.price,
                offered_price_rounded_off: data.price.round(),
                room_price: data.price,
                tax: 0.0,
                extra_guest_charge: 0.0,
                child_charge: 0.0,
                other_charges: 0.0,
                currency_code: currency.clone(),
            };
            blocked_rooms.push(DomainBlockedRoom {
                room_code: original_request.selected_room.room_unique_id.clone(),
                room_name: original_request.selected_room.room_name.clone(),
                room_type_code: Some(original_request.selected_room.room_unique_id.clone()),
                price: fallback_price,
                cancellation_policy: None,
                meal_plan: None,
            });
        }

        // Total price from provider (already includes multiple rooms if applicable)
        let total_price = DomainDetailedPrice {
            published_price: data.suggested_selling_price,
            published_price_rounded_off: data.suggested_selling_price.round(),
            offered_price: data.price,
            offered_price_rounded_off: data.price.round(),
            room_price: data.price,
            tax: 0.0,
            extra_guest_charge: 0.0,
            child_charge: 0.0,
            other_charges: 0.0,
            currency_code: currency.clone(),
        };

        DomainBlockRoomResponse {
            block_id: data.prebook_id.clone(),
            is_price_changed: data.price_difference_percent != 0.0,
            is_cancellation_policy_changed: data.cancellation_changed,
            blocked_rooms,
            total_price,
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
                            amount: room.flattened_rate.amount,
                            currency: room.flattened_rate.currency,
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
                .unwrap_or_default()
                .into_iter()
                .map(|policy| DomainCancelPolicyInfo {
                    cancel_time: policy.cancel_time,
                    amount: policy.amount,
                    policy_type: policy.policy_type,
                    timezone: policy.timezone,
                    currency: policy.currency,
                })
                .collect(),
            hotel_remarks: data
                .cancellation_policies
                .hotel_remarks
                .as_ref()
                .filter(|remarks| !remarks.is_empty())
                .map(|remarks| remarks.join(". ")),
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

    // <!-- GET BOOKING DETAILS MAPPING METHODS -->

    /// Map domain get booking request to LiteAPI request
    fn map_domain_get_booking_to_liteapi(
        request: &DomainGetBookingRequest,
    ) -> Result<LiteApiGetBookingRequest, ProviderError> {
        // Validate that at least one identifier is provided
        if request.client_reference.is_none() && request.guest_id.is_none() {
            return Err(ProviderError::validation_error(
                ProviderNames::LiteApi,
                "Either client_reference or guest_id must be provided for booking lookup"
                    .to_string(),
            ));
        }

        Ok(LiteApiGetBookingRequest {
            client_reference: request.client_reference.clone(),
            guest_id: request.guest_id.clone(),
            timeout: Some(4.0), // Default timeout for LiteAPI
        })
    }

    /// Map LiteAPI get booking response to domain response
    fn map_liteapi_get_booking_to_domain(
        response: LiteApiGetBookingResponse,
    ) -> Result<DomainGetBookingResponse, ProviderError> {
        use crate::domain::{
            DomainBookingDetails,
            DomainBookingHotelInfo,
            // <!-- Commented out unused imports for simplified struct -->
            // DomainBookingGuestInfo, DomainBookingRoomInfo,
            // DomainCancellationPolicies, DomainCancelPolicyInfo,
        };

        let bookings = response
            .data
            .into_iter()
            .map(|liteapi_booking| {
                // Map hotel info
                let hotel = DomainBookingHotelInfo {
                    hotel_id: liteapi_booking.hotel.hotel_id,
                    name: liteapi_booking.hotel.name,
                };

                // Map holder (from API response)
                let holder = DomainBookingHolder {
                    first_name: liteapi_booking.holder.first_name,
                    last_name: liteapi_booking.holder.last_name,
                    email: liteapi_booking.holder.email,
                    phone: liteapi_booking.holder.phone,
                };

                // <!-- Commented out unused mappings since fields are not in simplified struct -->

                // // Map cancellation policies
                // let cancellation_policies = DomainCancellationPolicies {
                //     cancel_policy_infos: liteapi_booking
                //         .cancellation_policies
                //         .cancel_policy_infos
                //         .unwrap_or_default()
                //         .into_iter()
                //         .map(|policy| DomainCancelPolicyInfo {
                //             cancel_time: policy.cancel_time,
                //             amount: policy.amount,
                //             policy_type: policy.policy_type,
                //             timezone: policy.timezone,
                //             currency: policy.currency,
                //         })
                //         .collect(),
                //     hotel_remarks: liteapi_booking.cancellation_policies.hotel_remarks,
                //     refundable_tag: liteapi_booking.cancellation_policies.refundable_tag,
                // };

                // // Map rooms
                // let rooms = liteapi_booking
                //     .rooms
                //     .into_iter()
                //     .map(|liteapi_room| {
                //         let guests = liteapi_room
                //             .guests
                //             .into_iter()
                //             .map(|guest| DomainBookingGuestInfo {
                //                 first_name: guest.first_name,
                //                 last_name: guest.last_name,
                //                 email: guest.email,
                //                 phone: guest.phone,
                //                 remarks: guest.remarks,
                //                 occupancy_number: guest.occupancy_number,
                //             })
                //             .collect();

                //         DomainBookingRoomInfo {
                //             adults: liteapi_room.adults,
                //             children: liteapi_room.children,
                //             first_name: liteapi_room.first_name,
                //             last_name: liteapi_room.last_name,
                //             children_ages: liteapi_room.children_ages,
                //             room_id: liteapi_room.room_id,
                //             occupancy_number: liteapi_room.occupancy_number,
                //             amount: liteapi_room.amount,
                //             currency: liteapi_room.currency,
                //             children_count: liteapi_room.children_count,
                //             remarks: liteapi_room.remarks,
                //             guests,
                //         }
                //     })
                //     .collect();

                DomainBookingDetails {
                    // <!-- Essential fields only -->
                    booking_id: liteapi_booking.booking_id,
                    client_reference: Some(liteapi_booking.client_reference),
                    status: liteapi_booking.status,
                    hotel,
                    holder,
                    price: liteapi_booking.price,
                    currency: liteapi_booking.currency,
                    // <!-- All other fields are commented out in the domain struct -->
                    // supplier_booking_id: liteapi_booking.supplier_booking_id,
                    // supplier_booking_name: liteapi_booking.supplier_booking_name,
                    // supplier: liteapi_booking.supplier,
                    // supplier_id: liteapi_booking.supplier_id,
                    // hotel_confirmation_code: liteapi_booking.hotel_confirmation_code,
                    // checkin: liteapi_booking.checkin,
                    // checkout: liteapi_booking.checkout,
                    // rooms,
                    // created_at: liteapi_booking.created_at,
                    // updated_at: liteapi_booking.updated_at,
                    // cancellation_policies,
                    // commission: liteapi_booking.commission,
                    // payment_status: liteapi_booking.payment_status,
                    // payment_transaction_id: liteapi_booking.payment_transaction_id,
                    // special_remarks: liteapi_booking.special_remarks,
                    // guest_id: liteapi_booking.guest_id,
                    // tracking_id: liteapi_booking.tracking_id,
                    // prebook_id: liteapi_booking.prebook_id,
                    // email: liteapi_booking.email,
                    // cancelled_at: liteapi_booking.cancelled_at,
                    // refunded_at: liteapi_booking.refunded_at,
                    // cancelled_by: liteapi_booking.cancelled_by,
                    // sandbox: liteapi_booking.sandbox,
                    // nationality: liteapi_booking.nationality,
                }
            })
            .collect();

        Ok(DomainGetBookingResponse { bookings })
    }

    fn map_liteapi_room_to_domain(
        rate: crate::api::liteapi::LiteApiRate,
        room_type_id: String,
        offer_id: String,
    ) -> DomainRoomOption {
        let room_price = rate
            .retail_rate
            .suggested_selling_price
            .first()
            .map(|amount| amount.amount)
            .unwrap_or(0.0);

        let currency_code = rate
            .retail_rate
            .suggested_selling_price
            .first()
            .map(|amount| amount.currency.clone())
            .unwrap_or_else(|| "USD".to_string());

        let meal_plan = {
            let board_name = rate.board_name.trim();
            let board_type = rate.board_type.trim();
            if board_name.is_empty() && board_type.is_empty() {
                None
            } else if board_type.is_empty() {
                Some(board_name.to_string())
            } else if board_name.is_empty() {
                Some(board_type.to_string())
            } else {
                Some(format!("{} ({})", rate.board_name, rate.board_type))
            }
        };

        let occupancy_info = Some(crate::domain::DomainRoomOccupancy {
            max_occupancy: Some(rate.max_occupancy),
            adult_count: Some(rate.adult_count),
            child_count: Some(rate.child_count),
        });

        DomainRoomOption {
            mapped_room_id: rate.mapped_room_id,
            price: crate::domain::DomainDetailedPrice {
                published_price: room_price,
                published_price_rounded_off: room_price,
                offered_price: room_price,
                offered_price_rounded_off: room_price,
                room_price,
                tax: 0.0,
                extra_guest_charge: 0.0,
                child_charge: 0.0,
                other_charges: 0.0,
                currency_code,
            },
            room_data: crate::domain::DomainRoomData {
                mapped_room_id: rate.mapped_room_id,
                occupancy_number: rate.occupancy_number,
                room_name: rate.name,
                room_unique_id: room_type_id,
                rate_key: rate.rate_id,
                offer_id,
            },
            meal_plan,
            occupancy_info,
        }
    }
}
