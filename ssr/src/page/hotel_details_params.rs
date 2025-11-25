use crate::{
    component::{ChildrenAgesSignalExt, GuestSelection, SelectedDateRange},
    log,
    utils::query_params::{
        build_query_string, individual_params, update_url_with_params, QueryParamsSync,
    },
    view_state_layer::{ui_search_state::UISearchCtx, view_state::HotelInfoCtx},
};
use chrono::Datelike;
use leptos::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Hotel Details page state that can be encoded in URL
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HotelDetailsParams {
    // Basic search parameters
    pub hotel_code: Option<String>,
    pub checkin: Option<String>,
    pub checkout: Option<String>,
    pub adults: Option<u32>,
    pub children: Option<u32>,
    pub rooms: Option<u32>,
    pub children_ages: Vec<u32>,
}

impl Default for HotelDetailsParams {
    fn default() -> Self {
        Self {
            hotel_code: None,
            checkin: None,
            checkout: None,
            adults: Some(2),
            children: Some(0),
            rooms: Some(1),
            children_ages: Vec::new(),
        }
    }
}

impl HotelDetailsParams {
    /// Create from current UI context state
    pub fn from_current_context() -> Option<Self> {
        let search_ctx: UISearchCtx = expect_context();
        let hotel_info_ctx: HotelInfoCtx = expect_context();

        let hotel_code = hotel_info_ctx.hotel_code.get_untracked();
        if hotel_code.is_empty() {
            return None; // Not enough info to create params
        }

        let date_range = search_ctx.date_range.get_untracked();
        let (checkin, checkout) = if date_range.start != (0, 0, 0) && date_range.end != (0, 0, 0) {
            (
                Some(format!(
                    "{:04}-{:02}-{:02}",
                    date_range.start.0, date_range.start.1, date_range.start.2
                )),
                Some(format!(
                    "{:04}-{:02}-{:02}",
                    date_range.end.0, date_range.end.1, date_range.end.2
                )),
            )
        } else {
            (None, None)
        };

        let guests = &search_ctx.guests;
        let adults = Some(guests.adults.get_untracked());
        let children = Some(guests.children.get_untracked());
        let rooms = Some(guests.rooms.get_untracked());
        let children_ages = guests.children_ages.get_untracked().into();

        Some(Self {
            hotel_code: Some(hotel_code),
            checkin,
            checkout,
            adults,
            children,
            rooms,
            children_ages,
        })
    }

