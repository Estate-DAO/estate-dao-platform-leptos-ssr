use leptos::prelude::*;
use reqwest::Method;
use serde::{Deserialize, Serialize};

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
pub struct Country {
    pub code: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiGetCountriesResponse {
    pub data: Vec<Country>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiGetCountriesRequest {
    pub timeout: Option<u32>,
}

impl LiteApiGetCountriesRequest {
    pub fn new() -> Self {
        Self { timeout: Some(2) }
    }
}

impl LiteApiReq for LiteApiGetCountriesRequest {
    fn path_suffix() -> &'static str {
        "data/countries"
    }
}

impl ApiRequestMeta for LiteApiGetCountriesRequest {
    const METHOD: Method = Method::GET;
    const GZIP: bool = false;
    type Response = LiteApiGetCountriesResponse;
}

impl ApiRequest for LiteApiGetCountriesRequest {
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

pub async fn liteapi_get_countries(
    request: LiteApiGetCountriesRequest,
) -> Result<LiteApiGetCountriesResponse, crate::api::ApiError> {
    let client = LiteApiHTTPClient::default();
    client.send(request).await
}

pub async fn get_countries_list() -> Result<LiteApiGetCountriesResponse, crate::api::ApiError> {
    let request = LiteApiGetCountriesRequest::new();
    liteapi_get_countries(request).await
}
