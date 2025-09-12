use leptos::*;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::api::api_client::{ApiClient, ApiRequest, ApiRequestMeta};
use crate::api::liteapi::client::LiteApiHTTPClient;
use crate::api::liteapi::traits::LiteApiReq;
use reqwest::header::HeaderMap;

cfg_if::cfg_if! {
    if #[cfg(feature = "mock-provab")] {
        use fake::{Dummy, Fake, Faker};
        use rand::rngs::StdRng;
        use rand::SeedableRng;
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
#[serde(rename_all = "camelCase")]
pub struct LiteApiGetPlaceResponse {
    pub data: Data,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
#[serde(rename_all = "camelCase")]
pub struct Data {
    pub address_components: Vec<AddressComponent>,
    pub location: Location,
    pub viewport: Viewport,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct AddressComponent {
    pub language_code: String,
    pub long_text: String,
    pub short_text: String,
    pub types: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct Viewport {
    pub high: High,
    pub low: Low,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct High {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct Low {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiGetPlaceRequest {
    pub place_id: String,
}

impl LiteApiGetPlaceRequest {
    pub fn new(place_id: String) -> Self {
        Self { place_id }
    }
}

impl LiteApiReq for LiteApiGetPlaceRequest {
    fn path_suffix() -> &'static str {
        "data/places"
    }
}

impl ApiRequestMeta for LiteApiGetPlaceRequest {
    const METHOD: Method = Method::GET;
    const GZIP: bool = false;
    type Response = LiteApiGetPlaceResponse;
}

impl ApiRequest for LiteApiGetPlaceRequest {
    fn base_path() -> String {
        <Self as LiteApiReq>::base_path()
    }

    fn path_suffix() -> &'static str {
        <Self as LiteApiReq>::path_suffix()
    }

    fn custom_headers() -> HeaderMap {
        <Self as LiteApiReq>::custom_headers()
    }

    fn full_path(&self) -> String {
        format!(
            "{}/{}/{}",
            <Self as LiteApiReq>::base_path(),
            <Self as LiteApiReq>::path_suffix(),
            self.place_id
        )
    }
}

#[instrument(skip(request), fields(
    place_id = %request.place_id,
    api_provider = "liteapi",
    operation = "get_place",
    service.name = "estate_fe_ssr",
    component = "liteapi_client"
))]
async fn liteapi_get_place(
    request: LiteApiGetPlaceRequest,
) -> Result<LiteApiGetPlaceResponse, crate::api::ApiError> {
    let client = LiteApiHTTPClient::default();
    client.send(request).await
}

#[instrument(fields(
    place_id = %place_id,
    api_provider = "liteapi",
    operation = "get_place",
    service.name = "estate_fe_ssr",
    component = "liteapi_client"
))]
pub async fn get_place(place_id: String) -> Result<LiteApiGetPlaceResponse, crate::api::ApiError> {
    let request = LiteApiGetPlaceRequest::new(place_id);
    liteapi_get_place(request).await
}
