use std::sync::Arc;

use super::consts::EnvVarConfig;
use super::{ApiClientResult, ApiError};
use colored::Colorize;
use error_stack::{report, Report, ResultExt};
use leptos::expect_context;
use leptos::logging::log;
use reqwest::header::HeaderMap;
use reqwest::{IntoUrl, Method, RequestBuilder, Response, Url};
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;
use std::io::Read;

cfg_if::cfg_if! {
    if #[cfg(feature = "mock-provab")] {
        // fake imports
        use fake::{Dummy, Fake, Faker};
        use rand::rngs::StdRng;
        use rand::SeedableRng;
    }
}

#[derive(Debug, Clone)]
pub enum DeserializableInput {
    Text(String),
    Bytes(Vec<u8>),
}

impl DeserializableInput {
    pub fn take(self, num: usize) -> Self {
        match self {
            Self::Text(body_string) => Self::Text(body_string.chars().take(num).collect()),
            Self::Bytes(body_bytes) => Self::Bytes(body_bytes.into_iter().take(num).collect()),
        }
    }

    pub fn to_string(self) -> String {
        match self {
            Self::Text(body_string) => body_string,
            Self::Bytes(body_bytes) => {
                match String::from_utf8(body_bytes) {
                    Ok(string) => {
                        println!("DeserializableInput - Bytes(Vec<u8>):");
                        string
                    }
                    Err(e) => {
                        println!("DeserializableInput - Bytes - could not convert - : {}", e);
                        // return empty string for now. since this is debug only.
                        String::new()
                    }
                }
            }
        }
    }
}

pub trait ProvabReqMeta: Sized + Send {
    const METHOD: Method;
    const GZIP: bool = true;

    #[cfg(feature = "mock-provab")]
    type Response: DeserializeOwned + Debug + Dummy<Faker>;

    #[cfg(not(feature = "mock-provab"))]
    type Response: DeserializeOwned + Debug;

    /// Deserialize the response from the API
    /// The default implementation that assumes the response is JSON encoded [crate::CfSuccessRes]
    /// and extracts the `result` field
    fn deserialize_response(
        response_bytes_or_string: DeserializableInput,
    ) -> ApiClientResult<Self::Response> {
        log!(
            "{}",
            format!(
                "gzip = {} , response_bytes_or_string : {}\n\n\n",
                Self::GZIP,
                response_bytes_or_string.clone().take(50).to_string()
            )
            .bright_yellow()
            .bold()
        );

        let decompressed_body = match response_bytes_or_string {
            DeserializableInput::Bytes(body_bytes) => {
                // use flate2::read::GzDecoder;

                // let mut d = GzDecoder::new(&body_bytes[..]);

                // let mut s = String::new();
                // d.read_to_string(&mut s).map_err(|e| {
                //     log!("\n\ndeserialize_response- DecompressionFailed: {e:?}\n\n");
                //     report!(ApiError::DecompressionFailed(e.to_string()))
                // })?;
                // s
                String::from_utf8(body_bytes).map_err(|e| {
                    report!(ApiError::DecompressionFailed(String::from(
                        "Could not convert from bytes to string"
                    )))
                })?
            }
            DeserializableInput::Text(body_string) => body_string,
        };

        let jd = &mut serde_json::Deserializer::from_str(&decompressed_body);
        let res: Self::Response = serde_path_to_error::deserialize(jd).map_err(|e| {
            let total_error = format!("path: {} - inner: {} ", e.path().to_string(), e.inner());
            log!("deserialize_response- JsonParseFailed: {:?}", total_error);
            report!(ApiError::JsonParseFailed(total_error))
        })?;

        // log!("\n\ndeserialize_response- Ok : {res:?}\n\n\n");

        Ok(res)
    }
}

pub trait ProvabReq: ProvabReqMeta {
    fn base_path() -> String {
        // get_provab_base_url_from_env().to_owned()
        // log!("base_path() BEFORE");

        let env_var_config: EnvVarConfig = expect_context();
        // log!("base_path(): {env_var_config:#?}");

        env_var_config.provab_base_url
    }

    fn path() -> String {
        format!("{}/{}", Self::base_path(), Self::path_suffix())
    }

    fn path_suffix() -> &'static str;

    fn headers() -> HeaderMap {
        // get_headers_from_env()
        log!("headers(): BEFORE");

        let env_var_config: EnvVarConfig = expect_context();

        env_var_config.get_headers()
    }

    fn custom_headers() -> HeaderMap {
        Self::headers()
    }
}

#[derive(Clone, Debug)]
pub struct Provab {
    client: reqwest::Client,
    // base_url: Arc<Url>,
}

impl Default for Provab {
    fn default() -> Self {
        Self {
            client: reqwest::Client::builder()
                // .gzip(true)
                // .deflate(true)
                .build()
                .unwrap(),
            // base_url: Arc::new(get_provab_base_url_from_env().parse().unwrap()),
        }
    }
}

impl Provab {
    /// Create a new client with the given base URL.
    /// Use [Default::default] to use the default base URL ([crate::consts::get_provab_base_url_from_env])
    pub fn new(base_url: Url) -> Self {
        Self {
            client: Default::default(),
            // base_url: Arc::new(base_url),
        }
    }

    fn req_builder(
        &self,
        method: Method,
        url: impl IntoUrl,
        // g: Option<&Credentials>,
    ) -> RequestBuilder {
        let reqb = self.client.request(method, url);
        reqb
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
            return response.text().await.map_or_else(
                |er| {
                    Err(ApiError::ResponseError).attach_printable_lazy(|| format!("Error: {er:?}"))
                },
                |t| Err(report!(ApiError::ResponseNotOK(t))),
            );
        }

        let input = if Req::GZIP {
            let body_bytes = response
                .bytes()
                .await
                .map_err(|_e| report!(ApiError::ResponseError))?;
            DeserializableInput::Bytes(body_bytes.into())
        } else {
            let body_string = response
                .text()
                .await
                .map_err(|_e| report!(ApiError::ResponseError))?;
            DeserializableInput::Text(body_string)
        };

        let res = Req::deserialize_response(input)?;

        Ok(res)
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

        let reqb = if Req::GZIP {
            set_gzip_accept_encoding(reqb)
        } else {
            reqb
        };

        // log!(" reqb - send_json: {reqb:?}");

        self.send_inner::<Req>(reqb).await
    }

    pub async fn send<Req: ProvabReq + ProvabReqMeta + Serialize>(
        &self,
        req: Req,
    ) -> ApiClientResult<Req::Response> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "mock-provab")] {
                let resp: Req::Response = Faker.fake();
                Ok(resp)
            } else {
                let reqb = self.req_builder(Req::METHOD, Req::path());
                // log!("reqb - send: {reqb:?}");
                self.send_json(req, reqb).await
            }
        }
    }
}
fn set_gzip_accept_encoding(reqb: RequestBuilder) -> RequestBuilder {
    // let headers = reqb.headers_ref().unwrap();
    // if !headers.contains_key("Accept-Encoding") && !headers.contains_key("Range") {
    reqb.header("Accept-Encoding", "gzip, deflate")
    // } else {
    //     reqb
    // }
}
