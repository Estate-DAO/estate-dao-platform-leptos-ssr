use crate::hotel_search_types::{DomainCancellationPolicies, DomainRoomOccupancy};
use serde::{Deserialize, Serialize};

/// Tax breakdown item for display
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GroupedTaxItem {
    pub description: String,
    pub amount: f64,
    pub currency_code: String,
    pub included: bool,
}

/// A single bookable room variant within a room type group
/// Represents a specific pricing option (e.g., "Room Only" vs "Breakfast Included")
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainRoomVariant {
    pub offer_id: String,
    pub rate_key: String,
    pub room_name: String, // Added to simplify frontend
    pub mapped_room_id: String,
    pub room_count: u32,
    pub room_unique_id: String,
    pub occupancy_number: Option<u32>,
    pub meal_plan: Option<String>,

    // Price details
    pub total_price_for_all_rooms: f64,
    pub total_price_for_one_room: f64,
    pub price_per_room_excluding_taxes: f64,
    pub currency_code: String,

    // Tax breakdown
    pub tax_breakdown: Vec<GroupedTaxItem>,

    // Other details
    pub occupancy_info: Option<DomainRoomOccupancy>,
    pub cancellation_info: Option<DomainCancellationPolicies>,
}

/// A logical room type grouping (e.g., "Deluxe Room" or "2 x Standard Room")
/// This maps to a single "card" in the UI
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainRoomGroup {
    /// Combined name of the room(s)
    pub name: String,

    /// Mapped room ID if it corresponds to a single static room type
    pub mapped_room_id: Option<String>,

    /// Minimum starting price for this group
    pub min_price: f64,
    pub currency_code: String,

    /// Visual assets from static data
    pub images: Vec<String>,

    /// Amenities from static data
    pub amenities: Vec<String>,

    /// Bed types from static data
    pub bed_types: Vec<String>,

    /// Available variants (rates) for this group, sorted by price
    pub room_types: Vec<DomainRoomVariant>,
}

/// Response structure for the grouped hotel rates API
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainGroupedRoomRates {
    pub room_groups: Vec<DomainRoomGroup>,
}
