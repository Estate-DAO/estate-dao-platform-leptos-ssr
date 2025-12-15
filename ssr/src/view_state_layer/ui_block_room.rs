use crate::domain::{DomainHotelDetails, DomainHotelStaticDetails, DomainRoomData, DomainTaxLine};
use crate::view_state_layer::{ui_search_state::UISearchCtx, GlobalStateForLeptos};
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

// <!-- Room selection summary for block room page -->
#[derive(Clone, Debug)]
pub struct RoomSelectionSummary {
    pub room_id: String,
    pub room_name: String,
    pub meal_plan: Option<String>,
    pub quantity: u32,
    pub price_per_night: f64,
    pub tax_lines: Vec<DomainTaxLine>,
    pub room_data: DomainRoomData,
}

impl RoomSelectionSummary {
    pub fn display_name(&self) -> String {
        if let Some(plan) = self
            .meal_plan
            .as_ref()
            .map(|plan| plan.trim())
            .filter(|plan| !plan.is_empty())
        {
            format!("{} - {}", self.room_name, plan)
        } else {
            self.room_name.clone()
        }
    }
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

    // <!-- Phase 3.2: Enhanced error handling -->
    pub api_error_type: RwSignal<Option<String>>, // "network", "validation", "room_unavailable", "server"
    pub error_details: RwSignal<Option<String>>,  // Detailed error message for debugging
    pub retry_count: RwSignal<u32>,               // Track retry attempts

    // Pricing data
    pub room_price: RwSignal<f64>,
    pub total_price: RwSignal<f64>,
    pub num_nights: RwSignal<u32>,

    // Block room response
    pub block_room_id: RwSignal<Option<String>>,
    pub block_room_called: RwSignal<bool>,

    // <!-- Room selection data from hotel details -->
    pub selected_rooms: RwSignal<HashMap<String, (u32, DomainRoomData)>>, // room_id -> (quantity, room_data)
    pub hotel_context: RwSignal<Option<DomainHotelStaticDetails>>,
    pub room_selection_summary: RwSignal<Vec<RoomSelectionSummary>>,
}

impl BlockRoomUIState {
    pub fn from_leptos_context() -> Self {
        expect_context()
    }

    // Form data methods
    pub fn create_adults(count: usize) {
        use crate::log;
        log!(
            "BlockRoomUIState::create_adults called with count: {}",
            count
        );
        let this: Self = expect_context();
        let new_adults = vec![AdultDetail::default(); count];
        log!("Created {} adults: {:?}", count, new_adults);
        this.adults.set(new_adults);

        // Verify it was set
        let verification = this.adults.get_untracked();
        log!(
            "After create_adults - verification: {} adults in state",
            verification.len()
        );
    }

    pub fn create_children(count: usize) {
        let this: Self = expect_context();
        let search_ctx: UISearchCtx = expect_context();
        let ages_from_search = search_ctx.guests.children_ages.get_untracked();

        let children: Vec<ChildDetail> = (0..count)
            .map(|idx| {
                let age = ages_from_search.get(idx).copied().map(|a| a as u8);
                ChildDetail {
                    age,
                    ..Default::default()
                }
            })
            .collect();

        this.children.set(children);
    }

    pub fn update_adult(index: usize, field: &str, value: String) {
        use crate::log;
        log!(
            "BlockRoomUIState::update_adult called - index: {}, field: '{}', value: '{}'",
            index,
            field,
            value
        );

        let this: Self = expect_context();
        let adults_count_before = this.adults.get_untracked().len();
        log!("Adults list length before update: {}", adults_count_before);

        this.adults.update(|list| {
            log!("Inside adults.update closure - list length: {}", list.len());
            if let Some(adult) = list.get_mut(index) {
                log!("Found adult at index {}, updating field '{}'", index, field);
                match field {
                    "first_name" => {
                        adult.first_name = value.clone();
                        log!("Updated first_name to: '{}'", value);
                    }
                    "last_name" => {
                        adult.last_name = Some(value.clone());
                        log!("Updated last_name to: '{}'", value);
                    }
                    "email" => {
                        if index == 0 {
                            adult.email = Some(value.clone());
                            log!("Updated email to: '{}'", value);
                        } else {
                            log!(
                                "Skipped email update for non-primary adult (index: {})",
                                index
                            );
                        }
                    }
                    "phone" => {
                        if index == 0 {
                            adult.phone = Some(value.clone());
                            log!("Updated phone to: '{}'", value);
                        } else {
                            log!(
                                "Skipped phone update for non-primary adult (index: {})",
                                index
                            );
                        }
                    }
                    _ => {
                        log!("Unknown field '{}', ignoring", field);
                    }
                }
            } else {
                log!("ERROR: No adult found at index {}", index);
            }
        });

        // Verify the update worked
        let adults_after = this.adults.get_untracked();
        log!("Adults list length after update: {}", adults_after.len());
        if let Some(adult) = adults_after.get(index) {
            log!(
                "After update verification - Adult {}: first_name='{}', email={:?}, phone={:?}",
                index,
                adult.first_name,
                adult.email,
                adult.phone
            );
        }
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
        // todo (validation): maybe validate phone numbers to have specific format
        true
        // phone.chars().all(|c| c.is_ascii_digit())
        // && phone.len() >= 10
    }

