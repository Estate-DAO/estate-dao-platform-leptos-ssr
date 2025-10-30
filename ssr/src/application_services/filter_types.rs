use crate::{
    domain::DomainHotelAfterSearch,
    utils::query_params::{ComparisonOp, FilterMap},
};
use serde::{Deserialize, Serialize};

const FILTER_KEY_MIN_STAR_RATING: &str = "min_star_rating";
const FILTER_KEY_MAX_PRICE: &str = "max_price_per_night";
const FILTER_KEY_MIN_PRICE: &str = "min_price_per_night";
const FILTER_KEY_AMENITIES: &str = "amenities";
const FILTER_KEY_PROPERTY_TYPES: &str = "property_types";

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct UISearchFilters {
    pub min_star_rating: Option<u8>,
    pub max_price_per_night: Option<f64>,
    pub min_price_per_night: Option<f64>,
    pub amenities: Option<Vec<String>>, // e.g., ["wifi", "pool", "spa"]
    pub property_types: Option<Vec<String>>, // e.g., ["hotel", "resort", "apartment"]
    pub hotel_name_search: Option<String>, // For searching by hotel name
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UISortOptions {
    pub sort_by: Option<DomainSortField>,
    pub sort_direction: Option<DomainSortDirection>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DomainSortField {
    Price,
    Rating,
    Name,
    // Add other sortable fields as needed
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DomainSortDirection {
    Ascending,
    Descending,
}

impl UISearchFilters {
    // <!-- Helper method to check if any filter is applied -->
    pub fn has_filters(&self) -> bool {
        self.min_star_rating.is_some()
            || self.max_price_per_night.is_some()
            || self.min_price_per_night.is_some()
            || self.amenities.as_ref().is_some_and(|a| !a.is_empty())
            || self.property_types.as_ref().is_some_and(|p| !p.is_empty())
            || self
                .hotel_name_search
                .as_ref()
                .is_some_and(|s| !s.is_empty())
    }

    // <!-- Helper method to get amenities as Vec<String> -->
    pub fn get_amenities(&self) -> Vec<String> {
        self.amenities.clone().unwrap_or_default()
    }

    // <!-- Helper method to get property types as Vec<String> -->
    pub fn get_property_types(&self) -> Vec<String> {
        self.property_types.clone().unwrap_or_default()
    }

    pub fn matches_hotel(&self, hotel: &DomainHotelAfterSearch) -> bool {
        let meets_rating = self
            .min_star_rating
            .map_or(true, |min_rating| hotel.star_rating >= min_rating);

        let room_price = hotel
            .price
            .as_ref()
            .map(|price| price.room_price)
            .filter(|value| value.is_finite() && *value > 0.0);

        let meets_max_price = self.max_price_per_night.map_or(true, |max_price| {
            room_price.map_or(false, |price| price <= max_price)
        });

        let meets_min_price = self.min_price_per_night.map_or(true, |min_price| {
            room_price.map_or(false, |price| price >= min_price)
        });

        let meets_amenities = self.amenities.as_ref().map_or(true, |wanted| {
            if wanted.is_empty() {
                return true;
            }
            let wanted_lc: Vec<String> = wanted.iter().map(|s| s.to_lowercase()).collect();
            let hotel_lc: Vec<String> = hotel.amenities.iter().map(|s| s.to_lowercase()).collect();
            wanted_lc.iter().any(|w| hotel_lc.iter().any(|h| h == w))
        });

        let meets_property_type = self.property_types.as_ref().map_or(true, |wanted_types| {
            if wanted_types.is_empty() {
                return true;
            }
            match &hotel.property_type {
                Some(pt) => {
                    let pt_lc = pt.to_lowercase();
                    wanted_types.iter().any(|t| t.to_lowercase() == pt_lc)
                }
                None => false,
            }
        });

        meets_rating && meets_min_price && meets_max_price && meets_amenities && meets_property_type
    }

    pub fn apply_filters(&self, hotels: &[DomainHotelAfterSearch]) -> Vec<DomainHotelAfterSearch> {
        let mut sorted_hotels: Vec<_> = hotels.to_vec();
        sorted_hotels.sort_by_key(|hotel| {
            match hotel.price.as_ref().map(|price| price.room_price) {
                Some(price) if price > 0.0 => 0,
                _ => 1,
            }
        });

        sorted_hotels
            .into_iter()
            .filter(|hotel| self.matches_hotel(hotel))
            .collect()
    }

    pub fn to_filter_map(&self) -> FilterMap {
        let mut map = FilterMap::new();

        if let Some(min_rating) = self.min_star_rating {
            map.insert(
                FILTER_KEY_MIN_STAR_RATING.to_string(),
                ComparisonOp::Gte(min_rating as f64),
            );
        }

        if let Some(max_price) = self.max_price_per_night {
            map.insert(
                FILTER_KEY_MAX_PRICE.to_string(),
                ComparisonOp::Lte(max_price),
            );
        }

        if let Some(min_price) = self.min_price_per_night {
            map.insert(
                FILTER_KEY_MIN_PRICE.to_string(),
                ComparisonOp::Gte(min_price),
            );
        }

        if let Some(amenities) = &self.amenities {
            if !amenities.is_empty() {
                map.insert(
                    FILTER_KEY_AMENITIES.to_string(),
                    ComparisonOp::In(amenities.clone()),
                );
            }
        }

        if let Some(property_types) = &self.property_types {
            if !property_types.is_empty() {
                map.insert(
                    FILTER_KEY_PROPERTY_TYPES.to_string(),
                    ComparisonOp::In(property_types.clone()),
                );
            }
        }

        // TODO: Map hotel_name_search when supported

        map
    }

    pub fn from_filter_map(map: &FilterMap) -> Self {
        let mut filters = UISearchFilters::default();

        if let Some(op) = map.get(FILTER_KEY_MIN_STAR_RATING) {
            let min_rating = match op {
                ComparisonOp::Eq(value) => value.parse::<u8>().ok(),
                ComparisonOp::Gte(value) | ComparisonOp::Gt(value) => Some((*value).round() as u8),
                _ => None,
            };
            filters.min_star_rating = min_rating;
        }

        if let Some(op) = map.get(FILTER_KEY_MAX_PRICE) {
            let max_price = match op {
                ComparisonOp::Eq(value) => value.parse::<f64>().ok(),
                ComparisonOp::Lte(value)
                | ComparisonOp::Lt(value)
                | ComparisonOp::Gte(value)
                | ComparisonOp::Gt(value) => Some(*value),
                _ => None,
            };
            filters.max_price_per_night = max_price;
        }

        if let Some(op) = map.get(FILTER_KEY_MIN_PRICE) {
            let min_price = match op {
                ComparisonOp::Eq(value) => value.parse::<f64>().ok(),
                ComparisonOp::Gte(value)
                | ComparisonOp::Gt(value)
                | ComparisonOp::Lte(value)
                | ComparisonOp::Lt(value) => Some(*value),
                _ => None,
            };
            filters.min_price_per_night = min_price;
        }

        if let Some(op) = map.get(FILTER_KEY_AMENITIES) {
            let values = match op {
                ComparisonOp::In(v) | ComparisonOp::All(v) => Some(v.clone()),
                _ => None,
            };
            filters.amenities = values;
        }

        if let Some(op) = map.get(FILTER_KEY_PROPERTY_TYPES) {
            let values = match op {
                ComparisonOp::In(v) | ComparisonOp::All(v) => Some(v.clone()),
                _ => None,
            };
            filters.property_types = values;
        }

        filters
    }
}

impl UISortOptions {
    // <!-- Helper method to check if sorting is specified -->
    pub fn has_sort(&self) -> bool {
        self.sort_by.is_some()
    }

    // <!-- Create a UISortOptions for price low to high -->
    pub fn price_low_to_high() -> Self {
        Self {
            sort_by: Some(DomainSortField::Price),
            sort_direction: Some(DomainSortDirection::Ascending),
        }
    }

    // <!-- Create a UISortOptions for price high to low -->
    pub fn price_high_to_low() -> Self {
        Self {
            sort_by: Some(DomainSortField::Price),
            sort_direction: Some(DomainSortDirection::Descending),
        }
    }

    // <!-- Create a UISortOptions for rating high to low -->
    pub fn rating_high_to_low() -> Self {
        Self {
            sort_by: Some(DomainSortField::Rating),
            sort_direction: Some(DomainSortDirection::Descending),
        }
    }
}
