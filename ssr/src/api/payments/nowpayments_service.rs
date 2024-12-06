
use super::ports::{
    CreateInvoiceRequest, CreateInvoiceResponse, GetPaymentStatusRequest, GetPaymentStatusResponse,
    PaymentGateway, PaymentGatewayParams,
};
use crate::api::consts::EnvVarConfig;
use leptos::*;
use reqwest::{IntoUrl, Method, RequestBuilder};
use serde::{Deserialize, Serialize};
use leptos::logging::log;

pub struct NowPayments {
    pub api_key: String,
    pub api_host: String,
    pub client: reqwest::Client,
}

impl NowPayments {
    pub fn new(api_key: String, api_host: String) -> Self {
        Self {
            api_key,
            api_host,
            client: reqwest::Client::default(),
        }
    }

    async fn send<Req: PaymentGateway + PaymentGatewayParams + Serialize>(
        &self,
        req: Req,
    ) -> anyhow::Result<Req::PaymentGatewayResponse> {
        let url = req.build_url(&self.api_host)?;

        let response = self
            .client
            .clone()
            .request(Req::METHOD, url)
            .header("x-api-key", &self.api_key)
            .json(&req)
            .send()
            .await?;

        let response_struct: Req::PaymentGatewayResponse = response.json().await?;
        log!("nowpayments reponse = {response_struct:#?}");
        Ok(response_struct)
    }
}

impl Default for NowPayments {
    fn default() -> Self {
        let env_var_config: EnvVarConfig = expect_context();

        NowPayments::new(
            env_var_config.nowpayments_api_key,
            "https://api.nowpayments.io".to_string(),
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

#[server]
pub async fn nowpayments_get_payment_status(
    request: GetPaymentStatusRequest,
) -> Result<GetPaymentStatusResponse, ServerFnError> {
    let nowpayments = NowPayments::default();
    match nowpayments.send(request).await {
        Ok(response) => Ok(response),
        Err(e) => Err(ServerFnError::ServerError(e.to_string())),
    }
}

//////////////////////////////
