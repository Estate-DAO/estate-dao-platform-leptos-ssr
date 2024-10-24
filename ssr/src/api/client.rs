use std::sync::Arc;

use super::{
    consts::{get_headers_from_env, get_provab_base_url_from_env},
    ApiClientResult, ApiError,
};
use error_stack::{report, Report, ResultExt};
use leptos::logging::log;
use reqwest::header::HeaderMap;
use reqwest::{IntoUrl, Method, RequestBuilder, Url};
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;
use std::io::Read;

pub trait ProvabReqMeta: Sized + Send {
    const METHOD: Method;
    const GZIP: bool = true;
    type Response: DeserializeOwned + Debug;

    /// Deserialize the response from the API
    /// The default implementation that assumes the response is JSON encoded [crate::CfSuccessRes]
    /// and extracts the `result` field
    fn deserialize_response(body: String) -> ApiClientResult<Self::Response> {
        use flate2::read::GzDecoder;

        // log!("deserialize_response- body:String : {body:?}\n\n\n");
        let decompressed_body = if Self::GZIP {
            let mut d = GzDecoder::new(body.as_bytes());
            let mut s = String::new();
            d.read_to_string(&mut s).map_err(|e| {
                // log!("\n\ndeserialize_response- DecompressionFailed: {e:?}\n\n");
                report!(ApiError::DecompressionFailed(e.to_string()))
            })?;
            s
        } else {
            body
        };

        let jd = &mut serde_json::Deserializer::from_str(&decompressed_body);
        let res: Self::Response  = serde_path_to_error::deserialize(jd).map_err(|e| {
                log!("deserialize_response- JsonParseFailed: {:?}", e.path().to_string());
                let total_error = format!("path: {} - inner: {} ", e.path().to_string(), e.inner() );
                report!(ApiError::JsonParseFailed( total_error))
            })?;


        // log!("\n\ndeserialize_response- Ok : {res:?}\n\n\n");

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
            client: reqwest::Client::builder()
                // .gzip(true)
                .build()
                .unwrap(),
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
        let reqb = self.req_builder(Req::METHOD, Req::path());
        // log!("reqb - send: {reqb:?}");
        self.send_json(req, reqb).await
    }
}
fn set_gzip_accept_encoding(reqb: RequestBuilder) -> RequestBuilder {
    // let headers = reqb.headers_ref().unwrap();
    // if !headers.contains_key("Accept-Encoding") && !headers.contains_key("Range") {
    reqb.header("Accept-Encoding", "gzip")
    // } else {
    //     reqb
    // }
}

// fn handle_gzip_response(mut response: reqwest::Response) -> reqwest::Response {
//     if response.headers().get("Content-Encoding") == Some(&"gzip".parse().unwrap()) {
//         response.headers_mut().remove("Content-Encoding");
//         response.headers_mut().remove("Content-Length");
//         // Assume response body is automatically decompressed by reqwest
//     }
//     response
// }
