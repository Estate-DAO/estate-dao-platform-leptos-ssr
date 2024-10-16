use reqwest::Method;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use super::{ProvabReq, ProvabReqMeta};

#[derive(Serialize, Deserialize, Debug)]
pub struct HotelDetailsLevel2 {
    checkin: String,
    checkout: String,
    #[serde(rename = "HotelName")]
    hotel_name: String,
    #[serde(rename = "HotelCode")]
    hotel_code: String,
    #[serde(rename = "StarRating")]
    star_rating: i32,
    #[serde(rename = "Description")]
    description: String,
    #[serde(rename = "Attractions")]
    attractions: Vec<String>,
    #[serde(rename = "HotelPolicy")]
    hotel_policy: String,
    #[serde(rename = "HotelFacilities")]
    hotel_facilities: Vec<String>,
    #[serde(rename = "Address")]
    address: String,
    #[serde(rename = "Latitude")]
    latitude: f64,
    #[serde(rename = "Longitude")]
    longitude: f64,
    #[serde(rename = "Images")]
    images: Vec<String>,
    first_room_details: FirstRoomDetails,
    first_rm_cancel_date: String,
    cancel_date: String,
    #[serde(rename = "Amenities")]
    amenities: Vec<String>,
    trip_adv_url: String,
    trip_rating: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FirstRoomDetails {
    #[serde(rename = "Price")]
    price: Price,
    #[serde(rename = "cancellation_policy")]
    cancellation_policy: Vec<CancellationPolicy>,
    #[serde(rename = "room_name")]
    room_name: String,
    #[serde(rename = "Room_data")]
    room_data: RoomData,
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
pub struct CancellationPolicy {
    #[serde(rename = "Charge")]
    charge: f64,
    #[serde(rename = "ChargeType")]
    charge_type: i32,
    #[serde(rename = "Currency")]
    currency: String,
    #[serde(rename = "FromDate")]
    from_date: String,
    #[serde(rename = "ToDate")]
    to_date: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RoomData {
    #[serde(rename = "RoomUniqueId")]
    room_unique_id: String,
    #[serde(rename = "rate_key")]
    rate_key: String,
    #[serde(rename = "group_code")]
    group_code: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HotelDetailsLevel1 {
    #[serde(rename = "HotelInfoResult")]
    hotel_info_result: HotelInfoResult,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HotelInfoResult {
    #[serde(rename = "HotelDetails")]
    hotel_details: HotelDetailsLevel2,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HotelInfoRequest {
    #[serde(rename = "ResultToken")]
    token: String,
}

#[derive(Serialize, Deserialize, Debug)]
// #[display("Status: {}, Message: {}", status, message)]
pub struct HotelInfoResponse {
    #[serde(rename = "Status")]
    status: i32,
    #[serde(rename = "Message")]
    message: String,
    #[serde(rename = "HotelDetails")]
    hotel_details: Option<HotelDetailsLevel1>,
}

impl ProvabReq for HotelInfoRequest {
    fn path_suffix() -> &'static str {
        "HotelDetails"
    }
}

impl ProvabReqMeta for HotelInfoRequest {
    const METHOD: Method = Method::POST;
    type Response = HotelInfoResponse;
}
