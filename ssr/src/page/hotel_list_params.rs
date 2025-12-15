use crate::{
    api::{
        self,
        client_side_api::{ClientSideApiClient, Place, PlaceData},
        consts::PAGINATION_LIMIT,
    },
    application_services::filter_types::{UISearchFilters, UISortOptions},
    component::{ChildrenAgesSignalExt, Destination, GuestSelection, SelectedDateRange},
    log,
    utils::query_params::{
        build_query_string, individual_params, update_url_with_params, update_url_with_state,
        FilterMap, QueryParamsSync, SortDirection,
    },
    view_state_layer::ui_search_state::{SearchListResults, UIPaginationState, UISearchCtx},
};
use chrono::Datelike;
use leptos::*;
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

/// Helper to extract display name from PlaceData address components
pub fn get_display_name_from_place_data(place_data: &PlaceData) -> String {
    // Define the hierarchy of place types to search for the display name.
    // More specific types come first.
    let type_hierarchy = [
        "neighborhood",
        "sublocality_level_1",
        "sublocality",
        "locality",
        "administrative_area_level_1",
    ];

    for place_type in &type_hierarchy {
        if let Some(component) = place_data
            .address_components
            .iter()
            .find(|c| c.types.contains(&place_type.to_string()))
        {
            return component.long_text.clone();
        }
    }

    // Fallback to the first component if no preferred types are found.
    place_data
        .address_components
        .first()
        .map(|c| c.long_text.clone())
        .unwrap_or_default()
}

