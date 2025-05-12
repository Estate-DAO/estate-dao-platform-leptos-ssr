use std::sync::Arc;

use super::a03_block_room::BlockRoomResponse;
use super::consts::EnvVarConfig;
use super::{ApiClientResult, ApiError};
use colored::Colorize;
use error_stack::{report, Report, ResultExt};
use leptos::expect_context;
// use leptos::logging::log;

use crate::log;
use reqwest::header::HeaderMap;
use reqwest::{IntoUrl, Method, RequestBuilder, Response, Url};
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;
use std::io::Read;

#[cfg(feature = "ssr")]
use tokio;

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
                        // log!("DeserializableInput - Bytes(Vec<u8>): \n{}", string);
                        string
                    }
                    Err(e) => {
                        log!("DeserializableInput - Bytes - could not convert - : {}", e);
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
    type Response: DeserializeOwned + Debug + MockableResponse + Send;

    #[cfg(not(feature = "mock-provab"))]
    type Response: DeserializeOwned + Debug + Send;

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
                response_bytes_or_string.clone().to_string()
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

pub trait ProvabReq: ProvabReqMeta + Debug {
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
        // log!("headers(): BEFORE");

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
        let request = reqb
            .build()
            .map_err(|e| report!(ApiError::RequestFailed(e)))?;
        // Print the headers before sending the request
        // print_request_headers(&request)?;

        // let response = reqb
        //     .send()
        //     .await
        //     .map_err(|e| report!(ApiError::RequestFailed(e)))?;

        let response = self
            .client
            .execute(request)
            .await
            .map_err(|e| report!(ApiError::RequestFailed(e)))?;

        let response_status = response.status();
        if !response_status.is_success() {
            return response.text().await.map_or_else(
                |er| {
                    Err(ApiError::ResponseError).attach_printable_lazy(|| format!("Error: {er:?}"))
                },
                |t| {
                    Err(report!(ApiError::ResponseNotOK(format!(
                        "received status - {}, error- {t}",
                        response_status
                    ))))
                },
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
        log_json_payload(&req);

        let reqb = if Req::METHOD == Method::GET {
            reqb.query(&req)
        } else {
            reqb.json(&req)
        };
        let reqb = reqb.headers(Req::custom_headers());

        // TODO: in production, set headers to live. (via environment variables?)
        #[cfg(any(feature = "prod-consts", feature = "stage-consts"))]
        let reqb = set_live_headers(reqb);

        let reqb = if Req::GZIP {
            set_gzip_accept_encoding(reqb)
        } else {
            reqb
        };

        // log!(" reqb - send_json: {reqb:?}");

        self.send_inner::<Req>(reqb).await
    }

    #[cfg(feature = "mock-provab")]
    /// Send a request and return the response - mocked
    pub async fn send<Req>(&self, req: Req) -> ApiClientResult<Req::Response>
    where
        Req: ProvabReq + ProvabReqMeta + Serialize + Send + 'static,
        Req::Response: MockableResponse + Send + 'static,
    {
        // sleep by 3 seconds before response

        use std::time::Duration;
        #[cfg(all(feature = "ssr", feature = "mock-provab"))]
        tokio::time::sleep(Duration::from_millis(20000)).await;

        Ok(Req::Response::generate_mock_response(0.5))
    }

    #[cfg(not(feature = "mock-provab"))]
    /// Send a request and return the response
    pub async fn send<Req>(&self, req: Req) -> ApiClientResult<Req::Response>
    where
        Req: ProvabReq + ProvabReqMeta + Serialize + Send + 'static,
        Req::Response: Send + 'static,
    {
        // Real request implementation here
        let reqb = self.req_builder(Req::METHOD, Req::path());
        self.send_json(req, reqb).await
    }
}

fn set_gzip_accept_encoding(reqb: RequestBuilder) -> RequestBuilder {
    // Create a HeaderMap to store custom headers
    let mut headers = HeaderMap::new();

    // Insert headers with specific casing
    headers.insert(
        http::HeaderName::from_bytes(b"Accept-Encoding").unwrap(),
        http::HeaderValue::from_str("gzip, deflate").unwrap(),
    );

    reqb.headers(headers)
    // let headers = reqb.headers_ref().unwrap();
    // if !headers.contains_key("Accept-Encoding") && !headers.contains_key("Range") {
    // reqb.header("Accept-Encoding", "gzip, deflate")
    // } else {
    //     reqb
    // }
}

fn set_live_headers(reqb: RequestBuilder) -> RequestBuilder {
    // Create a HeaderMap to store custom headers
    let mut headers = HeaderMap::new();

    // Insert headers with specific casing
    headers.insert(
        http::HeaderName::from_bytes(b"x-System").unwrap(),
        http::HeaderValue::from_str("live").unwrap(),
    );

    reqb.headers(headers)
}

/// Function to print the headers of a reqwest::Request
fn print_request_headers(request: &reqwest::Request) -> ApiClientResult<()> {
    let headers_str = request
        .headers()
        .iter()
        .map(|(key, value)| {
            let key_str = key.as_str();
            let value_str = value.to_str().unwrap_or("<Invalid UTF-8>");
            format!("{}: {}", key_str, value_str)
        })
        .collect::<Vec<_>>()
        .join("\n");

    println!(
        "----- Request Headers -----\n{}\n-----------------------",
        headers_str
    );
    Ok(())
}

fn log_json_payload<T: Serialize + Debug>(req: &T) {
    match serde_json::to_string_pretty(req) {
        Ok(json) => log!(
            "{}",
            format!("json_payload = {} ", &json).bright_cyan().bold()
        ),

        Err(e) => log!(
            "{}",
            format!("Failed to serialize JSON payload: {:?}", e)
                .bright_red()
                .bold()
        ),
    }
}
