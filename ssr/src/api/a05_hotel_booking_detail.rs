use super::{ProvabReq, ProvabReqMeta};
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
    app_reference: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct HotelBookingDetailResponse {
    status: bool,
    message: String,
    update_hold_booking: BookingIds,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
// booking_id from previous API endpoint
pub struct BookingIds(Vec<String>);

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
