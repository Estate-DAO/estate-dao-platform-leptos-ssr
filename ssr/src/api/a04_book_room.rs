use super::{ProvabReq, ProvabReqMeta};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
// use leptos::ServerFnError;
use crate::api::Provab;
use leptos::logging::log;
use leptos::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BookRoomRequest {
    #[serde(rename = "ResultToken")]
    pub result_token: String,

    #[serde(rename = "BlockRoomId")]
    pub block_room_id: String,

    #[serde(rename = "AppReference")]
    pub app_reference: String,

    #[serde(rename = "RoomDetails")]
    pub room_details: Vec<RoomDetail>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RoomDetail {
    #[serde(rename = "PassengerDetails")]
    pub passenger_details: Vec<PassengerDetail>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PassengerDetail {
    #[serde(rename = "Title")]
    pub title: String, // Mr,Mrs,Ms possible values

    #[serde(rename = "FirstName")]
    // #[serde(validate(min_length = 2, max_length = 50))]
    pub first_name: String,

    #[serde(rename = "MiddleName")]
    pub middle_name: Option<String>,

    #[serde(rename = "LastName")]
    // #[serde(validate(min_length = 2, max_length = 50))]
    pub last_name: String,

    #[serde(rename = "Email")]
    pub email: String,

    #[serde(rename = "PaxType")]
    pub pax_type: PaxType,

    #[serde(rename = "LeadPassenger", default = "default_true")]
    pub lead_passenger: bool,

    #[serde(rename = "Age")]
    pub children_ages: Option<u32>, // Mandatory only for children, must be <= 18
}

fn default_true() -> bool {
    true
}

#[derive(Serialize_repr, Deserialize_repr, Debug, Clone)]
#[repr(u8)]
pub enum PaxType {
    Adult = 1,
    Child = 2,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BookRoomResponse {
    #[serde(rename = "Status")]
    pub status: BookingStatus,

    #[serde(rename = "Message")]
    pub message: Option<String>,

    #[serde(rename = "CommitBooking", default)]
    pub commit_booking: Vec<BookingDetails>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BookingDetails {
    #[serde(rename = "BookingId")]
    pub booking_id: String,

    #[serde(rename = "BookingRefNo")]
    pub booking_ref_no: String,

    #[serde(rename = "ConfirmationNo")]
    pub confirmation_no: String,

    #[serde(rename = "booking_status")]
    pub booking_status: String,
}

#[derive(Serialize_repr, Deserialize_repr, Debug, Clone)]
#[repr(u8)]
pub enum BookingStatus {
    BookFailed = 0,
    Confirmed = 1,
}

impl ProvabReqMeta for BookRoomRequest {
    const METHOD: Method = Method::POST;
    const GZIP: bool = false;
    type Response = BookRoomResponse;
}
impl ProvabReq for BookRoomRequest {
    fn path_suffix() -> &'static str {
        "CommitBooking"
    }
}

#[server(BlockRoom)]
pub async fn book_room(request: BookRoomRequest) -> Result<BookRoomResponse, ServerFnError> {
    let provab = Provab::default();

    match provab.send(request).await {
        Ok(response) => Ok(response),
        Err(e) => {
            log!("error: {:?}", e);
            Err(ServerFnError::ServerError(e.to_string()))
        }
    }
}
