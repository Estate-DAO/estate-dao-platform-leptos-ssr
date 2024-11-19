use reqwest::{IntoUrl, Method, RequestBuilder};
use crate::api::consts::EnvVarConfig;
use super::ports::{CreateInvoiceRequest, CreateInvoiceResponse, PaymentGateway, PaymentGatewayParams, PaymentStatus};
use leptos::*;
use serde::{Deserialize, Serialize};

pub struct NowPayments {
    pub api_key: String,
    pub api_host: String,
    pub client: reqwest::Client
}

impl NowPayments {
    pub fn new(api_key: String, api_host: String) -> Self {
        Self { api_key, api_host, client: reqwest::Client::default() }
    }

    async fn send<Req: PaymentGateway + PaymentGatewayParams + Serialize>(&self, req: Req) -> anyhow::Result<Req::PaymentGatewayResponse>{
        let url = Req::build_url(&self.api_host)?;

        let response = self.client
            .clone()
            .request(Req::METHOD, url)
            .header("x-api-key", &self.api_key)
            .json(&req)
            .send()
            .await?;
        
        let response_struct: Req::PaymentGatewayResponse = response.json().await?;
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
 
 
    // fn get_payment_status(&self, payment_id: &str) -> Result<PaymentStatus, String> {
        // let client = reqwest::Client::new();
        // let url = format!("{}/v1/payment/{}", self.api_host, payment_id);

        // let response = client
        //     .get(url)
        //     .header("x-api-key", &self.api_key)
        //     .send()
        //     .await
        //     .map_err(|e| e.to_string())?;

        // // Parse response and determine PaymentStatus
        // // Placeholder implementation; replace with actual logic
        // if response.status().is_success() {
        //     let payment_status_response: NowPaymentsPaymentStatusResponse =
        //         response.json().await.map_err(|e| e.to_string())?;

        //     let status = match payment_status_response.payment_status.as_str() {
        //         "waiting" => PaymentStatus::Waiting,
        //         "confirming" => PaymentStatus::Confirming,
        //         "confirmed" => PaymentStatus::Confirmed,
        //         "sending" => PaymentStatus::Sending,
        //         "partially_paid" => PaymentStatus::PartiallyPaid,
        //         "finished" => PaymentStatus::Finished,
        //         "refunded" => PaymentStatus::Refunded,
        //         "expired" => PaymentStatus::Expired,
        //         _ => PaymentStatus::Failed,
        //     };
        //     Ok(status)
        // } else {
        //     Err(format!(
        //         "Failed to get payment status: {}",
        //         response.status()
        //     ))
        // }



impl PaymentGatewayParams for  CreateInvoiceRequest{

    fn path_suffix() -> String {
        "/v1/invoice".to_owned()
    }
}

impl PaymentGateway for CreateInvoiceRequest{
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

let nowpayments = NowPayments::default() ;
    match nowpayments.send(request).await {
        Ok(response) => Ok(response),
        Err(e) => {
            // log!("server_fn_error: {}", e.to_string());
            Err(ServerFnError::ServerError(e.to_string()))
        }
    }
}
