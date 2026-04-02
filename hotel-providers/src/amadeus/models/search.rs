use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AmadeusHotelSearchResponse {
    pub data: Vec<AmadeusHotelSummary>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AmadeusHotelSummary {
    pub hotel_id: String,
}
