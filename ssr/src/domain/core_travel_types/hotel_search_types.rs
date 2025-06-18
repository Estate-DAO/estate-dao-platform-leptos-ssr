use serde::{Deserialize, Serialize};
use std::collections::HashMap;

//
// SEARCH CORE TYPES
//

#[derive(Clone, Debug, PartialEq)]
pub struct DomainDestination {
    // todo (liteapi) - should to be int?
    pub city_id: u32,
    pub city_name: String,
    pub country_code: String,
    pub country_name: String,
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

// ROOM TYPES and HOTEL TYPES
// Domain types for hotel search and booking - provider-agnostic

// <!-- Core domain types based on working Provab types -->

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainHotelSearchCriteria {
    // <!-- Destination information -->
    pub destination_city_id: u32,
    pub destination_city_name: String,
    pub destination_country_code: String,
    pub destination_country_name: String,

    // <!-- Date information -->
    pub check_in_date: (u32, u32, u32),  // YYYY-MM-DD
    pub check_out_date: (u32, u32, u32), // YYYY-MM-DD
    pub no_of_nights: u32,

    // <!-- Guest information -->
    pub no_of_rooms: u32,
    pub room_guests: Vec<DomainRoomGuest>,
    pub guest_nationality: String,
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
    pub price: DomainPrice,
    pub hotel_picture: String,
    // todo (liteapi) how does liteapi propagate tokens?
    pub result_token: String,
    // <!-- Additional fields can be added later -->
    // pub hotel_description: Option<String>,
    // pub hotel_address: Option<String>,
    // pub amenities: Option<Vec<String>>,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct DomainHotelListAfterSearch {
    pub hotel_results: Vec<DomainHotelAfterSearch>,
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
    pub room_name: String,
    pub room_unique_id: String,
    pub rate_key: String,
    pub offer_id: String, // LiteAPI: offerId, Provab: empty string
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainRoomOption {
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

pub struct DomainHotelDetails {
    pub checkin: String,
    pub checkout: String,
    pub hotel_name: String,
    pub hotel_code: String,
    pub star_rating: i32,
    pub description: String,
    pub hotel_facilities: Vec<String>,
    pub address: String,
    pub images: Vec<String>,
    pub all_rooms: Vec<DomainRoomOption>,
    pub amenities: Vec<String>,
}

// <!-- Default implementations -->

impl Default for DomainHotelSearchCriteria {
    fn default() -> Self {
        Self {
            destination_city_id: 1254, // <!-- Default to Mumbai -->
            destination_country_code: "IN".into(),
            check_in_date: (2025, 11, 12),
            check_out_date: (2025, 11, 12),
            destination_city_name: "Mumbai".into(),
            destination_country_name: "India".into(),
            no_of_nights: 1,
            guest_nationality: "IN".into(),
            no_of_rooms: 1,
            room_guests: vec![DomainRoomGuest {
                no_of_adults: 1,
                no_of_children: 0,
                children_ages: None,
            }],
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
