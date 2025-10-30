use crate::api::{client_side_api::Place, consts::PAGINATION_LIMIT};
use crate::utils::query_params::individual_params;
use chrono::{Datelike, Duration, Local, NaiveDate};
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::form_urlencoded;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchParamsV2 {
    // Location
    pub place_name: Option<String>,
    pub place_address: Option<String>, // For disambiguation

    // Dates (ISO format: YYYY-MM-DD)
    pub checkin: Option<String>,
    pub checkout: Option<String>,

    // Guests
    pub adults: Option<u32>,
    pub children: Option<u32>,
    pub rooms: Option<u32>,
    pub child_ages: Vec<u32>,

    // Filters
    pub stars: Option<u8>, // Exact star rating or minimum
    pub min_price: Option<f64>,
    pub max_price: Option<f64>,
    pub amenities: Vec<String>,
    pub property_types: Vec<String>,

    // Pagination
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

impl Default for SearchParamsV2 {
    /// Smart defaults for missing parameters
    fn default() -> Self {
        let today = Local::now().date_naive();
        let checkin = today + Duration::days(7);
        let checkout = checkin + Duration::days(1);

        Self {
            place_name: None,
            place_address: None,
            checkin: Some(checkin.format("%Y-%m-%d").to_string()),
            checkout: Some(checkout.format("%Y-%m-%d").to_string()),
            adults: Some(2),
            children: Some(0),
            rooms: Some(1),
            child_ages: Vec::new(),
            stars: None,
            min_price: None,
            max_price: None,
            amenities: Vec::new(),
            property_types: Vec::new(),
            page: Some(1),
            page_size: Some(500),
        }
    }
}

impl SearchParamsV2 {
    /// Parse search parameters from current URL
    ///
    /// This is the primary way to get search state - derive it from the URL.
    /// Missing parameters are filled with smart defaults.
    ///
    /// # Example
    /// ```
    /// // URL: /hotel-list?placeName=Jaipur&stars=4
    /// let params = SearchParamsV2::from_url_v2();
    /// assert_eq!(params.place_name, Some("Jaipur".to_string()));
    /// assert_eq!(params.stars, Some(4));
    /// assert_eq!(params.adults, Some(2)); // Default applied
    /// ```
    pub fn from_url_v2() -> Self {
        let query_map = use_query_map();
        let params = query_map.get();
        let map: HashMap<String, String> = params
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect();

        Self::from_map(&map)
    }

    /// Parse search parameters from a HashMap
    ///
    /// Useful for testing and server-side rendering.
    pub fn from_map(map: &HashMap<String, String>) -> Self {
        let defaults = Self::default();

        Self {
            // Location
            place_name: map.get("placeName").cloned(),
            place_address: map.get("placeAddress").cloned(),

            // Dates
            checkin: map.get("checkin").cloned().or(defaults.checkin),
            checkout: map.get("checkout").cloned().or(defaults.checkout),

            // Guests
            adults: map
                .get("adults")
                .and_then(|s| s.parse().ok())
                .or(defaults.adults),
            children: map
                .get("children")
                .and_then(|s| s.parse().ok())
                .or(defaults.children),
            rooms: map
                .get("rooms")
                .and_then(|s| s.parse().ok())
                .or(defaults.rooms),
            child_ages: map
                .get("childAges")
                .map(|s| individual_params::parse_comma_separated_u32(s))
                .unwrap_or_default(),

            // Filters
            stars: map.get("stars").and_then(|s| s.parse().ok()),
            min_price: map.get("minPrice").and_then(|s| s.parse().ok()),
            max_price: map.get("maxPrice").and_then(|s| s.parse().ok()),
            amenities: map
                .get("amenities")
                .map(|s| individual_params::parse_comma_separated(s))
                .unwrap_or_default(),
            property_types: map
                .get("propertyTypes")
                .map(|s| individual_params::parse_comma_separated(s))
                .unwrap_or_default(),

            // Pagination
            page: map
                .get("page")
                .and_then(|s| s.parse().ok())
                .or(defaults.page),
            page_size: map
                .get("pageSize")
                .and_then(|s| s.parse().ok())
                .or(map.get("perPage").and_then(|s| s.parse().ok()))
                .or(defaults.page_size),
        }
    }

    /// Convert search parameters to URL query string
    ///
    /// Only includes non-default values to keep URLs minimal.
    ///
    /// # Example
    /// ```
    /// let params = SearchParamsV2 {
    ///     place_name: Some("Jaipur".to_string()),
    ///     stars: Some(4),
    ///     ..Default::default()
    /// };
    ///
    /// let url = params.to_url_path("/hotel-list");
    /// // /hotel-list?placeName=Jaipur&stars=4
    /// ```
    pub fn to_url_path(&self, base_path: &str) -> String {
        let query_string = self.to_query_string();
        if query_string.is_empty() {
            base_path.to_string()
        } else {
            format!("{}?{}", base_path, query_string)
        }
    }

    /// Convert to query string (without base path)
    pub fn to_query_string(&self) -> String {
        let mut params = Vec::new();

        // Location (always include if present)
        if let Some(ref name) = self.place_name {
            params.push(("placeName", name.clone()));
        }
        if let Some(ref address) = self.place_address {
            params.push(("placeAddress", address.clone()));
        }

        // Dates (only if different from defaults)
        let defaults = Self::default();
        if self.checkin != defaults.checkin {
            if let Some(ref checkin) = self.checkin {
                params.push(("checkin", checkin.clone()));
            }
        }
        if self.checkout != defaults.checkout {
            if let Some(ref checkout) = self.checkout {
                params.push(("checkout", checkout.clone()));
            }
        }

        // Guests (only if different from defaults)
        if self.adults != defaults.adults {
            if let Some(adults) = self.adults {
                params.push(("adults", adults.to_string()));
            }
        }
        if self.children != defaults.children {
            if let Some(children) = self.children {
                params.push(("children", children.to_string()));
            }
        }
        if self.rooms != defaults.rooms {
            if let Some(rooms) = self.rooms {
                params.push(("rooms", rooms.to_string()));
            }
        }
        if !self.child_ages.is_empty() {
            params.push((
                "childAges",
                individual_params::join_comma_separated_u32(&self.child_ages),
            ));
        }

        // Filters (always include if present)
        if let Some(stars) = self.stars {
            params.push(("stars", stars.to_string()));
        }
        if let Some(min_price) = self.min_price {
            params.push(("minPrice", min_price.to_string()));
        }
        if let Some(max_price) = self.max_price {
            params.push(("maxPrice", max_price.to_string()));
        }
        if !self.amenities.is_empty() {
            params.push((
                "amenities",
                individual_params::join_comma_separated(&self.amenities),
            ));
        }
        if !self.property_types.is_empty() {
            params.push((
                "propertyTypes",
                individual_params::join_comma_separated(&self.property_types),
            ));
        }

        // Pagination (only if different from defaults)
        if self.page != defaults.page {
            if let Some(page) = self.page {
                params.push(("page", page.to_string()));
            }
        }
        if self.page_size != defaults.page_size {
            if let Some(page_size) = self.page_size {
                params.push(("pageSize", page_size.to_string()));
            }
        }

        form_urlencoded::Serializer::new(String::new())
            .extend_pairs(params)
            .finish()
    }

    /// Get checkin date as NaiveDate
    pub fn checkin_date(&self) -> Option<NaiveDate> {
        self.checkin
            .as_ref()
            .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
    }

    /// Get checkout date as NaiveDate
    pub fn checkout_date(&self) -> Option<NaiveDate> {
        self.checkout
            .as_ref()
            .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
    }

    /// Calculate number of nights
    pub fn number_of_nights(&self) -> u32 {
        if let (Some(checkin), Some(checkout)) = (self.checkin_date(), self.checkout_date()) {
            (checkout - checkin).num_days().max(1) as u32
        } else {
            1
        }
    }

    /// Validate dates (checkin before checkout, not in past)
    pub fn validate_dates(&self) -> Result<(), String> {
        let today = Local::now().date_naive();

        if let Some(checkin) = self.checkin_date() {
            if checkin < today {
                return Err("Check-in date cannot be in the past".to_string());
            }
        }

        if let (Some(checkin), Some(checkout)) = (self.checkin_date(), self.checkout_date()) {
            if checkout <= checkin {
                return Err("Check-out must be after check-in".to_string());
            }
        }

        Ok(())
    }

    /// Check if search has minimum required parameters
    pub fn is_valid_for_search(&self) -> bool {
        self.place_name.is_some() && self.validate_dates().is_ok()
    }

    /// Check if any filters are applied
    pub fn has_filters(&self) -> bool {
        self.stars.is_some()
            || self.min_price.is_some()
            || self.max_price.is_some()
            || !self.amenities.is_empty()
            || !self.property_types.is_empty()
    }

    /// Clone with modified field (builder pattern for navigation)
    pub fn with_place_name(mut self, name: String) -> Self {
        self.place_name = Some(name);
        self
    }

    pub fn with_dates(mut self, checkin: String, checkout: String) -> Self {
        self.checkin = Some(checkin);
        self.checkout = Some(checkout);
        self
    }

    pub fn with_guests(mut self, adults: u32, children: u32, rooms: u32) -> Self {
        self.adults = Some(adults);
        self.children = Some(children);
        self.rooms = Some(rooms);
        self
    }

    pub fn with_stars(mut self, stars: Option<u8>) -> Self {
        self.stars = stars;
        self
    }

    pub fn with_price_range(mut self, min: Option<f64>, max: Option<f64>) -> Self {
        self.min_price = min;
        self.max_price = max;
        self
    }

    pub fn with_amenities(mut self, amenities: Vec<String>) -> Self {
        self.amenities = amenities;
        self
    }

    pub fn with_property_types(mut self, types: Vec<String>) -> Self {
        self.property_types = types;
        self
    }

    pub fn with_page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    pub fn with_page_size(mut self, page_size: u32) -> Self {
        self.page_size = Some(page_size);
        self
    }

    /// Get current page (1-indexed)
    pub fn current_page(&self) -> u32 {
        self.page.unwrap_or(1)
    }

    /// Get page size
    pub fn current_page_size(&self) -> u32 {
        self.page_size.unwrap_or(PAGINATION_LIMIT as u32)
    }

    /// Check if there could be a next page (based on result count)
    pub fn has_next_page(&self, result_count: usize) -> bool {
        result_count >= self.current_page_size() as usize
    }

    /// Check if there is a previous page
    pub fn has_previous_page(&self) -> bool {
        self.current_page() > 1
    }

    /// Toggle an amenity (add if not present, remove if present)
    pub fn toggle_amenity(mut self, amenity: String) -> Self {
        if let Some(pos) = self.amenities.iter().position(|a| a == &amenity) {
            self.amenities.remove(pos);
        } else {
            self.amenities.push(amenity);
        }
        self
    }

    /// Toggle a property type
    pub fn toggle_property_type(mut self, property_type: String) -> Self {
        if let Some(pos) = self.property_types.iter().position(|p| p == &property_type) {
            self.property_types.remove(pos);
        } else {
            self.property_types.push(property_type);
        }
        self
    }

    /// Clear all filters
    pub fn clear_filters(mut self) -> Self {
        self.stars = None;
        self.min_price = None;
        self.max_price = None;
        self.amenities.clear();
        self.property_types.clear();
        self
    }

    /// Reset to first page (useful when filters change)
    pub fn reset_to_first_page(mut self) -> Self {
        self.page = Some(1);
        self
    }
}

/// Hotel details parameters (minimal URL design)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HotelDetailsParamsV2 {
    pub hotel_code: String,

    // Inherit search context
    pub place_name: Option<String>,
    pub place_address: Option<String>,

    // Dates
    pub checkin: Option<String>,
    pub checkout: Option<String>,

    // Guests
    pub adults: Option<u32>,
    pub children: Option<u32>,
    pub rooms: Option<u32>,
    pub child_ages: Vec<u32>,
}

