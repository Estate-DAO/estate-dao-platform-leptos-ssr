use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AmadeusBookingResponse {
    pub data: AmadeusBookingData,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AmadeusBookingData {
    pub id: String,
}
