use crate::booking::client::{BookingApiClient, BookingClient, BookingMockClient};
use crate::booking::mapper::BookingMapper;
use crate::booking::models::AccommodationsDetailsInput;
use crate::domain::*;
use crate::ports::{HotelProviderPort, ProviderError, ProviderKeys, ProviderNames, UISearchFilters};
use async_trait::async_trait;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct BookingDriver {
    client: BookingApiClient,
}

impl BookingDriver {
    pub fn new_real(token: String, affiliate_id: i64, base_url: String, currency: String) -> Self {
        Self {
            client: BookingApiClient::Real(BookingClient::new(
                token,
                affiliate_id,
                base_url,
                currency,
            )),
        }
    }

    pub fn new_mock(currency: String) -> Self {
        Self {
            client: BookingApiClient::Mock(BookingMockClient::new(currency)),
        }
    }
}

#[async_trait]
impl HotelProviderPort for BookingDriver {
    fn key(&self) -> &'static str {
        ProviderKeys::Booking
    }

    fn name(&self) -> &'static str {
        ProviderNames::Booking
    }

    async fn search_hotels(
        &self,
        criteria: DomainHotelSearchCriteria,
        ui_filters: UISearchFilters,
    ) -> Result<DomainHotelListAfterSearch, ProviderError> {
        let req = BookingMapper::map_domain_search_to_booking(
            &criteria,
            &ui_filters,
            self.client.currency(),
        );
        let resp = self.client.search_accommodations(&req).await?;

        if resp.data.is_empty() {
            return Ok(DomainHotelListAfterSearch {
                hotel_results: Vec::new(),
                pagination: None,
                provider: Some(ProviderNames::Booking.to_string()),
            });
        }

        let ids: Vec<i64> = resp.data.iter().map(|d| d.id).collect();
        let details = self
            .client
            .get_accommodation_details(&AccommodationsDetailsInput {
                accommodations: Some(ids),
                extras: Some(vec!["photos".to_string(), "facilities".to_string()]),
                languages: Some(vec!["en-gb".to_string()]),
            })
            .await
            .ok();

        let details_map = details.map(|d| {
            d.data
                .into_iter()
                .map(|detail| (detail.id, detail))
                .collect::<HashMap<_, _>>()
        });

        Ok(BookingMapper::map_booking_search_to_domain(
            resp,
            details_map,
        ))
    }

    async fn get_hotel_static_details(
        &self,
        hotel_id: &str,
    ) -> Result<DomainHotelStaticDetails, ProviderError> {
        let req = AccommodationsDetailsInput {
            accommodations: Some(vec![hotel_id.parse::<i64>().unwrap_or_default()]),
            extras: Some(vec![
                "description".to_string(),
                "facilities".to_string(),
                "photos".to_string(),
                "rooms".to_string(),
            ]),
            languages: Some(vec!["en-gb".to_string()]),
        };
        let resp = self.client.get_accommodation_details(&req).await?;
        let detail = resp.data.into_iter().next().ok_or_else(|| {
            ProviderError::not_found(
                "Booking.com",
                crate::ports::ProviderSteps::HotelDetails,
                "Accommodation not found",
            )
        })?;

        Ok(BookingMapper::map_booking_details_to_domain_static(detail))
    }

    async fn get_hotel_rates(
        &self,
        criteria: DomainHotelInfoCriteria,
    ) -> Result<DomainGroupedRoomRates, ProviderError> {
        let req = BookingMapper::map_domain_info_to_booking_availability(
            &criteria,
            self.client.currency(),
        )?;
        let resp = self.client.get_availability(&req).await?;
        Ok(BookingMapper::map_booking_availability_to_grouped_rates(
            resp,
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
        let req = BookingMapper::map_domain_search_to_booking_bulk_availability(
            &criteria,
            hotel_ids,
            self.client.currency(),
        )?;
        let resp = self.client.get_bulk_availability(&req).await?;
        Ok(BookingMapper::map_booking_bulk_availability_to_domain(
            resp,
            self.client.currency(),
        ))
    }

    async fn block_room(
        &self,
        request: DomainBlockRoomRequest,
    ) -> Result<DomainBlockRoomResponse, ProviderError> {
        let req =
            BookingMapper::map_domain_block_to_booking_preview(&request, self.client.currency())?;
        let resp = self.client.orders_preview(&req).await?;
        BookingMapper::map_booking_preview_to_domain_block(resp)
    }

    async fn book_room(
        &self,
        request: DomainBookRoomRequest,
    ) -> Result<DomainBookRoomResponse, ProviderError> {
        let req = BookingMapper::map_domain_book_to_booking_create(&request)?;
        let resp = self.client.orders_create(&req).await?;
        BookingMapper::map_booking_create_to_domain_book(resp, &request)
    }

    async fn get_booking_details(
        &self,
        request: DomainGetBookingRequest,
    ) -> Result<DomainGetBookingResponse, ProviderError> {
        let req = BookingMapper::map_domain_get_booking_to_booking_details(
            &request,
            self.client.currency(),
        );
        let resp = self.client.orders_details_accommodations(&req).await?;
        Ok(BookingMapper::map_booking_details_to_domain(resp))
    }
}
