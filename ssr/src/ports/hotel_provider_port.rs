use crate::api::ApiError;
use crate::application_services::filter_types::UISearchFilters;
use crate::domain::{
    DomainHotelDetails,
    // <!-- Future domain types for room operations, booking, etc. -->
    // DomainRoomOptions, DomainBlockRoomRequest, DomainBlockRoomResponse,
    // DomainBookRoomRequest, DomainBookRoomResponse,
    DomainHotelInfoCriteria,
    DomainHotelSearchCriteria,
    DomainHotelSearchResponse,
};
use async_trait::async_trait;

#[derive(Debug)]
pub struct ProviderError(pub String);

impl From<ApiError> for ProviderError {
    fn from(e: ApiError) -> Self {
        ProviderError(e.to_string())
    }
}

impl From<error_stack::Report<ApiError>> for ProviderError {
    fn from(e: error_stack::Report<ApiError>) -> Self {
        ProviderError(e.to_string())
    }
}

#[async_trait]
pub trait HotelProviderPort: Send + Sync {
    // <!-- Core search method that takes both essential criteria and UI filters -->
    // <!-- The adapter will try to use UI filters if the specific provider API supports them -->
    async fn search_hotels(
        &self,
        criteria: DomainHotelSearchCriteria,
        ui_filters: &UISearchFilters,
    ) -> Result<DomainHotelSearchResponse, ProviderError>;

    async fn get_hotel_details(
        &self,
        criteria: DomainHotelInfoCriteria,
    ) -> Result<DomainHotelDetails, ProviderError>;

    // <!-- Future operations to be implemented -->
    // async fn get_room_options(&self, hotel_id: String, token: String) -> Result<DomainRoomOptions, ProviderError>;

    // async fn block_room(&self, block_request: DomainBlockRoomRequest) -> Result<DomainBlockRoomResponse, ProviderError>;

    // async fn book_room(&self, book_request: DomainBookRoomRequest) -> Result<DomainBookRoomResponse, ProviderError>;

    // async fn get_booking_details(&self, booking_id: String) -> Result<DomainBookingDetails, ProviderError>;
}
