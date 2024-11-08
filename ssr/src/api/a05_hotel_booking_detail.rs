use super::{ProvabReq, ProvabReqMeta};
use reqwest::Method;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HotelBookingDetailRequest {
    app_reference: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HotelBookingDetailResponse {
    status: bool,
    message: String,
    update_hold_booking: BookingIds,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
// booking_id from previous API endpoint
pub struct BookingIds(Vec<String>);

impl ProvabReqMeta for HotelBookingDetailRequest {
    const METHOD: Method = Method::POST;
    const GZIP: bool = false;
    type Response = HotelBookingDetailResponse;
}
impl ProvabReq for HotelBookingDetailRequest {
    fn path_suffix() -> &'static str {
        "UpdateHoldBooking"
    }
}
