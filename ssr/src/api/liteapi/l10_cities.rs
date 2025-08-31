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
pub struct City {
    pub city: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiGetCitiesResponse {
    pub data: Vec<City>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct LiteApiGetCitiesRequest {
    #[serde(rename = "countryCode")]
    pub country_code: String,
    pub timeout: Option<u32>,
}

impl LiteApiGetCitiesRequest {
    pub fn new(country_code: String) -> Self {
        Self {
            country_code,
            timeout: Some(2),
        }
    }
}

impl LiteApiReq for LiteApiGetCitiesRequest {
    fn path_suffix() -> &'static str {
        "data/cities"
    }
}

impl ApiRequestMeta for LiteApiGetCitiesRequest {
    const METHOD: Method = Method::GET;
    const GZIP: bool = false;
    type Response = LiteApiGetCitiesResponse;
}

impl ApiRequest for LiteApiGetCitiesRequest {
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
    country_code = %request.country_code,
    timeout = request.timeout,
    api_provider = "liteapi",
    operation = "get_cities",
    service.name = "estate_fe_ssr",
    component = "liteapi_client"
))]
pub async fn liteapi_get_cities(
    request: LiteApiGetCitiesRequest,
) -> Result<LiteApiGetCitiesResponse, crate::api::ApiError> {
    let client = LiteApiHTTPClient::default();
    client.send(request).await
}

#[instrument(fields(
    country_code = %country_code,
    api_provider = "liteapi",
    operation = "get_cities_list",
    service.name = "estate_fe_ssr",
    component = "liteapi_client"
))]
pub async fn get_cities_list(
    country_code: String,
) -> Result<LiteApiGetCitiesResponse, crate::api::ApiError> {
    let request = LiteApiGetCitiesRequest::new(country_code);
    liteapi_get_cities(request).await
}

// Simple async iterator for getting all cities
pub struct AllCitiesIterator {
    countries: Vec<crate::api::liteapi::Country>,
    current_index: usize,
}

pub type CountryCitiesResult = Result<
    (crate::api::liteapi::Country, Vec<City>),
    (crate::api::liteapi::Country, crate::api::ApiError),
>;

impl AllCitiesIterator {
    #[instrument(skip(self), fields(
        current_index = self.current_index,
        total_countries = self.countries.len(),
        api_provider = "liteapi",
        operation = "iterate_next_country",
        service.name = "estate_fe_ssr",
        component = "all_cities_iterator"
    ))]
    pub async fn next(&mut self) -> Option<CountryCitiesResult> {
        if self.current_index >= self.countries.len() {
            return None;
        }

        let country = self.countries[self.current_index].clone();
        self.current_index += 1;

        match get_cities_list(country.code.clone()).await {
            Ok(response) => Some(Ok((country, response.data))),
            Err(error) => Some(Err((country, error))),
        }
    }

    pub fn progress(&self) -> (usize, usize) {
        (self.current_index, self.countries.len())
    }
}

// Simple function to get all cities - returns async iterator
#[instrument(fields(
    api_provider = "liteapi",
    operation = "get_all_cities",
    service.name = "estate_fe_ssr",
    component = "all_cities_iterator",
    countries.count = tracing::field::Empty
))]
pub async fn get_all_cities() -> Result<AllCitiesIterator, crate::api::ApiError> {
    let countries_response = crate::api::liteapi::get_countries_list().await?;

    // Record the number of countries for observability
    tracing::Span::current().record("countries.count", countries_response.data.len());

    Ok(AllCitiesIterator {
        countries: countries_response.data,
        current_index: 0,
    })
}
