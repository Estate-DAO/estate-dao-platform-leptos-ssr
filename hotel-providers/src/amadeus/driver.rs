use async_trait::async_trait;
use std::collections::HashMap;

use crate::amadeus::client::AmadeusClient;
use crate::domain::{
    DomainBlockRoomRequest, DomainBlockRoomResponse, DomainBookRoomRequest, DomainBookRoomResponse,
    DomainGetBookingRequest, DomainGetBookingResponse, DomainGroupedRoomRates,
    DomainHotelInfoCriteria, DomainHotelListAfterSearch, DomainHotelSearchCriteria,
    DomainHotelStaticDetails, DomainPrice,
};
use crate::ports::{
    HotelProviderPort, ProviderError, ProviderKeys, ProviderNames, ProviderSteps, UISearchFilters,
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
        _criteria: DomainHotelSearchCriteria,
        _ui_filters: UISearchFilters,
    ) -> Result<DomainHotelListAfterSearch, ProviderError> {
        Err(ProviderError::other(
            self.name(),
            ProviderSteps::HotelSearch,
            "Amadeus search is not implemented yet",
        ))
    }

    async fn get_hotel_static_details(
        &self,
        _hotel_id: &str,
    ) -> Result<DomainHotelStaticDetails, ProviderError> {
        Err(ProviderError::other(
            self.name(),
            ProviderSteps::HotelDetails,
            "Amadeus hotel details are not implemented yet",
        ))
    }

    async fn get_hotel_rates(
        &self,
        _criteria: DomainHotelInfoCriteria,
    ) -> Result<DomainGroupedRoomRates, ProviderError> {
        Err(ProviderError::other(
            self.name(),
            ProviderSteps::HotelRate,
            "Amadeus hotel rates are not implemented yet",
        ))
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
