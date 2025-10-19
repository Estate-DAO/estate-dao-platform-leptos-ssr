use crate::{
    api::client_side_api::{Place, PlaceData},
    component::{ChildrenAgesSignalExt, Destination, GuestSelection, SelectedDateRange},
    domain::DomainHotelSearchCriteria,
    utils::query_params::{update_url_with_state, QueryParamsSync},
    view_state_layer::{ui_search_state::UISearchCtx, view_state::HotelInfoCtx},
};
use chrono::Datelike;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Hotel Details page state that can be encoded in URL via base64
/// Contains essential search criteria needed to recreate hotel search context
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HotelDetailsParams {
    // Hotel identification
    pub hotel_code: String,

    // Essential search parameters for hotel search criteria
    // pub destination_city_id: u32,
    // pub destination_city_name: String,
    // pub destination_country_code: String,
    // pub destination_country_name: String,
    pub place: Place,
    pub place_details: PlaceData,
    pub destination_latitude: Option<f64>,
    pub destination_longitude: Option<f64>,

    // Date information
    pub checkin: String,  // YYYY-MM-DD format
    pub checkout: String, // YYYY-MM-DD format
    pub no_of_nights: u32,

    // Guest information
    pub adults: u32,
    pub children: u32,
    pub rooms: u32,
    pub children_ages: Vec<u32>,
    pub guest_nationality: String,
}

impl HotelDetailsParams {
    /// Create from current search context state and hotel info
    pub fn from_current_context() -> Option<Self> {
        let search_ctx: UISearchCtx = expect_context();
        let hotel_info_ctx: HotelInfoCtx = expect_context();

        let place = search_ctx.place.get_untracked()?;
        let place_details = search_ctx.place_details.get_untracked()?;

        let hotel_code = hotel_info_ctx.hotel_code.get_untracked();
        if hotel_code.is_empty() {
            return None;
        }

        let place = search_ctx.place.get_untracked()?;
        let date_range = search_ctx.date_range.get_untracked();

        // Use next week as fallback if dates are not available
        let (start_date, end_date, nights) =
            if date_range.start == (0, 0, 0) || date_range.end == (0, 0, 0) {
                // Calculate next week dates (1 night stay)
                let today = chrono::Local::now().naive_local().date();
                let next_week_start = today + chrono::Duration::days(7);
                let next_week_end = next_week_start + chrono::Duration::days(1); // Just 1 night

                let start = (
                    next_week_start.year() as u32,
                    next_week_start.month(),
                    next_week_start.day(),
                );
                let end = (
                    next_week_end.year() as u32,
                    next_week_end.month(),
                    next_week_end.day(),
                );
                let nights = 1u32; // 1 night stay

                (start, end, nights)
            } else {
                (date_range.start, date_range.end, date_range.no_of_nights())
            };

        let guests = &search_ctx.guests;

        Some(Self {
            // we don't need hotel code for serach query. only destination, date range and guests are needed
            // latitude and longitude help with search query
            hotel_code: hotel_code.clone(),
            destination_latitude: Some(place_details.location.latitude),
            destination_longitude: Some(place_details.location.longitude),
            place,
            place_details,
            checkin: format!(
                "{:04}-{:02}-{:02}",
                start_date.0, start_date.1, start_date.2
            ),
            checkout: format!("{:04}-{:02}-{:02}", end_date.0, end_date.1, end_date.2),
            no_of_nights: nights,
            adults: guests.adults.get_untracked(),
            children: guests.children.get_untracked(),
            rooms: guests.rooms.get_untracked(),
            children_ages: guests.children_ages.get_untracked().into(),
            guest_nationality: "US".to_string(), // Default nationality
        })
    }

    /// Convert to DomainHotelSearchCriteria for API calls
    pub fn to_domain_search_criteria(&self) -> DomainHotelSearchCriteria {
        use crate::domain::DomainRoomGuest;

        // Parse dates back to tuples
        let checkin_date = self.parse_date(&self.checkin).unwrap_or((2025, 1, 1));
        let checkout_date = self.parse_date(&self.checkout).unwrap_or((2025, 1, 2));

        // Create room guests
        let room_guests = vec![DomainRoomGuest {
            no_of_adults: self.adults,
            no_of_children: self.children,
            children_ages: if self.children > 0 {
                Some(
                    self.children_ages
                        .iter()
                        .map(|age| age.to_string())
                        .collect(),
                )
            } else {
                None
            },
        }];

        let place_id = self.place.place_id.clone();

        DomainHotelSearchCriteria {
            // destination_city_id: self.destination_city_id,
            // destination_city_name: self.destination_city_name.clone(),
            // destination_country_code: self.destination_country_code.clone(),
            // destination_country_name: self.destination_country_name.clone(),
            // destination_latitude: self.destination_latitude,
            // destination_longitude: self.destination_longitude,
            place_id,
            check_in_date: checkin_date,
            check_out_date: checkout_date,
            no_of_nights: self.no_of_nights,
            no_of_rooms: self.rooms,
            room_guests,
            guest_nationality: self.guest_nationality.clone(),
            pagination: None, // No pagination for hotel details
                              // ..Default::default()
        }
    }

