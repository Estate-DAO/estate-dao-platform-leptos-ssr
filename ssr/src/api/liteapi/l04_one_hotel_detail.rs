use leptos::*;
use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::api::api_client::{ApiClient, ApiRequest, ApiRequestMeta};
use crate::api::liteapi::client::LiteApiHTTPClient;
use crate::api::liteapi::traits::LiteApiReq;
use reqwest::header::HeaderMap;

cfg_if::cfg_if! {
    if #[cfg(feature = "mock-provab")] {
        use fake::{Dummy, Fake, Faker};
        use rand::rngs::StdRng;
        use rand::SeedableRng;
    }
}

// Hotel details structures based on API documentation
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
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
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiLocation {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiFacility {
    #[serde(rename = "facilityId")]
    pub facility_id: i32,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
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
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiRoomAmenity {
    // #[serde(rename = "amenitiesId")]
    // pub amenities_id: i32,
    pub name: String,
    // pub sort: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiRoomPhoto {
    pub url: String,
    #[serde(rename = "mainPhoto")]
    pub main_photo: bool,
    #[serde(rename = "hd_url")]
    pub hd_url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiRoomView {
    // Add view-related fields if needed
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiBedType {
    pub quantity: i32,
    #[serde(rename = "bedType")]
    pub bed_type: String,
    #[serde(rename = "bedSize")]
    pub bed_size: String,
    pub id: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
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
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiSingleHotelDetailData {
    pub id: String,
    pub name: String,
    #[serde(rename = "hotelDescription", default)]
    pub hotel_description: String,
    #[serde(rename = "hotelImportantInformation", default)]
    pub hotel_important_information: String,
    #[serde(rename = "checkinCheckoutTimes", default)]
    pub checkin_checkout_times: LiteApiCheckinCheckoutTimes,
    // for_claude_hint: hotel_images is not received in the final api response because of some reason
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
    // pub fax: String,
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
    pub parking: String,
    #[serde(rename = "groupRoomMin", default)]
    pub group_room_min: i32,
    #[serde(rename = "childAllowed", default)]
    pub child_allowed: bool,
    #[serde(rename = "petsAllowed", default)]
    pub pets_allowed: bool,
    pub rooms: Option<Vec<LiteApiRoom>>,
    // Simplified policies and sentiment_analysis fields for now
    // Can be expanded later if needed
}

impl LiteApiSingleHotelDetailData {
    // Populate main_photo from other image fields if it's empty
    pub fn populate_main_photo_if_empty(&mut self) {
        if self.main_photo.trim().is_empty() {
            // Try to use thumbnail first
            if let Some(ref thumbnail) = self.thumbnail {
                if !thumbnail.trim().is_empty() {
                    self.main_photo = thumbnail.clone();
                    return;
                }
            }

            // Try to use the first hotel image with default_image = true
            if let Some(default_image) = self
                .hotel_images
                .iter()
                .find(|img| img.default_image && !img.url.trim().is_empty())
            {
                self.main_photo = default_image.url.clone();
                return;
            }

            // Try to use the first hotel image with HD URL
            if let Some(first_hd_image) = self
                .hotel_images
                .iter()
                .find(|img| !img.url_hd.trim().is_empty())
            {
                self.main_photo = first_hd_image.url_hd.clone();
                return;
            }

            // Try to use the first hotel image with regular URL
            if let Some(first_image) = self
                .hotel_images
                .iter()
                .find(|img| !img.url.trim().is_empty())
            {
                self.main_photo = first_image.url.clone();
                return;
            }

            // As a last resort, try to use room photos (main photo from rooms)
            while let Some(room) = &self.rooms {
                if let Some(room_main_photo) = room
                    .iter()
                    .map(|f| {
                        f.photos
                            .iter()
                            .find(|photo| photo.main_photo && !photo.url.trim().is_empty())
                    })
                    .flatten()
                    .next()
                {
                    self.main_photo = room_main_photo.url.clone();
                    return;
                }
            }

            // If no main room photo, try any room photo with HD URL
            while let Some(room) = &self.rooms {
                if let Some(room_photo) = room
                    .iter()
                    .map(|f| {
                        f.photos
                            .iter()
                            .find(|photo| !photo.hd_url.trim().is_empty())
                    })
                    .flatten()
                    .next()
                {
                    self.main_photo = room_photo.hd_url.clone();
                    return;
                }
            }

            // If no HD room photo, try any room photo with regular URL
            while let Some(room) = &self.rooms {
                if let Some(room_photo) = room
                    .iter()
                    .map(|f| f.photos.iter().find(|photo| !photo.url.trim().is_empty()))
                    .flatten()
                    .next()
                {
                    self.main_photo = room_photo.url.clone();
                    return;
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiSingleHotelDetailResponse {
    pub data: LiteApiSingleHotelDetailData,
}

// Request structure for hotel details - just needs hotel ID
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiSingleHotelDetailRequest {
    #[serde(rename = "hotelId")]
    pub hotel_id: String,
}

impl LiteApiReq for LiteApiSingleHotelDetailRequest {
    fn path_suffix() -> &'static str {
        // get details of a single hotel from hotel_id
        "data/hotel"
    }
}

impl ApiRequestMeta for LiteApiSingleHotelDetailRequest {
    const METHOD: Method = Method::GET;
    const GZIP: bool = false; // Set to false to match rates API
    type Response = LiteApiSingleHotelDetailResponse;
}

impl ApiRequest for LiteApiSingleHotelDetailRequest {
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

impl LiteApiSingleHotelDetailRequest {
    pub fn new(hotel_id: String) -> Self {
        Self { hotel_id }
    }

    // Custom path method that includes the hotel ID
    pub fn path_with_id(&self) -> String {
        format!(
            "{}/{}/{}",
            <LiteApiSingleHotelDetailRequest as LiteApiReq>::base_path(),
            <LiteApiSingleHotelDetailRequest as LiteApiReq>::path_suffix(),
            self.hotel_id
        )
    }
}

pub async fn liteapi_hotel_details(
    request: LiteApiSingleHotelDetailRequest,
) -> Result<LiteApiSingleHotelDetailResponse, crate::api::ApiError> {
    let client = LiteApiHTTPClient::default();
    client.send(request).await
}
