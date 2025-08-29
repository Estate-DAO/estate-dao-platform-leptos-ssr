use leptos::*;
use reqwest::Method;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use super::client::{ProvabReq, ProvabReqMeta};

cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {
        use crate::api::api_client::ApiClient;
        use crate::api::provab::Provab;
    }
}

use crate::component::{Destination, GuestSelection};
use crate::{component::SelectedDateRange, view_state_layer::ui_search_state::UISearchCtx};
// use leptos::logging::log;
use crate::log;
use std::collections::HashMap;

cfg_if::cfg_if! {
    if #[cfg(feature = "mock-provab")] {
        // fake imports
        use fake::{Dummy, Fake, Faker};
        use rand::rngs::StdRng;
        use rand::SeedableRng;
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct RoomGuest {
    #[serde(rename = "NoOfAdults")]
    pub no_of_adults: u32,
    #[serde(rename = "NoOfChild")]
    pub no_of_child: u32,
    #[serde(rename = "ChildAge", skip_serializing_if = "Option::is_none")]
    pub child_age: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct Price {
    // #[serde(rename = "PublishedPrice")]
    // published_price: f64,
    // #[serde(rename = "PublishedPriceRoundedOff")]
    // published_price_rounded_off: u64,
    // #[serde(rename = "OfferedPrice")]
    // offered_price: f64,
    // #[serde(rename = "OfferedPriceRoundedOff")]
    // offered_price_rounded_off: u64,
    #[serde(rename = "RoomPrice")]
    pub room_price: f64,
    // #[serde(rename = "Tax")]
    // tax: f64,
    // #[serde(rename = "ExtraGuestCharge")]
    // extra_guest_charge: f64,
    // #[serde(rename = "ChildCharge")]
    // child_charge: f64,
    // #[serde(rename = "OtherCharges")]
    // other_charges: f64,
    // #[serde(rename = "Discount")]
    // discount: f64,
    // #[serde(rename = "AgentCommission")]
    // agent_commission: f64,
    // #[serde(rename = "AgentMarkUp")]
    // agent_mark_up: f64,
    // #[serde(rename = "ServiceTax")]
    // service_tax: f64,
    // #[serde(rename = "TDS")]
    // tds: f64,
    // #[serde(rename = "RoomPriceWoGST")]
    // room_price_wo_gst: f64,
    // #[serde(rename = "GSTPrice")]
    // gst_price: f64,
    #[serde(rename = "CurrencyCode")]
    pub currency_code: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HotelResult {
    // #[serde(rename = "ResultIndex")]
    // result_index: i32,
    #[serde(rename = "HotelCode")]
    pub hotel_code: String,
    #[serde(rename = "HotelName")]
    pub hotel_name: String,
    #[serde(rename = "HotelCategory")]
    pub hotel_category: String,
    #[serde(rename = "StarRating")]
    pub star_rating: u8,
    // #[serde(rename = "HotelDescription")]
    // hotel_description: String,
    // #[serde(rename = "HotelPolicy")]
    // hotel_policy: String,
    // #[serde(rename = "HotelPromotionContent")]
    // hotel_promotion_content: String,
    // #[serde(rename = "HotelPromotion")]
    // hotel_promotion: i32,
    #[serde(rename = "Price")]
    pub price: Price,
    #[serde(rename = "HotelPicture")]
    pub hotel_picture: String,
    // #[serde(rename = "ImageOrder")]
    // image_order: i32,
    // #[serde(rename = "HotelAddress")]
    // hotel_address: String,
    // #[serde(rename = "HotelContactNo")]
    // hotel_contact_no: String,
    // #[serde(rename = "HotelMap")]
    // hotel_map: String,
    // #[serde(rename = "Latitude")]
    // latitude: String,
    // #[serde(rename = "Longitude")]
    // longitude: String,
    // #[serde(rename = "HotelLocation")]
    // hotel_location: String,
    // #[serde(rename = "SupplierPrice")]
    // supplier_price: String,
    // #[serde(rename = "RoomDetails")]
    // room_details: Vec<String>,
    #[serde(rename = "ResultToken")]
    pub result_token: String,
    // #[serde(rename = "HotelAmenities")]
    // hotel_amenities: Vec<String>,
    // #[serde(rename = "Free_cancel_date")]
    // free_cancel_date: String,
    // #[serde(rename = "trip_adv_url")]
    // trip_adv_url: String,
    // #[serde(rename = "trip_rating")]
    // trip_rating: f64,
    // #[serde(rename = "trip_reviews", default)]
    // trip_reviews: u64,
    // #[serde(rename = "trip_review_url")]
    // trip_review_url: String,
    // #[serde(rename = "web_reviews_url")]
    // web_reviews_url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct HotelSearchResult {
    #[serde(rename = "HotelResults")]
    pub hotel_results: Vec<HotelResult>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Search {
    #[serde(rename = "HotelSearchResult")]
    pub hotel_search_result: HotelSearchResult,
    // #[serde(rename = "CityId")]
    // city_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct HotelSearchRequest {
    #[serde(rename = "CheckInDate")]
    pub check_in_date: String,
    #[serde(rename = "NoOfNights")]
    pub no_of_nights: u32,
    #[serde(rename = "CountryCode")]
    pub country_code: String,
    #[serde(rename = "CityId")]
    pub city_id: u32,
    #[serde(rename = "GuestNationality")]
    pub guest_nationality: String,
    #[serde(rename = "NoOfRooms")]
    pub no_of_rooms: u32,
    #[serde(rename = "RoomGuests")]
    pub room_guests: Vec<RoomGuest>,
}

// TODO: remove these defaults when going in prod
impl Default for HotelSearchRequest {
    fn default() -> Self {
        Self {
            check_in_date: "11-11-2024".into(),
            no_of_nights: 1,
            country_code: "IN".into(),
            city_id: 1254,
            guest_nationality: "IN".into(),
            no_of_rooms: 1,
            room_guests: vec![RoomGuest {
                no_of_adults: 1,
                no_of_child: 0,
                child_age: None,
            }],
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HotelSearchResponse {
    #[serde(rename = "Status")]
    pub status: i32,
    #[serde(rename = "Message")]
    pub message: String,
    #[serde(rename = "Search")]
    pub search: Option<Search>,
}

impl HotelSearchResponse {
    pub fn get_results_token_map(&self) -> HashMap<String, String> {
        let mut hotel_map = HashMap::new();

        if let Some(search) = self.search.clone() {
            for hotel in search.hotel_search_result.hotel_results {
                hotel_map.insert(hotel.hotel_code, hotel.result_token);
            }
        }

        hotel_map
    }

    pub fn hotel_results(&self) -> Vec<HotelResult> {
        self.search
            .clone()
            .map_or_else(Vec::new, |search| search.hotel_search_result.hotel_results)
    }
}

impl ProvabReq for HotelSearchRequest {
    fn path_suffix() -> &'static str {
        "Search"
    }
}

impl ProvabReqMeta for HotelSearchRequest {
    const METHOD: Method = Method::POST;
    const GZIP: bool = true;
    // const GZIP: bool = false;
    type Response = HotelSearchResponse;
}
