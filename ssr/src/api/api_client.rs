#[cfg(feature = "mock-provab")]
use crate::api::MockableResponse;

use crate::api::{ApiClientResult, ApiError};

cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {

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
                    &(response_bytes_or_string.clone().to_string())[..500]
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
                        ApiError::DecompressionFailed(String::from(
                            "Could not convert from bytes to string",
                        ))
                    })?
                }
                DeserializableInput::Text(body_string) => body_string,
            };

            let jd = &mut serde_json::Deserializer::from_str(&decompressed_body);
            let res: Self::Response = serde_path_to_error::deserialize(jd).map_err(|e| {
                let total_error = format!("path: {} - inner: {} ", e.path().to_string(), e.inner());
                log!("deserialize_response- JsonParseFailed: {:?}", total_error);
                ApiError::JsonParseFailed(total_error)
            })?;

            // log!("\n\ndeserialize_response- Ok : {res:?}\n\n\n");

            Ok(res)
        }
    }

    pub trait ProvabReq: ProvabReqMeta + Debug {
        fn base_path() -> String {
            let env_var_config = EnvVarConfig::expect_context_or_try_from_env();

            env_var_config.provab_base_url
        }

        fn path() -> String {
            format!("{}/{}", Self::base_path(), Self::path_suffix())
        }

        fn path_suffix() -> &'static str;

        fn headers() -> HeaderMap {
            let env_var_config = EnvVarConfig::expect_context_or_try_from_env();

            env_var_config.get_headers()
        }

        fn custom_headers() -> HeaderMap {
            Self::headers()
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

    }
}

use crate::{api::consts::EnvVarConfig, log};

use colored::Colorize;
use reqwest::{header::HeaderMap, Method, RequestBuilder};
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;

use crate::utils::route::join_base_and_path_url;

#[async_trait::async_trait]
/// Common trait for API clients that handle HTTP requests with JSON serialization/deserialization
pub trait ApiClient: Clone + Debug + Default {
    /// The base URL for this client's API endpoints
    fn base_url(&self) -> String;

    /// Default headers that should be included with every request
    fn default_headers(&self) -> HeaderMap;

    /// Get the underlying reqwest client
    fn http_client(&self) -> &reqwest::Client;

    /// Create a request builder with the given method and URL
    fn request_builder(&self, method: Method, url: impl reqwest::IntoUrl) -> RequestBuilder {
        self.http_client().request(method, url)
    }

    /// Send a request and deserialize the response
    async fn send<Req>(&self, req: Req) -> ApiClientResult<Req::Response>
    where
        Req: ApiRequestMeta + ApiRequest + Serialize + Send + 'static,
        Req::Response: Send + 'static,
    {
        let full_url = req.full_path();
        let reqb = self.request_builder(Req::METHOD, full_url);
        //  /*  join_base_and_path_url(self.base_url().as_str(), Req::path_suffix())
        //     .expect("Failed to join base and path URL") */;
        // let reqb = if Req::METHOD == Method::GET {
        //     match Req::DATA {
        //         RequestData::QueryParams => self.request_builder(Req::METHOD, full_url).query(&req),
        //         RequestData::PathParam(param) => {
        //             self.request_builder(Req::METHOD, format!("{}/{}", full_url, param))
        //         }
        //     }
        // } else {
        //     self.request_builder(Req::METHOD, full_url).json(&req)
        // };
        self.send_json_request(req, reqb).await
    }

    /// Internal method to handle JSON serialization and HTTP execution
    async fn send_json_request<Req>(
        &self,
        req: Req,
        reqb: RequestBuilder,
    ) -> ApiClientResult<Req::Response>
    where
        Req: ApiRequestMeta + ApiRequest + Serialize,
    {
        self.log_json_payload(&req);

        let reqb = if Req::METHOD == Method::GET {
            reqb.query(&req)
        } else {
            reqb.json(&req)
        };

        // Apply default headers
        let reqb = reqb.headers(self.default_headers());

        // Apply custom headers for this request type
        let reqb = reqb.headers(Req::custom_headers());

        // Apply conditional headers based on features
        let reqb = self.apply_conditional_headers(reqb);

        // Handle gzip if needed
        crate::log!(
            "{}",
            format!("send_json_request:gzip: {}", Req::GZIP)
                .bright_cyan()
                .bold()
        );
        let reqb = if Req::GZIP {
            self.set_gzip_headers(reqb)
        } else {
            reqb
        };

        self.execute_request::<Req>(reqb).await
    }

