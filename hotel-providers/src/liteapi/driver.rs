use crate::domain::{
    DomainBlockRoomRequest, DomainBlockRoomResponse, DomainBookRoomRequest, DomainBookRoomResponse,
    DomainGetBookingRequest, DomainGetBookingResponse, DomainGroupedRoomRates,
    DomainHotelInfoCriteria, DomainHotelListAfterSearch, DomainHotelSearchCriteria,
    DomainHotelStaticDetails, DomainPlaceDetails, DomainPlaceDetailsPayload, DomainPlacesResponse,
    DomainPlacesSearchPayload, DomainPrice,
};
use crate::liteapi::client::LiteApiClient;
use crate::liteapi::mapper::LiteApiMapper;
use crate::ports::{
    HotelProviderPort, PlaceProviderPort, ProviderError, ProviderNames, UISearchFilters,
};
use async_trait::async_trait;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct LiteApiDriver {
    pub client: LiteApiClient,
    pub room_mapping: bool,
}

impl LiteApiDriver {
    pub fn new(client: LiteApiClient, room_mapping: bool) -> Self {
        Self {
            client,
            room_mapping,
        }
    }
}

#[async_trait]
impl HotelProviderPort for LiteApiDriver {
    fn name(&self) -> &'static str {
        ProviderNames::LiteApi
    }

    async fn search_hotels(
        &self,
        criteria: DomainHotelSearchCriteria,
        ui_filters: UISearchFilters,
    ) -> Result<DomainHotelListAfterSearch, ProviderError> {
        let req = LiteApiMapper::map_domain_search_to_liteapi(&criteria, &ui_filters);
        let resp = self.client.get_hotels(&req).await?;

        // First map basic search results
        let mut domain_list =
            LiteApiMapper::map_liteapi_search_response_to_domain(resp, &criteria.pagination);

        if domain_list.hotel_results.is_empty() {
            return Ok(domain_list);
        }

        // Collect hotel IDs
        let hotel_ids: Vec<String> = domain_list
            .hotel_results
            .iter()
            .map(|h| h.hotel_code.clone())
            .collect();

        // Call rates API
        let rates_criteria = DomainHotelInfoCriteria {
            token: "".to_string(), // Unused for LiteAPI search
            hotel_ids,
            search_criteria: criteria.clone(),
        };

        let rates_req = LiteApiMapper::map_domain_info_to_liteapi_rates(
            &rates_criteria,
            self.client.currency(),
            self.room_mapping,
        )?;
        // We use client.get_hotel_rates directly to get raw response for pricing merging
        match self.client.get_hotel_rates(&rates_req).await {
            Ok(rates_response) => {
                // Merge pricing
                // Create a map of hotel_id -> price for quick lookup
                let mut hotel_prices = std::collections::HashMap::new();

                if let Some(data) = rates_response.data {
                    for hotel_data in data {
                        let mut min_price: Option<(f64, String)> = None;

                        // Iterate through ALL room types
                        for room_type in &hotel_data.room_types {
                            // Iterate through ALL rates in this room type
                            for rate in &room_type.rates {
                                // Use total (base price) and subtract included taxes for lowest display price
                                if let Some(amount) = rate.retail_rate.total.first() {
                                    let mut price = amount.amount;
                                    let currency = amount.currency.clone();

                                    // Subtract any included taxes to show the lowest possible price
                                    if let Some(taxes) = &rate.retail_rate.taxes_and_fees {
                                        for tax in taxes {
                                            if tax.included {
                                                price -= tax.amount;
                                            }
                                        }
                                    }

                                    // Ensure price doesn't go negative
                                    price = price.max(0.0);

                                    // Update minimum if this is lower or first price found
                                    match &min_price {
                                        None => min_price = Some((price, currency)),
                                        Some((current_min, _)) if price < *current_min => {
                                            min_price = Some((price, currency));
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }

                        // Store the minimum price found for this hotel
                        if let Some((price, currency)) = min_price {
                            hotel_prices.insert(
                                hotel_data.hotel_id.clone(),
                                DomainPrice {
                                    room_price: price,
                                    currency_code: currency,
                                },
                            );
                        }
                    }
                }

                // Update search results with pricing
                for hotel in &mut domain_list.hotel_results {
                    if let Some(price) = hotel_prices.get(&hotel.hotel_code) {
                        hotel.price = Some(price.clone());
                    }
                }

                tracing::info!(
                    "Search completed: {} hotels, {} with prices",
                    domain_list.hotel_results.len(),
                    domain_list
                        .hotel_results
                        .iter()
                        .filter(|h| h.price.is_some())
                        .count()
                );

                Ok(domain_list)
            }
            Err(e) => {
                // Log error but return list without prices (graceful degradation)
                tracing::warn!("Failed to fetch rates for search results: {:?}", e);
                Ok(domain_list)
            }
        }
    }

    async fn get_hotel_static_details(
        &self,
        hotel_id: &str,
    ) -> Result<DomainHotelStaticDetails, ProviderError> {
        let resp = self.client.get_hotel_static_details(hotel_id).await?;
        Ok(LiteApiMapper::map_liteapi_details_to_domain_static(
            resp.data,
        ))
    }

    async fn get_hotel_rates(
        &self,
        criteria: DomainHotelInfoCriteria,
    ) -> Result<DomainGroupedRoomRates, ProviderError> {
        let req = LiteApiMapper::map_domain_info_to_liteapi_rates(
            &criteria,
            self.client.currency(),
            self.room_mapping,
        )?;
        let resp = self.client.get_hotel_rates(&req).await?;
        let rates = LiteApiMapper::map_liteapi_rates_response_to_domain(resp);

        // Fetch static details for image/amenity enrichment if we are querying a single hotel
        let static_details_result = if criteria.hotel_ids.len() == 1 {
            self.get_hotel_static_details(&criteria.hotel_ids[0])
                .await
                .ok()
        } else {
            None
        };

        let static_rooms_ref = static_details_result.as_ref().map(|d| d.rooms.as_slice());

        Ok(crate::liteapi::grouping::group_liteapi_rates(
            rates,
            static_rooms_ref,
        ))
    }

    async fn get_min_rates(
        &self,
        criteria: DomainHotelSearchCriteria,
        hotel_ids: Vec<String>,
    ) -> Result<HashMap<String, DomainPrice>, ProviderError> {
        if hotel_ids.is_empty() {
            return Ok(HashMap::new());
        }

        // Map request
        let req = LiteApiMapper::map_domain_search_to_liteapi_min_rates(
            &criteria,
            hotel_ids,
            self.client.currency(),
        )?;

        // Call API
        let resp = self.client.get_min_rates(&req).await?;

        // Map response
        Ok(LiteApiMapper::map_liteapi_min_rates_response_to_domain(
            resp,
            self.client.currency(),
        ))
    }

    async fn block_room(
        &self,
        request: DomainBlockRoomRequest,
    ) -> Result<DomainBlockRoomResponse, ProviderError> {
        let req = LiteApiMapper::map_domain_block_to_liteapi_prebook(&request)?;
        let resp = self.client.prebook(&req).await?;
        Ok(LiteApiMapper::map_liteapi_prebook_to_domain_block(
            resp, &request,
        ))
    }

    async fn book_room(
        &self,
        request: DomainBookRoomRequest,
    ) -> Result<DomainBookRoomResponse, ProviderError> {
        let req = LiteApiMapper::map_domain_book_to_liteapi_book(&request)?;
        let resp = self.client.book(&req).await?;
        Ok(LiteApiMapper::map_liteapi_book_to_domain_book(
            resp, &request,
        ))
    }

    async fn get_booking_details(
        &self,
        request: DomainGetBookingRequest,
    ) -> Result<DomainGetBookingResponse, ProviderError> {
        let req = LiteApiMapper::map_domain_get_booking_to_liteapi(&request)?;
        let resp = self.client.get_booking_details(&req).await?;
        LiteApiMapper::map_liteapi_get_booking_to_domain(resp)
    }
}

#[async_trait]
impl PlaceProviderPort for LiteApiDriver {
    fn name(&self) -> &'static str {
        ProviderNames::LiteApi
    }

    async fn search_places(
        &self,
        payload: DomainPlacesSearchPayload,
    ) -> Result<DomainPlacesResponse, ProviderError> {
        let req = LiteApiMapper::map_places_domain_to_liteapi(payload);
        let resp = self.client.get_places(&req).await?;
        Ok(LiteApiMapper::map_liteapi_places_response_to_domain(resp))
    }

    async fn get_single_place_details(
        &self,
        payload: DomainPlaceDetailsPayload,
    ) -> Result<DomainPlaceDetails, ProviderError> {
        let req = LiteApiMapper::map_places_id_domain_to_liteapi(payload);
        let resp = self.client.get_place(&req).await?;
        Ok(LiteApiMapper::map_liteapi_place_details_response_to_domain(
            resp,
        ))
    }
}
