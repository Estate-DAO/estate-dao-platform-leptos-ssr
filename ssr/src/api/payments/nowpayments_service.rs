use super::ports::{
    CreateInvoiceRequest, CreateInvoiceResponse, GetPaymentStatusRequest, GetPaymentStatusResponse,
    PaymentGateway, PaymentGatewayParams,
};
use crate::api::consts::{env_w_default, EnvVarConfig};
use crate::cprintln;
use colored::Colorize;
// use leptos::logging::log;
use crate::{error, log, warn};
use leptos::prelude::*;
use reqwest::{IntoUrl, Method, RequestBuilder};
use serde::{Deserialize, Serialize};

cfg_if::cfg_if! {
    if #[cfg(feature = "mock-provab")] {
        // fake imports
        use fake::{Dummy, Fake, Faker};
        use rand::rngs::StdRng;
        use rand::SeedableRng;
    }
}

#[derive(Debug)]
pub struct NowPayments {
    pub api_key: String,
    pub api_host: String,
    pub ipn_secret: String,
    pub client: reqwest::Client,
}

impl NowPayments {
    pub fn new(api_key: String, api_host: String, ipn_secret: String) -> Self {
        Self {
            api_key,
            api_host,
            ipn_secret,
            client: reqwest::Client::default(),
        }
    }

    pub fn try_from_env() -> Self {
        let api_key = std::env::var("NOW_PAYMENTS_USDC_ETHEREUM_API_KEY")
            .expect("NOW_PAYMENTS_USDC_ETHEREUM_API_KEY must be set");

        let api_host = env_w_default("NOWPAYMENTS_API_HOST", "https://api.nowpayments.io").unwrap();

        let ipn_secret = env_w_default("NOWPAYMENTS_IPN_SECRET", "dummy-secret-for-now").unwrap();
        Self::new(api_key, api_host, ipn_secret)
    }

    pub async fn send<Req: PaymentGateway + PaymentGatewayParams + Serialize>(
        &self,
        req: Req,
    ) -> anyhow::Result<Req::PaymentGatewayResponse> {
        let url = req.build_url(&self.api_host)?;
        log!("nowpayments url = {url:#?}");

        let response = self
            .client
            .clone()
            .request(Req::METHOD, url)
            .header("x-api-key", &self.api_key)
            .json(&req)
            .send()
            .await?;

        let body_string = response.text().await?;
        log!("nowpayments reponse = {:#?}", body_string);

        let jd = &mut serde_json::Deserializer::from_str(&body_string);
        let response_struct: Req::PaymentGatewayResponse = serde_path_to_error::deserialize(jd)
            .map_err(|e| {
                let total_error = format!("path: {} - inner: {} ", e.path().to_string(), e.inner());
                error!("deserialize_response- JsonParseFailed: {:?}", total_error);
                e
            })?;

        log!("nowpayments reponse = {response_struct:#?}");
        Ok(response_struct)
    }
}

impl Default for NowPayments {
    fn default() -> Self {
        let env_var_config = EnvVarConfig::expect_context_or_try_from_env();

        NowPayments::new(
            env_var_config.nowpayments_api_key,
            "https://api.nowpayments.io".to_string(),
            env_var_config.ipn_secret,
        )
    }
}

impl PaymentGatewayParams for CreateInvoiceRequest {
    fn path_suffix(&self) -> String {
        "/v1/invoice".to_owned()
    }
}

impl PaymentGateway for CreateInvoiceRequest {
    const METHOD: Method = Method::POST;
    type PaymentGatewayResponse = CreateInvoiceResponse;
}

#[derive(Deserialize, Debug)]
pub struct NowPaymentsPaymentStatusResponse {
    payment_status: String,
}

#[server]
pub async fn nowpayments_create_invoice(
    request: CreateInvoiceRequest,
) -> Result<CreateInvoiceResponse, ServerFnError> {
    // log!("PAYMENT_API: {request:?}");

    let nowpayments = NowPayments::default();
    match nowpayments.send(request).await {
        Ok(response) => Ok(response),
        Err(e) => {
            // log!("server_fn_error: {}", e.to_string());
            Err(ServerFnError::ServerError(e.to_string()))
        }
    }
}

//////////////////////////////
// Get payments status
//////////////////////////////

impl PaymentGatewayParams for GetPaymentStatusRequest {
    fn path_suffix(&self) -> String {
        format!("/v1/payment/{}", self.payment_id)
    }
}

impl PaymentGateway for GetPaymentStatusRequest {
    const METHOD: Method = Method::GET;
    type PaymentGatewayResponse = GetPaymentStatusResponse;
}

// #[server]
pub async fn nowpayments_get_payment_status(
    request: GetPaymentStatusRequest,
) -> Result<GetPaymentStatusResponse, ServerFnError> {
    let nowpayments = NowPayments::default();
    println!("{:#?}", request);
    match nowpayments.send(request).await {
        Ok(response) => {
            // expect_context for counter_tx
            Ok(response)
        }
        Err(e) => Err(ServerFnError::ServerError(e.to_string())),
    }
}

//////////////////////////////