impl HotelDetailsParamsV2 {
    /// Parse from URL
    pub fn from_url_v2() -> Option<Self> {
        let query_map = use_query_map();
        let params = query_map.get();
        let map: HashMap<String, String> = params
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect();

        Self::from_map(&map)
    }

    /// Parse from HashMap
    pub fn from_map(map: &HashMap<String, String>) -> Option<Self> {
        let hotel_code = map.get("hotelCode")?.clone();
        let search_defaults = SearchParamsV2::default();

        Some(Self {
            hotel_code,
            place_name: map.get("placeName").cloned(),
            place_address: map.get("placeAddress").cloned(),
            checkin: map.get("checkin").cloned().or(search_defaults.checkin),
            checkout: map.get("checkout").cloned().or(search_defaults.checkout),
            adults: map
                .get("adults")
                .and_then(|s| s.parse().ok())
                .or(search_defaults.adults),
            children: map
                .get("children")
                .and_then(|s| s.parse().ok())
                .or(search_defaults.children),
            rooms: map
                .get("rooms")
                .and_then(|s| s.parse().ok())
                .or(search_defaults.rooms),
            child_ages: map
                .get("childAges")
                .map(|s| individual_params::parse_comma_separated_u32(s))
                .unwrap_or_default(),
        })
    }

