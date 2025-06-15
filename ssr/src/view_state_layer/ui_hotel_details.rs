use crate::domain::DomainHotelDetails;
use leptos::*;

use super::GlobalStateForLeptos;

#[derive(Debug, Clone, Default)]
pub struct HotelDetailsUIState {
    pub hotel_details: RwSignal<Option<DomainHotelDetails>>,
    pub loading: RwSignal<bool>,
    pub error: RwSignal<Option<String>>,
}

impl HotelDetailsUIState {
    pub fn from_leptos_context() -> Self {
        expect_context()
    }

    pub fn set_hotel_details(details: Option<DomainHotelDetails>) {
        let this: Self = expect_context();
        this.hotel_details.set(details);
    }

    pub fn set_loading(loading: bool) {
        let this: Self = expect_context();
        this.loading.set(loading);
    }

    pub fn set_error(error: Option<String>) {
        let this: Self = expect_context();
        this.error.set(error);
    }

    pub fn reset() {
        let this: Self = expect_context();
        this.hotel_details.set(None);
        this.loading.set(false);
        this.error.set(None);
    }

    pub fn get_hotel_details() -> Option<DomainHotelDetails> {
        let this: Self = expect_context();
        this.hotel_details.get()
    }

    pub fn is_loading() -> bool {
        let this: Self = expect_context();
        this.loading.get()
    }

    pub fn get_error() -> Option<String> {
        let this: Self = expect_context();
        this.error.get()
    }
}

impl GlobalStateForLeptos for HotelDetailsUIState {}
