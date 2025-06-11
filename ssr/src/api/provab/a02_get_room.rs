// use leptos::logging::log;
use crate::log;
use leptos::ServerFnError;
use leptos::*;
use reqwest::Method;

use serde::{Deserialize, Serialize};

cfg_if::cfg_if! {
    if #[cfg(feature = "mock-provab")] {
        // fake imports
        use fake::{Dummy, Fake, Faker};
        use rand::rngs::StdRng;
        use rand::SeedableRng;
    }
}

use super::client::{ProvabReq, ProvabReqMeta};

cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {
        use crate::api::api_client::ApiClient;
        use crate::api::provab::Provab;
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct HotelRoomRequest {
    #[serde(rename = "ResultToken")]
    pub token: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct RoomList {
    #[serde(rename = "GetHotelRoomResult")]
    pub get_hotel_room_result: GetHotelRoomResult,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct GetHotelRoomResult {
    #[serde(rename = "HotelRoomsDetails")]
    pub hotel_rooms_details: Vec<HotelRoomDetail>,
    #[serde(rename = "RoomCombinations")]
    pub room_combinations: RoomCombinations,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct HotelRoomDetail {
    #[serde(rename = "RoomUniqueId")]
    pub room_unique_id: String,
    // rate_key: String,
    // group_code: String,
    // room_code: String,
    #[serde(rename = "ChildCount")]
    child_count: u32,
    #[serde(rename = "RoomTypeName")]
    pub room_type_name: String,
    #[serde(rename = "Price")]
    pub price: Price,

    // #[serde(rename = "SmokingPreference")]
    // smoking_preference: String,

    // #[serde(rename = "RatePlanCode")]
    // rate_plan_code: String,
    #[serde(rename = "RoomTypeCode")]
    pub room_type_code: String,

    // #[serde(rename = "RatePlanName")]
    // rate_plan_name: String,

    // #[serde(rename = "Amenities")]
    // amenities: Vec<String>,

    // #[serde(rename = "OtherAmennities")]
    // other_amennities: Vec<String>,
    pub room_only: String,

    // #[serde(rename = "LastCancellationDate")]
    // last_cancellation_date: String,

    // #[serde(rename = "CancellationPolicies")]
    // cancellation_policies: Vec<CancellationPolicy>,

    // #[serde(rename = "CancellationPolicy")]
    // cancellation_policy: String,

    // todo: caution: expected String, found integer

    // #[serde(rename = "HOTEL_CODE")]
    // hotel_code: String,

    // #[serde(rename = "SEARCH_ID")]
    // search_id: String,
    #[serde(rename = "RoomIndex")]
    room_index: u32,
    // #[serde(rename = "InfoSource")]
    // info_source: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]

pub struct Price {
    // #[serde(rename = "PublishedPrice")]
    // published_price: f64,

    // #[serde(rename = "PublishedPriceRoundedOff")]
    // published_price_rounded_off: i32,
    #[serde(rename = "OfferedPrice")]
    pub offered_price: f64,

    // #[serde(rename = "OfferedPriceRoundedOff")]
    // offered_price_rounded_off: i32,
    #[serde(rename = "RoomPrice")]
    pub room_price: f32,

    // #[serde(rename = "Tax")]
    // tax: i32,
    #[serde(rename = "ExtraGuestCharge")]
    pub extra_guest_charge: f32,
    #[serde(rename = "ChildCharge")]
    pub child_charge: f32,
    #[serde(rename = "OtherCharges")]
    pub other_charges: f32,

    // #[serde(rename = "Discount")]
    // discount: i32,

    // #[serde(rename = "AgentCommission")]
    // agent_commission: i32,

    // #[serde(rename = "AgentMarkUp")]
    // agent_mark_up: i32,

    // #[serde(rename = "ServiceTax")]
    // service_tax: i32,

    // #[serde(rename = "TDS")]
    // tds: i32,

    // #[serde(rename = "RoomPriceWoGST")]
    // room_price_wo_gst: i32,

    // #[serde(rename = "GSTPrice")]
    // gst_price: i32,
    #[serde(rename = "CurrencyCode")]
    pub currency_code: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]

pub struct CancellationPolicy {
    #[serde(rename = "Charge")]
    charge: f32,
    #[serde(rename = "ChargeType")]
    charge_type: f32,
    #[serde(rename = "Currency")]
    currency: String,
    #[serde(rename = "FromDate")]
    from_date: String,
    #[serde(rename = "ToDate")]
    to_date: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct RoomCombinations {
    #[serde(rename = "InfoSource")]
    info_source: String,
    #[serde(rename = "IsPolicyPerStay")]
    is_policy_per_stay: bool,
    #[serde(rename = "RoomCombination")]
    room_combination: Vec<RoomCombination>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct RoomCombination {
    #[serde(rename = "RoomIndex")]
    room_index: Vec<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HotelRoomResponse {
    #[serde(rename = "Status")]
    pub status: u32,
    #[serde(rename = "Message")]
    pub message: String,
    #[serde(rename = "RoomList")]
    pub room_list: Option<RoomList>,
}

impl HotelRoomResponse {
    pub fn get_hotel_room_details(&self) -> Option<Vec<HotelRoomDetail>> {
        self.room_list
            .as_ref()
            .map(|room_list| room_list.get_hotel_room_result.hotel_rooms_details.clone())
    }

    pub fn get_room_unique_ids(&self) -> Vec<String> {
        self.room_list
            .as_ref()
            .map(|room_list| {
                room_list
                    .get_hotel_room_result
                    .hotel_rooms_details
                    .iter()
                    .map(|room| room.room_unique_id.clone())
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl ProvabReq for HotelRoomRequest {
    fn path_suffix() -> &'static str {
        "RoomList"
    }
}

impl ProvabReqMeta for HotelRoomRequest {
    const METHOD: Method = Method::POST;
    const GZIP: bool = true;
    type Response = HotelRoomResponse;
}

// #[server(GetRoom)]
// pub async fn get_room(request: HotelRoomRequest) -> Result<HotelRoomResponse, ServerFnError> {
//     use crate::api::provab::retry::RetryableRequest;
//     request.retry_with_backoff(3).await
// }