    /// Convert to URL path
    pub fn to_url_path(&self, base_path: &str) -> String {
        let query_string = self.to_query_string();
        format!("{}?{}", base_path, query_string)
    }

    /// Convert to query string
    pub fn to_query_string(&self) -> String {
        let mut params = vec![("hotelCode", self.hotel_code.clone())];

        if let Some(ref name) = self.place_name {
            params.push(("placeName", name.clone()));
        }
        if let Some(ref address) = self.place_address {
            params.push(("placeAddress", address.clone()));
        }
        if let Some(ref checkin) = self.checkin {
            params.push(("checkin", checkin.clone()));
        }
        if let Some(ref checkout) = self.checkout {
            params.push(("checkout", checkout.clone()));
        }
        if let Some(adults) = self.adults {
            params.push(("adults", adults.to_string()));
        }
        if let Some(children) = self.children {
            params.push(("children", children.to_string()));
        }
        if let Some(rooms) = self.rooms {
            params.push(("rooms", rooms.to_string()));
        }
        if !self.child_ages.is_empty() {
            params.push((
                "childAges",
                individual_params::join_comma_separated_u32(&self.child_ages),
            ));
        }

        form_urlencoded::Serializer::new(String::new())
            .extend_pairs(params)
            .finish()
    }

    /// Create from search params + hotel code
    pub fn from_search_params(search: &SearchParamsV2, hotel_code: String) -> Self {
        Self {
            hotel_code,
            place_name: search.place_name.clone(),
            place_address: search.place_address.clone(),
            checkin: search.checkin.clone(),
            checkout: search.checkout.clone(),
            adults: search.adults,
            children: search.children,
            rooms: search.rooms,
            child_ages: search.child_ages.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_dates() {
        let params = SearchParamsV2::default();

        // Checkin should be 7 days from today
        let checkin = params.checkin_date().unwrap();
        let today = Local::now().date_naive();
        assert_eq!(checkin, today + Duration::days(7));

        // Checkout should be 1 day after checkin
        let checkout = params.checkout_date().unwrap();
        assert_eq!(checkout, checkin + Duration::days(1));
    }

    #[test]
    fn test_parse_minimal_url() {
        let mut map = HashMap::new();
        map.insert("placeName".to_string(), "Jaipur".to_string());

        let params = SearchParamsV2::from_map(&map);
        assert_eq!(params.place_name, Some("Jaipur".to_string()));
        assert_eq!(params.adults, Some(2)); // Default
        assert_eq!(params.rooms, Some(1)); // Default
    }

    #[test]
    fn test_parse_with_filters() {
        let mut map = HashMap::new();
        map.insert("placeName".to_string(), "Jaipur".to_string());
        map.insert("stars".to_string(), "4".to_string());
        map.insert("amenities".to_string(), "wifi,pool".to_string());

        let params = SearchParamsV2::from_map(&map);
        assert_eq!(params.stars, Some(4));
        assert_eq!(params.amenities, vec!["wifi", "pool"]);
    }

    #[test]
    fn test_to_query_string_minimal() {
        let params = SearchParamsV2 {
            place_name: Some("Jaipur".to_string()),
            ..Default::default()
        };

        let query = params.to_query_string();
        assert!(query.contains("placeName=Jaipur"));
        // Should not contain default values
        assert!(!query.contains("adults=2"));
    }

    #[test]
    fn test_builder_pattern() {
        let params = SearchParamsV2::default()
            .with_place_name("Jaipur".to_string())
            .with_stars(Some(4))
            .with_page(2);

        assert_eq!(params.place_name, Some("Jaipur".to_string()));
        assert_eq!(params.stars, Some(4));
        assert_eq!(params.page, Some(2));
    }

    #[test]
    fn test_toggle_amenity() {
        let params = SearchParamsV2::default()
            .toggle_amenity("wifi".to_string())
            .toggle_amenity("pool".to_string());

        assert_eq!(params.amenities.len(), 2);

        // Toggle off
        let params = params.toggle_amenity("wifi".to_string());
        assert_eq!(params.amenities.len(), 1);
        assert_eq!(params.amenities[0], "pool");
    }

    #[test]
    fn test_validate_dates() {
        let mut params = SearchParamsV2::default();
        assert!(params.validate_dates().is_ok());

        // Set past date
        params.checkin = Some("2020-01-01".to_string());
        assert!(params.validate_dates().is_err());
    }

    #[test]
    fn test_hotel_details_from_search() {
        let search = SearchParamsV2 {
            place_name: Some("Jaipur".to_string()),
            adults: Some(2),
            ..Default::default()
        };

        let details = HotelDetailsParamsV2::from_search_params(&search, "ABC123".to_string());
        assert_eq!(details.hotel_code, "ABC123");
        assert_eq!(details.place_name, Some("Jaipur".to_string()));
        assert_eq!(details.adults, Some(2));
    }
}
