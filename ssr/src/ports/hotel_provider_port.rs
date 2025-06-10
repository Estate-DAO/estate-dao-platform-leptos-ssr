use crate::api::ApiError;
use crate::application_services::filter_types::UISearchFilters;
use crate::domain::{
    DomainHotelDetails, DomainHotelInfoCriteria, DomainHotelListAfterSearch,
    DomainHotelSearchCriteria,
};
use crate::ports::ProviderNames;
use async_trait::async_trait;
use futures::future::{BoxFuture, FutureExt, LocalBoxFuture};
use std::fmt;
use std::future::Future;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
pub struct ProviderErrorDetails {
    pub provider_name: ProviderNames,
    pub api_error: ApiError,
    pub error_step: ProviderSteps,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProviderSteps {
    HotelSearch,
    HotelDetails,
    HotelBlockRoom,
    HotelBookRoom,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProviderError(pub Arc<ProviderErrorDetails>);

// Added Display implementation
impl fmt::Display for ProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Provider '{:?}' failed during '{:?}' step. Details: {}",
            self.0.provider_name, self.0.error_step, self.0.api_error
        )
    }
}

impl ProviderError {
    pub fn from_api_error(
        api_error: ApiError,
        provider_name: ProviderNames,
        error_step: ProviderSteps,
    ) -> Self {
        ProviderError(Arc::new(ProviderErrorDetails {
            provider_name,
            api_error,
            error_step,
        }))
    }
}
#[async_trait::async_trait]
pub trait HotelProviderPort: Send + Sync + 'static {
    // pub trait HotelProviderPort: Send + Sync + 'static {
    // <!-- Core search method that takes both essential criteria and UI filters -->
    // <!-- The adapter will try to use UI filters if the specific provider API supports them -->
    async fn search_hotels(
        // async fn search_hotels(
        &self,
        criteria: DomainHotelSearchCriteria,
        ui_filters: UISearchFilters,
    ) -> LocalBoxFuture<'_, Result<DomainHotelListAfterSearch, ProviderError>>;

    async fn get_hotel_details(
        &self,
        criteria: DomainHotelInfoCriteria,
    ) -> LocalBoxFuture<'_, Result<DomainHotelDetails, ProviderError>>;

    // <!-- Future operations to be implemented -->
    // async fn get_room_options(&self, hotel_id: String, token: String) -> Result<DomainRoomOptions, ProviderError>;

    // async fn block_room(&self, block_request: DomainBlockRoomRequest) -> Result<DomainBlockRoomResponse, ProviderError>;

    // async fn book_room(&self, book_request: DomainBookRoomRequest) -> Result<DomainBookRoomResponse, ProviderError>;

    // async fn get_booking_details(&self, booking_id: String) -> Result<DomainBookingDetails, ProviderError>;
}
