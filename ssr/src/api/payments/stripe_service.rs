use chrono::NaiveDate;
#[cfg(feature = "ssr")]
use tracing::instrument;
// use sha2::digest::block_buffer::Block; // Removed as it's causing E0433 and seems unused
use super::ports::{
    CreateInvoiceRequest, CreateInvoiceResponse, GetPaymentStatusRequest, GetPaymentStatusResponse,
    PaymentGateway, PaymentGatewayParams,
};
use crate::api::consts::{
    env_w_default, get_payments_url, get_payments_url_v2, EnvVarConfig, PaymentProvider,
};
use crate::canister::backend::SelectedDateRange as BackendSelectedDateRange;
use crate::component::SelectedDateRange;
use crate::cprintln;
use crate::utils::app_reference::BookingId;
use crate::utils::booking_id::PaymentIdentifiers;
use crate::view_state_layer::ui_search_state::{
    // HotelInfoResults, SearchCtx, -- temporarily commented out to fix compilation
    SearchListResults,
};
use crate::view_state_layer::view_state::{BlockRoomCtx, HotelInfoCtx};
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

    #[cfg(feature = "ssr")]
    #[instrument(skip(self))]
    pub async fn send<Req: PaymentGateway + PaymentGatewayParams + Serialize + std::fmt::Debug>(
        &self,
        req: Req,
    ) -> anyhow::Result<Req::PaymentGatewayResponse> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "mock-provab")] {
                let resp: Req::PaymentGatewayResponse = Faker.fake();
                log!("[Stripe] Faker Response {:?}", resp);
                Ok(resp)
            } else {
                let url = req.build_url(&self.api_host)?;
                log!("[Stripe] url = {url:#?}");
                // Log serialized form data for debugging
                match serde_urlencoded::to_string(&req) {
                    Ok(form_str) => {
                        log!("[Stripe] Serialized form data: {}", form_str);
                        // Check specifically for line_items
                        if form_str.contains("line_items") {
                            log!("[Stripe] ✅ line_items found in serialized data");
                        } else {
                            error!("[Stripe] ❌ line_items MISSING from serialized data!");
                        }
                    },
                    Err(e) => error!("[Stripe] Failed to serialize form data with serde_urlencoded: {:?}", e),
                }

                let request = self
                    .client
                    .clone()
                    .request(Req::METHOD, url)
                    // todo(stripe) change this in prod
                    .basic_auth(&self.api_key, Some(""))
                    .form(&req);

                // log!("[Stripe] Request Headers = {:#?}", request.headers());

                let response = request.send().await?;

                let response_status = response.status();
                log!("[Stripe] Response Status = {}", response_status);
                let response_text_value = response.text().await?;

                if !response_status.is_success() {
                    log!("[Stripe] Error Response = {:#?}", response_text_value);
                }

                let body_string = response_text_value;
                log!("[Stripe] stripe response = {:#?}", body_string);

                let jd = &mut serde_json::Deserializer::from_str(&body_string);
                let response_struct: Req::PaymentGatewayResponse = serde_path_to_error::deserialize(jd)
                    .map_err(|e| {
                        let total_error = format!("path: {} - inner: {}", e.path().to_string(), e.inner());
                        error!("[Stripe] deserialize_response- JsonParseFailed: {:?}", total_error);
                        e
                    })?;

                log!("[Stripe] stripe response struct = {response_struct:#?}");
                Ok(response_struct)
            }
        }
    }
}

