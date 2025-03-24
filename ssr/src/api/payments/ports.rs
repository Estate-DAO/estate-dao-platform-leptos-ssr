use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::canister::backend::BePaymentApiResponse;
use anyhow::{anyhow, Result};
use reqwest::{IntoUrl, Method, RequestBuilder, Url};
use std::fmt::Debug;

cfg_if::cfg_if! {
    if #[cfg(feature = "mock-provab")] {
        // fake imports
        use fake::{Dummy, Fake, Faker};
        use rand::rngs::StdRng;
        use rand::SeedableRng;
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreateInvoiceRequest {
    pub price_amount: u32,
    pub price_currency: String,
    pub order_id: String,
    pub order_description: String,
    pub ipn_callback_url: String,
    pub success_url: String,
    pub cancel_url: String,
    pub partially_paid_url: String,
    pub is_fixed_rate: bool,
    pub is_fee_paid_by_user: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct CreateInvoiceResponse {
    pub id: String,
    pub token_id: String,
    pub order_id: String,
    pub order_description: String,
    pub price_amount: String,
    pub price_currency: String,
    pub pay_currency: Option<String>,
    pub ipn_callback_url: String,
    pub invoice_url: String,
    pub success_url: String,
    pub cancel_url: String,
    pub customer_email: Option<String>,
    pub partially_paid_url: String,
    pub payout_currency: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub is_fixed_rate: bool,
    pub is_fee_paid_by_user: bool,
    pub source: Option<String>,
}

// #[derive(Debug, Clone, PartialEq, Eq)]
// pub enum PaymentStatus {
//     Waiting,
//     Confirming,
//     Confirmed,
//     Sending,
//     PartiallyPaid,
//     Finished,
//     Failed,
//     Refunded,
//     Expired,
// }

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GetPaymentStatusRequest {
    pub payment_id: u64,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct SuccessGetPaymentStatusResponse {
    pub payment_id: u64,
    pub invoice_id: u64,
    pub payment_status: String,
    // pub pay_address: String,
    // pub payin_extra_id: Option<String>,
    pub price_amount: u64,
    pub price_currency: String,
    pub pay_amount: f64,
    pub actually_paid: f64,
    pub pay_currency: String,
    pub order_id: String,
    pub order_description: String,
    pub purchase_id: u64,
    // pub outcome_amount: f64,
    // pub outcome_currency: String,
    // pub payout_hash: Option<String>,
    // pub payin_hash: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    // pub burning_percent: Option<String>,
    // #[serde(rename = "type")]
    // pub type_field: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct FailureGetPaymentStatusResponse {
    status: bool,
    #[serde(rename = "statusCode")]
    status_code: u16,
    code: String,
    message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub enum GetPaymentStatusResponse {
    Success(SuccessGetPaymentStatusResponse),
    Failure(FailureGetPaymentStatusResponse),
}

impl GetPaymentStatusResponse {
    pub fn get_payment_status(&self) -> String {
        match self {
            GetPaymentStatusResponse::Success(success) => success.get_payment_status(),
            GetPaymentStatusResponse::Failure(failure) => failure.get_payment_status(),
        }
    }
}

impl SuccessGetPaymentStatusResponse {
    pub fn get_payment_status(&self) -> String {
        self.payment_status.clone()
    }
}

impl FailureGetPaymentStatusResponse {
    pub fn get_payment_status(&self) -> String {
        "ApiFailure".into()
    }
}

impl From<(GetPaymentStatusResponse, String)> for BePaymentApiResponse {
    fn from((response, provider): (GetPaymentStatusResponse, String)) -> Self {
        match response {
            GetPaymentStatusResponse::Success(response) => Self {
                updated_at: response.updated_at,
                actually_paid: response.actually_paid,
                provider,
                invoice_id: response.invoice_id,
                order_description: response.order_description,
                pay_amount: response.pay_amount,
                pay_currency: response.pay_currency,
                created_at: response.created_at,
                payment_status: response.payment_status,
                price_amount: response.price_amount,
                purchase_id: response.purchase_id,
                order_id: response.order_id,
                price_currency: response.price_currency,
                payment_id: response.payment_id,
            },
            GetPaymentStatusResponse::Failure(failed_resp) => {
                let mut default_response = Self::default();
                default_response.payment_status = format!("FAILED: {}", failed_resp.message);
                default_response.provider = provider;
                default_response
            }
        }
    }
}

pub trait PaymentGateway {
    const METHOD: Method;

    #[cfg(feature = "mock-provab")]
    type PaymentGatewayResponse: DeserializeOwned + Debug + Dummy<Faker>;

    #[cfg(not(feature = "mock-provab"))]
    type PaymentGatewayResponse: DeserializeOwned + Debug;

    // fn get_payment_status(&self, payment_id: &str) -> Result<PaymentStatus, String>;
}
pub trait PaymentGatewayParams {
    fn path_suffix(&self) -> String;

    fn build_url(&self, base_url: &str) -> Result<Url> {
        let path_suffix = self.path_suffix();
        // Parse the base URL first
        let base = Url::parse(base_url).map_err(|e| anyhow!("Invalid base URL: {}", e))?;

        // Join the path suffix, trimming any leading/trailing slashes
        let path = path_suffix.trim_matches('/');

        // Combine the base URL with the path
        base.join(path)
            .map_err(|e| anyhow!("Failed to join URL parts: {}", e))
    }
}
