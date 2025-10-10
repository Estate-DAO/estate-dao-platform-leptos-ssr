use leptos::*;
use reqwest::Method;
use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize};

use crate::api::api_client::{ApiClient, ApiRequest, ApiRequestMeta};
use crate::api::liteapi::client::LiteApiHTTPClient;
use crate::api::liteapi::traits::LiteApiReq;
use crate::component::{Destination, GuestSelection};
use crate::{api::consts::EnvVarConfig, log};
use crate::{component::SelectedDateRange, view_state_layer::ui_search_state::UISearchCtx};
use reqwest::header::HeaderMap;
use std::collections::HashMap;

cfg_if::cfg_if! {
    if #[cfg(feature = "mock-provab")] {
        use fake::{Dummy, Fake, Faker};
        use rand::rngs::StdRng;
        use rand::SeedableRng;
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct AccessibilityAttributes {
    pub attributes: Option<String>,
    #[serde(rename = "showerChair")]
    pub shower_chair: Option<String>,
    #[serde(rename = "entranceType")]
    pub entrance_type: Option<String>,
    #[serde(rename = "petFriendly")]
    pub pet_friendly: String,
    #[serde(rename = "rampAngle")]
    pub ramp_angle: i32,
    #[serde(rename = "rampLength")]
    pub ramp_length: i32,
    #[serde(rename = "entranceDoorWidth")]
    pub entrance_door_width: i32,
    #[serde(rename = "roomMaxGuestsNumber")]
    pub room_max_guests_number: i32,
    #[serde(rename = "distanceFromTheElevatorToTheAccessibleRoom")]
    pub distance_from_elevator_to_accessible_room: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiHotelResult {
    pub id: String,
    pub name: String,
    #[serde(rename = "hotelDescription")]
    pub hotel_description: String,
    #[serde(rename = "hotelTypeId", skip_serializing_if = "Option::is_none")]
    pub hotel_type_id: Option<i32>,
    // #[serde(rename = "chainId", skip_serializing_if = "Option::is_none")]
    // pub chain_id: Option<i32>,
    // pub chain: String,
    pub currency: String,
    pub country: String,
    pub city: String,
    // pub latitude: f64,
    // pub longitude: f64,
    pub address: String,
    pub zip: String,
    #[serde(rename = "main_photo")]
    pub main_photo: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
    #[serde(deserialize_with = "deserialize_stars")]
    pub stars: i32,
    pub rating: f64,
    #[serde(rename = "reviewCount")]
    pub review_count: i32,
    #[serde(rename = "facilityIds")]
    pub facility_ids: Vec<i32>,
    // #[serde(rename = "accessibilityAttributes")]
    // pub accessibility_attributes: AccessibilityAttributes,
    // #[serde(rename = "deletedAt")]
    // pub deleted_at: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiHotelSearchResponse {
    pub data: Vec<LiteApiHotelResult>,
    #[serde(rename = "hotelIds", deserialize_with = "deserialize_hotel_ids")]
    pub hotel_ids: Vec<String>,
    pub total: i32,
}

impl LiteApiHotelSearchResponse {
    pub fn get_results_token_map(&self) -> HashMap<String, String> {
        let mut hotel_map = HashMap::new();
        for hotel in &self.data {
            // For LiteAPI, we use the hotel ID as both key and token since there's no separate result token
            hotel_map.insert(hotel.id.clone(), hotel.id.clone());
        }
        hotel_map
    }

    pub fn hotel_results(&self) -> Vec<LiteApiHotelResult> {
        self.data.clone()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiHotelSearchRequest {
    // #[serde(rename = "aiSearch")]
    // pub ai_search: String,
    #[serde(rename = "placeId")]
    pub place_id: String,
    // #[serde(rename = "countryCode")]
    // pub country_code: String,
    // #[serde(rename = "cityName")]
    // pub city_name: String,
    pub offset: i32,
    pub limit: i32,
    // #[serde(skip_serializing, rename = "latitude")]
    // pub destination_latitude: Option<f64>,
    // #[serde(skip_serializing, rename = "longitude")]
    // pub destination_longitude: Option<f64>,
    // // radius in meters, min 1000
    // #[serde(skip_serializing, rename = "radius")]
    // pub radius: Option<u32>,
    pub distance: u32,
}

impl Default for LiteApiHotelSearchRequest {
    fn default() -> Self {
        Self {
            // ai_search: "New York".into(),
            // country_code: "US".into(),
            // city_name: "New York".into(),
            offset: 0,
            limit: 50,
            // destination_latitude: Some(40.7),
            // destination_longitude: Some(-74.0),
            place_id: "ChIJOwg_06VPwokRYv534QaPC8g".into(),
            distance: 100000,
            // radius: Some(1000),
        }
    }
}

impl LiteApiHotelSearchRequest {
    pub fn new(place_id: String) -> Self {
        Self {
            place_id,

            distance: 100000,
            // ai_search: destination.city.clone(),
            // country_code: destination.country_code.clone(),
            // city_name: destination.city.clone(),
            offset: 0,
            limit: 50,
            // destination_latitude: destination.latitude,
            // destination_longitude: destination.longitude,
            // radius: Some(1000),
        }
    }
}

impl LiteApiReq for LiteApiHotelSearchRequest {
    fn path_suffix() -> &'static str {
        "data/hotels"
    }
}

impl ApiRequestMeta for LiteApiHotelSearchRequest {
    const METHOD: Method = Method::GET;
    const GZIP: bool = false;
    type Response = LiteApiHotelSearchResponse;
}

impl ApiRequest for LiteApiHotelSearchRequest {
    fn base_path() -> String {
        <Self as LiteApiReq>::base_path()
    }

    fn path_suffix() -> &'static str {
        <Self as LiteApiReq>::path_suffix()
    }

    fn custom_headers() -> HeaderMap {
        <Self as LiteApiReq>::custom_headers()
    }
}

pub async fn liteapi_hotel_search(
    request: LiteApiHotelSearchRequest,
) -> Result<LiteApiHotelSearchResponse, crate::api::ApiError> {
    let client = LiteApiHTTPClient::default();
    client.send(request).await
}

// #[cfg(feature = "ssr")]
pub async fn search_hotels_from_place_id(
    place_id: String,
) -> Result<LiteApiHotelSearchResponse, crate::api::ApiError> {
    let request = LiteApiHotelSearchRequest::new(place_id);
    liteapi_hotel_search(request).await
}

// Custom deserializer for stars field to handle both integer and floating point values
fn deserialize_stars<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de;

    // First try to deserialize as f64 to handle both int and float cases
    let value = f64::deserialize(deserializer)?;
    Ok(value.floor() as i32)
}

// Custom deserializer for hotel_ids field to handle both Vec<String> and empty string
fn deserialize_hotel_ids<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;

    struct HotelIdsVisitor;

    impl<'de> Visitor<'de> for HotelIdsVisitor {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a vector of strings or an empty string")
        }

        fn visit_str<E>(self, value: &str) -> Result<Vec<String>, E>
        where
            E: de::Error,
        {
            if value.is_empty() {
                Ok(vec![])
            } else {
                Ok(vec![value.to_string()])
            }
        }

        fn visit_seq<A>(self, seq: A) -> Result<Vec<String>, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            Vec::deserialize(de::value::SeqAccessDeserializer::new(seq))
        }
    }

    deserializer.deserialize_any(HotelIdsVisitor)
}
