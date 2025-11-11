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
    HotelRate,
    HotelBlockRoom,
    HotelBookRoom,
    GetBookingDetails,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProviderError(pub Arc<ProviderErrorDetails>);

// Added Display implementation
impl fmt::Display for ProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let error_message = match self.0.error_step {
            ProviderSteps::PlaceSearch => {
                "There was a problem searching for the place. Please try again."
            }
            ProviderSteps::PlaceDetails => {
                "Could not retrieve details for the selected place. Please try again."
            }
            ProviderSteps::HotelSearch => {
                "There was a problem searching for hotels. Please try again."
            }
            ProviderSteps::HotelDetails => {
                "Could not retrieve details for the selected hotel. Please try again."
            }
            ProviderSteps::HotelRate => {
                "This hotel is fully booked for your selected dates. Please choose different dates or another hotel."
            }
            ProviderSteps::HotelBlockRoom => {
                "We were unable to reserve the room. Please try again."
            }
            ProviderSteps::HotelBookRoom => {
                "There was a problem confirming your booking. Please try again."
            }
            ProviderSteps::GetBookingDetails => {
                "Could not retrieve booking details. Please try again."
            }
        };
        write!(f, "{}", error_message)
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
