use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AmadeusHotelDetailsResponse {
    pub data: Vec<AmadeusHotelDetails>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AmadeusHotelDetails {
    pub hotel_id: String,
}