    /// Execute the HTTP request and handle the response
    #[inline]
    fn format_response_body_for_log(&self, response: String) -> String {
        cfg_if::cfg_if! {
            if #[cfg(feature = "debug_log")] {
                response
            } else {
                if response.len() > 500 {
                    format!("{}...[truncated]", &response[..500])
                } else {
                    response
                }
            }
        }
    }

    async fn execute_request<Req: ApiRequestMeta>(
        &self,
        reqb: RequestBuilder,
    ) -> ApiClientResult<Req::Response> {
        let reqb_clone = reqb.try_clone().unwrap();
        // First build the request to log details
        let request = reqb
            .build()
            .map_err(|e| ApiError::RequestFailed(e.into()))?;

        // Log the URL and query parameters
        let url = request.url();
        let query_params = url
            .query_pairs()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");

        crate::log!(
            "execute_request: method: {} path: {}\nQuery Params: {}\nFull URL: {}",
            request.method(),
            url.path(),
            if query_params.is_empty() {
                "<none>"
            } else {
                &query_params
            },
            url.as_str()
        );

        // Rebuild the request for execution since we can't reuse the built request
        let request = reqb_clone
            .build()
            .map_err(|e| ApiError::RequestFailed(e.into()))?;

        let response = self
            .http_client()
            .execute(request)
            .await
            .map_err(|e| ApiError::RequestFailed(e.into()))?;

        let status = response.status();

        // Log response status for all responses
        crate::log!(
            "{}",
            format!("Response Status: {}", status)
                .bright_magenta()
                .bold()
        );

        // Log response headers and content length for debugging
        let headers = response.headers();
        let content_length = headers
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown");
        let content_type = headers
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown");

        crate::log!(
            "{}",
            format!(
                "Response Headers - Content-Length: {}, Content-Type: {}",
                content_length, content_type
            )
            .bright_cyan()
            .bold()
        );

        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read error response".to_string());
            crate::log!(
                "{}",
                format!("Response Error - Status: {}, Body: {}", status, error_text)
                    .bright_red()
                    .bold()
            );
            return Err(ApiError::ResponseNotOK(format!(
                "Status: {}, Body: {}",
                status, error_text
            )));
        }

        let response = response
            .text()
            .await
            .map_err(|e| ApiError::ResponseError(e.to_string()))?;

        // Log raw response body for debugging
        crate::log!(
            "{}",
            format!(
                "Raw Response Body (length: {}): '{}'",
                response.len(),
                self.format_response_body_for_log(response.clone())
            )
            .bright_yellow()
            .bold()
        );

        let input = if Req::GZIP {
            let body_bytes = response.as_bytes();
            DeserializableInput::Bytes(body_bytes.to_vec())
        } else {
            DeserializableInput::Text(response.clone())
        };

        let res = Req::deserialize_response(input)?;
        Ok(res)
    }

    // /// Execute the HTTP request and handle the response
    // async fn execute_request<Req: ApiRequestMeta>(
    //     &self,
    //     reqb: RequestBuilder,
    // ) -> ApiClientResult<Req::Response> {
    //     let request = reqb
    //         .build()
    //         .map_err(|e| ApiError::RequestFailed(e.into()))?;

    //     let response = self
    //         .http_client()
    //         .execute(request)
    //         .await
    //         .map_err(|e| (ApiError::RequestFailed(e.into())))?;

    //     let response_status = response.status();
    //     if !response_status.is_success() {
    //         let response_text = response.text().await;
    //         return response_text.map_or_else(
    //             |er| Err(ApiError::ResponseError(er.to_string())),
    //             |t| {
    //                 Err(ApiError::ResponseNotOK(format!(
    //                     "received status - {}, error- {t}",
    //                     response_status
    //                 )))
    //             },
    //         );
    //     }

    //     let input = if Req::GZIP {
    //         let body_bytes = response
    //             .bytes()
    //             .await
    //             .map_err(|_e| ApiError::ResponseError(_e.to_string()))?;
    //         DeserializableInput::Bytes(body_bytes.into())
    //     } else {
    //         let body_string = response
    //             .text()
    //             .await
    //             .map_err(|_e| ApiError::ResponseError(_e.to_string()))?;
    //         DeserializableInput::Text(body_string)
    //     };

    //     let res = Req::deserialize_response(input)?;
    //     Ok(res)
    // }

    /// Apply conditional headers based on compile-time features
    fn apply_conditional_headers(&self, reqb: RequestBuilder) -> RequestBuilder {
        #[cfg(any(feature = "prod-consts", feature = "stage-consts"))]
        {
            self.set_live_headers(reqb)
        }
        #[cfg(not(any(feature = "prod-consts", feature = "stage-consts")))]
        {
            reqb
        }
    }

    /// Set gzip accept encoding headers
    fn set_gzip_headers(&self, reqb: RequestBuilder) -> RequestBuilder {
        let mut headers = HeaderMap::new();
        headers.insert(
            http::HeaderName::from_bytes(b"Accept-Encoding").unwrap(),
            http::HeaderValue::from_str("gzip, deflate").unwrap(),
        );
        reqb.headers(headers)
    }

    /// Set live system headers
    fn set_live_headers(&self, reqb: RequestBuilder) -> RequestBuilder {
        let mut headers = HeaderMap::new();
        headers.insert(
            http::HeaderName::from_bytes(b"x-System").unwrap(),
            http::HeaderValue::from_str("live").unwrap(),
        );
        reqb.headers(headers)
    }

    /// Log the JSON payload for debugging
    fn log_json_payload<T: Serialize + Debug>(&self, req: &T) {
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
}

