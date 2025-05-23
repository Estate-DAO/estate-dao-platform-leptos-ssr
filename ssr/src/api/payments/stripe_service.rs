
use chrono::NaiveDate;
// use sha2::digest::block_buffer::Block; // Removed as it's causing E0433 and seems unused
use super::ports::{
    CreateInvoiceRequest, CreateInvoiceResponse, GetPaymentStatusRequest, GetPaymentStatusResponse,
    PaymentGateway, PaymentGatewayParams,
};
use crate::api::consts::{env_w_default, get_payments_url, EnvVarConfig};
use crate::canister::backend::SelectedDateRange as BackendSelectedDateRange;
use crate::component::SelectedDateRange;
use crate::cprintln;
use crate::state::search_state::{HotelInfoResults, SearchCtx, SearchListResults};
use crate::state::view_state::{BlockRoomCtx, HotelInfoCtx};
use crate::utils::app_reference::BookingId;
use crate::utils::booking_id::PaymentIdentifiers;
use colored::Colorize;
use std::{collections::HashMap, fmt};
use thiserror::Error;
// use leptos::logging::log;
use crate::{error, log, warn};
use leptos::*;
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
pub struct StripeEstate {
    pub api_key: String,
    pub api_host: String,
    pub ipn_secret: String,
    pub client: reqwest::Client,
}

impl StripeEstate {
    pub fn new(api_key: String, api_host: String, ipn_secret: String) -> Self {
        Self {
            api_key,
            api_host,
            ipn_secret,
            client: reqwest::Client::default(),
        }
    }

    pub fn try_from_env() -> Self {
        let api_key = std::env::var("STRIPE_SECRET_KEY").expect("STRIPE_SECRET_KEY must be set");

        let api_host = env_w_default("STRIPE_API_HOST", "https://api.stripe.com").unwrap();
        let ipn_secret = env_w_default("STRIPE_WEBHOOK_SECRET", "dummy-secret-for-now").unwrap();

        Self::new(api_key, api_host, ipn_secret)
    }

    pub async fn send<Req: PaymentGateway + PaymentGatewayParams + Serialize>(
        &self,
        req: Req,
    ) -> anyhow::Result<Req::PaymentGatewayResponse> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "mock-provab")] {
                let resp: Req::PaymentGatewayResponse = Faker.fake();
                log!("Faker Response {:?}", resp);
                Ok(resp)
            } else {
                let url = req.build_url(&self.api_host)?;
                log!("stripe url = {url:#?}");

                let response = self
                    .client
                    .clone()
                    .request(Req::METHOD, url)
                    // todo(stripe) change this in prod
                    .basic_auth(&self.api_key, Some(""))
                    .form(&req)
                    // .json(&req)
                    .send()
                    .await?;

                let body_string = response.text().await?;
                log!("stripe response = {:#?}", body_string);

                let jd = &mut serde_json::Deserializer::from_str(&body_string);
                let response_struct: Req::PaymentGatewayResponse = serde_path_to_error::deserialize(jd)
                    .map_err(|e| {
                        let total_error = format!("path: {} - inner: {}", e.path().to_string(), e.inner());
                        error!("deserialize_response- JsonParseFailed: {:?}", total_error);
                        e
                    })?;

                log!("stripe response struct = {response_struct:#?}");
                Ok(response_struct)
            }
        }
    }
}

impl Default for StripeEstate {
    fn default() -> Self {
        let env_var_config: EnvVarConfig = expect_context();

        StripeEstate::new(
            env_var_config.stripe_secret_key,
            "https://api.stripe.com".to_string(),
            env_var_config.stripe_webhook_secret,
        )
    }
}

#[derive(Debug, Serialize)]
/// Error type for metadata validation
#[derive(Error, PartialEq)]
pub enum MetadataValidationError {
    #[error("metadata cannot contain more than 50 key-value pairs")]
    TooManyKeys,
    #[error("key '{0}' is too long (max 40 characters)")]
    KeyTooLong(String),
    #[error("value for key '{0}' is too long (max 500 characters)")]
    ValueTooLong(String),
    #[error("key '{0}' contains invalid characters (square brackets are not allowed)")]
    InvalidKeyCharacters(String),
}

/// Validated metadata for Stripe checkout session
#[derive(Debug, Clone, Default)]
pub struct StripeMetadata(HashMap<String, String>);

