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

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
#[serde(rename_all = "camelCase")]
pub struct Place {
    pub place_id: String,
    pub display_name: String,
    pub formatted_address: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiGetPlacesResponse {
    pub data: Vec<Place>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiGetPlacesRequest {
    #[serde(rename = "textQuery")]
    pub text_query: String,
}

impl LiteApiGetPlacesRequest {
    pub fn new(text_query: String) -> Self {
        Self { text_query }
    }
}

impl LiteApiReq for LiteApiGetPlacesRequest {
    fn path_suffix() -> &'static str {
        "data/places"
    }
}

impl ApiRequestMeta for LiteApiGetPlacesRequest {
    const METHOD: Method = Method::GET;
    const GZIP: bool = false;
    type Response = LiteApiGetPlacesResponse;
}

impl ApiRequest for LiteApiGetPlacesRequest {
    fn base_path() -> String {
        <Self as LiteApiReq>::base_path()
    }

    fn path_suffix() -> &'static str {
        <Self as LiteApiReq>::path_suffix()
    }

    fn custom_headers() -> HeaderMap {
        <Self as LiteApiReq>::custom_headers()
    }
}

#[instrument(skip(request), fields(
    text_query = %request.text_query,
    api_provider = "liteapi",
    operation = "get_places",
    service.name = "estate_fe_ssr",
    component = "liteapi_client"
))]
async fn liteapi_get_places(
    request: LiteApiGetPlacesRequest,
) -> Result<LiteApiGetPlacesResponse, crate::api::ApiError> {
    let client = LiteApiHTTPClient::default();
    client.send(request).await
}

#[instrument(fields(
    text_query = %text_query,
    api_provider = "liteapi",
    operation = "get_places_list",
    service.name = "estate_fe_ssr",
    component = "liteapi_client"
))]
pub async fn get_places_list(
    text_query: String,
) -> Result<LiteApiGetPlacesResponse, crate::api::ApiError> {
    let request = LiteApiGetPlacesRequest::new(text_query);
    liteapi_get_places(request).await
}
