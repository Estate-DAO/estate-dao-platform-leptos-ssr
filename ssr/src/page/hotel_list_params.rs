use crate::{
    api::{
        self,
        client_side_api::{ClientSideApiClient, Place, PlaceData},
    },
    application_services::filter_types::UISearchFilters,
    component::{ChildrenAgesSignalExt, Destination, GuestSelection, SelectedDateRange},
    utils::query_params::{update_url_with_state, FilterMap, QueryParamsSync, SortDirection},
    view_state_layer::ui_search_state::UISearchCtx,
};
use chrono::Datelike;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// #[server(LookupDestinationById)]
// pub async fn lookup_destination_by_id(city: String) -> Result<Destination, ServerFnError> {
//     use std::io::BufReader;

//     let res =
//         duck_searcher::search_city_by_name(&city).map_err(|e| ServerFnError::new(e.to_string()))?;
//     Ok(Destination {
//         city_id: res.city_code,
//         city: res.city_name,
//         country_name: res.country_name,
//         country_code: res.country_code,
//         latitude: Some(res.latitude),
//         longitude: Some(res.longitude),
//     })
// }

pub async fn lookup_place_by_id(place_id: String) -> Result<PlaceData, ServerFnError> {
    let api_client = ClientSideApiClient::new();
    let place = api_client
        .get_place_details_by_id(place_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(place)
}

/// Hotel List page state that can be encoded in URL via base64
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HotelListParams {
    // Basic search parameters
    pub checkin: Option<String>,
    pub checkout: Option<String>,
    pub adults: Option<u32>,
    pub children: Option<u32>,
    pub rooms: Option<u32>,
    pub children_ages: Vec<u32>,

    // Advanced filtering using nested objects (works seamlessly with base64)
    pub filters: FilterMap,

    // Sorting (multi-column support)
    pub sort: Vec<(String, SortDirection)>,

    // Pagination
    pub page: Option<u32>,
    pub per_page: Option<u32>,

    // Destination
    // destination - city name
    pub place_details: Option<PlaceData>,
    pub place: Option<Place>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

impl Default for HotelListParams {
    fn default() -> Self {
        Self {
            place_details: None,
            place: None,
            checkin: None,
            checkout: None,
            adults: Some(2),
            children: Some(0),
            rooms: Some(1),
            children_ages: Vec::new(),
            filters: HashMap::new(),
            sort: Vec::new(),
            page: Some(1),
            per_page: Some(20),
            latitude: None,
            longitude: None,
        }
    }
}

impl HotelListParams {
    /// Create from current search context state
    pub fn from_search_context(search_ctx: &UISearchCtx) -> Self {
        let place_details = search_ctx.place_details.get_untracked();

        let place = search_ctx.place.get_untracked();

        let latitude = place_details.as_ref().map(|f| f.location.latitude);

        let longitude = place_details.as_ref().map(|f| f.location.longitude);

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

        let filters_map = search_ctx.filters.get_untracked().to_filter_map();

        Self {
            place_details,
            place,
            checkin,
            checkout,
            adults,
            children,
            rooms,
            children_ages,
            filters: filters_map,
            sort: Vec::new(),
            page: Some(1),
            per_page: Some(20),
            latitude,
            longitude,
        }
    }

    /// Get a user-friendly description of current filters
    pub fn get_filter_description(&self) -> String {
        if self.filters.is_empty() {
            return "No filters applied".to_string();
        }

        let descriptions: Vec<String> = self
            .filters
            .iter()
            .map(|(field, op)| match op {
                crate::utils::query_params::ComparisonOp::Eq(val) => format!("{} = {}", field, val),
                crate::utils::query_params::ComparisonOp::Gt(val) => format!("{} > {}", field, val),
                crate::utils::query_params::ComparisonOp::Gte(val) => {
                    format!("{} >= {}", field, val)
                }
                crate::utils::query_params::ComparisonOp::Lt(val) => format!("{} < {}", field, val),
                crate::utils::query_params::ComparisonOp::Lte(val) => {
                    format!("{} <= {}", field, val)
                }
                crate::utils::query_params::ComparisonOp::In(vals) => {
                    format!("{} in [{}]", field, vals.join(", "))
                }
                crate::utils::query_params::ComparisonOp::All(vals) => {
                    format!("{} includes all [{}]", field, vals.join(", "))
                }
                crate::utils::query_params::ComparisonOp::Near(lat, lng, radius) => {
                    format!("{} near ({}, {}) within {}km", field, lat, lng, radius)
                }
            })
            .collect();

        descriptions.join("; ")
    }

    /// Get a user-friendly description of current sorting
    pub fn get_sort_description(&self) -> String {
        if self.sort.is_empty() {
            return "No sorting applied".to_string();
        }

        let descriptions: Vec<String> = self
            .sort
            .iter()
            .map(|(field, direction)| format!("{} ({})", field, direction.to_string()))
            .collect();

        format!("Sorted by: {}", descriptions.join(", "))
    }

    /// Manual update URL with current state (call this when you want to update the URL)
    pub fn update_url(&self) {
        update_url_with_state(self);
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

impl QueryParamsSync<HotelListParams> for HotelListParams {
    fn sync_to_app_state(&self) {
        let search_ctx: UISearchCtx = expect_context();

        // Set destination if available
        if let Some(place) = &self.place {
            UISearchCtx::set_place(place.clone());
            UISearchCtx::set_place_details(self.place_details.clone());
        }
        // if let Some(place) = &self.place.clone().map(|p| p.place_id) {
        //     // Spawn async lookup task
        //     let place = place.clone();
        //     spawn_local(async move {
        //         if let Ok(place) = lookup_place_by_id(place.clone()).await {
        //             UISearchCtx::set_place_details(Some(place));
        //         }
        //     });
        // }

        // Set date range if available
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

        let filters = if self.filters.is_empty() {
            UISearchFilters::default()
        } else {
            UISearchFilters::from_filter_map(&self.filters)
        };

        UISearchCtx::set_filters(filters);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::query_params::{decode_state, encode_state};

    #[test]
    fn test_hotel_list_params_serialization() {
        let mut params = HotelListParams::default();
        params.place = Some(Place {
            place_id: "1254".to_string(),
            display_name: "Test Place".to_string(),
            formatted_address: "123 Test St, Test City, TC 12345".to_string(),
        });
        params.checkin = Some("2025-01-15".to_string());
        params.checkout = Some("2025-01-20".to_string());
        params.adults = Some(2);
        params.children = Some(1);
        params.children_ages = vec![8, 10];

        // Test base64 encoding/decoding
        let encoded = encode_state(&params);
        assert!(!encoded.is_empty());

        let decoded: HotelListParams = decode_state(&encoded).unwrap();
        assert_eq!(params, decoded);
    }

    #[test]
    fn test_from_url_params() {
        let params = HotelListParams {
            place: Some(Place {
                place_id: "NYC".to_string(),
                display_name: "New York City".to_string(),
                formatted_address: "New York, NY 10001".to_string(),
            }),
            adults: Some(2),
            ..Default::default()
        };

        let query_params = params.to_url_params();
        let parsed = HotelListParams::from_url_params(&query_params.into_iter().collect()).unwrap();

        assert_eq!(params, parsed);
    }

    #[test]
    fn test_filter_description() {
        let mut params = HotelListParams::default();
        params.filters.insert(
            "price".to_string(),
            crate::utils::query_params::ComparisonOp::Gte(100.0),
        );
        params.filters.insert(
            "amenities".to_string(),
            crate::utils::query_params::ComparisonOp::In(vec![
                "wifi".to_string(),
                "pool".to_string(),
            ]),
        );

        let description = params.get_filter_description();
        assert!(description.contains("price"));
        assert!(description.contains("amenities"));
    }
}
