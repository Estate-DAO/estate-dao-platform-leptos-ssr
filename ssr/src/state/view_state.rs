use crate::{
    api::{HotelSearchRequest, HotelSearchResponse},
    component::{GuestSelection, SelectedDateRange},
};
use leptos::RwSignal;
use leptos::*;
use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct HotelInfoCtx {
    pub hotel_code: RwSignal<Option<String>>,
}

#[derive(Clone, Default, Debug)]
pub struct Names {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(Clone, Default, Debug)]
pub struct BlockRoomCtx {
    pub name: RwSignal<Vec<Names>>,
    pub email: RwSignal<Option<String>>,
    pub phone_number: RwSignal<Option<String>>
}

impl BlockRoomCtx {
    pub fn set_first_name(name: Vec<Names>) {
        let this: Self = expect_context();

        this.name.set(name);
    }
    
    pub fn set_phone_number(phone_number: String) {
        let this: Self = expect_context();

        this.phone_number.set(Some(phone_number));
    }
    pub fn set_email(email: String) {
        let this: Self = expect_context();

        this.email.set(Some(email));
    }
}
