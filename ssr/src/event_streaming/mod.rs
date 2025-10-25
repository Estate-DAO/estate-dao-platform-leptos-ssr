use std::env;

use gloo_utils::format::JsValueSerdeExt;
use leptos::prelude::*;
use serde::Serialize;
use serde_json::json;
use wasm_bindgen::prelude::*;

// use crate::consts::GTAG_MEASUREMENT_ID;

#[cfg(feature = "ga4")]
use crate::api::consts::GTAG_MEASUREMENT_ID;

pub mod events;

#[derive(Debug, Serialize)]
struct GA4Event {
    client_id: String,
    user_id: Option<String>,
    events: Vec<Event>,
}

#[derive(Debug, Serialize)]
struct Event {
    name: String,
    params: serde_json::Value,
}

// #[cfg(feature = "ssr")]
// pub mod warehouse_events {
//     tonic::include_proto!("warehouse_events");
// }

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = gtag)]
    pub fn gtag(cmd: &str, event_name: &str, params: &JsValue);
}

#[derive(Clone, Default)]
pub struct EventHistory {
    pub event_name: RwSignal<String>,
}

#[cfg(feature = "ga4")]
#[server]
pub async fn send_event_ssr(event_name: String, params: String) -> Result<(), ServerFnError> {
    use crate::utils::host::get_host;

    let params = serde_json::from_str::<serde_json::Value>(&params).unwrap();

    let host_str = get_host();
    let mut params = params.clone();
    params["host"] = json!(host_str);

    if params["page_location"].is_null() {
        params["page_location"] = json!(format!("https://{}", host_str));
    }

    // Warehouse
    // send_event_warehouse(&event_name, &params).await;

    // GA4
    // get client_id as user_id from params
    let user_id = params["user_id"].as_str().unwrap_or("0");
    let res = send_event_ga4(user_id, &event_name, &params).await;

    if let Err(e) = res {
        log::error!("Error sending event to GA4: {:?}", e);
    }

    Ok(())
}

#[cfg(feature = "ga4")]
pub fn send_event_ssr_spawn(event_name: String, params: String) {
    // use leptos::task::spawn_local;
    use leptos::{task::spawn_local, *};

    let mut params = serde_json::from_str::<serde_json::Value>(&params).unwrap();
    params["page_location"] = json!(window().location().href().unwrap().to_string());
    let params = serde_json::to_string(&params).unwrap();

    spawn_local(async move {
        let _ = send_event_ssr(event_name, params).await;
    });
}

#[cfg(feature = "ga4")]
pub fn send_user_id(user_id: String) {
    let gtag_measurement_id = &GTAG_MEASUREMENT_ID;

    gtag(
        "config",
        gtag_measurement_id,
        &JsValue::from_serde(&json!({
            "user_id": user_id,
        }))
        .unwrap(),
    );
}

fn convert_leaf_values_to_string(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(mut obj) => {
            for (_, val) in obj.iter_mut() {
                *val = convert_leaf_values_to_string(val.clone());
            }
            serde_json::Value::Object(obj)
        }
        serde_json::Value::Array(mut arr) => {
            for item in arr.iter_mut() {
                *item = convert_leaf_values_to_string(item.clone());
            }
            serde_json::Value::Array(arr)
        }
        serde_json::Value::Number(n) => serde_json::Value::String(n.to_string()),
        serde_json::Value::Null => serde_json::Value::String("".to_string()),
        serde_json::Value::Bool(value) => serde_json::Value::String(value.to_string()),
        serde_json::Value::String(value) => serde_json::Value::String(value),
    }
}

#[cfg(all(feature = "ga4", feature = "ssr"))]
pub async fn send_event_ga4(
    user_id: &str,
    event_name: &str,
    params: &serde_json::Value,
) -> Result<(), Box<dyn std::error::Error>> {
    use reqwest::Client;

    let measurement_id: &str = &GTAG_MEASUREMENT_ID;
    let api_secret = env::var("GA4_API_SECRET")?;

    let client = Client::new();
    let url = format!(
        "https://www.google-analytics.com/mp/collect?measurement_id={}&api_secret={}",
        measurement_id, api_secret
    );

    let params = convert_leaf_values_to_string(params.clone());

    let payload = GA4Event {
        client_id: "nofeebooking-prod-202504".to_string(), // Should be some unique id
        user_id: Some(user_id.to_string()),
        events: vec![Event {
            name: event_name.to_string(),
            params: params.clone(),
        }],
    };

    let response = client.post(&url).json(&payload).send().await?;

    if !response.status().is_success() {
        return Err(format!("GA4 request failed: {:?}", response.status()).into());
    }

    Ok(())
}
