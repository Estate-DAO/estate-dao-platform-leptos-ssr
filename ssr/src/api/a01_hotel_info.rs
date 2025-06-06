use crate::api::Provab;
// use leptos::logging::log;
use crate::api::api_client::ApiClient;
use crate::log;
use leptos::ServerFnError;
use leptos::*;
use reqwest::Method;
use serde::{Deserialize, Deserializer, Serialize};

use super::{ProvabReq, ProvabReqMeta};

cfg_if::cfg_if! {
    if #[cfg(feature = "mock-provab")] {
        // fake imports
        use fake::{Dummy, Fake, Faker};
        use rand::rngs::StdRng;
        use rand::SeedableRng;
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HotelDetailsLevel2 {
    pub checkin: String,
    pub checkout: String,
    #[serde(rename = "HotelName")]
    pub hotel_name: String,
    #[serde(rename = "HotelCode")]
    pub hotel_code: String,
    #[serde(rename = "StarRating")]
    pub star_rating: i32,
    #[serde(rename = "Description")]
    pub description: String,
    // #[serde(rename = "Attractions")]
    // attractions: Vec<String>,
    // #[serde(rename = "HotelPolicy")]
    // hotel_policy: String,
    #[serde(rename = "HotelFacilities")]
    #[serde(deserialize_with = "deserialize_optional_string_vec")]
    pub hotel_facilities: Vec<String>,
    #[serde(rename = "Address")]
    pub address: String,
    // #[serde(rename = "Latitude")]
    // latitude: f64,
    // #[serde(rename = "Longitude")]
    // longitude: f64,
    #[serde(rename = "Images")]
    pub images: Vec<String>,
    pub first_room_details: FirstRoomDetails,
    // first_rm_cancel_date: String,
    // cancel_date: String,
    #[serde(rename = "Amenities")]
    #[serde(deserialize_with = "deserialize_optional_string_vec")]
    pub amenities: Vec<String>,
    // trip_adv_url: String,
    // trip_rating: String,
}

/// this is used to deserialize
// [null] or []
fn deserialize_optional_string_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt_vec: Option<Vec<Option<String>>> = Deserialize::deserialize(deserializer)?;

    Ok(opt_vec
        .unwrap_or_default()
        .into_iter()
        .filter_map(|item| item)
        .collect())
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
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
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
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
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
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
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct RoomData {
    #[serde(rename = "RoomUniqueId")]
    room_unique_id: String,
    #[serde(rename = "rate_key")]
    rate_key: String,
    // #[serde(rename = "group_code")]
    // group_code: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct HotelDetailsLevel1 {
    #[serde(rename = "HotelInfoResult")]
    hotel_info_result: HotelInfoResult,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct HotelInfoResult {
    #[serde(rename = "HotelDetails")]
    hotel_details: HotelDetailsLevel2,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct HotelInfoRequest {
    #[serde(rename = "ResultToken")]
    pub token: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
// #[display("Status: {}, Message: {}", status, message)]
pub struct HotelInfoResponse {
    #[serde(rename = "Status")]
    pub status: i32,
    #[serde(rename = "Message")]
    pub message: String,
    #[serde(rename = "HotelDetails")]
    pub hotel_details: Option<HotelDetailsLevel1>,
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
    pub fn get_location(&self) -> String {
        self.hotel_details.as_ref().map_or_else(
            || "".to_owned(),
            |details| details.hotel_info_result.hotel_details.address.clone(),
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
    const GZIP: bool = true;
    type Response = HotelInfoResponse;
}

#[server(HotelInfo)]
pub async fn hotel_info(
    request: HotelInfoRequest,
    // hote_code: String
) -> Result<HotelInfoResponse, ServerFnError> {
    // log!("SEARCH_HOTEL_API: {request:?}");
    // let search_list_page: SearchListResults = expect_context();
    // let hotel_code_token = search_list_page.get_hotel_code_results_token_map().get(&hotel_code).unwrap();

    // let provab = Provab::default();
    let provab: Provab = expect_context();

    // log!("provab_default: {provab:?}");
    match provab.send(request).await {
        Ok(response) => Ok(response),
        Err(e) => {
            // log!("server_fn_error: {}", e.to_string());
            Err(ServerFnError::ServerError(e.to_string()))
        }
    }
}