pub enum RequestData {
    QueryParams,
    PathParam(String),
}
/// Trait for request metadata (similar to your ProvabReqMeta)
pub trait ApiRequestMeta: Sized + Send {
    const METHOD: Method;
    const GZIP: bool = false;
    const DATA: RequestData = RequestData::QueryParams;

    #[cfg(feature = "mock-provab")]
    type Response: DeserializeOwned + Debug + MockableResponse + Send;

    #[cfg(not(feature = "mock-provab"))]
    type Response: DeserializeOwned + Debug + Send;

    /// Deserialize the response from the API
    #[cfg(feature = "ssr")]
    fn deserialize_response(
        response_bytes_or_string: DeserializableInput,
    ) -> ApiClientResult<Self::Response> {
        // Same implementation as your current one
        log!(
            "{}",
            format!(
                "gzip = {} , response_bytes_or_string : {}\n\n\n",
                Self::GZIP,
                format_deserializable_input_for_log(response_bytes_or_string.clone())
            )
            .bright_yellow()
            .bold()
        );

        let decompressed_body = match response_bytes_or_string {
            DeserializableInput::Bytes(body_bytes) => {
                String::from_utf8(body_bytes).map_err(|e| {
                    ApiError::DecompressionFailed(String::from(
                        "Could not convert from bytes to string",
                    ))
                })?
            }
            DeserializableInput::Text(body_string) => body_string,
        };

        let jd = &mut serde_json::Deserializer::from_str(&decompressed_body);
        let res: Self::Response = serde_path_to_error::deserialize(jd).map_err(|e| {
            let total_error = format!("path: {} - inner: {} ", e.path().to_string(), e.inner());
            log!("deserialize_response- JsonParseFailed: {:?}", total_error);
            ApiError::JsonParseFailed(total_error)
        })?;

        Ok(res)
    }
}

#[inline]
fn format_deserializable_input_for_log(input: DeserializableInput) -> String {
    let s = input.to_string();
    cfg_if::cfg_if! {
        if #[cfg(feature = "debug_log")] {
            s
        } else {
            s.chars().take(4000).collect::<String>()
        }
    }
}

/// Trait for request configuration (similar to your ProvabReq)
pub trait ApiRequest: ApiRequestMeta + Debug {
    fn base_path() -> String;

    fn path() -> String {
        join_base_and_path_url(Self::base_path().as_str(), Self::path_suffix())
            .expect("Failed to join base and path URL")
    }

    fn full_path(&self) -> String {
        Self::path()
    }

    /// The path suffix for this request (without the base URL)
    fn path_suffix() -> &'static str;

    fn headers() -> HeaderMap {
        HeaderMap::new()
    }

    /// Custom headers specific to this request type
    fn custom_headers() -> HeaderMap {
        Self::headers()
    }
}

//
// Blanket implementation: any type that implements ProvabReq automatically gets ApiRequest
//
impl<T> ApiRequest for T
where
    T: ProvabReq + ApiRequestMeta + Debug,
{
    fn base_path() -> String {
        T::base_path()
    }

    fn headers() -> HeaderMap {
        let env_var_config = EnvVarConfig::expect_context_or_try_from_env();
        env_var_config.get_headers()
    }

    fn path_suffix() -> &'static str {
        T::path_suffix()
    }

    fn custom_headers() -> HeaderMap {
        T::custom_headers()
    }
}

//
// Blanket implementation: any type that implements ProvabReqMeta automatically gets ApiRequestMeta
//
impl<T> ApiRequestMeta for T
where
    T: ProvabReqMeta + Sized + Send,
{
    const METHOD: Method = T::METHOD;
    const GZIP: bool = T::GZIP;

    #[cfg(feature = "mock-provab")]
    type Response = T::Response;

    #[cfg(not(feature = "mock-provab"))]
    type Response = T::Response;

    fn deserialize_response(
        response_bytes_or_string: DeserializableInput,
    ) -> ApiClientResult<Self::Response> {
        T::deserialize_response(response_bytes_or_string)
    }
}
