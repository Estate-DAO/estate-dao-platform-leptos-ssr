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
    PlaceSearch,
    PlaceDetails,
    HotelSearch,
    HotelDetails,
    HotelBlockRoom,
    HotelBookRoom,
    GetBookingDetails,
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

    pub fn validation_error(provider_name: ProviderNames, message: String) -> Self {
        ProviderError(Arc::new(ProviderErrorDetails {
            provider_name,
            api_error: ApiError::Other(message),
            error_step: ProviderSteps::GetBookingDetails,
        }))
    }
}
