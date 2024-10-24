use super::{ProvabReq, ProvabReqMeta};
use reqwest::Method;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct HotelBookingDetailRequest {
    app_reference: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HotelBookingDetailResponse {
    status: bool,
    message: String,
    update_hold_booking: BookingIds,
}

#[derive(Debug, Serialize, Deserialize)]
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
