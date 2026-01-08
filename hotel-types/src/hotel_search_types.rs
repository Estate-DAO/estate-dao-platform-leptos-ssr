//! Hotel search domain types - provider-agnostic
//!
//! These types are used for hotel search, details, and room operations.

use crate::grouped_rooms::DomainRoomGroup;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

//
// SEARCH CORE TYPES
//

#[derive(Clone, Debug, PartialEq)]
pub struct DomainDestination {
    pub place_id: String,
}

#[derive(Debug, Clone)]
pub struct DomainSelectedDateRange {
    pub end: (u32, u32, u32),
    pub start: (u32, u32, u32),
}

impl DomainSelectedDateRange {
    pub fn to_string(&self) -> String {
        let start_str = format!(
            "{:04}-{:02}-{:02}",
            self.start.0, self.start.1, self.start.2
        );
        let end_str = format!("{:04}-{:02}-{:02}", self.end.0, self.end.1, self.end.2);
        format!("{} - {}", start_str, end_str)
    }

    pub fn display_string(&self) -> String {
        if self.start == (0, 0, 0) && self.end == (0, 0, 0) {
            return "Check in - Check out".to_string();
        }

        let (start_date, end_date) = if self.start != (0, 0, 0) && self.end != (0, 0, 0) {
            if self.start > self.end {
                (self.end, self.start)
            } else {
                (self.start, self.end)
            }
        } else {
            (self.start, self.end)
        };

        let format_date = |(y, m, d): (u32, u32, u32)| -> String {
            if (y, m, d) == (0, 0, 0) {
                return "".to_string();
            }
            let suffix = match d {
                1 | 21 | 31 => "st",
                2 | 22 => "nd",
                3 | 23 => "rd",
                _ => "th",
            };
            let month = match m {
                1 => "January",
                2 => "February",
                3 => "March",
                4 => "April",
                5 => "May",
                6 => "June",
                7 => "July",
                8 => "August",
                9 => "September",
                10 => "October",
                11 => "November",
                12 => "December",
                _ => "",
            };
            format!("{d}{suffix} {month} {y}")
        };

        if start_date == (0, 0, 0) {
            return format!("Check in - {}", format_date(end_date));
        }
        if end_date == (0, 0, 0) {
            return format!("{} - Check out", format_date(start_date));
        }

        let (sy, sm, _sd) = start_date;
        let (ey, em, _ed) = end_date;

        if sy == ey && sm == em {
            let (_, _, sd) = start_date;
            let (_, _, ed) = end_date;
            let suffix_start = match sd {
                1 | 21 | 31 => "st",
                2 | 22 => "nd",
                3 | 23 => "rd",
                _ => "th",
            };
            let suffix_end = match ed {
                1 | 21 | 31 => "st",
                2 | 22 => "nd",
                3 | 23 => "rd",
                _ => "th",
            };
            let month = match sm {
                1 => "January",
                2 => "February",
                3 => "March",
                4 => "April",
                5 => "May",
                6 => "June",
                7 => "July",
                8 => "August",
                9 => "September",
                10 => "October",
                11 => "November",
                12 => "December",
                _ => "",
            };
            return format!("{sd}{suffix_start}-{ed}{suffix_end} {month} {sy}");
        }

        format!("{} - {}", format_date(start_date), format_date(end_date))
    }

    pub fn no_of_nights(&self) -> u32 {
        let (start_year, start_month, start_day) = self.start;
        let (end_year, end_month, end_day) = self.end;
        if self.start == (0, 0, 0) || self.end == (0, 0, 0) {
            return 0;
        }
        let start_date = chrono::NaiveDate::from_ymd_opt(start_year as i32, start_month, start_day);
        let end_date = chrono::NaiveDate::from_ymd_opt(end_year as i32, end_month, end_day);
        if let (Some(start), Some(end)) = (start_date, end_date) {
            if end > start {
                return (end - start).num_days() as u32;
            }
        }
        0
    }

