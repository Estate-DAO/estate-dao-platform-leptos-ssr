use std::sync::Arc;

use super::consts::{get_headers_from_env, get_provab_base_url_from_env};
use anyhow::Result;
use reqwest::{IntoUrl, Method, RequestBuilder, Url};
use serde::{de::DeserializeOwned, Serialize};

use error_stack::{report, Report, ResultExt};

use crate::api::http_methods::{ApiClientResult, ApiError};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::collections::HashMap;

pub trait ProvabReqMeta: Sized + Send {
    const METHOD: Method;
    type Response: DeserializeOwned;

    /// Deserialize the response from the API
    /// The default implementation that assumes the response is JSON encoded [crate::CfSuccessRes]
    /// and extracts the `result` field
    fn deserialize_response(body: String) -> ApiClientResult<Self::Response> {
        let res: Self::Response =
            serde_json::from_str(&body).map_err(|e| report!(ApiError::JsonParseFailed(e)))?;
        Ok(res)
    }
}

pub trait ProvabReq: ProvabReqMeta {
    fn base_path() -> String {
        get_provab_base_url_from_env().to_owned()
    }

    fn path() -> String {
        format!("{}/{}", Self::base_path(), Self::path_suffix())
    }

    fn path_suffix() -> &'static str;

    fn headers() -> HeaderMap {
        get_headers_from_env()
    }

    fn custom_headers() -> HeaderMap {
        Self::headers()
    }
}

#[derive(Clone, Debug)]
pub struct Provab {
    client: reqwest::Client,
    base_url: Arc<Url>,
}

impl Default for Provab {
    fn default() -> Self {
        Self {
            client: reqwest::Client::builder().gzip(true).build().unwrap(),
            base_url: Arc::new(get_provab_base_url_from_env().parse().unwrap()),
        }
    }
}

impl Provab {
    /// Create a new client with the given base URL.
    /// Use [Default::default] to use the default base URL ([crate::consts::get_provab_base_url_from_env])
    pub fn new(base_url: Url) -> Self {
        Self {
            client: Default::default(),
            base_url: Arc::new(base_url),
        }
    }

    fn req_builder(
        &self,
        method: Method,
        url: impl IntoUrl,
        // g: Option<&Credentials>,
    ) -> RequestBuilder {
        let reqb = self.client.request(method, url);
        // if let Some(creds) = auth {
        // reqb.bearer_auth(&creds.token)
        // } else {
        reqb
        // }
    }

    async fn send_inner<Req: ProvabReqMeta>(
        &self,
        reqb: RequestBuilder,
    ) -> ApiClientResult<Req::Response> {
        let response = reqb
            .send()
            .await
            .map_err(|e| report!(ApiError::RequestFailed(e)))?;

        if !response.status().is_success() {
            response.text().await.map_or_else(
                |er| {
                    Err(ApiError::ResponseError).attach_printable_lazy(|| format!("Error: {er:?}"))
                },
                |t| Err(report!(ApiError::ResponseNotOK(t))),
            )
        } else {
            let res = Req::deserialize_response(
                response
                    .text()
                    .await
                    .map_err(|_e| report!(ApiError::ResponseError))?,
            )?;
            Ok(res)
        }
    }

    async fn send_json<Req: ProvabReq + ProvabReqMeta + Serialize>(
        &self,
        req: Req,
        reqb: RequestBuilder,
    ) -> ApiClientResult<Req::Response> {
        let reqb = if Req::METHOD == Method::GET {
            reqb.query(&req)
        } else {
            reqb.json(&req)
        };
        let reqb = reqb.headers(Req::custom_headers());
        self.send_inner::<Req>(reqb).await
    }
}
