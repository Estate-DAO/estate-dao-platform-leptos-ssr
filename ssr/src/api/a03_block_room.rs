use super::{ProvabReq, ProvabReqMeta};
use crate::api::Provab;
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

macro_rules! delegate_method {
    ($enum_var:expr, $method:ident) => {
        match $enum_var {
            BlockRoomResponse::Success(success_response) => success_response.$method(),
            BlockRoomResponse::Failure(failure_response) => failure_response.$method(),
        }
    };
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct BlockRoomRequest {
    #[serde(rename = "ResultToken")]
    pub token: String,
    #[serde(rename = "RoomUniqueId")]
    pub room_unique_id: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct BlockRoomResult {
    #[serde(rename = "IsPriceChanged")]
    pub is_price_changed: bool,

    #[serde(rename = "IsCancellationPolicyChanged")]
    pub is_cancellation_policy_changed: bool,

    #[serde(rename = "BlockRoomId")]
    pub block_room_id: String,

    #[serde(rename = "HotelRoomsDetails")]
    pub hotel_rooms_details: Vec<HotelRoomDetail>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct SuccessBlockRoomResponse {
    #[serde(rename = "Status")]
    pub status: u32,

    #[serde(rename = "Message")]
    pub message: Option<String>,

    #[serde(rename = "BlockRoom")]
    pub block_room: BlockRoomContainer,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct FailureBlockRoomResponse {
    #[serde(rename = "Status")]
    pub status: u32,

    #[serde(rename = "Message")]
    pub message: Option<String>,
}

impl Default for FailureBlockRoomResponse {
    fn default() -> Self {
        // Provide default values for the fields of FailureBlockRoomResponse
        FailureBlockRoomResponse {
            status: 0,
            message: Some("generated_from_ impl Default ".into()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
#[serde(untagged)]
pub enum BlockRoomResponse {
    Success(SuccessBlockRoomResponse),
    Failure(FailureBlockRoomResponse),
}

impl Default for BlockRoomResponse {
    fn default() -> Self {
        BlockRoomResponse::Failure(FailureBlockRoomResponse::default())
    }
}

impl SuccessBlockRoomResponse {
    pub fn get_block_room_hotel_details(&self) -> Vec<HotelRoomDetail> {
        self.block_room
            .block_room_result
            .hotel_rooms_details
            .clone()
    }

    pub fn get_block_room_id(&self) -> Option<String> {
        Some(self.block_room.block_room_result.block_room_id.clone())
    }
}

impl FailureBlockRoomResponse {
    pub fn get_block_room_hotel_details(&self) -> Vec<HotelRoomDetail> {
        Vec::new()
    }

    pub fn get_block_room_id(&self) -> Option<String> {
        None
    }
}

impl BlockRoomResponse {
    pub fn get_block_room_hotel_details(&self) -> Vec<HotelRoomDetail> {
        // match self {
        //     BlockRoomResponse::Success(success_response) => {
        //         success_response.get_block_room_hotel_details()
        //     }
        //     BlockRoomResponse::Failure(failre_response) => {
        //         failre_response.get_block_room_hotel_details()
        //     }
        // }
        delegate_method!(self, get_block_room_hotel_details)
    }

    pub fn get_room_price_summed(&self) -> f64 {
        self.get_block_room_hotel_details()
            .iter()
            .map(|detail| detail.get_offered_price())
            .sum()
    }

    pub fn get_block_room_id(&self) -> Option<String> {
        delegate_method!(self, get_block_room_id)
    }
}

impl HotelRoomDetail {
    pub fn get_offered_price(&self) -> f64 {
        self.price.offered_price
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct BlockRoomContainer {
    #[serde(rename = "BlockRoomResult")]
    pub block_room_result: BlockRoomResult,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct HotelRoomDetail {
    // #[serde(rename = "RoomUniqueId")]
    // pub room_unique_id: String,
    pub room_code: String,

    #[serde(rename = "HotelCode")]
    pub hotel_code: String,

    // #[serde(rename = "RoomIndex")]
    // pub room_index: i32,

    // #[serde(rename = "RatePlanCode")]
    // pub rate_plan_code: Option<String>,
    #[serde(rename = "RoomTypeCode")]
    pub room_type_code: Option<String>,

    #[serde(rename = "RoomTypeName")]
    pub room_type_name: String,

    #[serde(rename = "Price")]
    pub price: Price,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct Price {
    // #[serde(rename = "PublishedPrice")]
    // published_price: f64,
    // #[serde(rename = "PublishedPriceRoundedOff")]
    // published_price_rounded_off: i32,
    #[serde(rename = "OfferedPrice")]
    offered_price: f64,
    // #[serde(rename = "OfferedPriceRoundedOff")]
    // offered_price_rounded_off: i32,
    // #[serde(rename = "RoomPrice")]
    // room_price: f64,
    // #[serde(rename = "Tax")]
    // tax: i32,
    // #[serde(rename = "ExtraGuestCharge")]
    // extra_guest_charge: f64,
    // #[serde(rename = "ChildCharge")]
    // child_charge: f64,
    // #[serde(rename = "OtherCharges")]
    // other_charges: f64,
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
    const GZIP: bool = true;
    type Response = BlockRoomResponse;
}

#[server(BlockRoom)]
pub async fn block_room(request: BlockRoomRequest) -> Result<BlockRoomResponse, ServerFnError> {
    let retry_count = 3;

    use crate::api::RetryableRequest;
    request.retry_with_backoff(retry_count).await
}
