use std::future::Future;

use crate::{
    application_services::UISearchFilters,
    domain::{
        DomainHotelDetails, DomainHotelInfoCriteria, DomainHotelListAfterSearch,
        DomainHotelSearchCriteria,
    },
    ports::hotel_provider_port::ProviderError,
};

#[async_trait::async_trait]
// pub trait HotelProviderPort: Send + Sync + 'static {
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

    async fn get_hotel_details(
        &self,
        criteria: DomainHotelInfoCriteria,
    ) -> Result<DomainHotelDetails, ProviderError>;

    // fn get_hotel_details(
    //     &self,
    //     criteria: DomainHotelInfoCriteria,
    // ) -> impl Future<Output = Result<DomainHotelDetails, ProviderError>> + Send;

    // <!-- Future operations to be implemented -->
    // async fn get_room_options(&self, hotel_id: String, token: String) -> Result<DomainRoomOptions, ProviderError>;

    // async fn block_room(&self, block_request: DomainBlockRoomRequest) -> Result<DomainBlockRoomResponse, ProviderError>;

    // async fn book_room(&self, book_request: DomainBookRoomRequest) -> Result<DomainBookRoomResponse, ProviderError>;

    // async fn get_booking_details(&self, booking_id: String) -> Result<DomainBookingDetails, ProviderError>;
}
