use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiteApiGetPlaceResponse {
    pub data: Data,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Data {
    pub address_components: Vec<AddressComponent>,
    pub location: Location,
    pub viewport: Viewport,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddressComponent {
    pub language_code: String,
    pub long_text: String,
    pub short_text: String,
    pub types: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Viewport {
    pub high: High,
    pub low: Low,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct High {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Low {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct LiteApiGetPlaceRequest {
    pub place_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Place {
    pub place_id: String,
    pub display_name: String,
    pub formatted_address: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LiteApiGetPlacesResponse {
    pub data: Vec<Place>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct LiteApiGetPlacesRequest {
    #[serde(rename = "textQuery")]
    pub text_query: String,
}
