use std::fmt;

use super::{ApiClientResult, ProvabReq, ProvabReqMeta};
use crate::api::{ApiError, Provab};
use crate::canister::backend::{
    self, AdultDetail, BeBookRoomResponse, Booking, ChildDetail, UserDetails,
};
use leptos::logging::log;
use leptos::*;
use reqwest::Method;
use serde::Deserializer;
use serde::{Deserialize, Serialize};

cfg_if::cfg_if! {
    if #[cfg(feature = "mock-provab")] {
        // fake imports
        use fake::{Dummy, Fake, Faker};
        use rand::rngs::StdRng;
        use rand::SeedableRng;
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
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
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct RoomDetail {
    #[serde(rename = "PassengerDetails")]
    pub passenger_details: Vec<PassengerDetail>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
/// customer serializer is implemented to ensure validation
/// and override the serde's default behaviour around None values
pub struct PassengerDetail {
    // #[serde(rename = "Title")]
    pub title: String, // Mr,Mrs,Ms possible values

    // #[serde(rename = "FirstName")]
    // #[serde(validate(min_length = 2, max_length = 50))]
    pub first_name: String,

    // #[serde(rename = "MiddleName")]
    pub middle_name: Option<String>,

    // #[serde(rename = "LastName")]
    // #[serde(validate(min_length = 2, max_length = 50))]
    pub last_name: String,

    // #[serde(rename = "Email")]
    pub email: String,

    // #[serde(rename = "PaxType")]
    pub pax_type: PaxType,

    // #[serde(rename = "LeadPassenger", default = "default_true")]
    pub lead_passenger: bool,

    // #[serde(rename = "Age", default = "_default_passenger_age")]
    // #[serde( default = "_default_passenger_age")]
    pub age: u32, //for children, must be <= 18

    // #[serde(rename = "Phoneno")]
    pub phone_number: String,
}

impl Default for PassengerDetail {
    fn default() -> Self {
        Self {
            title: "Mr".to_string(),
            first_name: "".to_string(),
            middle_name: Some("".to_string()),
            last_name: "".to_string(),
            email: "".to_string(),
            pax_type: PaxType::Adult,
            lead_passenger: false,
            age: 33,
            // todo [UAT] - don't hardcode the phone number  - use it from the form user fills.
            phone_number: "9090909090".to_string(),
        }
    }
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

pub fn _default_passenger_age() -> u32 {
    25
}

fn default_true() -> bool {
    true
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
#[repr(u8)]
pub enum PaxType {
    #[default]
    Adult = 1,

    Child = 2,
}

// todo [UAT] - show the user that booking cannot be done if the Failure happens in API call
#[derive(Serialize, PartialEq, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct SuccessBookRoomResponse {
    #[serde(rename = "Status")]
    pub status: BookingStatus,
    // pub status: BookingStatus,
    #[serde(rename = "Message")]
    pub message: String,

    #[serde(rename = "CommitBooking")]
    pub commit_booking: BookingDetailsContainer,
}

#[derive(Serialize, PartialEq, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct FailureBookRoomResponse {
    #[serde(rename = "Status")]
    pub status: u32,

    #[serde(rename = "Message")]
    pub message: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
#[serde(untagged)]
pub enum BookRoomResponse {
    Success(SuccessBookRoomResponse),
    Failure(FailureBookRoomResponse),
}

#[derive(Serialize, PartialEq, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct BookingDetailsContainer {
    #[serde(rename = "BookingDetails")]
    pub booking_details: BookingDetails,
}

#[derive(Serialize, PartialEq, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct BookingDetails {
    #[serde(rename = "BookingId")]
    pub travelomatrix_id: String,

    #[serde(rename = "BookingRefNo")]
    pub booking_ref_no: String,

    #[serde(rename = "ConfirmationNo")]
    pub confirmation_no: String,

    #[serde(rename = "booking_status")]
    pub booking_status: String,
}

#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
// #[repr(u8)]
pub enum BookingStatus {
    // #[serde(rename = "BookFailed")]
    BookFailed = 0,
    // #[serde(rename = "Confirmed")]
    Confirmed = 1,
}

impl<'de> Deserialize<'de> for BookingStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        match value {
            0 => Ok(BookingStatus::BookFailed),
            1 => Ok(BookingStatus::Confirmed),
            _ => Err(serde::de::Error::custom("Invalid BookingStatus value")),
        }
    }
}

impl Serialize for BookingStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = match self {
            BookingStatus::Confirmed => serializer.serialize_u8(1),
            BookingStatus::BookFailed => serializer.serialize_u8(0),
        };

        value.map_err(|e| serde::ser::Error::custom(format!("Invalid BookingStatus value: {}", e)))
    }
}