impl StripeMetadata {
    /// Creates a new StripeMetadata instance after validation
    pub fn new(metadata: HashMap<String, String>) -> Result<Self, MetadataValidationError> {
        // Check number of keys
        if metadata.len() > 50 {
            return Err(MetadataValidationError::TooManyKeys);
        }

        // Validate each key-value pair
        for (key, value) in &metadata {
            // Check key length
            if key.len() > 40 {
                return Err(MetadataValidationError::KeyTooLong(key.clone()));
            }

            // Check for invalid characters in key
            if key.contains('[') || key.contains(']') {
                return Err(MetadataValidationError::InvalidKeyCharacters(key.clone()));
            }

            // Check value length
            if value.len() > 500 {
                return Err(MetadataValidationError::ValueTooLong(key.clone()));
            }
        }

        Ok(StripeMetadata(metadata))
    }

    /// Get a reference to the inner HashMap
    pub fn inner(&self) -> &HashMap<String, String> {
        &self.0
    }

    /// Consume self and return the inner HashMap
    pub fn into_inner(self) -> HashMap<String, String> {
        self.0
    }
}

impl serde::Serialize for StripeMetadata {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for StripeMetadata {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let map = HashMap::<String, String>::deserialize(deserializer)?;
        StripeMetadata::new(map).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Serialize)]
pub struct StripeCreateCheckoutSession {
    pub success_url: String,
    pub cancel_url: String,
    pub line_items: Vec<StripeLineItem>,
    // mod=payment
    #[serde(default = "default_payment_mode")]
    pub mode: String,
    /// You can specify up to 50 keys, with key names up to 40 characters long and values up to 500 characters long.
    /// Keys and values are stored as strings and can contain any characters
    /// with one exception: you can't use square brackets ([ and ]) in keys.
    pub metadata: Option<StripeMetadata>,
    /// this is order_id resp-encoded
    pub client_reference_id: String,
    /// we have this during hotel booking
    pub customer_email: String,
    pub ui_mode: StripeUIModeEnum,
}
#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum StripeUIModeEnum {
    #[default]
    Hosted,
    Embedded,
    Custom,
}

fn default_payment_mode() -> String {
    "payment".to_string()
}

#[derive(Debug, Serialize)]
pub struct StripeLineItem {
    pub price_data: StripePriceData,
    pub quantity: u32,
}

#[derive(Debug, Serialize)]
pub struct StripePriceData {
    pub currency: String,
    pub product_data: StripeProductData,
    pub unit_amount: u32,
}

#[derive(Debug, Serialize)]
pub struct StripeProductData {
    pub name: String,
    /// This is what is displayed to the user in the checkout page
    pub description: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
}

/// Builder for creating a well-formatted product description for Stripe checkout
#[derive(Debug, Clone)]
pub struct StripeProductDescription {
    hotel_name: String,
    hotel_location: String,
    date_range: SelectedDateRange,
    total_price: f64,
    user_email: String,
    user_phone: String,
    user_name: String,
    num_adults: u32,
    num_children: u32,
}

impl StripeProductDescription {
    pub fn new(
        hotel_name: impl Into<String>,
        location: impl Into<String>,
        date_range: SelectedDateRange,
        total_price: f64,
        user_email: impl Into<String>,
        user_phone: impl Into<String>,
        user_name: impl Into<String>,
        num_adults: u32,
        num_children: u32,
    ) -> Self {
        Self {
            hotel_name: hotel_name.into(),
            hotel_location: location.into(),
            date_range,
            total_price,
            user_email: user_email.into(),
            user_phone: user_phone.into(),
            user_name: user_name.into(),
            num_adults,
            num_children,
        }
    }
}

impl fmt::Display for StripeProductDescription {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Hotel Details:")?;
        writeln!(f, "- Hotel: {}", self.hotel_name)?;
        writeln!(f, "- Location: {}", self.hotel_location)?;
        writeln!(f, "- Dates: {}", self.date_range.to_string())?;
        writeln!(f, "- Total: ${:.2}", self.total_price)?;
        writeln!(f)?;
        writeln!(f, "Guest Details:")?;
        writeln!(f, "- Name: {}", self.user_name)?;
        writeln!(f, "- Email: {}", self.user_email)?;
        writeln!(f, "- Phone: {}", self.user_phone)?;
        write!(f, "- Guests: {} adult(s)", self.num_adults)?;

        if self.num_children > 0 {
            write!(f, ", {} child(ren)", self.num_children)?;
        }

        Ok(())
    }
}

impl StripeProductData {
    /// Creates a new StripeProductData with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            metadata: None,
        }
    }

    /// Sets the description using a StripeProductDescription
    pub fn with_description(self, description: impl fmt::Display) -> Self {
        Self {
            description: Some(description.to_string()),
            ..self
        }
    }

    /// Adds metadata to the product data
    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

