use serde::{de::DeserializeOwned, Deserialize, Serialize};

use anyhow::{anyhow, Result};
use reqwest::{IntoUrl, Method, RequestBuilder, Url};
use std::fmt::Debug;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaymentStatus {
    Waiting,
    Confirming,
    Confirmed,
    Sending,
    PartiallyPaid,
    Finished,
    Failed,
    Refunded,
    Expired,
}

// #[async_trait(?Send)]
pub trait PaymentGateway {
    const METHOD: Method;

    type PaymentGatewayResponse: DeserializeOwned + Debug;

    // fn get_payment_status(&self, payment_id: &str) -> Result<PaymentStatus, String>;
}
pub trait PaymentGatewayParams {
    fn path_suffix() -> String;

    fn build_url(base_url: &str) -> Result<Url> {
        let path_suffix = Self::path_suffix();
        // Parse the base URL first
        let base = Url::parse(base_url).map_err(|e| anyhow!("Invalid base URL: {}", e))?;

        // Join the path suffix, trimming any leading/trailing slashes
        let path = path_suffix.trim_matches('/');

        // Combine the base URL with the path
        base.join(path)
            .map_err(|e| anyhow!("Failed to join URL parts: {}", e))
    }
}
