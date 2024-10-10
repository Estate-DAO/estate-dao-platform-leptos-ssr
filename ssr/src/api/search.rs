
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use reqwest::Method;


use super::{consts::{get_headers_from_env, get_provab_base_url_from_env}, ProvabReq, ProvabReqMeta};

#[derive(Serialize, Deserialize, Debug)]
pub struct RoomGuest {
    #[serde(rename = "NoOfAdults")]
    no_of_adults: u32,
    #[serde(rename = "NoOfChild")]
    no_of_child: u32,
    #[serde(rename = "ChildAge", skip_serializing_if = "Option::is_none")]
    child_age: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Price {
    #[serde(rename = "PublishedPrice")]
    published_price: f64,
    #[serde(rename = "PublishedPriceRoundedOff")]
    published_price_rounded_off: f64,
    #[serde(rename = "OfferedPrice")]
    offered_price: f64,
    #[serde(rename = "OfferedPriceRoundedOff")]
    offered_price_rounded_off: f64,
    #[serde(rename = "RoomPrice")]
    room_price: f64,
    #[serde(rename = "Tax")]
    tax: f64,
    #[serde(rename = "ExtraGuestCharge")]
    extra_guest_charge: f64,
    #[serde(rename = "ChildCharge")]
    child_charge: f64,
    #[serde(rename = "OtherCharges")]
    other_charges: f64,
    #[serde(rename = "Discount")]
    discount: f64,
    #[serde(rename = "AgentCommission")]
    agent_commission: f64,
    #[serde(rename = "AgentMarkUp")]
    agent_mark_up: f64,
    #[serde(rename = "ServiceTax")]
    service_tax: f64,
    #[serde(rename = "TDS")]
    tds: f64,
    #[serde(rename = "RoomPriceWoGST")]
    room_price_wo_gst: f64,
    #[serde(rename = "GSTPrice")]
    gst_price: f64,
    #[serde(rename = "CurrencyCode")]
    currency_code: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HotelResult {
    #[serde(rename = "ResultIndex")]
    result_index: i32,
    #[serde(rename = "HotelCode")]
    hotel_code: String,
    #[serde(rename = "HotelName")]
    hotel_name: String,
    #[serde(rename = "HotelCategory")]
    hotel_category: String,
    #[serde(rename = "StarRating")]
    star_rating: i32,
    #[serde(rename = "HotelDescription")]
    hotel_description: String,
    #[serde(rename = "HotelPolicy")]
    hotel_policy: String,
    #[serde(rename = "HotelPromotionContent")]
    hotel_promotion_content: String,
    #[serde(rename = "HotelPromotion")]
    hotel_promotion: i32,
    #[serde(rename = "Price")]
    price: Price,
    #[serde(rename = "HotelPicture")]
    hotel_picture: String,
    #[serde(rename = "ImageOrder")]
    image_order: i32,
    #[serde(rename = "HotelAddress")]
    hotel_address: String,
    #[serde(rename = "HotelContactNo")]
    hotel_contact_no: String,
    #[serde(rename = "HotelMap")]
    hotel_map: String,
    #[serde(rename = "Latitude")]
    latitude: f64,
    #[serde(rename = "Longitude")]
    longitude: f64,
    #[serde(rename = "HotelLocation")]
    hotel_location: String,
    #[serde(rename = "SupplierPrice")]
    supplier_price: String,
    #[serde(rename = "RoomDetails")]
    room_details: Vec<String>,
    #[serde(rename = "ResultToken")]
    result_token: String,
    #[serde(rename = "HotelAmenities")]
    hotel_amenities: Vec<String>,
    #[serde(rename = "Free_cancel_date")]
    free_cancel_date: String,
    #[serde(rename = "trip_adv_url")]
    trip_adv_url: String,
    #[serde(rename = "trip_rating")]
    trip_rating: String,
    #[serde(rename = "trip_reviews")]
    trip_reviews: String,
    #[serde(rename = "trip_review_url")]
    trip_review_url: String,
    #[serde(rename = "web_reviews_url")]
    web_reviews_url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HotelSearchResult {
    #[serde(rename = "HotelResults")]
    hotel_results: Vec<HotelResult>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Search {
    #[serde(rename = "HotelSearchResult")]
    hotel_search_result: HotelSearchResult,
    #[serde(rename = "CityId")]
    city_id: String,
}



#[derive(Serialize, Deserialize, Debug)]
pub struct SearchRequest {
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



#[derive(Serialize, Deserialize, Debug)]
pub struct SearchResponse {
    #[serde(rename = "Status")]
    status: i32,
    #[serde(rename = "Message")]
    message: String,
    #[serde(rename = "Search")]
    search: Option<Search>,
}



impl ProvabReq for SearchRequest {
    fn path_suffix() -> &'static str {
        "Search"
    }
}

impl ProvabReqMeta for SearchRequest { 
    const METHOD: Method = Method::POST;
    type Response = SearchResponse;
}