use colored::Colorize;
use error_stack::{report, Report, ResultExt};
use std::io::Read;

impl ProvabReqMeta for BookRoomRequest {
    const METHOD: Method = Method::POST;
    const GZIP: bool = false;
    type Response = BookRoomResponse;

    fn deserialize_response(body: String) -> ApiClientResult<Self::Response> {
        use flate2::read::GzDecoder;

        println!(
            "{}",
            format!("deserialize_response- body:String : {body}\n\n\n")
                .bright_yellow()
                .bold()
        );
        let decompressed_body = if Self::GZIP {
            let mut d = GzDecoder::new(body.as_bytes());
            let mut s = String::new();
            d.read_to_string(&mut s)
                .map_err(|e| report!(ApiError::DecompressionFailed(e.to_string())))?;
            s
        } else {
            body
        };

        let json_value: serde_json::Value =
            serde_json::from_str(&decompressed_body).map_err(|e| {
                // let total_error = format!("path: {} - inner: {} ", e.path().to_string(), e.inner());
                log!("deserialize_response- JsonParseFailed: {:#?}", e);
                report!(ApiError::JsonParseFailed(e.to_string()))
            })?;

        if json_value.get("CommitBooking").is_some() {
            println!(
                "{}",
                format!("json_value - {}", json_value).bright_green().bold()
            );

            let res: SuccessBookRoomResponse = serde_json::from_value(json_value).map_err(|e| {
                // let total_error = format!("path: {} - inner: {} ", e.path().to_string(), e.inner());
                log!("deserialize_response- JsonParseFailed: {:?}", e.to_string());
                report!(ApiError::JsonParseFailed(e.to_string()))
            })?;
            Ok(BookRoomResponse::Success(res))
        } else {
            let res: FailureBookRoomResponse = serde_json::from_value(json_value).map_err(|e| {
                log!("deserialize_response- JsonParseFailed: {:?}", e.to_string());
                report!(ApiError::JsonParseFailed(e.to_string()))
            })?;
            Ok(BookRoomResponse::Failure(res))
        }
    }
}

impl ProvabReq for BookRoomRequest {
    fn path_suffix() -> &'static str {
        "CommitBooking"
    }
}

#[server(BlockRoom)]
pub async fn book_room(request: String) -> Result<String, ServerFnError> {
    // pub async fn book_room(request: String) -> Result<BookRoomResponse, ServerFnError> {
    let provab = Provab::default();

    let request_struct = serde_json::from_str::<BookRoomRequest>(&request)
        .map_err(|er| ServerFnError::new(format!("Could not deserialize Booking: Err = {er:?}")))?;

    println!("book_request - {request_struct:?}");

    match provab.send(request_struct).await {
        Ok(response) => {
            println!("{}", format!("{:?}", response).green().on_black());
            let response_str = serde_json::to_string(&response).unwrap();
            Ok(response_str)
        }
        Err(e) => {
            println!(
                "{}",
                format!("error: {:?}", e)
                    .bright_black()
                    .bold()
                    .on_bright_red()
            );
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
        state.serialize_field("Phoneno", &self.phone_number)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for PassengerDetail {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[cfg_attr(feature = "mock-provab", derive(Dummy))]
        struct PassengerDetailHelper {
            #[serde(rename = "Title")]
            pub title: String,

            #[serde(rename = "FirstName")]
            pub first_name: String,

            #[serde(rename = "MiddleName")]
            pub middle_name: Option<String>,

            #[serde(rename = "LastName")]
            pub last_name: String,

            #[serde(rename = "Email")]
            pub email: String,

            #[serde(rename = "PaxType")]
            pub pax_type: String,

            #[serde(rename = "LeadPassenger")]
            pub lead_passenger: bool,

            #[serde(rename = "Age")]
            pub age: u32,

            #[serde(rename = "Phoneno")]
            pub phone_number: String,
        }

        let helper = PassengerDetailHelper::deserialize(deserializer)?;

        // Validate first and last name length
        if helper.first_name.len() < 2 || helper.first_name.len() >= 50 {
            return Err(serde::de::Error::custom(
                "First name must have at least 2 characters, max 50 characters",
            ));
        }

        if helper.last_name.len() < 2 || helper.last_name.len() >= 50 {
            return Err(serde::de::Error::custom(
                "Last name must have at least 2 characters, max 50 characters",
            ));
        }

        let pax_type = match helper.pax_type.parse::<u8>() {
            Ok(1) => PaxType::Adult,
            Ok(2) => PaxType::Child,
            _ => return Err(serde::de::Error::custom("Invalid PaxType value")),
        };

        Ok(PassengerDetail {
            title: helper.title,
            first_name: helper.first_name,
            middle_name: helper.middle_name,
            last_name: helper.last_name,
            email: helper.email,
            pax_type,
            lead_passenger: helper.lead_passenger,
            age: helper.age,
            phone_number: helper.phone_number,
        })
    }
}

impl fmt::Display for BookingStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BookingStatus::Confirmed => write!(f, "Confirmed"),
            BookingStatus::BookFailed => write!(f, "BookFailed"),
        }
    }
}