/// Helper to build formatted address from PlaceData address components
/// Note: Does NOT include the component used for the display_name to avoid duplication.
pub fn get_formatted_address_from_place_data(place_data: &PlaceData) -> String {
    let display_name = get_display_name_from_place_data(place_data);

    // Collect address parts, excluding the one used for the display name.
    let parts: Vec<&str> = place_data
        .address_components
        .iter()
        .filter_map(|c| {
            // Exclude the component if its long_text matches the display_name
            if c.long_text != display_name {
                Some(c.long_text.as_str())
            } else {
                None
            }
        })
        .collect();

    // A more robust way to join parts, removing duplicates that might still appear
    // (e.g. "Jaipur", "Jaipur").
    let mut unique_parts = Vec::new();
    for part in parts {
        if !unique_parts.contains(&part) {
            unique_parts.push(part);
        }
    }

    // Let's try to build a sensible address from what's left.
    // A common pattern is City, State, Country.
    // We can try to find these components.
    let locality = place_data
        .address_components
        .iter()
        .find(|c| c.types.contains(&"locality".to_string()))
        .map(|c| c.long_text.as_str());

    let admin_area = place_data
        .address_components
        .iter()
        .find(|c| c.types.contains(&"administrative_area_level_1".to_string()))
        .map(|c| c.long_text.as_str());

    let country = place_data
        .address_components
        .iter()
        .find(|c| c.types.contains(&"country".to_string()))
        .map(|c| c.long_text.as_str());

    let mut address_parts = Vec::new();
    if let Some(loc) = locality {
        if loc != display_name && !address_parts.contains(&loc) {
            address_parts.push(loc);
        }
    }
    if let Some(admin) = admin_area {
        if admin != display_name && !address_parts.contains(&admin) {
            address_parts.push(admin);
        }
    }
    if let Some(cnt) = country {
        if cnt != display_name && !address_parts.contains(&cnt) {
            address_parts.push(cnt);
        }
    }

    address_parts.join(", ")
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

    // Place name to search (when placeId is not available in URL)
    #[serde(skip)]
    pub place_name_to_search: Option<String>,
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
            per_page: Some(500),
            latitude: None,
            longitude: None,
            place_name_to_search: None,
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

        // Get sort options from context
        let sort_options = search_ctx.sort_options.get_untracked();
        let sort = if sort_options.has_sort() {
            // Convert UISortOptions to the sort Vec format used by HotelListParams
            vec![(
                sort_options.to_string(),
                match sort_options.sort_direction {
                    Some(
                        crate::application_services::filter_types::DomainSortDirection::Ascending,
                    ) => SortDirection::Asc,
                    Some(
                        crate::application_services::filter_types::DomainSortDirection::Descending,
                    ) => SortDirection::Desc,
                    None => SortDirection::Desc, // Default to descending
                },
            )]
        } else {
            Vec::new()
        };

        // Get pagination from context instead of hardcoding
        // Only include page/per_page if they differ from defaults to avoid polluting URLs
        let pagination_state: UIPaginationState = expect_context();
        let current_page = pagination_state.current_page.get_untracked();
        let current_page_size = pagination_state.page_size.get_untracked();

        // Only include page if > 1 (to avoid polluting URL with defaults)
        let page = if current_page > 1 {
            Some(current_page)
        } else {
            None
        };

        // Only include per_page if different from backend default
        let per_page = if current_page_size != PAGINATION_LIMIT as u32 {
            Some(current_page_size)
        } else {
            None
        };

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
            sort,
            page,
            per_page,
            latitude,
            longitude,
            place_name_to_search: None,
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
        let params = self.to_query_params();
        update_url_with_params("/hotel-list", &params);
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
    pub fn to_url_params(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();
        let encoded = crate::utils::query_params::encode_state(self);
        params.insert("state".to_string(), encoded);
        params
    }

    /// Create from individual query parameters (NEW - human-readable format)
    /// Accepts HashMap<String, String> which is converted from leptos_router params
    pub fn from_query_params(params: &HashMap<String, String>) -> Option<Self> {
        use chrono::{Duration, Local};
        use individual_params::*;

        log!("[from_query_params] Parsing URL params: {:?}", params);

        // Check for legacy format first
        if params.contains_key("state") {
            log!("[from_query_params] Found legacy base64 state param");
            return Self::from_url_params(params);
        }

        // Parse Place from placeId and placeName
        let place_id_opt = params.get("placeId");
        let place_name_opt = params.get("placeName");

        let (place, place_name_to_search) = match (place_id_opt, place_name_opt) {
            // Case 1: Both placeId and placeName present (normal case)
            (Some(place_id), name_opt) => {
                log!("[from_query_params] placeId: {}", place_id);
                let place = Place {
                    place_id: place_id.clone(),
                    display_name: name_opt.cloned().unwrap_or_default(),
                    formatted_address: params.get("placeAddress").cloned().unwrap_or_default(),
                };
                (Some(place), None)
            }

            // Case 2: Only placeName present - need to search for placeId
            (None, Some(place_name)) => {
                log!(
                    "[from_query_params] Only placeName provided: '{}', will search for placeId",
                    place_name
                );
                (None, Some(place_name.clone()))
            }

            // Case 3: Neither present - invalid, cannot proceed
            (None, None) => {
                log!("[from_query_params] Missing both placeId and placeName, cannot proceed");
                return None;
            }
        };

        // Parse dates, ensuring they are not in the past.
        let today = Local::now().date_naive();

        let checkin = params
            .get("checkin")
            .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
            .filter(|&date| date >= today)
            .map(|date| date.format("%Y-%m-%d").to_string())
            .or_else(|| Some(today.format("%Y-%m-%d").to_string()));

        let checkout = params
            .get("checkout")
            .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
            .filter(|&date| date > today)
            .map(|date| date.format("%Y-%m-%d").to_string())
            .or_else(|| Some((today + Duration::days(1)).format("%Y-%m-%d").to_string()));

        // Parse guest information
        let adults = params.get("adults").and_then(|s| s.parse().ok());
        let children = params.get("children").and_then(|s| s.parse().ok());
        let rooms = params.get("rooms").and_then(|s| s.parse().ok());

        // Parse children ages
        let children_ages = params
            .get("childAges")
            .map(|s| parse_comma_separated_u32(&s))
            .unwrap_or_default();

        // Parse filters
        let mut filters = FilterMap::new();

        // Price filters - use keys that match UISearchFilters constants
        if let Some(min_price) = params.get("minPrice").and_then(|s| s.parse().ok()) {
            filters.insert(
                "min_price_per_night".to_string(),
                crate::utils::query_params::ComparisonOp::Gte(min_price),
            );
        }
        if let Some(max_price) = params.get("maxPrice").and_then(|s| s.parse().ok()) {
            filters.insert(
                "max_price_per_night".to_string(),
                crate::utils::query_params::ComparisonOp::Lte(max_price),
            );
        }

        // Star rating filter - check both "stars" and "minStars" for compatibility
        if let Some(min_stars) = params
            .get("stars")
            .or_else(|| params.get("minStars"))
            .and_then(|s| s.parse::<u32>().ok())
        {
            filters.insert(
                "min_star_rating".to_string(),
                crate::utils::query_params::ComparisonOp::Gte(min_stars as f64),
            );
        }

        // Amenities filter
        if let Some(amenities_str) = params.get("amenities") {
            log!(
                "[from_query_params] Found amenities param: {}",
                amenities_str
            );
            let amenities = parse_comma_separated(&amenities_str);
            log!("[from_query_params] Parsed amenities: {:?}", amenities);
            if !amenities.is_empty() {
                filters.insert(
                    "amenities".to_string(),
                    crate::utils::query_params::ComparisonOp::All(amenities),
                );
            }
        }

        // Property types filter (check both "propertyTypes" and "property_types")
        if let Some(types_str) = params
            .get("propertyTypes")
            .or_else(|| params.get("property_types"))
        {
            log!(
                "[from_query_params] Found propertyTypes param: {}",
                types_str
            );
            let types = parse_comma_separated(&types_str);
            log!("[from_query_params] Parsed property types: {:?}", types);
            if !types.is_empty() {
                filters.insert(
                    "property_types".to_string(),
                    crate::utils::query_params::ComparisonOp::In(types),
                );
            }
        }

        // Pagination - support both pageSize and perPage (perPage for backward compat)
        // Only set if present in URL - otherwise leave as None to use backend default
        let page = params.get("page").and_then(|s| s.parse().ok());
        let per_page = params
            .get("pageSize")
            .and_then(|s| s.parse().ok())
            .or_else(|| params.get("perPage").and_then(|s| s.parse().ok()));

        // Coordinates (optional)
        let latitude = params.get("lat").and_then(|s| s.parse().ok());
        let longitude = params.get("lng").and_then(|s| s.parse().ok());

        // Sort options
        let sort = if let Some(sort_str) = params.get("sort") {
            // Parse sort string like "price_low_to_high" to sort vector
            vec![(sort_str.clone(), SortDirection::Desc)] // Default direction, will be inferred from sort_str
        } else {
            Vec::new()
        };

        log!("[from_query_params] Parsed {} filters", filters.len());
        log!(
            "[from_query_params] Filter keys: {:?}",
            filters.keys().collect::<Vec<_>>()
        );

        Some(Self {
            place,
            place_details: None, // Will be fetched async if needed
            checkin,
            checkout,
            adults,
            children,
            rooms,
            children_ages,
            filters,
            sort,
            page,
            per_page,
            latitude,
            longitude,
            place_name_to_search,
        })
    }

    /// Convert to individual query parameters (NEW - human-readable format)
    pub fn to_query_params(&self) -> HashMap<String, String> {
        use individual_params::*;
        let mut params = HashMap::new();

        // Place information - always include if present
        if let Some(ref place) = self.place {
            params.insert("placeId".to_string(), place.place_id.clone());
        }

        // Dates - always include if present
        if let Some(ref checkin) = self.checkin {
            params.insert("checkin".to_string(), checkin.clone());
        }
        if let Some(ref checkout) = self.checkout {
            params.insert("checkout".to_string(), checkout.clone());
        }

        // Guest information - always include if present (even if default values)
        if let Some(adults) = self.adults {
            params.insert("adults".to_string(), adults.to_string());
        }
        if let Some(children) = self.children {
            params.insert("children".to_string(), children.to_string());
        }
        if let Some(rooms) = self.rooms {
            params.insert("rooms".to_string(), rooms.to_string());
        }

        // Children ages - only if children > 0
        if !self.children_ages.is_empty() {
            params.insert(
                "childAges".to_string(),
                join_comma_separated_u32(&self.children_ages),
            );
        }

        // Filters - only include if set
        for (key, op) in &self.filters {
            match (key.as_str(), op) {
                ("min_price_per_night", crate::utils::query_params::ComparisonOp::Gte(val)) => {
                    params.insert("minPrice".to_string(), val.to_string());
                }
                ("max_price_per_night", crate::utils::query_params::ComparisonOp::Lte(val)) => {
                    params.insert("maxPrice".to_string(), val.to_string());
                }
                ("min_star_rating", crate::utils::query_params::ComparisonOp::Gte(val)) => {
                    params.insert("stars".to_string(), (*val as u32).to_string());
                }
                ("amenities", crate::utils::query_params::ComparisonOp::All(vals))
                | ("amenities", crate::utils::query_params::ComparisonOp::In(vals)) => {
                    params.insert("amenities".to_string(), join_comma_separated(vals));
                }
                ("property_types", crate::utils::query_params::ComparisonOp::In(vals)) => {
                    params.insert("propertyTypes".to_string(), join_comma_separated(vals));
                }
                _ => {} // Skip unknown filters
            }
        }

        // Pagination - only include if different from defaults
        // Only add page if > 1 (default is 1)
        if let Some(page) = self.page {
            if page > 1 {
                params.insert("page".to_string(), page.to_string());
            }
        }

        // Only add pageSize if explicitly set (backend will use PAGINATION_LIMIT by default)
        // Don't include if it's None - let backend use its default
        if let Some(per_page) = self.per_page {
            params.insert("pageSize".to_string(), per_page.to_string());
        }

        // Sort options - only include if not default/empty
        if !self.sort.is_empty() {
            if let Some((sort_field, _)) = self.sort.first() {
                params.insert("sort".to_string(), sort_field.clone());
            }
        }

        // NOTE: Removed lat/lng/placeAddress - these can be derived from placeId via API

        params
    }

    /// Generate a shareable URL for hotel list with all search parameters
    pub fn to_shareable_url(&self) -> String {
        let params = self.to_query_params();
        let query_string = build_query_string(&params);

        if query_string.is_empty() {
            "/hotel-list".to_string()
        } else {
            format!("/hotel-list?{}", query_string)
        }
    }
}

