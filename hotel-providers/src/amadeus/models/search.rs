use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AmadeusHotelListResponse {
    pub data: Vec<AmadeusHotelListEntry>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AmadeusHotelListEntry {
    #[serde(rename = "hotelId")]
    pub hotel_id: String,
    pub name: String,
    #[serde(rename = "geoCode")]
    pub geo_code: Option<AmadeusGeoCode>,
    pub address: Option<AmadeusAddress>,
    pub distance: Option<AmadeusDistance>,
    #[serde(rename = "chainCode")]
    pub chain_code: Option<String>,
    #[serde(rename = "iataCode")]
    pub iata_code: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AmadeusGeoCode {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AmadeusAddress {
    pub lines: Option<Vec<String>>,
    pub city_name: Option<String>,
    pub country_code: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AmadeusDistance {
    pub value: f64,
    pub unit: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AmadeusHotelOffersResponse {
    pub data: Vec<AmadeusHotelOffer>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AmadeusHotelOffer {
    pub hotel: AmadeusOfferHotel,
    pub available: bool,
    pub offers: Vec<AmadeusOffer>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AmadeusOfferHotel {
    #[serde(rename = "hotelId")]
    pub hotel_id: String,
    pub name: String,
    pub city_code: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AmadeusOffer {
    pub id: String,
    pub check_in_date: String,
    pub check_out_date: String,
    pub price: AmadeusOfferPrice,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AmadeusOfferPrice {
    pub currency: String,
    pub total: String,
    pub base: Option<String>,
}