impl From<BookingStatus> for backend::BookingStatus {
    fn from(status: BookingStatus) -> Self {
        match status {
            BookingStatus::BookFailed => backend::BookingStatus::BookFailed,
            BookingStatus::Confirmed => backend::BookingStatus::Confirmed,
        }
    }
}

impl From<backend::BookingStatus> for BookingStatus {
    fn from(status: backend::BookingStatus) -> Self {
        match status {
            backend::BookingStatus::BookFailed => BookingStatus::BookFailed,
            backend::BookingStatus::Confirmed => BookingStatus::Confirmed,
        }
    }
}

impl Default for BeBookRoomResponse {
    fn default() -> Self {
        BeBookRoomResponse {
            status: String::default(),
            commit_booking: backend::BookingDetails::default(),
            message: String::default(),
        }
    }
}

impl From<BookingDetailsContainer> for BookingDetails {
    fn from(input: BookingDetailsContainer) -> Self {
        let value = input.booking_details;
        Self {
            travelomatrix_id: value.travelomatrix_id,
            booking_ref_no: value.booking_ref_no,
            confirmation_no: value.confirmation_no,
            booking_status: value.booking_status,
        }
    }
}

impl Default for backend::BookingDetails {
    fn default() -> Self {
        backend::BookingDetails {
            booking_ref_no: String::default(),
            booking_status: String::default(),
            confirmation_no: String::default(),
            booking_id: backend::BookingId::default(),
            travelomatrix_id: String::default(),
            api_status: backend::BookingStatus::default(),
        }
    }
}

impl Default for backend::BookingStatus {
    fn default() -> Self {
        backend::BookingStatus::BookFailed
    }
}

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
                phone_number: "".to_string(),
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
                phone_number: "".to_string(),
            });
        }

        passenger_details
    }
}

impl FromIterator<AdultDetail> for std::vec::Vec<crate::state::view_state::AdultDetail> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = AdultDetail>,
    {
        iter.into_iter()
            .map(|detail| crate::state::view_state::AdultDetail {
                first_name: detail.first_name,
                last_name: detail.last_name,
                email: detail.email,
                phone: detail.phone,
            })
            .collect()
    }
}

impl FromIterator<ChildDetail> for std::vec::Vec<crate::state::view_state::ChildDetail> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = ChildDetail>,
    {
        iter.into_iter()
            .map(|detail| crate::state::view_state::ChildDetail {
                first_name: detail.first_name,
                last_name: detail.last_name,
                age: Some(detail.age),
            })
            .collect()
    }
}

impl From<UserDetails> for Vec<crate::state::view_state::AdultDetail> {
    fn from(user_details: UserDetails) -> Self {
        user_details
            .adults
            .into_iter()
            .map(|a| AdultDetail {
                first_name: a.first_name,
                last_name: a.last_name,
                email: a.email,
                phone: a.phone,
            })
            .collect()
    }
}

impl From<UserDetails> for Vec<crate::state::view_state::ChildDetail> {
    fn from(user_details: UserDetails) -> Self {
        user_details
            .children
            .into_iter()
            .map(|c| ChildDetail {
                first_name: c.first_name,
                last_name: c.last_name,
                age: c.age,
            })
            .collect()
    }
}

mod test {
    use super::*; // Imports from the parent module (a04_book_room)

    #[test]
    fn test_deserialize_response_success() {
        let body = r#"{"Status":1,"Message":"","CommitBooking":{"BookingDetails":{"ConfirmationNo":"218-3379918","BookingRefNo":"218-3379918","BookingId":"TM-218-3379918","booking_status":"BOOKING_CONFIRMED"}}}"#.to_string();
        let result = super::BookRoomRequest::deserialize_response(body);
        assert!(result.is_ok());
        match result.unwrap() {
            super::BookRoomResponse::Success(_) => (),
            _ => panic!("Expected SuccessBookRoomResponse"),
        }
    }

    #[test]
    fn test_deserialize_response_failure() {
        let body = r#"{"Status":0,"Message":"Booking Failed"}"#.to_string();
        let result = super::BookRoomRequest::deserialize_response(body);
        assert!(result.is_ok());
        match result.unwrap() {
            super::BookRoomResponse::Failure(_) => (),
            _ => panic!("Expected FailureBookRoomResponse"),
        }
    }
}