    /// Helper to parse date string back to tuple
    fn parse_date(&self, date_str: &str) -> Option<(u32, u32, u32)> {
        if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            Some((date.year() as u32, date.month(), date.day()))
        } else {
            None
        }
    }

    /// Generate a shareable URL for this hotel with all search parameters (NEW - human-readable)
    /// This can be called from hotel list or other pages to create deep links
    pub fn to_shareable_url(&self) -> String {
        use crate::app::AppRoutes;

        let query_params = self.to_query_params();
        let query_string = build_query_string(&query_params);

        format!("{}?{}", AppRoutes::HotelDetails.to_string(), query_string)
    }

    /// Create from URL query parameters (LEGACY - base64 encoded state)
    pub fn from_url_params(params: &HashMap<String, String>) -> Option<Self> {
        let encoded_state = params.get("state").cloned();
        if let Some(encoded) = encoded_state {
            crate::utils::query_params::decode_state(&encoded[..]).ok()
        } else {
            None
        }
    }

    /// Convert to URL query parameters (LEGACY - base64 encoded state)
    // pub fn to_url_params_legacy(&self) -> HashMap<String, String> {
    //     let mut params = HashMap::new();
    //     let encoded = crate::utils::query_params::encode_state(self);
    //     params.insert("state".to_string(), encoded);
    //     params
    // }

    /// Manual update URL with current state (call this when you want to update the URL)
    pub fn update_url(&self) {
        let params = self.to_query_params();
        update_url_with_params("/hotel-details", &params);
    }

    /// Create from individual query parameters (human-readable format)
    pub fn from_query_params(params: &HashMap<String, String>) -> Option<Self> {
        use chrono::{Duration, Local};
        use individual_params::*;

        log!(
            "[from_query_params] Parsing URL params for Hotel Details: {:?}",
            params
        );

        // Hotel code is essential
        let hotel_code = params.get("hotelCode").cloned();
        if hotel_code.is_none() {
            log!("[from_query_params] Missing hotelCode, cannot proceed");
            return None;
        }

        // Parse dates with defaults if missing (next week + 1 night)
        let checkin = params.get("checkin").cloned().or_else(|| {
            let date = Local::now().date_naive() + Duration::days(7);
            Some(date.format("%Y-%m-%d").to_string())
        });
        let checkout = params.get("checkout").cloned().or_else(|| {
            let date = Local::now().date_naive() + Duration::days(8);
            Some(date.format("%Y-%m-%d").to_string())
        });

        // Parse guest information
        let adults = params
            .get("adults")
            .and_then(|s| s.parse().ok())
            .or(Some(2));
        let children = params
            .get("children")
            .and_then(|s| s.parse().ok())
            .or(Some(0));
        let rooms = params.get("rooms").and_then(|s| s.parse().ok()).or(Some(1));

        // Parse children ages
        let children_ages = params
            .get("childAges")
            .map(|s| parse_comma_separated_u32(s))
            .unwrap_or_default();

        Some(Self {
            hotel_code,
            checkin,
            checkout,
            adults,
            children,
            rooms,
            children_ages,
        })
    }

    /// Convert to individual query parameters (human-readable format)
    pub fn to_query_params(&self) -> HashMap<String, String> {
        use individual_params::*;
        let mut params = HashMap::new();

        // Hotel code
        if let Some(ref code) = self.hotel_code {
            params.insert("hotelCode".to_string(), code.clone());
        }

        // Dates
        if let Some(ref checkin) = self.checkin {
            params.insert("checkin".to_string(), checkin.clone());
        }
        if let Some(ref checkout) = self.checkout {
            params.insert("checkout".to_string(), checkout.clone());
        }

        // Guest information
        if let Some(adults) = self.adults {
            params.insert("adults".to_string(), adults.to_string());
        }
        if let Some(children) = self.children {
            params.insert("children".to_string(), children.to_string());
        }
        if let Some(rooms) = self.rooms {
            params.insert("rooms".to_string(), rooms.to_string());
        }

        // Children ages
        if !self.children_ages.is_empty() {
            params.insert(
                "childAges".to_string(),
                join_comma_separated_u32(&self.children_ages),
            );
        }

        params
    }
}

impl QueryParamsSync<HotelDetailsParams> for HotelDetailsParams {
    fn sync_to_app_state(&self) {
        let hotel_info_ctx: HotelInfoCtx = expect_context();

        // Batch the sync so downstream reactive effects (like rates fetch)
        // only run once with the final, consolidated state.
        batch(|| {
            // Set hotel code
            if let Some(code) = &self.hotel_code {
                hotel_info_ctx.hotel_code.set(code.clone());
            }

            // Set date range
            if let (Some(checkin), Some(checkout)) = (&self.checkin, &self.checkout) {
                if let (Ok(start_date), Ok(end_date)) = (
                    chrono::NaiveDate::parse_from_str(checkin, "%Y-%m-%d"),
                    chrono::NaiveDate::parse_from_str(checkout, "%Y-%m-%d"),
                ) {
                    let date_range = SelectedDateRange {
                        start: (
                            start_date.year() as u32,
                            start_date.month(),
                            start_date.day(),
                        ),
                        end: (end_date.year() as u32, end_date.month(), end_date.day()),
                    };
                    UISearchCtx::set_date_range(date_range);
                }
            }

            // Set guest information
            let guest_selection = GuestSelection::default();
            if let Some(adults) = self.adults {
                guest_selection.adults.set(adults);
            }
            if let Some(children) = self.children {
                guest_selection.children.set(children);
            }
            if let Some(rooms) = self.rooms {
                guest_selection.rooms.set(rooms);
            }
            guest_selection
                .children_ages
                .set_ages(self.children_ages.clone());

            UISearchCtx::set_guests(guest_selection);
        });
    }
}
