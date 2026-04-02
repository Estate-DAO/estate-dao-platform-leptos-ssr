use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AmadeusBookingResponse {
    pub data: AmadeusBookingData,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AmadeusBookingData {
    pub id: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AmadeusBlockRoomSnapshot {
    pub offer_id: String,
    pub hotel_id: String,
    pub room_name: String,
    pub total_price: f64,
    pub currency_code: String,
    pub cancellation_policy: Option<String>,
}