    // Validation logic
    pub fn validate_form() -> bool {
        let this: Self = expect_context();
        let adult_list = this.adults.get_untracked();
        let child_list = this.children.get_untracked();
        let ui_search_ctx: UISearchCtx = expect_context();
        let required_primary_contacts = ui_search_ctx.guests.rooms.get_untracked().max(1) as usize;

        // Validate primary adult
        let primary_adult_valid = adult_list.first().is_some_and(|adult| {
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

        // Need at least one primary contact per room
        let has_primary_contact_per_room = adult_list.len() >= required_primary_contacts;

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

        let is_valid = has_primary_contact_per_room
            && primary_adult_valid
            && other_adults_valid
            && children_valid
            && terms_valid;
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
        this.total_price.get()
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

    // <!-- Phase 3.2: Enhanced error handling methods -->
    pub fn set_api_error(
        error_type: Option<String>,
        user_message: Option<String>,
        details: Option<String>,
    ) {
        let this: Self = expect_context();
        this.api_error_type.set(error_type);
        this.error.set(user_message);
        this.error_details.set(details);
    }

    pub fn increment_retry_count() {
        let this: Self = expect_context();
        this.retry_count.update(|count| *count += 1);
    }

    pub fn reset_retry_count() {
        let this: Self = expect_context();
        this.retry_count.set(0);
    }

    pub fn get_retry_count() -> u32 {
        let this: Self = expect_context();
        this.retry_count.get_untracked()
    }

    pub fn can_retry() -> bool {
        Self::get_retry_count() < 3 // Max 3 retry attempts
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

    // Update pricing from block room API response
    pub fn update_pricing_from_api_response(total_price: f64, room_price: f64) {
        let this: Self = expect_context();
        this.total_price.set(total_price);
        this.room_price.set(room_price);
    }

    // Detect if price has changed from original search price
    pub fn has_price_changed_from_original(new_total_price: f64) -> bool {
        let this: Self = expect_context();
        let original_total = this.total_price.get_untracked();
        (new_total_price - original_total).abs() > 0.01 // Allow for small floating point differences
    }

    // Get the difference between original and updated price
    pub fn get_price_difference(new_total_price: f64) -> f64 {
        let this: Self = expect_context();
        let original_total = this.total_price.get_untracked();
        new_total_price - original_total
    }

    // Static getter methods for reactive access
    pub fn get_room_price() -> f64 {
        let this: Self = expect_context();
        this.room_price.get()
    }

    pub fn get_num_nights() -> u32 {
        let this: Self = expect_context();
        this.num_nights.get()
    }

    pub fn get_room_selection_summary() -> Vec<RoomSelectionSummary> {
        let this: Self = expect_context();
        this.room_selection_summary.get()
    }

    pub fn get_calculated_total_from_summary() -> f64 {
        let this: Self = expect_context();
        let nights = this.num_nights.get();
        let summary = this.room_selection_summary.get();
        let base_total = summary
            .iter()
            .map(|room| {
                // Round price to 2 decimals to match displayed value
                let rounded_price = (room.price_per_night * 100.0).round() / 100.0;
                rounded_price * room.quantity as f64 * nights as f64
            })
            .sum::<f64>();
        let total_with_tax = base_total + Self::get_included_tax_total();
        // Round final total to 2 decimals to match display and eliminate floating point errors
        (total_with_tax * 100.0).round() / 100.0
    }

    pub fn get_included_tax_total() -> f64 {
        let this: Self = expect_context();
        this.room_selection_summary
            .get_untracked()
            .iter()
            .map(|room| {
                room.tax_lines
                    .iter()
                    .filter(|line| line.included)
                    .map(|line| line.amount * room.quantity as f64)
                    .sum::<f64>()
            })
            .sum()
    }

    pub fn get_loading() -> bool {
        let this: Self = expect_context();
        this.loading.get()
    }

    pub fn get_block_room_called() -> bool {
        let this: Self = expect_context();
        this.block_room_called.get_untracked()
    }

    pub fn get_adults() -> Vec<AdultDetail> {
        let this: Self = expect_context();
        this.adults.get()
    }

    pub fn get_show_payment_modal() -> bool {
        let this: Self = expect_context();
        this.show_payment_modal.get()
    }

    pub fn get_error() -> Option<String> {
        let this: Self = expect_context();
        this.error.get()
    }

    pub fn get_hotel_context() -> Option<DomainHotelStaticDetails> {
        let this: Self = expect_context();
        this.hotel_context.get()
    }

    pub fn get_form_valid() -> bool {
        let this: Self = expect_context();
        this.form_valid.get()
    }

    pub fn get_api_error_type() -> Option<String> {
        let this: Self = expect_context();
        this.api_error_type.get()
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

    // <!-- Room selection data management methods -->
    pub fn set_selected_rooms(rooms: HashMap<String, (u32, DomainRoomData)>) {
        let this: Self = expect_context();
        this.selected_rooms.set(rooms);
    }

    pub fn get_selected_rooms_untracked() -> HashMap<String, (u32, DomainRoomData)> {
        let this: Self = expect_context();
        this.selected_rooms.get_untracked()
    }

    pub fn set_hotel_context(hotel_details: Option<DomainHotelStaticDetails>) {
        let this: Self = expect_context();
        this.hotel_context.set(hotel_details);
    }

    pub fn get_hotel_context_untracked() -> Option<DomainHotelStaticDetails> {
        let this: Self = expect_context();
        this.hotel_context.get_untracked()
    }

    pub fn set_room_selection_summary(summary: Vec<RoomSelectionSummary>) {
        let this: Self = expect_context();
        this.room_selection_summary.set(summary);
    }

    pub fn get_room_selection_summary_untracked() -> Vec<RoomSelectionSummary> {
        let this: Self = expect_context();
        this.room_selection_summary.get_untracked()
    }

    // <!-- Helper method to calculate total from room selections -->
    pub fn calculate_total_from_room_selections() -> f64 {
        let this: Self = expect_context();
        let summary = this.room_selection_summary.get_untracked();
        let num_nights = this.num_nights.get_untracked();

        let total: f64 = summary
            .iter()
            .map(|room| {
                // Round price to 2 decimals to match displayed value
                let rounded_price = (room.price_per_night * 100.0).round() / 100.0;
                rounded_price * room.quantity as f64 * num_nights as f64
            })
            .sum();

        this.total_price.set(total);
        total
    }

    // <!-- Helper method to get total number of selected rooms -->
    pub fn get_total_selected_rooms() -> u32 {
        let this: Self = expect_context();
        this.room_selection_summary
            .get_untracked()
            .iter()
            .map(|room| room.quantity)
            .sum()
    }

    // Batch update methods to avoid signal borrow conflicts
    pub fn batch_update_on_success(block_id: String, total_price: f64, room_price: f64) {
        let this: Self = expect_context();

        // Use untracked updates to avoid borrow conflicts
        this.block_room_id.set(Some(block_id));
        this.total_price.set(total_price);
        this.room_price.set(room_price);

        this.block_room_called.set(true);
        this.show_payment_modal.set(true);
        this.loading.set(false);

        this.error.set(None);
        this.api_error_type.set(None);
        this.error_details.set(None);
    }

    pub fn batch_update_on_success_with_backend_error(
        block_id: String,
        total_price: f64,
        room_price: f64,
        error_message: Option<String>,
    ) {
        let this: Self = expect_context();

        // Use untracked updates to avoid borrow conflicts
        this.block_room_id.set(Some(block_id));
        this.block_room_called.set(true);
        this.total_price.set(total_price);
        this.room_price.set(room_price);

        this.loading.set(false);

        this.error.set(error_message);
        this.api_error_type.set(Some("backend".to_string()));
    }

    pub fn batch_update_on_error(
        error_type: Option<String>,
        user_message: Option<String>,
        details: Option<String>,
    ) {
        let this: Self = expect_context();

        // Use untracked updates to avoid borrow conflicts
        this.loading.set(false);
        this.api_error_type.set(error_type);
        this.error.set(user_message);
        this.error_details.set(details);
        // Close payment modal when error occurs so error is properly visible
        this.show_payment_modal.set(false);
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
        this.selected_rooms.set(HashMap::new());
        this.hotel_context.set(None);
        this.room_selection_summary.set(vec![]);
        this.api_error_type.set(None);
        this.error_details.set(None);
        this.retry_count.set(0);
    }
}

impl GlobalStateForLeptos for BlockRoomUIState {}

impl From<DomainHotelStaticDetails> for DomainHotelDetails {
    fn from(static_details: DomainHotelStaticDetails) -> Self {
        DomainHotelDetails {
            hotel_name: static_details.hotel_name,
            hotel_code: static_details.hotel_code,
            star_rating: static_details.star_rating,
            rating: static_details.rating,
            review_count: static_details.review_count,
            categories: static_details.categories,
            description: static_details.description,
            hotel_facilities: static_details.hotel_facilities,
            address: static_details.address,
            images: static_details.images,
            amenities: static_details.amenities,
            all_rooms: vec![], // No room rates in static details
            checkin: "".to_string(),
            checkout: "".to_string(),
            search_info: None,
            search_criteria: None,
        }
    }
}