impl QueryParamsSync<HotelListParams> for HotelListParams {
    fn sync_to_app_state(&self) {
        let search_ctx: UISearchCtx = expect_context();

        // Set destination if available
        if let Some(place) = &self.place {
            // Check if we need to fetch place details:
            // 1. If place_details is None, OR
            // 2. If the incoming placeId is different from the currently stored place
            let current_place = search_ctx.place.get_untracked();
            let should_fetch = self.place_details.is_none()
                || current_place.as_ref().map_or(true, |current| {
                    // Fetch if placeId has changed
                    current.place_id != place.place_id
                });

            // If place has changed, clear search results to trigger hotel search resource
            if current_place
                .as_ref()
                .map_or(false, |current| current.place_id != place.place_id)
            {
                log!(
                    "[sync_to_app_state] Place changed from {:?} to {:?}, clearing search results",
                    current_place.as_ref().map(|p| &p.place_id),
                    Some(&place.place_id)
                );
                SearchListResults::reset();
                UIPaginationState::reset_to_first_page();
            }

            if should_fetch {
                let place_id = place.place_id.clone();
                let place_for_update = place.clone();
                spawn_local(async move {
                    if let Ok(place_details) = lookup_place_by_id(place_id).await {
                        // Update the Place with proper display_name and formatted_address from place_details
                        let updated_place = Place {
                            place_id: place_for_update.place_id.clone(),
                            display_name: get_display_name_from_place_data(&place_details),
                            formatted_address: get_formatted_address_from_place_data(
                                &place_details,
                            ),
                        };

                        // Set both place (for DestinationPicker display) and place_details
                        UISearchCtx::set_place(updated_place);
                        UISearchCtx::set_place_details(Some(place_details));
                    }
                });
            } else {
                // If we already have place_details and placeId matches, update the place with proper names
                if let Some(ref details) = self.place_details {
                    let updated_place = Place {
                        place_id: place.place_id.clone(),
                        display_name: get_display_name_from_place_data(details),
                        formatted_address: get_formatted_address_from_place_data(details),
                    };
                    UISearchCtx::set_place(updated_place);
                } else {
                    UISearchCtx::set_place(place.clone());
                }
                UISearchCtx::set_place_details(self.place_details.clone());
            }
        }

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
            log!("[sync_to_app_state] No filters in URL params");
            UISearchFilters::default()
        } else {
            log!(
                "[sync_to_app_state] Parsing filters from URL: {:?}",
                self.filters
            );
            let parsed_filters = UISearchFilters::from_filter_map(&self.filters);
            log!("[sync_to_app_state] Parsed filters: stars={:?}, price={:?}-{:?}, amenities={:?}, property_types={:?}",
                parsed_filters.min_star_rating,
                parsed_filters.min_price_per_night,
                parsed_filters.max_price_per_night,
                parsed_filters.amenities,
                parsed_filters.property_types
            );
            parsed_filters
        };

