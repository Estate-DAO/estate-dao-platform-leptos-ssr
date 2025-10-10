use std::future::Future;

use crate::{
    application_services::UISearchFilters,
    domain::{
        DomainBlockRoomRequest, DomainBlockRoomResponse, DomainBookRoomRequest,
        DomainBookRoomResponse, DomainGetBookingRequest, DomainGetBookingResponse,
        DomainHotelDetails, DomainHotelDetailsWithoutRates, DomainHotelInfoCriteria,
        DomainHotelListAfterSearch, DomainHotelSearchCriteria, DomainPlaceDetails,
        DomainPlaceDetailsPayload, DomainPlacesResponse, DomainPlacesSearchPayload,
    },
    ports::hotel_provider_port::ProviderError,
};

#[async_trait::async_trait]
pub trait HotelProviderPort {
    // <!-- Core search method that takes both essential criteria and UI filters -->
    // <!-- The adapter will try to use UI filters if the specific provider API supports them -->

    async fn search_hotels(
        // async fn search_hotels(
        &self,
        criteria: DomainHotelSearchCriteria,
        ui_filters: UISearchFilters,
    ) -> Result<DomainHotelListAfterSearch, ProviderError>;

    // fn search_hotels(
    //     // async fn search_hotels(
    //     &self,
    //     criteria: DomainHotelSearchCriteria,
    //     ui_filters: UISearchFilters,
    // ) -> impl Future<Output = Result<DomainHotelListAfterSearch, ProviderError>> + Send;
    // // ) -> LocalBoxFuture<'_, Result<DomainHotelListAfterSearch, ProviderError>>;

    async fn get_single_hotel_details(
        &self,
        criteria: DomainHotelInfoCriteria,
    ) -> Result<DomainHotelDetails, ProviderError>;

    // <!-- Get hotel details without rates - returns basic hotel information without pricing -->
    async fn get_hotel_details_without_rates(
        &self,
        hotel_id: String,
    ) -> Result<DomainHotelDetailsWithoutRates, ProviderError>;

    // fn get_hotel_details(
    //     &self,
    //     criteria: DomainHotelInfoCriteria,
    // ) -> impl Future<Output = Result<DomainHotelDetails, ProviderError>> + Send;

    // <!-- Block room operation - reserves room before payment -->
    async fn block_room(
        &self,
        block_request: DomainBlockRoomRequest,
    ) -> Result<DomainBlockRoomResponse, ProviderError>;

    // <!-- Book room operation - finalizes the booking with payment -->
    async fn book_room(
        &self,
        book_request: DomainBookRoomRequest,
    ) -> Result<DomainBookRoomResponse, ProviderError>;

    // <!-- Get hotel rates - for providers that need separate rates call -->
    async fn get_hotel_rates(
        &self,
        criteria: DomainHotelInfoCriteria,
    ) -> Result<DomainHotelDetails, ProviderError>;

    // <!-- Get booking details - retrieve booking information -->
    async fn get_booking_details(
        &self,
        request: DomainGetBookingRequest,
    ) -> Result<DomainGetBookingResponse, ProviderError>;

    // <!-- Future operations to be implemented -->
    // async fn get_room_options(&self, hotel_id: String, token: String) -> Result<DomainRoomOptions, ProviderError>;
}

#[async_trait::async_trait]
pub trait PlaceProviderPort {
    // <!-- Core search method that takes both essential criteria and UI filters -->
    // <!-- The adapter will try to use UI filters if the specific provider API supports them -->

    async fn search_places(
        // async fn search_hotels(
        &self,
        criteria: DomainPlacesSearchPayload,
    ) -> Result<DomainPlacesResponse, ProviderError>;

    async fn get_single_place_details(
        &self,
        payload: DomainPlaceDetailsPayload,
    ) -> Result<DomainPlaceDetails, ProviderError>;
}
