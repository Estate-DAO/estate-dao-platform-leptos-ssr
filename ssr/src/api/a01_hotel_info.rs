use crate::state::search_state::SearchListResults;
use leptos::logging::log;
use leptos::ServerFnError;
use leptos::*;
use reqwest::Method;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use super::{ProvabReq, ProvabReqMeta};
use crate::api::Provab;

#[derive(Serialize, Deserialize, Clone, Debug)]
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
    // #[serde(rename = "Attractions")]
    // attractions: Vec<String>,
    // #[serde(rename = "HotelPolicy")]
    // hotel_policy: String,
    #[serde(rename = "HotelFacilities")]
    hotel_facilities: Vec<String>,
    #[serde(rename = "Address")]
    address: String,
    // #[serde(rename = "Latitude")]
    // latitude: f64,
    // #[serde(rename = "Longitude")]
    // longitude: f64,
    #[serde(rename = "Images")]
    images: Vec<String>,
    first_room_details: FirstRoomDetails,
    // first_rm_cancel_date: String,
    // cancel_date: String,
    #[serde(rename = "Amenities")]
    amenities: Vec<String>,
    // trip_adv_url: String,
    // trip_rating: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FirstRoomDetails {
    #[serde(rename = "Price")]
    price: Price,
    // #[serde(rename = "cancellation_policy")]
    // cancellation_policy: Vec<CancellationPolicy>,
    #[serde(rename = "room_name")]
    room_name: String,
    #[serde(rename = "Room_data")]
    room_data: RoomData,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RoomData {
    #[serde(rename = "RoomUniqueId")]
    room_unique_id: String,
    #[serde(rename = "rate_key")]
    rate_key: String,
    // #[serde(rename = "group_code")]
    // group_code: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HotelDetailsLevel1 {
    #[serde(rename = "HotelInfoResult")]
    hotel_info_result: HotelInfoResult,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HotelInfoResult {
    #[serde(rename = "HotelDetails")]
    hotel_details: HotelDetailsLevel2,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HotelInfoRequest {
    #[serde(rename = "ResultToken")]
    pub token: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
// #[display("Status: {}, Message: {}", status, message)]
pub struct HotelInfoResponse {
    #[serde(rename = "Status")]
    status: i32,
    #[serde(rename = "Message")]
    message: String,
    #[serde(rename = "HotelDetails")]
    hotel_details: Option<HotelDetailsLevel1>,
}

impl HotelInfoResponse {
    pub fn get_address(&self) -> String {
        self.hotel_details.as_ref().map_or_else(
            || "".to_owned(),
            |details| details.hotel_info_result.hotel_details.address.clone(),
        )
    }
    pub fn get_description(&self) -> String {
        self.hotel_details.as_ref().map_or_else(
            || "".to_string(),
            |details| details.hotel_info_result.hotel_details.description.clone(),
        )
    }

    pub fn get_amenities(&self) -> Vec<String> {
        self.hotel_details.as_ref().map_or_else(
            || vec![],
            |details| details.hotel_info_result.hotel_details.amenities.clone(),
        )
    }
    pub fn get_images(&self) -> Vec<String> {
        self.hotel_details.as_ref().map_or_else(
            || vec![],
            |details| details.hotel_info_result.hotel_details.images.clone(),
        )
    }
    pub fn get_hotel_name(&self) -> String {
        self.hotel_details.as_ref().map_or_else(
            || "".to_owned(),
            |details| details.hotel_info_result.hotel_details.hotel_name.clone(),
        )
    }

    pub fn get_star_rating(&self) -> i32 {
        self.hotel_details.as_ref().map_or_else(
            || 0,
            |details| details.hotel_info_result.hotel_details.star_rating,
        )
    }
    pub fn get_room_price(&self) -> f64 {
        self.hotel_details.as_ref().map_or_else(
            || 0.0,
            |details| {
                details
                    .hotel_info_result
                    .hotel_details
                    .first_room_details
                    .price
                    .room_price
            },
        )
    }
}

impl ProvabReq for HotelInfoRequest {
    fn path_suffix() -> &'static str {
        "HotelDetails"
    }
}

impl ProvabReqMeta for HotelInfoRequest {
    const METHOD: Method = Method::POST;
    const GZIP: bool = false;
    type Response = HotelInfoResponse;
}

#[server(HotelInfo, "/hotel_info")]
pub async fn hotel_info(
    request: HotelInfoRequest,
    // hote_code: String
) -> Result<HotelInfoResponse, ServerFnError> {
    // log!("SEARCH_HOTEL_API: {request:?}");
    // let search_list_page: SearchListResults = expect_context();
    // let hotel_code_token = search_list_page.get_hotel_code_results_token_map().get(&hotel_code).unwrap();

    let provab = Provab::default();

    // log!("provab_default: {provab:?}");
    match provab.send(request).await {
        Ok(response) => Ok(response),
        Err(e) => {
            // log!("server_fn_error: {}", e.to_string());
            Err(ServerFnError::ServerError(e.to_string()))
        }
    }
}
