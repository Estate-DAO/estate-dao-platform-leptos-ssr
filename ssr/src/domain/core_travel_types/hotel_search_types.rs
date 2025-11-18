use serde::{Deserialize, Serialize};
use std::collections::HashMap;

//
// SEARCH CORE TYPES
//

#[derive(Clone, Debug, PartialEq)]
pub struct DomainDestination {
    // todo (liteapi) - should to be int?
    pub place_id: String,
    // pub city_id: Option<u32>,
    // pub city_name: Option<String>,
    // pub country_code: Option<String>,
    // pub country_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DomainSelectedDateRange {
    pub end: (u32, u32, u32),
    pub start: (u32, u32, u32),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainRoomGuest {
    pub no_of_adults: u32,
    pub no_of_children: u32,
    pub children_ages: Option<Vec<String>>,
}

// <!-- Pagination types for hotel search -->

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainPaginationParams {
    pub page: Option<u32>,      // 1-based page number
    pub page_size: Option<u32>, // Results per page (default: 200, max: 1000)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainPaginationMeta {
    pub page: u32,
    pub page_size: u32,
    pub total_results: Option<i32>, // If available from API
    pub has_next_page: bool,
    pub has_previous_page: bool,
}

// ROOM TYPES and HOTEL TYPES
// Domain types for hotel search and booking - provider-agnostic

// <!-- Core domain types based on working Provab types -->

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainHotelSearchCriteria {
    // <!-- Destination information -->
    // pub destination_city_id: Option<u32>, // Provab city ID
    // pub destination_city_name: Option<String>,
    // pub destination_country_code: Option<String>,
    // pub destination_country_name: Option<String>,
    // pub destination_latitude: Option<f64>,
    // pub destination_longitude: Option<f64>,
    pub place_id: String,

    // <!-- Date information -->
    pub check_in_date: (u32, u32, u32),  // YYYY-MM-DD
    pub check_out_date: (u32, u32, u32), // YYYY-MM-DD
    pub no_of_nights: u32,

    // <!-- Guest information -->
    pub no_of_rooms: u32,
    pub room_guests: Vec<DomainRoomGuest>,
    pub guest_nationality: String,

    // <!-- Pagination information -->
    pub pagination: Option<DomainPaginationParams>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainPlacesSearchPayload {
    pub text_query: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainPlaceDetailsPayload {
    pub place_id: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainPlaceDetails {
    pub data: DomainPlaceData,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainPlaceData {
    pub address_components: Vec<DomainAddressComponent>,
    pub location: DomainLocation,
    pub viewport: DomainViewport,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainAddressComponent {
    pub language_code: String,
    pub long_text: String,
    pub short_text: String,
    pub types: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainLocation {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainViewport {
    pub high: DomainHigh,
    pub low: DomainLow,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainHigh {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainLow {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DomainPrice {
    pub room_price: f64,
    pub currency_code: String,
    // <!-- Additional price fields can be added later -->
    // pub published_price: Option<f64>,
    // pub offered_price: Option<f64>,
    // pub tax: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainHotelAfterSearch {
    pub hotel_code: String,
    pub hotel_name: String,
    pub hotel_category: String,
    pub star_rating: u8,
    pub price: Option<DomainPrice>,
    pub hotel_picture: String,
    // Derived from provider result where available
    pub amenities: Vec<String>,
    pub property_type: Option<String>,
    // todo (liteapi) how does liteapi propagate tokens?
    pub result_token: String,
    // <!-- Additional fields can be added later -->
    // pub hotel_description: Option<String>,
    pub hotel_address: Option<String>,
    // Distance from center in kilometers (calculated if coordinates available)
    pub distance_from_center_km: Option<f64>,
    // pub amenities: Option<Vec<String>>,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct DomainHotelListAfterSearch {
    pub hotel_results: Vec<DomainHotelAfterSearch>,
    pub pagination: Option<DomainPaginationMeta>,
}

// <!-- Hotel Details Types -->

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainHotelInfoCriteria {
    // Hotel-specific information
    pub token: String,          // Used by Provab, hotel ID for LiteAPI
    pub hotel_ids: Vec<String>, // For LiteAPI

    // Search criteria (reused from search) because some api providers need the search criteria
    pub search_criteria: DomainHotelSearchCriteria,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainHotelCodeId {
    pub hotel_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainFirstRoomDetails {
    pub price: DomainDetailedPrice,
    pub room_data: DomainRoomData,
}

#[derive(Clone, Debug, Serialize, Deserialize)]

pub struct DomainDetailedPrice {
    pub published_price: f64,
    pub published_price_rounded_off: f64,
    pub offered_price: f64,
    pub offered_price_rounded_off: f64,
    pub room_price: f64,
    pub tax: f64,
    pub extra_guest_charge: f64,
    pub child_charge: f64,
    pub other_charges: f64,
    pub currency_code: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]

pub struct DomainRoomData {
    pub mapped_room_id: u32,
    pub room_name: String,
    pub room_unique_id: String,
    pub rate_key: String,
    pub offer_id: String, // LiteAPI: offerId, Provab: empty string
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainRoomOption {
    pub mapped_room_id: u32,
    pub price: DomainDetailedPrice,
    pub room_data: DomainRoomData,
    pub meal_plan: Option<String>, // Board type + board name (e.g., "Room Only")
    pub occupancy_info: Option<DomainRoomOccupancy>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainRoomOccupancy {
    pub max_occupancy: Option<u32>,
    pub adult_count: Option<u32>,
    pub child_count: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainStaticRoom {
    pub room_id: String,
    pub room_name: String,
    pub description: String,
    pub room_size_square: Option<f64>,
    pub room_size_unit: Option<String>,
    pub max_adults: Option<u32>,
    pub max_children: Option<u32>,
    pub max_occupancy: Option<u32>,
    pub amenities: Vec<String>,
    pub photos: Vec<String>,
    pub bed_types: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]

pub struct DomainHotelDetails {
    pub checkin: String,
    pub checkout: String,
    pub hotel_name: String,
    pub hotel_code: String,
    pub star_rating: i32,
    pub rating: Option<f64>,
    pub review_count: Option<u32>,
    pub categories: Vec<DomainReviewCategory>,
    pub description: String,
    pub hotel_facilities: Vec<String>,
    pub address: String,
    pub images: Vec<String>,
    pub all_rooms: Vec<DomainRoomOption>,
    pub amenities: Vec<String>,
    pub search_info: Option<DomainHotelSearchInfo>,
    pub search_criteria: Option<DomainHotelSearchCriteria>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainHotelStaticDetails {
    pub hotel_name: String,
    pub hotel_code: String,
    pub star_rating: i32,
    pub rating: Option<f64>,
    pub review_count: Option<u32>,
    pub categories: Vec<DomainReviewCategory>,
    pub description: String,
    pub hotel_facilities: Vec<String>,
    pub address: String,
    pub images: Vec<String>,
    pub amenities: Vec<String>,
    pub rooms: Vec<DomainStaticRoom>,
    pub location: Option<DomainLocation>,
    pub checkin_checkout_times: Option<DomainCheckinCheckoutTimes>,
    pub policies: Vec<DomainPolicy>,
}

impl DomainHotelStaticDetails {
    pub fn get_domain_hotel_details(
        &self,
        checkin: String,
        checkout: String,
        all_rooms: Vec<DomainRoomOption>,
        search_info: Option<DomainHotelSearchInfo>,
        search_criteria: Option<DomainHotelSearchCriteria>,
    ) -> DomainHotelDetails {
        DomainHotelDetails {
            checkin: checkin,
            checkout: checkout,
            hotel_name: self.hotel_name.clone(),
            hotel_code: self.hotel_code.clone(),
            star_rating: self.star_rating,
            rating: self.rating,
            review_count: self.review_count,
            categories: self.categories.clone(),
            description: self.description.clone(),
            hotel_facilities: self.hotel_facilities.clone(),
            address: self.address.clone(),
            images: self.images.clone(),
            all_rooms,
            amenities: self.amenities.clone(),
            search_info,
            search_criteria,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct DomainCheckinCheckoutTimes {
    pub checkin: String,
    pub checkout: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct DomainPolicy {
    pub policy_type: Option<String>,
    pub name: String,
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct DomainReviewCategory {
    pub name: String,
    pub rating: f32,
    pub description: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainHotelSearchInfo {
    pub hotel_code: String,
    pub hotel_name: String,
    pub star_rating: i32,
    pub price: Option<DomainPrice>,
    pub hotel_picture: String,
    // Derived from provider result where available
    pub amenities: Vec<String>,
    pub property_type: Option<String>,
    // todo (liteapi) how does liteapi propagate tokens?
    pub result_token: String,
    // <!-- Additional fields can be added later -->
    // pub hotel_description: Option<String>,
    pub hotel_address: Option<String>,
    // Distance from center in kilometers (calculated if coordinates available)
    pub distance_from_center_km: Option<f64>,
    // pub amenities: Option<Vec<String>>,
}

// <!-- Default implementations -->

impl Default for DomainHotelSearchCriteria {
    fn default() -> Self {
        Self {
            // destination_city_id: None, // <!-- Default to Mumbai -->
            // destination_country_code: None,
            // destination_latitude: Some(19.07),
            // destination_longitude: Some(72.87),
            // destination_city_name: None,
            // destination_country_name: None,
            check_in_date: (2025, 11, 12),
            check_out_date: (2025, 11, 12),
            no_of_nights: 1,
            guest_nationality: "IN".into(),
            no_of_rooms: 1,
            room_guests: vec![DomainRoomGuest {
                no_of_adults: 1,
                no_of_children: 0,
                children_ages: None,
            }],
            place_id: "ChIJOwg_06VPwokRYv534QaPC8g".into(),
            pagination: None, // <!-- Default to no pagination (first page) -->
        }
    }
}

// <!-- Helper implementations -->

// <!-- Integration with UI State (SearchCtx) -->
// <!-- This will be enabled when needed -->
/*
use crate::component::{Destination, SelectedDateRange};
use crate::state::search_state::SearchCtx;

impl DomainHotelSearchCriteria {
    fn get_room_guests_from_ctx(search_ctx: &SearchCtx) -> Vec<DomainRoomGuest> {
        let guest_selection = search_ctx.guests;

        let no_of_adults = guest_selection.get_untracked().adults.get_untracked();
        let no_of_child = guest_selection.get_untracked().children.get_untracked();
        let children_ages: Vec<String> = guest_selection
            .get_untracked()
            .children_ages
            .get_untracked()
            .iter()
            .map(|age| age.to_string())
            .collect();

        let child_age = if no_of_child > 0 {
            Some(children_ages)
        } else {
            None
        };

        vec![DomainRoomGuest {
            no_of_adults,
            no_of_child,
            child_age,
        }]
    }
}

impl From<SearchCtx> for DomainHotelSearchCriteria {
    fn from(ctx: SearchCtx) -> Self {
        let check_in_date = SelectedDateRange::format_date(ctx.date_range.get_untracked().start);
        let no_of_nights = ctx.date_range.get_untracked().no_of_nights();
        let request = DomainHotelSearchCriteria {
            destination_city_id: Destination::get_city_id(&ctx),
            destination_country_code: Destination::get_country_code(&ctx),
            check_in_date,
            no_of_nights,
            room_guests: Self::get_room_guests_from_ctx(&ctx),
            ..Default::default()
        };

        request
    }
}
*/

// needed on block room page - where the user fills the form.
//
// GUEST types
//

#[derive(Debug, Clone, Serialize, Deserialize)]
/// This is used in the form we will on the block room page
/// stored in backend
pub struct DomainChildDetail {
    pub age: u8,
    pub first_name: String,
    pub last_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainAdultDetail {
    pub email: Option<String>,
    pub first_name: String,
    pub last_name: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainUserDetails {
    pub children: Vec<DomainChildDetail>,
    pub adults: Vec<DomainAdultDetail>,
}

// Block Room Types
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainSelectedRoomWithQuantity {
    pub room_data: DomainRoomData,
    pub quantity: u32,
    pub price_per_night: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainBlockRoomRequest {
    // Hotel and search context
    pub hotel_info_criteria: DomainHotelInfoCriteria,

    // Guest details for the booking
    pub user_details: DomainUserDetails,

    // Selected rooms with quantities (supports multiple room types)
    pub selected_rooms: Vec<DomainSelectedRoomWithQuantity>,

    // Backward compatibility: First room (for providers that don't support multiple rooms yet)
    pub selected_room: DomainRoomData,

    // Additional booking context
    pub total_guests: u32,
    pub special_requests: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainBlockRoomResponse {
    // Unique identifier for this block
    // liteapi calls this prebookId
    // differnt providers give different names to this.
    pub block_id: String,

    // Flags for changes since search
    pub is_price_changed: bool,
    pub is_cancellation_policy_changed: bool,

    // Blocked room details
    pub blocked_rooms: Vec<DomainBlockedRoom>,

    // Total pricing for all rooms
    pub total_price: DomainDetailedPrice,

    // Provider-specific data that might be needed for booking
    pub provider_data: Option<String>, // JSON string for flexibility
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainBlockedRoom {
    pub room_code: String,
    pub room_name: String,
    pub room_type_code: Option<String>,
    pub price: DomainDetailedPrice,
    pub cancellation_policy: Option<String>,
    pub meal_plan: Option<String>,
}

// get room types
