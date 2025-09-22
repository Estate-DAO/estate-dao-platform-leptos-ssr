use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