impl PaymentGatewayParams for StripeCreateCheckoutSession {
    fn path_suffix(&self) -> String {
        "/v1/checkout/sessions".to_owned()
    }
}

impl PaymentGateway for StripeCreateCheckoutSession {
    const METHOD: Method = Method::POST;
    type PaymentGatewayResponse = StripeCreateCheckoutSessionResponse;
}

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct StripeCreateCheckoutSessionResponse {
    id: String, // Session ID
    /// Checkout URL - this is the one we will use to redirect the user    
    url: String,
    amount_total: i64,
    currency: String,
    metadata: HashMap<String, String>,
    /// stripe's notation of payment_status
    payment_status: StripePaymentStatusEnum,
    status: StripeCheckoutSessionStatusEnum,
    /// that we had set in request
    success_url: String,
    /// that we had set in request
    cancel_url: Option<String>,
    customer_email: Option<String>,
    created: u64,
    // expires_at: u64,
    client_reference_id: String,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
#[serde(rename_all = "snake_case")]
/// Use this status to fullfill the order
pub enum StripePaymentStatusEnum {
    Unpaid,
    Paid,
    NoPaymentRequired,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
#[serde(rename_all = "snake_case")]
pub enum StripeCheckoutSessionStatusEnum {
    /// The checkout session is complete. Payment processing may still be in progress
    Complete,
    /// The checkout session has expired. No further processing will occur
    Expired,
    /// The checkout session is still in progress. Payment processing has not started
    Open,
}

impl From<StripeCreateCheckoutSessionResponse> for CreateInvoiceResponse {
    fn from(response: StripeCreateCheckoutSessionResponse) -> Self {
        CreateInvoiceResponse {
            id: response.id,
            token_id: String::new(), // Not available in Stripe response
            order_id: response.client_reference_id,
            order_description: String::new(), // Will be set by the caller
            price_amount: (response.amount_total as f64).to_string(),
            price_currency: response.currency,
            pay_currency: None,
            ipn_callback_url: String::new(), // Will be set by the caller
            invoice_url: response.url,
            success_url: response.success_url,
            cancel_url: response.cancel_url.unwrap_or_default(),
            customer_email: response.customer_email,
            partially_paid_url: String::new(), // Not applicable for Stripe
            payout_currency: None,
            created_at: response.created.to_string(),
            updated_at: response.created.to_string(), // Same as created for new session
            is_fixed_rate: true,                      // Stripe handles currency conversion
            is_fee_paid_by_user: false,               // Default to merchant pays fees
            source: Some("stripe".to_string()),
        }
    }
}

/// Used on block_room page to crate a checkout url to which the user will be redirected to complete the payment
pub fn create_stripe_checkout_session(
    total_price: f64,
) -> Result<StripeCreateCheckoutSession, Box<dyn std::error::Error>> {
    let email = BlockRoomCtx::get_email_untracked();
    let booking_id = BookingId::get_backend_compatible_booking_id_untracked(email.clone());

    let order_id =
        PaymentIdentifiers::order_id_from_app_reference(&booking_id.app_reference, &email);

    let destination = SearchCtx::get_backend_compatible_destination_untracked();
    let date_range = SearchCtx::get_backend_compatible_date_range_untracked();
    let hotel_code = HotelInfoCtx::get_hotel_code_untracked();
    let hotel_token = SearchListResults::get_result_token(hotel_code.clone());

    let hotel_name = HotelInfoResults::get_hotel_name_untracked();
    let hotel_location = HotelInfoResults::get_hotel_location_untracked();

    let user_phone = BlockRoomCtx::get_user_phone_untracked();
    let user_name = BlockRoomCtx::get_user_name_untracked();

    let num_adults = BlockRoomCtx::get_num_adults_untracked();
    let num_children = BlockRoomCtx::get_num_children_untracked();

    let stripe_product_description = StripeProductDescription::new(
        hotel_name,
        hotel_location,
        date_range.into(),
        total_price,
        email.clone(),
        user_phone,
        user_name,
        num_adults,
        num_children,
    );

    Ok(StripeCreateCheckoutSession {
        success_url: get_payments_url("success"),
        cancel_url: get_payments_url("cancel"),
        line_items: vec![],
        mode: "payment".to_string(),
        client_reference_id: order_id,
        customer_email: email.clone(),
        ui_mode: StripeUIModeEnum::Hosted,
        metadata: Some(StripeMetadata(HashMap::new())),
    })
}
