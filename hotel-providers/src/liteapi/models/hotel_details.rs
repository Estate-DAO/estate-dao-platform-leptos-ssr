use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteApiHotelDataResponse {
    pub data: Vec<LiteApiHotelData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteApiHotelData {
    pub id: String,
    pub name: String,
    #[serde(rename = "hotelDescription")]
    pub hotel_description: Option<String>,
    pub currency: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub address: Option<String>,
    pub zip: Option<String>,
    pub main_photo: Option<String>,
    pub thumbnail: Option<String>,
    #[serde(rename = "hotelTypeId")]
    pub hotel_type_id: Option<i32>,
    #[serde(rename = "chainId")]
    pub chain_id: Option<i32>,
    pub chain: Option<String>,
    pub stars: Option<f64>,
    #[serde(rename = "facilityIds")]
    pub facility_ids: Option<Vec<i32>>,
    #[serde(rename = "reviewCount")]
    pub review_count: Option<i32>,
    pub rating: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteApiPlaceResponse {
    pub data: Vec<LiteApiPlace>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteApiPlace {
    #[serde(rename = "placeId")]
    pub place_id: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "formattedAddress")]
    pub formatted_address: Option<String>,
    pub types: Option<Vec<String>>,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteApiHotelSearchRequest {
    #[serde(rename = "placeId", skip_serializing_if = "Option::is_none")]
    pub place_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latitude: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub longitude: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distance: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i32>,
    #[serde(rename = "countryCode", skip_serializing_if = "Option::is_none")]
    pub country_code: Option<String>,
    #[serde(rename = "cityName", skip_serializing_if = "Option::is_none")]
    pub city_name: Option<String>,
    #[serde(rename = "hotelName", skip_serializing_if = "Option::is_none")]
    pub hotel_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteApiHotelSearchResponse {
    pub data: Vec<LiteApiHotelResult>,
    pub total: Option<i32>, // Optional because some endpoints might behave differently? But openapi says integer. I'll make it Option to be safe or verify strictness. Openapi says it's property of response.
    pub place: Option<LiteApiPlaceInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteApiHotelResult {
    pub id: String,
    pub name: String,
    pub currency: String,
    pub country: String,
    pub city: String,
    pub latitude: f64,
    pub longitude: f64,
    pub address: String,
    pub zip: Option<String>,
    #[serde(rename = "main_photo")]
    pub main_photo: String,
    pub thumbnail: Option<String>,
    #[serde(rename = "hotelDescription")]
    pub hotel_description: String,
    pub stars: Option<f64>,
    #[serde(rename = "facilityIds")]
    pub facility_ids: Vec<i32>,
    #[serde(rename = "hotelTypeId")]
    pub hotel_type_id: Option<i32>,
    #[serde(rename = "reviewCount")]
    pub review_count: Option<i32>,
    pub rating: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteApiPlaceInfo {
    #[serde(rename = "placeId")]
    pub place_id: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub location: Option<LiteApiLocation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteApiLocation {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LiteApiHotelImage {
    pub url: String,
    #[serde(rename = "urlHd")]
    pub url_hd: String,
    pub caption: String,
    pub order: i32,
    #[serde(rename = "defaultImage")]
    pub default_image: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LiteApiFacility {
    #[serde(rename = "facilityId")]
    pub facility_id: i32,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LiteApiCheckinCheckoutTimes {
    #[serde(default)]
    pub checkout: String,
    #[serde(default)]
    pub checkin: String,
}

impl Default for LiteApiCheckinCheckoutTimes {
    fn default() -> Self {
        Self {
            checkout: String::new(),
            checkin: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LiteApiRoomAmenity {
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LiteApiRoomPhoto {
    pub url: String,
    #[serde(rename = "mainPhoto")]
    pub main_photo: bool,
    #[serde(rename = "hd_url")]
    pub hd_url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LiteApiRoomView {}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LiteApiPolicy {
    pub id: Option<i32>,
    #[serde(rename = "policy_type", default)]
    pub policy_type: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub child_allowed: String,
    #[serde(default)]
    pub pets_allowed: String,
    #[serde(default)]
    pub parking: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LiteApiCategory {
    pub name: String,
    pub rating: f64,
    #[serde(default)]
    pub description: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct LiteApiSentimentAnalysis {
    #[serde(default)]
    pub categories: Vec<LiteApiCategory>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LiteApiBedType {
    #[serde(default)]
    pub quantity: i32,
    #[serde(rename = "bedType", default)]
    pub bed_type: String,
    #[serde(rename = "bedSize", default)]
    pub bed_size: String,
    pub id: Option<i32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LiteApiRoom {
    pub id: i32,
    #[serde(rename = "roomName")]
    pub room_name: String,
    pub description: String,
    #[serde(rename = "roomSizeSquare")]
    pub room_size_square: Option<f64>,
    #[serde(rename = "roomSizeUnit", default)]
    pub room_size_unit: Option<String>,
    #[serde(rename = "hotelId")]
    pub hotel_id: String,
    #[serde(rename = "maxAdults")]
    pub max_adults: i32,
    #[serde(rename = "maxChildren")]
    pub max_children: i32,
    #[serde(rename = "maxOccupancy")]
    pub max_occupancy: i32,
    #[serde(rename = "bedTypes", default)]
    pub bed_types: Vec<LiteApiBedType>,
    #[serde(rename = "roomAmenities")]
    pub room_amenities: Vec<LiteApiRoomAmenity>,
    pub photos: Vec<LiteApiRoomPhoto>,
    pub views: Vec<LiteApiRoomView>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LiteApiSingleHotelDetailData {
    pub id: String,
    pub name: String,
    #[serde(rename = "hotelDescription", default)]
    pub hotel_description: String,
    #[serde(rename = "hotelImportantInformation", default)]
    pub hotel_important_information: String,
    #[serde(rename = "checkinCheckoutTimes", default)]
    pub checkin_checkout_times: LiteApiCheckinCheckoutTimes,
    #[serde(rename = "hotelImages", default)]
    pub hotel_images: Vec<LiteApiHotelImage>,
    #[serde(default)]
    pub main_photo: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
    pub country: String,
    pub city: String,
    #[serde(rename = "starRating", default)]
    pub star_rating: i32,
    pub location: Option<LiteApiLocation>,
    #[serde(default)]
    pub address: String,
    #[serde(rename = "hotelFacilities", default)]
    pub hotel_facilities: Vec<String>,
    #[serde(default)]
    pub zip: String,
    #[serde(default)]
    pub chain: String,
    #[serde(default)]
    pub facilities: Vec<LiteApiFacility>,
    #[serde(default)]
    pub phone: String,
    #[serde(default)]
    pub email: String,
    #[serde(rename = "hotelType", default)]
    pub hotel_type: String,
    #[serde(rename = "hotelTypeId", default)]
    pub hotel_type_id: i32,
    #[serde(rename = "airportCode", default)]
    pub airport_code: String,
    #[serde(default)]
    pub rating: f64,
    #[serde(rename = "reviewCount", default)]
    pub review_count: i32,
    #[serde(default)]
    pub categories: Vec<LiteApiCategory>,
    #[serde(default)]
    pub sentiment_analysis: Option<LiteApiSentimentAnalysis>,
    #[serde(default)]
    pub policies: Vec<LiteApiPolicy>,
    #[serde(default)]
    pub parking: String,
    #[serde(rename = "groupRoomMin", default)]
    pub group_room_min: i32,
    #[serde(rename = "childAllowed", default)]
    pub child_allowed: bool,
    #[serde(rename = "petsAllowed", default)]
    pub pets_allowed: bool,
    pub rooms: Option<Vec<LiteApiRoom>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LiteApiSingleHotelDetailResponse {
    pub data: LiteApiSingleHotelDetailData,
}
