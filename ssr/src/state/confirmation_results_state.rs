use crate::api::payments::ports::GetPaymentStatusResponse;
use crate::api::{BookRoomResponse, HotelResult};
use crate::canister::backend::{self};
use leptos::*;

use crate::component::SelectedDateRange;
// use crate::page::SortedRoom;

use super::hotel_details_state::RoomDetailsForPricingComponent;
use super::view_state::{AdultDetail, ChildDetail};
use super::GlobalStateForLeptos;

#[derive(Debug, Clone, Default)]
pub struct ConfirmationResultsState {
    pub booking_details: RwSignal<Option<backend::Booking>>,
    pub adults: RwSignal<Vec<AdultDetail>>,
    pub children: RwSignal<Vec<ChildDetail>>,
    pub date_range: RwSignal<SelectedDateRange>,
    // hotel data
    pub hotel_code: RwSignal<String>,
    pub hotel_name: RwSignal<String>,
    pub hotel_image: RwSignal<String>,
    pub hotel_location: RwSignal<String>,
    // pub sorted_rooms: RwSignal<Vec<SortedRoom>>,
    pub sorted_rooms: RwSignal<Vec<RoomDetailsForPricingComponent>>,
    pub hotel_token: RwSignal<String>,
    pub block_room_id: RwSignal<String>,
    // payment data
    pub payment_status_results_from_api: RwSignal<Option<GetPaymentStatusResponse>>, // book room
}

impl GlobalStateForLeptos for ConfirmationResultsState {}

impl ConfirmationResultsState {
    pub fn display(&self) -> String {
        let json_repr = serde_json::json!({
            "hotel_code": self.hotel_code.get_untracked(),
            "hotel_name": self.hotel_name.get_untracked(),
            "hotel_image": self.hotel_image.get_untracked(),
            "hotel_location": self.hotel_location.get_untracked(),
            "date_range": self.date_range.get_untracked(),
            "booking_details": self.booking_details.get_untracked(),
            "hotel_token": self.hotel_token.get_untracked(),
            "block_room_id": self.block_room_id.get_untracked(),
            "payment_status_results_from_api": self.payment_status_results_from_api.get_untracked()
        });

        serde_json::to_string_pretty(&json_repr)
            .unwrap_or_else(|_| "Failed to serialize".to_string())
    }

    // setters

    pub fn set_booking_details(booking_details: Option<backend::Booking>) {
        Self::get().booking_details.set(booking_details);
    }

    pub fn set_adults(adults: Vec<AdultDetail>) {
        Self::get().adults.set(adults);
    }

    pub fn set_children(children: Vec<ChildDetail>) {
        Self::get().children.set(children);
    }
    pub fn set_date_range(date_range: SelectedDateRange) {
        Self::get().date_range.set(date_range);
    }

    pub fn set_selected_hotel_details(
        code: String,
        name: String,
        image: String,
        location: String,
        hotel_token: String,
        block_room_id: String,
    ) {
        let this = Self::get();
        this.hotel_code.set(code);
        this.hotel_name.set(name);
        this.hotel_image.set(image);
        this.hotel_location.set(location);
        this.hotel_token.set(hotel_token);
        this.block_room_id.set(block_room_id);
    }

    pub fn set_sorted_rooms(sorted_rooms: Vec<RoomDetailsForPricingComponent>) {
        Self::get().sorted_rooms.set(sorted_rooms);
    }

    pub fn set_payment_results_from_api(resp: Option<GetPaymentStatusResponse>) {
        Self::get().payment_status_results_from_api.set(resp);
    }

    // getters
    pub fn get_payment_status() -> Option<String> {
        match Self::get().booking_details.get() {
            Some(booking) => Some(booking.payment_details.payment_status.to_string()),
            None => None,
        }
    }
    pub fn get_destination() -> Option<backend::Destination> {
        Self::get()
            .booking_details
            .get()
            .and_then(|booking| booking.get_destination())
    }

    pub fn payment_status_from_backend_is_finished_check() -> bool {
        Self::get_payment_status().map_or(false, |status| status == "finished")
    }

    pub fn payment_status_from_api_is_finished_check() -> bool {
        Self::payment_status_response_from_api().map_or(false, |status| status.is_finished())
    }

    pub fn payment_status_response_from_api() -> Option<GetPaymentStatusResponse> {
        Self::get().payment_status_results_from_api.get_untracked()
    }

    pub fn maybe_get_hotel_token() -> Option<String> {
        Self::maybe_string(Self::get().hotel_token.get_untracked())
    }

    pub fn maybe_get_block_room_id() -> Option<String> {
        Self::maybe_string(Self::get().block_room_id.get_untracked())
    }

    pub fn get_adult_details() -> Vec<AdultDetail> {
        Self::get().adults.get_untracked()
    }

    pub fn get_children_details() -> Vec<ChildDetail> {
        Self::get().children.get_untracked()
    }

    fn maybe_string(s: String) -> Option<String> {
        if s.is_empty() {
            None
        } else {
            Some(s)
        }
    }
}
