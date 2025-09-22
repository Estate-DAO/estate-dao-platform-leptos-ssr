use crate::api::ApiError;
// use crate::api::{ApiError, BlockRoomResponse};
use crate::log;
use crate::ports::hotel_provider_port::{ProviderError, ProviderSteps};
use crate::view_state_layer::GlobalStateForLeptos;
use base64::write;
use leptos::*;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiErrorType {
    PlaceSearch,
    PlaceDetails,
    BlockRoom,
    BookRoom,
    HotelSearch,
    HotelInfo,
    HotelRoom,
    Payment,
    GetBookingDetails,
    Generic,
}

impl From<ProviderError> for ApiErrorType {
    fn from(value: ProviderError) -> Self {
        match value.0.error_step {
            ProviderSteps::HotelSearch => ApiErrorType::HotelSearch,
            ProviderSteps::HotelDetails => ApiErrorType::HotelInfo,
            ProviderSteps::HotelBlockRoom => ApiErrorType::BlockRoom,
            ProviderSteps::HotelBookRoom => ApiErrorType::BookRoom,
            ProviderSteps::GetBookingDetails => ApiErrorType::GetBookingDetails,
            ProviderSteps::PlaceSearch => ApiErrorType::PlaceSearch,
            ProviderSteps::PlaceDetails => ApiErrorType::PlaceDetails,
        }
    }
}

impl fmt::Display for ApiErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiErrorType::BlockRoom => write!(f, "Block Room"),
            ApiErrorType::BookRoom => write!(f, "Book Room"),
            ApiErrorType::HotelSearch => write!(f, "Hotel Search"),
            ApiErrorType::HotelInfo => write!(f, "Hotel Information"),
            ApiErrorType::HotelRoom => write!(f, "Hotel Room"),
            ApiErrorType::Payment => write!(f, "Payment"),
            ApiErrorType::GetBookingDetails => write!(f, "Get Booking Details"),
            ApiErrorType::Generic => write!(f, "API"),
            ApiErrorType::PlaceSearch => write!(f, "Place Search"),
            ApiErrorType::PlaceDetails => write!(f, "Place Details"),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ApiErrorState {
    pub has_error: RwSignal<bool>,
    pub error_type: RwSignal<Option<ApiErrorType>>,
    pub error_message: RwSignal<String>,
    pub show_popup: RwSignal<bool>,
}
impl GlobalStateForLeptos for ApiErrorState {}

impl ApiErrorState {
    pub fn from_leptos_context() -> Self {
        Self::get()
    }

    pub fn reset() {
        let this = Self::from_leptos_context();
        this.has_error.set(false);
        this.error_type.set(None);
        this.error_message.set(String::new());
        this.show_popup.set(false);
    }

    pub fn set_error(&self, error_type: ApiErrorType, message: String) {
        self.has_error.set(true);
        self.error_type.set(Some(error_type));
        self.error_message.set(message);
        self.show_popup.set(true);
    }

    pub fn set_error_from_api_error(&self, error_type: ApiErrorType, error: &ApiError) {
        let message = error.to_string();
        self.set_error(error_type, message);
    }

    pub fn clear_error(&self) {
        self.has_error.set(false);
        self.error_type.set(None);
        self.error_message.set(String::new());
        self.show_popup.set(false);
    }

    pub fn dismiss_popup(&self) {
        self.show_popup.set(false);
    }

    pub fn has_error_of_type(&self, error_type: ApiErrorType) -> bool {
        if let Some(current_type) = self.error_type.get() {
            self.has_error.get() && current_type == error_type
        } else {
            false
        }
    }

    // /// Handles error state for BlockRoomResponse with optional custom error message
    // pub fn handle_block_room_response(
    //     &self,
    //     result: Option<BlockRoomResponse>,
    //     custom_message: Option<String>,
    // ) -> bool {
    //     if result.is_none() {
    //         let message =
    //             custom_message.unwrap_or_else(|| "API returned an empty response".to_string());
    //         self.set_error(ApiErrorType::BlockRoom, message);
    //         return true;
    //     }

    //     if let Some(BlockRoomResponse::Success(response)) = &result {
    //         if response.status == 400 || response.status == 0 {
    //             log!(
    //                 "[ApiErrorState] BlockRoom API error: {:?}",
    //                 &response.message
    //             );
    //             let message = custom_message.unwrap_or_else(|| {
    //                 "Selected room is already booked. Please choose another room.".to_string()
    //             });
    //             self.set_error(ApiErrorType::BlockRoom, message);
    //             return true;
    //         }
    //     }

    //     if let Some(BlockRoomResponse::Failure(failure_response)) = &result {
    //         if failure_response.status == 400 {
    //             log!(
    //                 "[ApiErrorState] BlockRoom API failure: {:?}",
    //                 &failure_response.message
    //             );
    //             let message = custom_message.unwrap_or_else(|| {
    //                 failure_response
    //                     .message
    //                     .clone()
    //                     .unwrap_or("Room is no longer available".to_string())
    //             });
    //             self.set_error(ApiErrorType::BlockRoom, message);
    //             return true;
    //         }
    //     }

    //     false
    // }
}
