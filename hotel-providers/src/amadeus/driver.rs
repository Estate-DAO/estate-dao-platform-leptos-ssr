use async_trait::async_trait;
use std::collections::HashMap;

use crate::amadeus::client::AmadeusClient;
use crate::amadeus::mapper::AmadeusMapper;
use crate::domain::{
    DomainBlockRoomRequest, DomainBlockRoomResponse, DomainBookRoomRequest, DomainBookRoomResponse,
    DomainGetBookingRequest, DomainGetBookingResponse, DomainGroupedRoomRates,
    DomainHotelInfoCriteria, DomainHotelListAfterSearch, DomainHotelSearchCriteria,
    DomainHotelStaticDetails, DomainPrice,
};
use crate::ports::{
    HotelProviderPort, ProviderError, ProviderErrorKind, ProviderKeys, ProviderNames,
    ProviderSteps, UISearchFilters,
};

#[derive(Clone, Debug, Default)]
pub struct AmadeusDriver {
    client: AmadeusClient,
}

impl AmadeusDriver {
    pub fn new(client: AmadeusClient) -> Self {
        Self { client }
    }

    pub fn new_mock() -> Self {
        Self::new(AmadeusClient::new_mock())
    }

    pub fn client(&self) -> &AmadeusClient {
        &self.client
    }
}

#[async_trait]
impl HotelProviderPort for AmadeusDriver {
    fn key(&self) -> &'static str {
        ProviderKeys::Amadeus
    }

    fn name(&self) -> &'static str {
        ProviderNames::Amadeus
    }

    async fn search_hotels(
        &self,
        criteria: DomainHotelSearchCriteria,
        _ui_filters: UISearchFilters,
    ) -> Result<DomainHotelListAfterSearch, ProviderError> {
        let (latitude, longitude) = criteria.latitude.zip(criteria.longitude).ok_or_else(|| {
            ProviderError::new(
                self.name(),
                ProviderErrorKind::InvalidRequest,
                ProviderSteps::HotelSearch,
                "Amadeus search requires latitude and longitude",
            )
        })?;

        let hotel_list = self.client.get_hotels_by_geocode(latitude, longitude).await?;
        if hotel_list.data.is_empty() {
            return Ok(DomainHotelListAfterSearch {
                hotel_results: Vec::new(),
                pagination: None,
                provider: Some(self.name().to_string()),
            });
        }

        let hotel_ids = hotel_list
            .data
            .iter()
            .map(|hotel| hotel.hotel_id.clone())
            .collect::<Vec<_>>();
        let offers = self.client.get_hotel_offers(&hotel_ids, &criteria).await?;

        Ok(AmadeusMapper::map_search_to_domain(
            hotel_list,
            offers,
            &criteria.pagination,
        ))
    }

    async fn get_hotel_static_details(
        &self,
        hotel_id: &str,
    ) -> Result<DomainHotelStaticDetails, ProviderError> {
        let response = self
            .client
            .get_hotels_by_ids(&[hotel_id.to_string()])
            .await?;

        AmadeusMapper::map_hotel_details_to_domain(response).ok_or_else(|| {
            ProviderError::not_found(
                self.name(),
                ProviderSteps::HotelDetails,
                "Amadeus hotel not found",
            )
        })
    }

    async fn get_hotel_rates(
        &self,
        criteria: DomainHotelInfoCriteria,
    ) -> Result<DomainGroupedRoomRates, ProviderError> {
        if criteria.hotel_ids.is_empty() {
            return Ok(DomainGroupedRoomRates {
                room_groups: Vec::new(),
                provider: Some(self.name().to_string()),
            });
        }

        let offers = self
            .client
            .get_hotel_offers(&criteria.hotel_ids, &criteria.search_criteria)
            .await?;

        Ok(AmadeusMapper::map_offers_to_grouped_rates(offers))
    }

    async fn get_min_rates(
        &self,
        _criteria: DomainHotelSearchCriteria,
        _hotel_ids: Vec<String>,
    ) -> Result<HashMap<String, DomainPrice>, ProviderError> {
        Err(ProviderError::other(
            self.name(),
            ProviderSteps::HotelRate,
            "Amadeus min rates are not implemented yet",
        ))
    }

    async fn block_room(
        &self,
        _block_request: DomainBlockRoomRequest,
    ) -> Result<DomainBlockRoomResponse, ProviderError> {
        Err(ProviderError::other(
            self.name(),
            ProviderSteps::HotelBlockRoom,
            "Amadeus block room is not implemented yet",
        ))
    }

    async fn book_room(
        &self,
        _book_request: DomainBookRoomRequest,
    ) -> Result<DomainBookRoomResponse, ProviderError> {
        Err(ProviderError::other(
            self.name(),
            ProviderSteps::HotelBookRoom,
            "Amadeus booking is not implemented yet",
        ))
    }

    async fn get_booking_details(
        &self,
        _request: DomainGetBookingRequest,
    ) -> Result<DomainGetBookingResponse, ProviderError> {
        Err(ProviderError::other(
            self.name(),
            ProviderSteps::GetBookingDetails,
            "Amadeus booking lookup is not implemented yet",
        ))
    }
}