    pub fn normalize(&self) -> Self {
        if self.start == (0, 0, 0) || self.end == (0, 0, 0) {
            return self.clone();
        }
        if self.start > self.end {
            return DomainSelectedDateRange {
                start: self.end,
                end: self.start,
            };
        }
        self.clone()
    }

    pub fn format_date(date: (u32, u32, u32)) -> String {
        format!("{:02}-{:02}-{:04}", date.2, date.1, date.0)
    }

    pub fn format_as_human_readable_date(&self) -> String {
        let format_date = |(year, month, day): (u32, u32, u32)| {
            chrono::NaiveDate::from_ymd_opt(year as i32, month, day)
                .map(|d| d.format("%a, %b %d").to_string())
                .unwrap_or_default()
        };
        format!("{} - {}", format_date(self.start), format_date(self.end))
    }

    pub fn format_mmm_dd(&self) -> String {
        let format_md = |(year, month, day): (u32, u32, u32)| {
            chrono::NaiveDate::from_ymd_opt(year as i32, month, day)
                .map(|d| d.format("%b, %d").to_string())
                .unwrap_or_default()
        };
        format!("{} - {}", format_md(self.start), format_md(self.end))
    }

    fn dd_month_yyyy(date: (u32, u32, u32)) -> String {
        chrono::NaiveDate::from_ymd_opt(date.0 as i32, date.1, date.2)
            .map(|d| d.format("%d %B %Y").to_string())
            .unwrap_or("-".to_string())
    }

    pub fn dd_month_yyyy_start(&self) -> String {
        Self::dd_month_yyyy(self.start)
    }
    pub fn dd_month_yyyy_end(&self) -> String {
        Self::dd_month_yyyy(self.end)
    }

    pub fn format_dd_month_yyyy(&self) -> String {
        format!(
            "{} - {}",
            Self::dd_month_yyyy(self.start),
            Self::dd_month_yyyy(self.end)
        )
    }