    /// Helper to parse date string back to tuple
    fn parse_date(&self, date_str: &str) -> Option<(u32, u32, u32)> {
        if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            Some((date.year() as u32, date.month(), date.day()))
        } else {
            None
        }
    }

    /// Manual update URL with current state (call this when you want to update the URL)
    pub fn update_url(&self) {
        update_url_with_state(self);
    }

    /// Generate a shareable URL for this hotel with all search parameters
    /// This can be called from hotel list or other pages to create deep links
    pub fn to_shareable_url(&self) -> String {
        use crate::app::AppRoutes;
        use url::form_urlencoded;

        let params = self.to_url_params();
        let query_string = form_urlencoded::Serializer::new(String::new())
            .extend_pairs(&params)
            .finish();

        format!("{}?{}", AppRoutes::HotelDetails.to_string(), query_string)
    }

    /// Create from URL query parameters
    pub fn from_url_params(params: &HashMap<String, String>) -> Option<Self> {
        let encoded_state = params.get("state").cloned();
        if let Some(encoded) = encoded_state {
            crate::utils::query_params::decode_state(&encoded[..]).ok()
        } else {
            None
        }
    }

    /// Convert to URL query parameters
    pub fn to_url_params(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();
        let encoded = crate::utils::query_params::encode_state(self);
        params.insert("state".to_string(), encoded);
        params
    }
}

impl QueryParamsSync<HotelDetailsParams> for HotelDetailsParams {
    fn sync_to_app_state(&self) {
        let search_ctx: UISearchCtx = expect_context();
        let hotel_info_ctx: HotelInfoCtx = expect_context();

        // Set hotel code
        hotel_info_ctx.hotel_code.set(self.hotel_code.clone());

        UISearchCtx::set_place(self.place.clone());
        UISearchCtx::set_place_details(Some(self.place_details.clone()));

        // Set date range
        if let (Some(start_date), Some(end_date)) = (
            self.parse_date(&self.checkin),
            self.parse_date(&self.checkout),
        ) {
            let date_range = SelectedDateRange {
                start: start_date,
                end: end_date,
            };
            UISearchCtx::set_date_range(date_range);
        }

        // Set guest information
        let guest_selection = GuestSelection::default();
        guest_selection.adults.set(self.adults);
        guest_selection.children.set(self.children);
        guest_selection.rooms.set(self.rooms);
        guest_selection
            .children_ages
            .set_ages(self.children_ages.clone());

        UISearchCtx::set_guests(guest_selection);
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::utils::query_params::{decode_state, encode_state};

//     #[test]
//     fn test_hotel_details_params_serialization() {
//         let params = HotelDetailsParams {
//             hotel_code: "hotel123".to_string(),
//             destination_city_id: 1254,
//             destination_city_name: "Mumbai".to_string(),
//             destination_country_code: "IN".to_string(),
//             destination_country_name: "India".to_string(),
//             checkin: "2025-01-15".to_string(),
//             checkout: "2025-01-20".to_string(),
//             no_of_nights: 5,
//             adults: 2,
//             children: 1,
//             rooms: 1,
//             children_ages: vec![8],
//             guest_nationality: "US".to_string(),
//             destination_latitude: Some(12.34),
//             destination_longitude: Some(56.78),
//         };

//         // Test base64 encoding/decoding
//         let encoded = encode_state(&params);
//         assert!(!encoded.is_empty());

//         let decoded: HotelDetailsParams = decode_state(&encoded).unwrap();
//         assert_eq!(params, decoded);
//     }

//     #[test]
//     fn test_from_url_params() {
//         let params = HotelDetailsParams {
//             hotel_code: "hotel456".to_string(),
//             destination_city_id: 1254,
//             destination_city_name: "Mumbai".to_string(),
//             destination_country_code: "IN".to_string(),
//             destination_country_name: "India".to_string(),
//             checkin: "2025-02-01".to_string(),
//             checkout: "2025-02-05".to_string(),
//             no_of_nights: 4,
//             adults: 2,
//             children: 0,
//             rooms: 1,
//             children_ages: vec![],
//             guest_nationality: "US".to_string(),
//             destination_latitude: Some(12.34),
//             destination_longitude: Some(56.78),
//         };

//         let query_params = params.to_url_params();
//         let parsed =
//             HotelDetailsParams::from_url_params(&query_params.into_iter().collect()).unwrap();

//         assert_eq!(params, parsed);
//     }

//     #[test]
//     fn test_to_domain_search_criteria() {
//         let params = HotelDetailsParams {
//             hotel_code: "hotel789".to_string(),
//             destination_city_id: 1254,
//             destination_city_name: "Mumbai".to_string(),
//             destination_country_code: "IN".to_string(),
//             destination_country_name: "India".to_string(),
//             checkin: "2025-03-10".to_string(),
//             checkout: "2025-03-15".to_string(),
//             no_of_nights: 5,
//             adults: 2,
//             children: 1,
//             rooms: 1,
//             children_ages: vec![10],
//             guest_nationality: "US".to_string(),
//             destination_latitude: Some(12.34),
//             destination_longitude: Some(56.78),
//         };

//         let domain_criteria = params.to_domain_search_criteria();

//         // assert_eq!(domain_criteria.destination_city_id, 1254);
//         // assert_eq!(domain_criteria.destination_city_name, "Mumbai");
//         assert_eq!(domain_criteria.check_in_date, (2025, 3, 10));
//         assert_eq!(domain_criteria.check_out_date, (2025, 3, 15));
//         assert_eq!(domain_criteria.no_of_nights, 5);
//         assert_eq!(domain_criteria.no_of_rooms, 1);
//         assert_eq!(domain_criteria.room_guests.len(), 1);
//         assert_eq!(domain_criteria.room_guests[0].no_of_adults, 2);
//         assert_eq!(domain_criteria.room_guests[0].no_of_children, 1);
//         assert_eq!(
//             domain_criteria.room_guests[0].children_ages,
//             Some(vec!["10".to_string()])
//         );
//     }
// }
