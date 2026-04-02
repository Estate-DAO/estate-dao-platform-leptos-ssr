use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AmadeusHotelDetailsResponse {
    pub data: Vec<AmadeusHotelDetailsEntry>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AmadeusHotelDetailsEntry {
    #[serde(rename = "hotelId")]
    pub hotel_id: String,
    pub name: String,
    #[serde(rename = "geoCode")]
    pub geo_code: Option<crate::amadeus::models::search::AmadeusGeoCode>,
    pub address: Option<crate::amadeus::models::search::AmadeusAddress>,
    #[serde(rename = "chainCode")]
    pub chain_code: Option<String>,
    #[serde(rename = "iataCode")]
    pub iata_code: Option<String>,
}