    pub fn formatted_nights(&self) -> String {
        let nights = self.no_of_nights();
        if nights > 0 {
            format!("{} Night{}", nights, if nights > 1 { "s" } else { "" })
        } else {
            "-".to_string()
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainRoomGuest {
    pub no_of_adults: u32,
    pub no_of_children: u32,
    pub children_ages: Option<Vec<String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainPaginationParams {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainPaginationMeta {
    pub page: u32,
    pub page_size: u32,
    pub total_results: Option<i32>,
    pub has_next_page: bool,
    pub has_previous_page: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainHotelSearchCriteria {
    pub place_id: String,
    pub check_in_date: (u32, u32, u32),
    pub check_out_date: (u32, u32, u32),
    pub no_of_nights: u32,
    pub no_of_rooms: u32,
    pub room_guests: Vec<DomainRoomGuest>,
    pub guest_nationality: String,
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
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainHotelAfterSearch {
    pub hotel_code: String,
    pub hotel_name: String,
    pub hotel_category: String,
    pub star_rating: u8,
    pub price: Option<DomainPrice>,
    pub hotel_picture: String,
    pub amenities: Vec<String>,
    pub property_type: Option<String>,
    pub result_token: String,
    pub hotel_address: Option<String>,
    pub distance_from_center_km: Option<f64>,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct DomainHotelListAfterSearch {
    pub hotel_results: Vec<DomainHotelAfterSearch>,
    pub pagination: Option<DomainPaginationMeta>,
}

impl DomainHotelListAfterSearch {
    pub fn get_results_token_map(&self) -> HashMap<String, String> {
        let mut hotel_map = HashMap::new();
        for hotel in &self.hotel_results {
            hotel_map.insert(hotel.hotel_code.clone(), hotel.result_token.clone());
        }
        hotel_map
    }

    pub fn hotel_list(&self) -> Vec<DomainHotelAfterSearch> {
        self.hotel_results.clone()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainHotelInfoCriteria {
    pub token: String,
    pub hotel_ids: Vec<String>,
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
    pub suggested_selling_price: f64,
    pub suggested_selling_price_rounded_off: f64,
    pub room_price: f64,
    pub tax: f64,
    pub extra_guest_charge: f64,
    pub child_charge: f64,
    pub other_charges: f64,
    pub currency_code: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainCurrencyAmount {
    pub amount: f64,
    pub currency_code: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainTaxLine {
    pub description: String,
    pub amount: f64,
    pub currency_code: String,
    pub included: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainRoomData {
    pub mapped_room_id: String,
    pub occupancy_number: Option<u32>,
    pub room_name: String,
    pub room_unique_id: String,
    pub rate_key: String,
    pub offer_id: String,
}

/// Perk/benefit associated with a room rate (e.g., breakfast, property credit, room upgrade)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainPerk {
    pub name: String,
    pub amount: Option<f64>,
    pub currency: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainRoomOption {
    pub mapped_room_id: String,
    pub price: DomainDetailedPrice,
    pub tax_lines: Vec<DomainTaxLine>,
    pub offer_retail_rate: Option<DomainCurrencyAmount>,
    pub room_data: DomainRoomData,
    pub meal_plan: Option<String>,
    pub occupancy_info: Option<DomainRoomOccupancy>,
    pub cancellation_policies: Option<DomainCancellationPolicies>,
    pub promotions: Option<String>,
    pub remarks: Option<String>,
    /// Perks included with this rate (e.g., breakfast, property credit, room upgrade)
    pub perks: Vec<DomainPerk>,
    /// Original price before discounts (for strikethrough display)
    pub original_price: Option<f64>,
    /// Board type code (e.g., "RO" for Room Only, "BI" for Breakfast Included)
    pub board_type_code: Option<String>,
    /// Payment types supported (e.g., "NUITEE_PAY", "PROPERTY_PAY")
    pub payment_types: Vec<String>,
}

impl DomainRoomOption {
    pub fn included_taxes_total(&self) -> f64 {
        self.tax_lines
            .iter()
            .filter(|line| line.included)
            .map(|line| line.amount)
            .sum()
    }

    pub fn price_excluding_included_taxes(&self) -> f64 {
        let base_price = self.price.room_price - self.included_taxes_total();
        if base_price.is_sign_negative() {
            0.0
        } else {
            base_price
        }
    }
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
    pub all_rooms: Vec<DomainRoomGroup>,
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
        all_rooms: Vec<DomainRoomGroup>,
        search_info: Option<DomainHotelSearchInfo>,
        search_criteria: Option<DomainHotelSearchCriteria>,
    ) -> DomainHotelDetails {
        DomainHotelDetails {
            checkin,
            checkout,
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
    pub amenities: Vec<String>,
    pub property_type: Option<String>,
    pub result_token: String,
    pub hotel_address: Option<String>,
    pub distance_from_center_km: Option<f64>,
}

impl Default for DomainHotelSearchCriteria {
    fn default() -> Self {
        Self {
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
            pagination: None,
        }
    }
}

// GUEST types

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub hotel_info_criteria: DomainHotelInfoCriteria,
    pub user_details: DomainUserDetails,
    pub selected_rooms: Vec<DomainSelectedRoomWithQuantity>,
    pub selected_room: DomainRoomData,
    pub total_guests: u32,
    pub special_requests: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainBlockRoomResponse {
    pub block_id: String,
    pub is_price_changed: bool,
    pub is_cancellation_policy_changed: bool,
    pub blocked_rooms: Vec<DomainBlockedRoom>,
    pub total_price: DomainDetailedPrice,
    pub provider_data: Option<String>,
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

// Cancellation policies
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainCancellationPolicies {
    pub cancel_policy_infos: Vec<DomainCancelPolicyInfo>,
    pub hotel_remarks: Option<String>,
    pub refundable_tag: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainCancelPolicyInfo {
    pub cancel_time: String,
    pub amount: f64,
    pub policy_type: String,
    pub timezone: String,
    pub currency: String,
}
