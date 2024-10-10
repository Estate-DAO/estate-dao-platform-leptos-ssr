use error_stack::{report, Context, Report, ResultExt};
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Client, Url
};
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Response Status not 200")]
    ResponseNotOK(String),
    
    #[error("Provab response error")]
    ResponseError,  

    #[error("HTTP request failed")]
    RequestFailed(#[from] reqwest::Error),
    #[error("JSON parsing failed")]
    JsonParseFailed(#[from] serde_json::Error),
    #[error("Invalid header Value")]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),
    #[error("Invalid header Name")]
    InvalidHeaderName(#[from] reqwest::header::InvalidHeaderName),
}

pub type ApiClientResult<T> = error_stack::Result<T, ApiError>;

pub async fn method_control(
    http_method: &str,
    url_typed: Url,
    body: String,
    headers: HashMap<String, String>,
) -> ApiClientResult<String> {
    let url = url_typed.to_string();
    let json_data = serde_json::from_str(&body)
    .map_err(|e| report!(ApiError::JsonParseFailed(e)))?;

    match http_method {
        // "POST" => post_request(url, json_data, headers).await,
        // "PUT" => put_request(url, json_data, headers).await,
        // "DELETE" => delete_request(url, headers).await,
        // _ => get_request(url, headers).await,
        _ => post_request(url, json_data, headers).await,
    }
}

pub async fn post_request(
    url: String,
    json_data: Value,
    headers: HashMap<String, String>,
) -> ApiClientResult<String> {
    let client = reqwest::Client::builder()
        .gzip(true)
        .build()
        .map_err(|e| report!(ApiError::RequestFailed(e)))?;


    let headers = transform_headers(&headers)?;

    let response = client
        .post(url)
        .headers(headers)
        .json(&json_data)
        .send()
        .await
        .map_err(|e| report!(ApiError::RequestFailed(e)))?;


    let response_body = response.text().await
    .map_err(|e| report!(ApiError::RequestFailed(e)))?;

    Ok(response_body)
}

pub fn transform_headers(headers: &HashMap<String, String>) -> ApiClientResult<HeaderMap> {
    let mut header_map = HeaderMap::new();
    for (key, value) in headers {
        
        let header_name = HeaderName::from_bytes(key.as_bytes())
        .map_err(|e| report!(ApiError::InvalidHeaderName(e)))?;

        let header_value = HeaderValue::from_str(&value)
        .map_err(|e| report!(ApiError::InvalidHeaderValue(e)))?;
        
        header_map.insert(header_name, header_value);
    }
    Ok(header_map)
}



pub async fn get_request(url: String, headers: HashMap<String, String>) -> ApiClientResult<String> {
    let client = Client::new();
    let headers = transform_headers(&headers)?;
    // let formed_client = client.get(url.clone()).headers(headers.clone());
    // warn!("formed_client: {:#?}", formed_client);

    let response = client
        .get(url)
        .headers(headers) // Headers merged here
        .send()
        .await.map_err(|e| report!(ApiError::RequestFailed(e)))?;

    // info!("Response: {:#?}", response);
    let body = response.text().await.map_err(|e| report!(ApiError::RequestFailed(e)))?;
    // info!("Response body: {:#?}", body);

    Ok(body)
}