impl Default for StripeEstate {
    fn default() -> Self {
        let env_var_config = EnvVarConfig::expect_context_or_try_from_env();

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StripeCreateCheckoutSession {
    pub success_url: String,
    pub cancel_url: String,
    #[serde(skip)]
    pub line_items: Vec<StripeLineItem>,
    // mod=payment
    #[serde(default = "default_payment_mode")]
    pub mode: String,
    /// You can specify up to 50 keys, with key names up to 40 characters long and values up to 500 characters long.
    /// Keys and values are stored as strings and can contain any characters
    /// with one exception: you can't use square brackets ([ and ]) in keys.
    #[serde(skip)]
    pub metadata: Option<StripeMetadata>,
    /// this is order_id resp-encoded
    pub client_reference_id: String,
    /// we have this during hotel booking
    pub customer_email: String,
    pub ui_mode: StripeUIModeEnum,
    #[serde(flatten)]
    pub form_fields: HashMap<String, String>,
}

fn build_form_fields(
    line_items_data: &Vec<StripeLineItem>, // Changed name to match struct field
    metadata_data: Option<&StripeMetadata>, // Changed name to match struct field
) -> HashMap<String, String> {
    let mut fields = HashMap::new();

    // --- Line Items ---
    // This part remains the same as it correctly processes the Vec<StripeLineItem>
    for (idx, item) in line_items_data.iter().enumerate() {
        let line_item_base_key = format!("line_items[{}]", idx);

        // Price Data for the current line item
        let price_data_prefix = format!("{}[price_data]", line_item_base_key);
        fields.insert(
            format!("{}[currency]", price_data_prefix),
            item.price_data.currency.clone(),
        );
        // Ensure item.price_data.unit_amount is i64 (integer cents)
        fields.insert(
            format!("{}[unit_amount]", price_data_prefix), // Using unit_amount
            item.price_data.unit_amount.to_string(),
        );

        // Product Data for the current line item's price data
        let product_data_prefix = format!("{}[product_data]", price_data_prefix);
        fields.insert(
            format!("{}[name]", product_data_prefix),
            item.price_data.product_data.name.clone(),
        );
        if let Some(desc) = &item.price_data.product_data.description {
            fields.insert(
                format!("{}[description]", product_data_prefix),
                desc.clone(),
            );
        }
        // Optional: Handling metadata within product_data (if your StripeProductData has it)
        if let Some(prod_meta) = &item.price_data.product_data.metadata {
            for (meta_key, meta_val) in prod_meta.iter() {
                fields.insert(
                    format!("{}[metadata][{}]", product_data_prefix, meta_key),
                    meta_val.clone(),
                );
            }
        }

        // Quantity for the current line item
        fields.insert(
            format!("{}[quantity]", line_item_base_key),
            item.quantity.to_string(),
        );
    }

    // --- Top-Level Metadata ---
    // This part remains the same as it correctly processes Option<StripeMetadata>
    if let Some(stripe_metadata) = metadata_data {
        // StripeMetadata is a newtype `StripeMetadata(HashMap<String, String>)`
        // So, access the inner HashMap using .0
        for (key, value) in stripe_metadata.0.iter() {
            // Stripe's form encoding for metadata is metadata[key]=value
            fields.insert(format!("metadata[{}]", key), value.clone());
        }
    }

    // Note: Fields like success_url, cancel_url, mode, client_reference_id, customer_email, ui_mode
    // are part of the main StripeCreateCheckoutSession struct and will be serialized directly
    // by serde_urlencoded because they are not marked #[serde(skip)].
    // The #[serde(flatten)] on form_fields will merge this map with those direct fields.

    fields
}

impl StripeCreateCheckoutSession {
    pub fn new(
        success_url: String,
        cancel_url: String,
        line_items_vec: Vec<StripeLineItem>, // Renamed to avoid confusion with field name
        mode: String,
        optional_metadata: Option<StripeMetadata>, // Renamed
        client_reference_id: String,
        customer_email: String,
        ui_mode: StripeUIModeEnum,
    ) -> Self {
        // Call build_form_fields to populate the map
        log!(
            "[Stripe] Constructor: Processing {} line items",
            line_items_vec.len()
        );
        let generated_form_fields = build_form_fields(&line_items_vec, optional_metadata.as_ref());
        log!(
            "[Stripe] Constructor: Generated {} form fields",
            generated_form_fields.len()
        );

        // Log first few form fields for debugging
        for (key, value) in generated_form_fields.iter().take(5) {
            log!("[Stripe] Form field: {}={}", key, value);
        }

        Self {
            success_url,
            cancel_url,
            line_items: line_items_vec, // Store the original data (it's skipped for serialization)
            mode,                       // This will be serialized directly
            metadata: optional_metadata, // Store the original data (skipped for serialization)
            client_reference_id,        // This will be serialized directly
            customer_email,             // This will be serialized directly
            ui_mode,                    // This will be serialized directly
            form_fields: generated_form_fields, // This map will be flattened
        }
    }
}

#[derive(Debug, Serialize, Default, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StripeLineItem {
    pub price_data: StripePriceData,
    pub quantity: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StripePriceData {
    pub currency: String,
    pub product_data: StripeProductData,
    // pub unit_amount_decimal: f64,
    pub unit_amount: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

impl From<StripeProductDescription> for StripeProductData {
    fn from(desc: StripeProductDescription) -> Self {
        let desc_clone = desc.clone();
        let mut metadata = HashMap::new();
        metadata.insert("hotel_location".to_string(), desc.hotel_location);
        metadata.insert("date_range".to_string(), desc.date_range.to_string());
        metadata.insert("user_email".to_string(), desc.user_email);
        metadata.insert("user_phone".to_string(), desc.user_phone);
        metadata.insert("user_name".to_string(), desc.user_name);
        metadata.insert("num_adults".to_string(), desc.num_adults.to_string());
        metadata.insert("num_children".to_string(), desc.num_children.to_string());
        metadata.insert("total_price".to_string(), desc.total_price.to_string());

        Self {
            name: desc.hotel_name.clone(),
            description: Some(desc_clone.displayable_description()),
            metadata: Some(metadata),
        }
    }
}

impl StripeProductDescription {
    /// Returns a simple text description with hotel, location, dates and phone
    pub fn displayable_description(&self) -> String {
        format!(
            "{} at {}, during {} \nPhone: {}",
            self.hotel_name,
            self.hotel_location,
            self.date_range.to_string(),
            self.user_phone
        )
    }

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

#[derive(Debug, Deserialize, Serialize, Clone)]
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

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
#[serde(rename_all = "snake_case")]
/// Use this status to fullfill the order
pub enum StripePaymentStatusEnum {
    Unpaid,
    Paid,
    NoPaymentRequired,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
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

/// Stripe checkout session retrieval request
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StripeGetCheckoutSession {
    #[serde(skip)]
    pub session_id: String,
}

impl PaymentGatewayParams for StripeGetCheckoutSession {
    fn path_suffix(&self) -> String {
        format!("/v1/checkout/sessions/{}", self.session_id)
    }
}

impl PaymentGateway for StripeGetCheckoutSession {
    const METHOD: Method = Method::GET;
    type PaymentGatewayResponse = StripeGetCheckoutSessionResponse;
}

/// Stripe checkout session retrieval response
#[derive(Debug, Deserialize, Serialize, Clone)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct StripeGetCheckoutSessionResponse {
    pub id: String,
    pub object: String,
    pub amount_subtotal: Option<i64>,
    pub amount_total: Option<i64>,
    pub currency: Option<String>,
    pub customer_email: Option<String>,
    pub payment_status: StripePaymentStatusEnum,
    pub status: StripeCheckoutSessionStatusEnum,
    pub metadata: HashMap<String, String>,
    pub client_reference_id: Option<String>,
    pub created: u64,
    pub expires_at: Option<u64>,
    pub success_url: Option<String>,
    pub cancel_url: Option<String>,
}

impl From<StripePaymentStatusEnum> for crate::api::payments::domain::PaymentStatus {
    fn from(stripe_status: StripePaymentStatusEnum) -> Self {
        match stripe_status {
            StripePaymentStatusEnum::Unpaid => crate::api::payments::domain::PaymentStatus::Pending,
            StripePaymentStatusEnum::Paid => crate::api::payments::domain::PaymentStatus::Completed,
            StripePaymentStatusEnum::NoPaymentRequired => {
                crate::api::payments::domain::PaymentStatus::Completed
            }
        }
    }
}

impl From<StripeCheckoutSessionStatusEnum> for crate::api::payments::domain::PaymentStatus {
    fn from(session_status: StripeCheckoutSessionStatusEnum) -> Self {
        match session_status {
            StripeCheckoutSessionStatusEnum::Open => {
                crate::api::payments::domain::PaymentStatus::Pending
            }
            StripeCheckoutSessionStatusEnum::Complete => {
                crate::api::payments::domain::PaymentStatus::Completed
            }
            StripeCheckoutSessionStatusEnum::Expired => {
                crate::api::payments::domain::PaymentStatus::Expired
            }
        }
    }
}

// #[server]
pub async fn stripe_create_invoice(
    request: String,
) -> Result<StripeCreateCheckoutSessionResponse, ServerFnError> {
    log!("[Stripe] PAYMENT_API: {request:?}");

    let request: StripeCreateCheckoutSession = serde_json::from_str(&request)
        .expect(format!("Failed to parse request. Got: {}", request).as_str());

    let stripe = StripeEstate::default();
    match stripe.send(request).await {
        Ok(response) => Ok(response),
        Err(e) => Err(ServerFnError::ServerError(e.to_string())),
    }
}

// #[server]
pub async fn stripe_get_session_status(
    session_id: String,
) -> Result<StripeGetCheckoutSessionResponse, ServerFnError> {
    log!("[Stripe] GET_CHECKOUT_SESSION: {session_id:?}");

    let request = StripeGetCheckoutSession { session_id };

    let stripe = StripeEstate::default();
    match stripe.send(request).await {
        Ok(response) => Ok(response),
        Err(e) => Err(ServerFnError::ServerError(e.to_string())),
    }
}

// TODO: Re-enable this unified server function after fixing import issues
// We don't need this for V2 implementation as we use PaymentServiceImpl directly in payment_handler.rs

// Unified server function to get payment status from any payment provider
/*
pub async fn get_payment_status(
    payment_id: String,
    provider: Option<String>, // Optional provider hint ("stripe" or "nowpayments")
) -> Result<crate::api::payments::domain::DomainGetPaymentStatusResponse, ServerFnError> {
    use crate::api::payments::domain::{
        DomainGetPaymentStatusRequest, PaymentProvider, PaymentService,
    };
    use crate::api::payments::service::PaymentServiceImpl;

    log!(
        "[Payment] GET_STATUS: payment_id={}, provider={:?}",
        payment_id,
        provider
    );

    // Convert optional string provider to PaymentProvider enum
    let provider_enum = provider.as_ref().and_then(|p| match p.as_str() {
        "stripe" => Some(PaymentProvider::Stripe),
        "nowpayments" => Some(PaymentProvider::NowPayments),
        _ => None,
    });

    let request = DomainGetPaymentStatusRequest {
        payment_id,
        provider: provider_enum,
    };

    let payment_service = PaymentServiceImpl::new();
    match payment_service.get_payment_status(request).await {
        Ok(response) => Ok(response),
        Err(e) => Err(ServerFnError::ServerError(e.to_string())),
    }
}
*/

/// Used on block_room page to crate a checkout url to which the user will be redirected to complete the payment
pub fn create_stripe_checkout_session(
    total_price: f64,
) -> Result<StripeCreateCheckoutSession, Box<dyn std::error::Error>> {
    let email = BlockRoomCtx::get_email_untracked();
    let booking_id = BookingId::get_backend_compatible_booking_id_untracked(email.clone());

    let order_id =
        PaymentIdentifiers::order_id_from_app_reference(&booking_id.app_reference, &email);

    // TODO: Fix these context imports - commented out for compilation
    // let destination = UISearchCtx::get_backend_compatible_destination_untracked();
    // let date_range = UISearchCtx::get_backend_compatible_date_range_untracked();
    let hotel_code = HotelInfoCtx::get_hotel_code_untracked();
    let hotel_token = SearchListResults::get_result_token(hotel_code.clone());

    // let hotel_name = HotelInfoResults::get_hotel_name_untracked();
    // let hotel_location = HotelInfoResults::get_hotel_location_untracked();
    let hotel_name = "Hotel Name".to_string(); // Temporary placeholder
    let hotel_location = "Hotel Location".to_string(); // Temporary placeholder

    // Temporary placeholder for date range
    let date_range = SelectedDateRange::default();

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

    let stripe_line_item = StripeLineItem {
        price_data: StripePriceData {
            currency: "usd".to_string(),
            product_data: stripe_product_description.into(),
            unit_amount: (total_price * 100.0).round() as u32,
        },
        quantity: 1,
    };

    Ok(StripeCreateCheckoutSession::new(
        get_payments_url_v2("success", PaymentProvider::Stripe),
        get_payments_url_v2("cancel", PaymentProvider::Stripe),
        vec![stripe_line_item],
        "payment".to_string(),
        Some(StripeMetadata(HashMap::new())),
        order_id,
        email.clone(),
        StripeUIModeEnum::Hosted,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_stripe_checkout_session_form_serialization() {
        // Arrange: Create test data
        let line_item = StripeLineItem {
            price_data: StripePriceData {
                currency: "USD".to_string(),
                unit_amount: 18400,
                product_data: StripeProductData {
                    name: "Hotel Test".to_string(),
                    description: Some("Test booking".to_string()),
                    metadata: None,
                },
            },
            quantity: 1,
        };

        let mut metadata_map = HashMap::new();
        metadata_map.insert("order_id".to_string(), "test_order_123".to_string());
        let metadata = StripeMetadata::new(metadata_map).unwrap();

        let session = StripeCreateCheckoutSession::new(
            "https://example.com/success".to_string(),
            "https://example.com/cancel".to_string(),
            vec![line_item],
            "payment".to_string(),
            Some(metadata),
            "test_order_123".to_string(),
            "test@example.com".to_string(),
            StripeUIModeEnum::Hosted,
        );

        // Act: Serialize to form data
        let form_data = serde_urlencoded::to_string(&session);

        // Assert: Check expected format
        if let Err(e) = &form_data {
            println!("Serialization error: {:?}", e);
        }
        assert!(
            form_data.is_ok(),
            "Serialization should succeed, but got error: {:?}",
            form_data.err()
        );
        let form_string = form_data.unwrap();

        println!("Form data: {}", form_string);

        // Assert line items
        assert!(
            form_string.contains("line_items%5B0%5D%5Bprice_data%5D%5Bcurrency%5D=USD")
                || form_string.contains("line_items[0][price_data][currency]=USD")
        );
        assert!(
            form_string.contains("line_items%5B0%5D%5Bprice_data%5D%5Bunit_amount%5D=18400")
                || form_string.contains("line_items[0][price_data][unit_amount]=18400")
        );
        assert!(
            form_string.contains("line_items%5B0%5D%5Bquantity%5D=1")
                || form_string.contains("line_items[0][quantity]=1")
        );

        // Assert metadata
        assert!(
            form_string.contains("metadata%5Border_id%5D=test_order_123")
                || form_string.contains("metadata[order_id]=test_order_123")
        );

        // Assert client reference ID
        assert!(form_string.contains("client_reference_id=test_order_123"));

        // Assert customer email
        assert!(form_string.contains("customer_email=test%40example.com"));
    }

    #[test]
    fn test_stripe_checkout_session_form_serialization_with_actual_client_id() {
        // Arrange: Create test data
        let line_item = StripeLineItem {
            price_data: StripePriceData {
                currency: "USD".to_string(),
                unit_amount: 18400,
                product_data: StripeProductData {
                    name: "Hotel Test".to_string(),
                    description: Some("Test booking".to_string()),
                    metadata: None,
                },
            },
            quantity: 1,
        };

        let mut metadata_map = HashMap::new();
        metadata_map.insert("order_id".to_string(), "test_order_123".to_string());
        let metadata = StripeMetadata::new(metadata_map).unwrap();

        let session = StripeCreateCheckoutSession::new(
            "https://example.com/success".to_string(),
            "https://example.com/cancel".to_string(),
            vec![line_item],
            "payment".to_string(),
            Some(metadata),
            "NP$6:ABC123$34:tripathi.abhishek.iitkgp@gmail.com".to_string(),
            "tripathi.abhishek.iitkgp@gmail.com".to_string(),
            StripeUIModeEnum::Hosted,
        );

        // Act: Serialize to form data
        let form_data = serde_urlencoded::to_string(&session);

        // Assert: Check expected format
        assert!(form_data.is_ok(), "Serialization should succeed");
        let form_string = form_data.unwrap();

        println!("Form data: {}", form_string);

        // Assert line items
        assert!(
            form_string.contains("line_items%5B0%5D%5Bprice_data%5D%5Bcurrency%5D=USD")
                || form_string.contains("line_items[0][price_data][currency]=USD")
        );
        assert!(
            form_string.contains("line_items%5B0%5D%5Bprice_data%5D%5Bunit_amount%5D=18400")
                || form_string.contains("line_items[0][price_data][unit_amount]=18400")
        );
        assert!(
            form_string.contains("line_items%5B0%5D%5Bquantity%5D=1")
                || form_string.contains("line_items[0][quantity]=1")
        );

        // Assert metadata
        assert!(
            form_string.contains("metadata%5Border_id%5D=test_order_123")
                || form_string.contains("metadata[order_id]=test_order_123")
        );

        // Assert client reference ID
        assert!(
            form_string.contains(
                "client_reference_id=NP%246%3AABC123%2434%3Atripathi.abhishek.iitkgp%40gmail.com"
            ) || form_string
                .contains("client_reference_id=NP$6:ABC123$34:tripathi.abhishek.iitkgp@gmail.com")
        );

        // Assert customer email
        assert!(
            form_string.contains("customer_email=tripathi.abhishek.iitkgp%40gmail.com")
                || form_string.contains("customer_email=tripathi.abhishek.iitkgp@gmail.com")
        );
    }
}
