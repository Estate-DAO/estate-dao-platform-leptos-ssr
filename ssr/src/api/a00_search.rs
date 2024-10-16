use leptos::*;
use reqwest::Method;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use super::{ProvabReq, ProvabReqMeta};
use crate::api::Provab;
use crate::{component::SelectedDateRange, state::search_state::SearchCtx};
use leptos::logging::log;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RoomGuest {
    #[serde(rename = "NoOfAdults")]
    no_of_adults: u32,
    #[serde(rename = "NoOfChild")]
    no_of_child: u32,
    #[serde(rename = "ChildAge", skip_serializing_if = "Option::is_none")]
    child_age: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
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
    currency_code: String,
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
pub struct HotelSearchResult {
    #[serde(rename = "HotelResults")]
    hotel_results: Vec<HotelResult>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Search {
    #[serde(rename = "HotelSearchResult")]
    hotel_search_result: HotelSearchResult,
    // #[serde(rename = "CityId")]
    // city_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HotelSearchRequest {
    #[serde(rename = "CheckInDate")]
    check_in_date: String,
    #[serde(rename = "NoOfNights")]
    no_of_nights: u32,
    #[serde(rename = "CountryCode")]
    country_code: String,
    #[serde(rename = "CityId")]
    city_id: u32,
    #[serde(rename = "GuestNationality")]
    guest_nationality: String,
    #[serde(rename = "NoOfRooms")]
    no_of_rooms: u32,
    #[serde(rename = "RoomGuests")]
    room_guests: Vec<RoomGuest>,
}

// TODO: remove these defaults when going in prod
impl Default for HotelSearchRequest {
    fn default() -> Self {
        Self {
            check_in_date: String::new(),
            no_of_nights: 1,
            country_code: "AE".into(),
            city_id: 804,
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
    status: i32,
    #[serde(rename = "Message")]
    message: String,
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
    const GZIP: bool = false;
    type Response = HotelSearchResponse;
}

impl From<SearchCtx> for HotelSearchRequest {
    fn from(ctx: SearchCtx) -> Self {
        // let check_in_date = SelectedDateRange::format_date(ctx.date_range.get().start);
        // let no_of_nights = ctx.date_range.get().no_of_nights();
        HotelSearchRequest {
            check_in_date: "31-10-2024".into(),
            no_of_nights: 1,
            ..Default::default()
        }
    }
}

#[server(SearchHotel, "/search_hotel")]
pub async fn search_hotel(
    request: HotelSearchRequest,
) -> Result<HotelSearchResponse, ServerFnError> {
    log!("SEARCH_HOTEL_API: {request:?}");
    let provab = Provab::default();

    log!("provab_default: {provab:?}");
    match provab.send(request).await {
        Ok(response) => Ok(response),
        Err(e) => {
            log!("server_fn_error: {}", e.to_string());
            Err(ServerFnError::ServerError(e.to_string()))
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_create_hotel_map() {
//         let hotel_results = vec![
//             HotelResult {
//                 hotel_code: "123".to_string(),
//                 hotel_name: "Hotel A".to_string(),
//                 hotel_category: "Luxury".to_string(),
//                 star_rating: 5,
//                 price: Price {
//                     room_price: 200.0,
//                     currency_code: "USD".to_string(),
//                 },
//                 hotel_picture: "url_a".to_string(),
//                 result_token: "token_a".to_string(),
//             },
//             HotelResult {
//                 hotel_code: "456".to_string(),
//                 hotel_name: "Hotel B".to_string(),
//                 hotel_category: "Budget".to_string(),
//                 star_rating: 3,
//                 price: Price {
//                     room_price: 100.0,
//                     currency_code: "USD".to_string(),
//                 },
//                 hotel_picture: "url_b".to_string(),
//                 result_token: "token_b".to_string(),
//             },
//         ];

//         let search = Search {
//             hotel_search_result: HotelSearchResult { hotel_results },
//         };

//         let response = HotelSearchResponse {
//             status: 1,
//             message: "Success".to_string(),
//             search: Some(search),
//         };

//         let hotel_map = response.get_results_token();

//         assert_eq!(hotel_map.get("123"), Some(&"token_a".to_string()));
//         assert_eq!(hotel_map.get("456"), Some(&"token_b".to_string()));
//         assert_eq!(hotel_map.len(), 2);
//     }
// }
