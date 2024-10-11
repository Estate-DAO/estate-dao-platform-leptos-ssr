
use serde::{Deserialize, Serialize};
use reqwest::Method;
use super::{consts::{get_headers_from_env, get_provab_base_url_from_env}, ProvabReq, ProvabReqMeta};



#[derive(Serialize, Deserialize, Debug)]
pub struct HotelRoomRequest {
    #[serde(rename = "ResultToken")]
    token: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RoomList {
    #[serde(rename = "GetHotelRoomResult")]
    get_hotel_room_result: GetHotelRoomResult,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetHotelRoomResult {
    #[serde(rename = "HotelRoomsDetails")]
    hotel_rooms_details: Vec<HotelRoomDetail>,
    #[serde(rename = "RoomCombinations")]
    room_combinations: RoomCombinations,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HotelRoomDetail {
    #[serde(rename = "RoomUniqueId")]
    room_unique_id: String,
    rate_key: String,
    group_code: String,
    room_code: String,
    #[serde(rename = "ChildCount")]
    child_count: i32,
    #[serde(rename = "RoomTypeName")]
    room_type_name: String,
    #[serde(rename = "Price")]
    price: Price,
    #[serde(rename = "SmokingPreference")]
    smoking_preference: String,
    #[serde(rename = "RatePlanCode")]
    rate_plan_code: String,
    #[serde(rename = "RoomTypeCode")]
    room_type_code: String,
    #[serde(rename = "RatePlanName")]
    rate_plan_name: String,
    #[serde(rename = "Amenities")]
    amenities: Vec<String>,
    #[serde(rename = "OtherAmennities")]
    other_amennities: Vec<String>,
    room_only: String,
    #[serde(rename = "LastCancellationDate")]
    last_cancellation_date: String,
    #[serde(rename = "CancellationPolicies")]
    cancellation_policies: Vec<CancellationPolicy>,
    #[serde(rename = "CancellationPolicy")]
    cancellation_policy: String,
    #[serde(rename = "HOTEL_CODE")]
    hotel_code: String,
    #[serde(rename = "SEARCH_ID")]
    search_id: String,
    #[serde(rename = "RoomIndex")]
    room_index: i32,
    #[serde(rename = "InfoSource")]
    info_source: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Price {
    #[serde(rename = "PublishedPrice")]
    published_price: f64,
    #[serde(rename = "PublishedPriceRoundedOff")]
    published_price_rounded_off: i32,
    #[serde(rename = "OfferedPrice")]
    offered_price: f64,
    #[serde(rename = "OfferedPriceRoundedOff")]
    offered_price_rounded_off: i32,
    #[serde(rename = "RoomPrice")]
    room_price: i32,
    #[serde(rename = "Tax")]
    tax: i32,
    #[serde(rename = "ExtraGuestCharge")]
    extra_guest_charge: i32,
    #[serde(rename = "ChildCharge")]
    child_charge: i32,
    #[serde(rename = "OtherCharges")]
    other_charges: i32,
    #[serde(rename = "Discount")]
    discount: i32,
    #[serde(rename = "AgentCommission")]
    agent_commission: i32,
    #[serde(rename = "AgentMarkUp")]
    agent_mark_up: i32,
    #[serde(rename = "ServiceTax")]
    service_tax: i32,
    #[serde(rename = "TDS")]
    tds: i32,
    #[serde(rename = "RoomPriceWoGST")]
    room_price_wo_gst: i32,
    #[serde(rename = "GSTPrice")]
    gst_price: i32,
    #[serde(rename = "CurrencyCode")]
    currency_code: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CancellationPolicy {
    #[serde(rename = "Charge")]
    charge: i32,
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
pub struct RoomCombinations {
    #[serde(rename = "InfoSource")]
    info_source: String,
    #[serde(rename = "IsPolicyPerStay")]
    is_policy_per_stay: bool,
    #[serde(rename = "RoomCombination")]
    room_combination: Vec<RoomCombination>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RoomCombination {
    #[serde(rename = "RoomIndex")]
    room_index: Vec<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HotelRoomResponse {
    #[serde(rename = "Status")]
    status: i32,
    #[serde(rename = "Message")]
    message: String,
    #[serde(rename = "RoomList")]
    room_list: Option<RoomList>,
}


impl ProvabReq for HotelRoomRequest {
    fn path_suffix() -> &'static str {
        "RoomList"
    }
}

impl ProvabReqMeta for HotelRoomRequest { 
    const METHOD: Method = Method::POST;
    type Response = HotelRoomResponse;
}
