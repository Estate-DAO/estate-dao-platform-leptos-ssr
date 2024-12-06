use super::{ProvabReq, ProvabReqMeta};
use reqwest::Method;
use serde::{Deserialize, Serialize};
// use leptos::ServerFnError;
use crate::api::Provab;
use crate::canister::backend::{AdultDetail, Booking, UserDetails};
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

#[derive(Deserialize, Default, Debug, Clone)]
/// customer serializer is implemented to ensure validation
/// and override the serde's default behaviour around None values
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

    #[serde(rename = "Age", default = "_default_passenger_age")]
    pub age: u32, //for children, must be <= 18
}

// impl PartialEq for AdultDetail{
//     fn eq(&self, other: &Self) -> bool {
//         self.email == other.email &&
//         self.first_name == other.first_name &&
//         self.last_name == other.last_name &&
//         self.phone == other.phone &&
//         self.age == other.age
//     }
// }

impl From<&UserDetails> for Vec<PassengerDetail> {
    fn from(user_details: &UserDetails) -> Self {
        let mut passenger_details = Vec::new();

        // Process adults
        for (index, adult) in user_details.adults.iter().enumerate() {
            passenger_details.push(PassengerDetail {
                title: "Mr".to_string(),
                first_name: adult.first_name.clone(),
                middle_name: None,
                last_name: adult.last_name.clone().unwrap_or_default(),
                email: adult.email.clone().unwrap_or_default(),
                pax_type: PaxType::Adult,
                lead_passenger: index == 0,
                age: 25,
            });
        }

        // Process children
        for (index, child) in user_details.children.iter().enumerate() {
            passenger_details.push(PassengerDetail {
                title: "Mr".to_string(),
                first_name: child.first_name.clone(),
                middle_name: None,
                last_name: child.last_name.clone().unwrap_or_default(),
                email: "".to_string(),
                pax_type: PaxType::Child,
                lead_passenger: false,
                age: child.age as u32,
            });
        }

        passenger_details
    }
}

pub fn _default_passenger_age() -> u32 {
    25
}

fn default_true() -> bool {
    true
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[repr(u8)]
pub enum PaxType {
    #[default]
    Adult = 1,

    Child = 2,
}

#[derive(Serialize, PartialEq, Deserialize, Debug, Clone)]
pub struct BookRoomResponse {
    #[serde(rename = "Status")]
    pub status: BookingStatus,

    #[serde(rename = "Message")]
    pub message: Option<String>,

    #[serde(rename = "CommitBooking")]
    pub commit_booking: Vec<BookingDetails>,
}

#[derive(Serialize, PartialEq, Deserialize, Debug, Clone)]
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

#[derive(Serialize, PartialEq, Deserialize, Debug, Clone)]
pub enum BookingStatus {
    #[serde(rename = "BookFailed")]
    BookFailed = 0,
    #[serde(rename = "Confirmed")]
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

// ======================  custom serializer for the PassengerDetail ======================
use serde::ser::{SerializeStruct, Serializer};

impl Serialize for PassengerDetail {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if self.first_name.len() < 2 || self.first_name.len() >= 50 {
            return Err(serde::ser::Error::custom(
                "First name must have at least 2 characters, max 50 characters",
            ));
        }

        if self.last_name.len() < 2 || self.last_name.len() >= 50 {
            return Err(serde::ser::Error::custom(
                "Last name must have at least 2 characters, max 50 characters",
            ));
        }

        let mut state = serializer.serialize_struct("PassengerDetail", 8)?;
        state.serialize_field("Title", &self.title)?;

        state.serialize_field("FirstName", &self.first_name)?;
        state.serialize_field(
            "MiddleName",
            &self.middle_name.clone().unwrap_or_else(|| "".to_string()),
        )?;

        state.serialize_field("LastName", &self.last_name)?;

        state.serialize_field("Email", &self.email)?;
        state.serialize_field("PaxType", &(self.pax_type.clone() as u8).to_string())?;
        state.serialize_field("LeadPassenger", &self.lead_passenger)?;
        state.serialize_field("Age", &self.age)?;
        state.end()
    }
}
