use crate::view_state_layer::GlobalStateForLeptos;
use leptos::*;
use std::collections::HashMap;

// Domain types for form data
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

#[derive(Debug, Clone, Default)]
pub struct BlockRoomUIState {
    // Form data
    pub adults: RwSignal<Vec<AdultDetail>>,
    pub children: RwSignal<Vec<ChildDetail>>,
    pub terms_accepted: RwSignal<bool>,

    // Validation
    pub form_valid: RwSignal<bool>,
    pub validation_errors: RwSignal<HashMap<String, String>>,

    // UI state
    pub loading: RwSignal<bool>,
    pub error: RwSignal<Option<String>>,
    pub show_payment_modal: RwSignal<bool>,

    // Pricing data
    pub room_price: RwSignal<f64>,
    pub total_price: RwSignal<f64>,
    pub num_nights: RwSignal<u32>,

    // Block room response
    pub block_room_id: RwSignal<Option<String>>,
    pub block_room_called: RwSignal<bool>,
}

impl BlockRoomUIState {
    pub fn from_leptos_context() -> Self {
        expect_context()
    }

    // Form data methods
    pub fn create_adults(count: usize) {
        let this: Self = expect_context();
        this.adults.set(vec![AdultDetail::default(); count]);
    }

    pub fn create_children(count: usize) {
        let this: Self = expect_context();
        this.children.set(vec![ChildDetail::default(); count]);
    }

    pub fn update_adult(index: usize, field: &str, value: String) {
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
                    _ => {}
                }
            }
        });
    }

    pub fn update_child(index: usize, field: &str, value: String) {
        let this: Self = expect_context();
        this.children.update(|list| {
            if let Some(child) = list.get_mut(index) {
                match field {
                    "first_name" => child.first_name = value,
                    "last_name" => child.last_name = Some(value),
                    "age" => child.age = value.parse().ok(),
                    _ => {}
                }
            }
        });
    }

    // Validation methods
    pub fn set_terms_accepted(accepted: bool) {
        let this: Self = expect_context();
        this.terms_accepted.set(accepted);
    }

    pub fn set_form_valid(valid: bool) {
        let this: Self = expect_context();
        this.form_valid.set(valid);
    }

    pub fn get_form_valid_untracked() -> bool {
        let this: Self = expect_context();
        this.form_valid.get_untracked()
    }

    // Helper function for email validation
    pub fn is_valid_email(email: &str) -> bool {
        email.contains('@') && email.contains('.')
    }

    // Helper function for phone validation
    pub fn is_valid_phone(phone: &str) -> bool {
        phone.chars().all(|c| c.is_ascii_digit()) && phone.len() >= 10
    }

    // Validation logic
    pub fn validate_form() -> bool {
        let this: Self = expect_context();
        let adult_list = this.adults.get_untracked();
        let child_list = this.children.get_untracked();

        // Validate primary adult
        let primary_adult_valid = adult_list.first().map_or(false, |adult| {
            !adult.first_name.trim().is_empty()
                && adult
                    .email
                    .as_ref()
                    .map_or(false, |e| !e.trim().is_empty() && Self::is_valid_email(e))
                && adult
                    .phone
                    .as_ref()
                    .map_or(false, |p| !p.trim().is_empty() && Self::is_valid_phone(p))
        });

        // Validate other adults
        let other_adults_valid = adult_list
            .iter()
            .skip(1)
            .all(|adult| !adult.first_name.trim().is_empty());

        // Validate children
        let children_valid = child_list
            .iter()
            .all(|child| !child.first_name.trim().is_empty() && child.age.is_some());

        // Check if terms are accepted
        let terms_valid = this.terms_accepted.get_untracked();

        let is_valid = primary_adult_valid && other_adults_valid && children_valid && terms_valid;
        this.form_valid.set(is_valid);
        is_valid
    }

    // Pricing methods
    pub fn set_room_price(price: f64) {
        let this: Self = expect_context();
        this.room_price.set(price);
    }

    pub fn set_num_nights(nights: u32) {
        let this: Self = expect_context();
        this.num_nights.set(nights);
    }

    pub fn calculate_total_price() -> f64 {
        let this: Self = expect_context();
        let room_price = this.room_price.get_untracked();
        let num_nights = this.num_nights.get_untracked();
        let total = room_price * num_nights as f64;
        this.total_price.set(total);
        total
    }

    pub fn get_total_price() -> f64 {
        let this: Self = expect_context();
        this.total_price.get_untracked()
    }

    // UI state methods
    pub fn set_loading(loading: bool) {
        let this: Self = expect_context();
        this.loading.set(loading);
    }

    pub fn set_error(error: Option<String>) {
        let this: Self = expect_context();
        this.error.set(error);
    }

    pub fn set_show_payment_modal(show: bool) {
        let this: Self = expect_context();
        this.show_payment_modal.set(show);
    }

    // Block room methods
    pub fn set_block_room_id(id: Option<String>) {
        let this: Self = expect_context();
        this.block_room_id.set(id);
    }

    pub fn set_block_room_called(called: bool) {
        let this: Self = expect_context();
        this.block_room_called.set(called);
    }

    // Getter methods for untracked access
    pub fn get_adults_untracked() -> Vec<AdultDetail> {
        let this: Self = expect_context();
        this.adults.get_untracked()
    }

    pub fn get_children_untracked() -> Vec<ChildDetail> {
        let this: Self = expect_context();
        this.children.get_untracked()
    }

    pub fn get_primary_email_untracked() -> String {
        let this: Self = expect_context();
        this.adults
            .get_untracked()
            .first()
            .and_then(|adult| adult.email.clone())
            .unwrap_or_default()
    }

    pub fn get_primary_phone_untracked() -> String {
        let this: Self = expect_context();
        this.adults
            .get_untracked()
            .first()
            .and_then(|adult| adult.phone.clone())
            .unwrap_or_default()
    }

    pub fn get_primary_name_untracked() -> String {
        let this: Self = expect_context();
        this.adults
            .get_untracked()
            .first()
            .map(|adult| adult.first_name.clone())
            .unwrap_or_default()
    }

    // Reset method
    pub fn reset() {
        let this: Self = expect_context();
        this.adults.set(vec![]);
        this.children.set(vec![]);
        this.terms_accepted.set(false);
        this.form_valid.set(false);
        this.validation_errors.set(HashMap::new());
        this.loading.set(false);
        this.error.set(None);
        this.show_payment_modal.set(false);
        this.room_price.set(0.0);
        this.total_price.set(0.0);
        this.num_nights.set(0);
        this.block_room_id.set(None);
        this.block_room_called.set(false);
    }
}

impl GlobalStateForLeptos for BlockRoomUIState {}
