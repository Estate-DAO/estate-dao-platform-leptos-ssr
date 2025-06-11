use std::sync::Arc;

use colored::Colorize;
use leptos::{expect_context, use_context};

use crate::api::api_client::ApiClient;
use crate::api::consts::EnvVarConfig;
use crate::api::provab::DeserializableInput;
use crate::api::{ApiClientResult, ApiError};
use crate::log;
use reqwest::header::HeaderMap;
use reqwest::{IntoUrl, Method, RequestBuilder, Response, Url};
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;
use std::io::Read;

// #[cfg(feature = "ssr")]
// use tokio;

cfg_if::cfg_if! {
    if #[cfg(feature = "mock-provab")] {
        // fake imports
        use fake::{Dummy, Fake, Faker};
        use rand::rngs::StdRng;
        use rand::SeedableRng;

        use crate::api::mock::mock_utils::MockableResponse;
        use crate::api::mock::mock_utils::MockResponseGenerator;
    }
}

#[derive(Clone, Debug)]
pub struct LiteApiHTTPClient {
    client: reqwest::Client,
    // base_url: Arc<Url>,
}

impl Default for LiteApiHTTPClient {
    fn default() -> Self {
        Self {
            client: reqwest::Client::builder()
                // .gzip(true)
                .build()
                .unwrap(),
            // base_url: Arc::new(get_provab_base_url_from_env().parse().unwrap()),
        }
    }
}

impl ApiClient for LiteApiHTTPClient {
    fn base_url(&self) -> String {
        "https://api.liteapi.travel/v3.0".to_string()
    }

    fn default_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("Accept", "application/json".parse().unwrap());
        headers
    }

    fn http_client(&self) -> &reqwest::Client {
        &self.client
    }
}

// pub fn from_leptos_context_or_axum_ssr() -> LiteApiHTTPClient {
//     let context = use_context::<LiteApiHTTPClient>();
//     match context {
//         Some(liteapi) => liteapi,
//         None => LiteApiHTTPClient::default(),
//         // None => get_provab_client().clone()
//     }
// }
