use serde::{Deserialize, Serialize};
use std::collections::HashMap;

cfg_if::cfg_if! {
    if #[cfg(feature = "mock-provab")] {
        // fake imports
        use fake::{Dummy, Fake, Faker};
        use rand::rngs::StdRng;
        use rand::SeedableRng;
    }
}

//
// SEARCH CORE TYPES
//

#[derive(Clone, Debug, PartialEq)]
pub struct Destination {
    pub city: String,
    pub country_name: String,
    pub country_code: String,
    pub city_id: String,
}

#[derive(Debug, Clone)]
pub struct DomainSelectedDateRange {
    pub end: (u32, u32, u32),
    pub start: (u32, u32, u32),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct DomainRoomGuest {
    pub no_of_adults: u32,
    pub no_of_children: u32,
    pub children_ages: Option<Vec<String>>,
}

// ROOM TYPES and HOTEL TYPES
// Domain types for hotel search and booking - provider-agnostic

// <!-- Core domain types based on working Provab types -->

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct DomainHotelSearchCriteria {
    // <!-- Destination information -->
    pub destination_city_id: u32,
    pub destination_country_code: String,

    // <!-- Date information -->
    pub check_in_date: String, // <!-- Format: "DD-MM-YYYY" -->
    pub no_of_nights: u32,

    // <!-- Guest information -->
    pub no_of_rooms: u32,
    pub room_guests: Vec<DomainRoomGuest>,
    pub guest_nationality: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct DomainPrice {
    pub room_price: f64,
    pub currency_code: String,
    // <!-- Additional price fields can be added later -->
    // pub published_price: Option<f64>,
    // pub offered_price: Option<f64>,
    // pub tax: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct DomainHotelResult {
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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct DomainHotelSearchResult {
    pub hotel_results: Vec<DomainHotelResult>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct DomainHotelSearchResponse {
    // some Api providers give string, some give int
    pub status: String,
    pub message: String,
    pub search: Option<DomainHotelSearchResult>,
}

// <!-- Hotel Details Types -->

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct DomainHotelInfoCriteria {
    pub token: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct DomainFirstRoomDetails {
    pub price: DomainDetailedPrice,
    pub room_data: DomainRoomData,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
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
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct DomainRoomData {
    pub room_name: String,
    pub room_unique_id: String,
    pub rate_key: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
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
    pub first_room_details: DomainFirstRoomDetails,
    pub amenities: Vec<String>,
}

// <!-- Default implementations -->

impl Default for DomainHotelSearchCriteria {
    fn default() -> Self {
        Self {
            destination_city_id: 1254, // <!-- Default to Mumbai -->
            destination_country_code: "IN".into(),
            check_in_date: "11-11-2024".into(),
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

impl DomainHotelSearchResponse {
    pub fn get_results_token_map(&self) -> HashMap<String, String> {
        let mut hotel_map = HashMap::new();

        if let Some(search) = self.search.clone() {
            for hotel in search.hotel_results {
                hotel_map.insert(hotel.hotel_code, hotel.result_token);
            }
        }

        hotel_map
    }

    pub fn hotel_results(&self) -> Vec<DomainHotelResult> {
        self.search
            .clone()
            .map_or_else(Vec::new, |search| search.hotel_results)
    }
}

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

#[derive(Debug, Clone)]
/// This is used in the form we will on the block room page
/// stored in backend
pub struct DomainChildDetail {
    pub age: u8,
    pub first_name: String,
    pub last_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DomainAdultDetail {
    pub email: Option<String>,
    pub first_name: String,
    pub last_name: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DomainUserDetails {
    pub children: Vec<DomainChildDetail>,
    pub adults: Vec<DomainAdultDetail>,
}

// get room types

// DomainHotelSearchCriteria, DomainHotelSearchResponse, DomainSearch,
// DomainHotelSearchResult, DomainHotelResult, DomainPrice,
// // DomainRoomGuest,
// DomainHotelInfoCriteria,
