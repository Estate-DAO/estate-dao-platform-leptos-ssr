use super::{ProvabReq, ProvabReqMeta};
use crate::api::Provab;
use leptos::logging::log;
use leptos::ServerFnError;
use leptos::*;
use reqwest::Method;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockRoomRequest {
    #[serde(rename = "ResultToken")]
    pub token: String,
    #[serde(rename = "RoomUniqueId")]
    pub room_unique_id: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockRoomResponse {
    #[serde(rename = "IsPriceChanged")]
    pub is_price_changed: bool,

    #[serde(rename = "IsCancellationPolicyChanged")]
    pub is_cancellation_policy_changed: bool,

    #[serde(rename = "HotelRoomDetails")]
    pub hotel_room_details: Vec<HotelRoomDetail>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HotelRoomDetail {
    #[serde(rename = "RoomUniqueId")]
    pub room_unique_id: String,

    pub room_code: String,

    #[serde(rename = "RoomIndex")]
    pub room_index: i32,

    #[serde(rename = "RatePlanCode")]
    pub rate_plan_code: Option<String>,

    #[serde(rename = "RoomTypeCode")]
    pub room_type_code: Option<String>,

    #[serde(rename = "RoomTypeName")]
    pub room_type_name: String,

    #[serde(rename = "Price")]
    pub price: Price,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Price {
    // #[serde(rename = "PublishedPrice")]
    // published_price: f64,
    // #[serde(rename = "PublishedPriceRoundedOff")]
    // published_price_rounded_off: i32,
    // #[serde(rename = "OfferedPrice")]
    // offered_price: f64,
    // #[serde(rename = "OfferedPriceRoundedOff")]
    // offered_price_rounded_off: i32,
    #[serde(rename = "RoomPrice")]
    room_price: i32,
    // #[serde(rename = "Tax")]
    // tax: i32,
    #[serde(rename = "ExtraGuestCharge")]
    extra_guest_charge: i32,
    #[serde(rename = "ChildCharge")]
    child_charge: i32,
    #[serde(rename = "OtherCharges")]
    other_charges: i32,
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
    currency_code: String,
}

impl ProvabReq for BlockRoomRequest {
    fn path_suffix() -> &'static str {
        "BlockRoom"
    }
}

impl ProvabReqMeta for BlockRoomRequest {
    const METHOD: Method = Method::POST;
    const GZIP: bool = false;
    type Response = BlockRoomResponse;
}

#[server(BlockRoom, "/block_room")]
pub async fn block_room(request: BlockRoomRequest) -> Result<BlockRoomResponse, ServerFnError> {
    let provab = Provab::default();

    match provab.send(request).await {
        Ok(response) => Ok(response),
        Err(e) => {
            log!("error: {:?}", e);
            Err(ServerFnError::ServerError(e.to_string()))
        }
    }
}
