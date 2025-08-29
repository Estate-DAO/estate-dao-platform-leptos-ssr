use crate::api::api_client::ApiRequestMeta;
use crate::utils::route::join_base_and_path_url;
use reqwest::header::HeaderMap;

/// Common trait for all LiteAPI requests
pub trait LiteApiReq: ApiRequestMeta + std::fmt::Debug {
    fn base_path() -> String {
        "https://api.liteapi.travel/v3.0".to_string()
    }

    fn path() -> String {
        join_base_and_path_url(&Self::base_path(), &Self::path_suffix())
            .expect("Failed to join base and path URL")
    }

    fn path_suffix() -> &'static str;

    fn headers() -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("Accept", "application/json".parse().unwrap());
        headers.insert("Content-Type", "application/json".parse().unwrap());
        headers
    }

    fn custom_headers() -> HeaderMap {
        Self::headers()
    }
}