        UISearchCtx::set_filters(filters);
        log!("[sync_to_app_state] Filters set in context");

        // Sync sort options from URL
        let sort_options = if !self.sort.is_empty() {
            if let Some((sort_field, _)) = self.sort.first() {
                UISortOptions::from_string(sort_field)
            } else {
                UISortOptions::default_sort()
            }
        } else {
            UISortOptions::default_sort()
        };

        UISearchCtx::set_sort_options(sort_options);
        log!(
            "[sync_to_app_state] Sort options set in context: {:?}",
            self.sort
        );

        // Sync pagination state from URL
        use crate::view_state_layer::ui_search_state::UIPaginationState;

        if let Some(page) = self.page {
            UIPaginationState::set_current_page(page);
            log!("[sync_to_app_state] Set current page to: {}", page);
        } else {
            // Reset to page 1 if not in URL
            UIPaginationState::set_current_page(1);
        }

        if let Some(per_page) = self.per_page {
            UIPaginationState::set_page_size(per_page);
            log!("[sync_to_app_state] Set page size to: {}", per_page);
        } else {
            // Reset to default page size (PAGINATION_LIMIT) if not in URL
            UIPaginationState::set_page_size(crate::api::consts::PAGINATION_LIMIT as u32);
        }
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
    fn test_individual_query_params() {
        let params = HotelListParams {
            place: Some(Place {
                place_id: "ChIJOwg".to_string(),
                display_name: "New York".to_string(),
                formatted_address: "New York, NY, USA".to_string(),
            }),
            checkin: Some("2025-01-15".to_string()),
            checkout: Some("2025-01-20".to_string()),
            adults: Some(2),
            children: Some(1),
            rooms: Some(1),
            children_ages: vec![8],
            ..Default::default()
        };

        let query_params = params.to_query_params();

        assert_eq!(query_params.get("placeId"), Some(&"ChIJOwg".to_string()));
        assert_eq!(query_params.get("checkin"), Some(&"2025-01-15".to_string()));
        assert_eq!(
            query_params.get("checkout"),
            Some(&"2025-01-20".to_string())
        );
        assert_eq!(query_params.get("adults"), Some(&"2".to_string()));
        assert_eq!(query_params.get("children"), Some(&"1".to_string()));
        assert_eq!(query_params.get("rooms"), Some(&"1".to_string()));
        assert_eq!(query_params.get("childAges"), Some(&"8".to_string()));
    }

    #[test]
    fn test_shareable_url() {
        let params = HotelListParams {
            place: Some(Place {
                place_id: "ChIJOwg".to_string(),
                display_name: "NYC".to_string(),
                formatted_address: "New York, NY".to_string(),
            }),
            checkin: Some("2025-01-15".to_string()),
            checkout: Some("2025-01-20".to_string()),
            adults: Some(2),
            ..Default::default()
        };

        let url = params.to_shareable_url();
        assert!(url.starts_with("/hotel-list?"));
        assert!(url.contains("placeId=ChIJOwg"));
        assert!(url.contains("checkin=2025-01-15"));
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
