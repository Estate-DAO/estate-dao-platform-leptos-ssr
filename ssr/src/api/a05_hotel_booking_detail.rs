use crate::api::{a04_book_room::from_leptos_context_or_axum_ssr, Provab};

use super::{ProvabReq, ProvabReqMeta};
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

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct HotelBookingDetailRequest {
    pub app_reference: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct HotelBookingDetailResponse {
    #[serde(rename = "Status")]
    pub status: bool,
    #[serde(rename = "Message")]
    pub message: String,
    #[serde(rename = "UpdateHoldBooking")]
    pub update_hold_booking: BookingProviderBookingIds,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
// booking_id from previous API endpoint
pub struct BookingProviderBookingIds(Vec<String>);

impl BookingProviderBookingIds {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl ProvabReqMeta for HotelBookingDetailRequest {
    const METHOD: Method = Method::POST;
    const GZIP: bool = true;
    type Response = HotelBookingDetailResponse;
}
impl ProvabReq for HotelBookingDetailRequest {
    fn path_suffix() -> &'static str {
        "UpdateHoldBooking"
    }
}

#[server(HotelBookingDetail)]
pub async fn get_hotel_booking_detail_from_travel_provider(
    request: HotelBookingDetailRequest,
) -> Result<HotelBookingDetailResponse, ServerFnError> {
    let provab: Provab = from_leptos_context_or_axum_ssr();

    println!("hotel booking detail request - {request:?}");

    match provab.send(request).await {
        Ok(response) => Ok(response),
        Err(e) => {
            // log!("server_fn_error: {}", e.to_string());
            Err(ServerFnError::ServerError(e.to_string()))
        }
    }
}

#[server(HotelBookingDetailV2)]
pub async fn get_hotel_booking_detail_from_travel_provider_v2(
    request: HotelBookingDetailRequest,
) -> Result<HotelBookingDetailResponse, ServerFnError> {
    let retry_count = 3;

    use crate::api::RetryableRequest;
    request.retry_with_backoff(retry_count).await
}
