use crate::{
    // api::provab::{HotelRoomResponse, HotelSearchRequest, HotelSearchResponse},
    component::{GuestSelection, SelectedDateRange},
    // state::search_state::HotelInfoResults,
};
// use leptos::logging::log;
use crate::log;
use leptos::prelude::*;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Default, Clone)]
pub struct HotelInfoCtx {
    pub hotel_code: RwSignal<String>,
    pub selected_hotel_name: RwSignal<String>,
    pub selected_hotel_image: RwSignal<String>,
    pub selected_hotel_location: RwSignal<String>,
}

impl HotelInfoCtx {
    pub fn display(&self) -> String {
        let json_repr = serde_json::json!({
            "hotel_code": self.hotel_code.get_untracked(),
            "selected_hotel_name": self.selected_hotel_name.get_untracked(),
            "selected_hotel_image": self.selected_hotel_image.get_untracked(),
            "selected_hotel_location": self.selected_hotel_location.get_untracked(),
        });

        serde_json::to_string_pretty(&json_repr)
            .unwrap_or_else(|_| "Failed to serialize".to_string())
    }

    pub fn set_selected_hotel_details(code: String, name: String, image: String, location: String) {
        let this: Self = expect_context();
        this.hotel_code.set(code);
        this.selected_hotel_name.set(name);
        this.selected_hotel_image.set(image);
        this.selected_hotel_location.set(location);
    }

    pub fn get_hotel_code_untracked() -> String {
        let this: Self = expect_context();
        this.hotel_code.get_untracked()
    }
}

#[derive(Clone, Default, Debug)]
pub struct Names {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(Clone, Default, Debug)]
pub struct BlockRoomCtx {
    pub adults: RwSignal<Vec<AdultDetail>>,
    pub children: RwSignal<Vec<ChildDetail>>,
    pub terms_accepted: RwSignal<bool>,
}

#[derive(Default, Clone, Debug)]
pub struct AdultDetail {
    pub first_name: String,
    pub last_name: Option<String>,
    pub email: Option<String>, // Only for first adult
    pub phone: Option<String>, // Only for first adult
}

#[derive(Default, Clone, Debug)]
pub struct ChildDetail {
    pub first_name: String,
    pub last_name: Option<String>,
    pub age: Option<u8>,
}

impl BlockRoomCtx {
    pub fn create_adults(count: usize) {
        log!("create_adults");

        let this: Self = expect_context();
        this.adults.set(vec![AdultDetail::default(); count]);
    }

    pub fn create_children(count: usize) {
        log!("create_children");

        let this: Self = expect_context();
        this.children.set(vec![ChildDetail::default(); count]);
    }

    pub fn set_adults(adults: Vec<AdultDetail>) {
        let this: Self = expect_context();
        this.adults.set(adults);
    }

    pub fn set_children(children: Vec<ChildDetail>) {
        let this: Self = expect_context();
        this.children.set(children);
    }

    pub fn set_terms_accepted(value: bool) {
        log!("set_terms_accepted");

        let this: Self = expect_context();
        this.terms_accepted.set(value);
    }

    pub fn update_adult(index: usize, field: &str, value: String) {
        log!("update_adult");

        let this: Self = expect_context();
        this.adults.update(|list| {
            if let Some(adult) = list.get_mut(index) {
                match field {
                    "first_name" => adult.first_name = value,
                    "last_name" => adult.last_name = Some(value),
                    "email" => {
                        if index == 0 {
                            adult.email = Some(value)
                        }
                    }
                    "phone" => {
                        if index == 0 {
                            adult.phone = Some(value)
                        }
                    }
                    other => {
                        log!("adults.update: {other:?}");
                    }
                }
            }
        });
    }

    pub fn update_child(index: usize, field: &str, value: String) {
        log!("update_child");

        let this: Self = expect_context();
        this.children.update(|list| {
            if let Some(child) = list.get_mut(index) {
                match field {
                    "first_name" => child.first_name = value,
                    "last_name" => child.last_name = Some(value),
                    "age" => child.age = value.parse().ok(),
                    other => {
                        log!("chlidren.update: {other:?}");
                    }
                }
            }
        });
    }

    pub fn get_adults_untracked() -> Vec<AdultDetail> {
        let this: Self = expect_context();
        this.adults.get_untracked()
    }

    pub fn get_email_untracked() -> String {
        let this: Self = expect_context();
        this.adults
            .get_untracked()
            .first()
            .and_then(|adult| adult.email.clone())
            .unwrap_or_default()
    }

    pub fn get_children_untracked() -> Vec<ChildDetail> {
        let this: Self = expect_context();
        this.children.get_untracked()
    }

    pub fn get_user_name_untracked() -> String {
        let this: Self = expect_context();
        this.adults
            .get_untracked()
            .first()
            .map(|adult| adult.first_name.clone())
            .unwrap_or_default()
    }

    pub fn get_user_phone_untracked() -> String {
        let this: Self = expect_context();
        this.adults
            .get_untracked()
            .first()
            .map(|adult| adult.phone.clone().unwrap_or_default())
            .unwrap_or_default()
    }

    pub fn get_num_adults_untracked() -> u32 {
        let this: Self = expect_context();
        this.adults.get_untracked().len() as u32
    }

    pub fn get_num_children_untracked() -> u32 {
        let this: Self = expect_context();
        this.children.get_untracked().len() as u32
    }
}
