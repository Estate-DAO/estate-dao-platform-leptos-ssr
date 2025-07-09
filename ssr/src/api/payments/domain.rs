use crate::api::payments::ports::{CreateInvoiceRequest, CreateInvoiceResponse};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Payment provider enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PaymentProvider {
    NowPayments,
    Stripe,
}

impl PaymentProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            PaymentProvider::NowPayments => "nowpayments",
            PaymentProvider::Stripe => "stripe",
        }
    }
}

/// Convert domain PaymentProvider to consts PaymentProvider
impl From<PaymentProvider> for crate::api::consts::PaymentProvider {
    fn from(domain_provider: PaymentProvider) -> Self {
        match domain_provider {
            PaymentProvider::NowPayments => crate::api::consts::PaymentProvider::NowPayments,
            PaymentProvider::Stripe => crate::api::consts::PaymentProvider::Stripe,
        }
    }
}

/// Domain struct for invoice creation request (provider-agnostic)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainCreateInvoiceRequest {
    pub price_amount: u32,
    pub price_currency: String,
    pub order_id: String,
    pub order_description: String,
    pub customer_email: String,
    pub callback_urls: DomainCallbackUrls,
    pub payment_settings: DomainPaymentSettings,
    pub provider: PaymentProvider,
}

/// Callback URLs for payment flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainCallbackUrls {
    pub ipn_callback_url: String,
    pub success_url: String,
    pub cancel_url: String,
    pub partially_paid_url: String,
}

/// Payment configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainPaymentSettings {
    pub is_fixed_rate: bool,
    pub is_fee_paid_by_user: bool,
}

/// Domain struct for invoice creation response (provider-agnostic)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainCreateInvoiceResponse {
    pub invoice_id: String,
    pub payment_url: String,
    pub order_id: String,
    pub provider: PaymentProvider,
    pub provider_response: ProviderResponseData,
}

/// Provider-specific response data stored for reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderResponseData {
    pub raw_response: String, // JSON string of provider response
    pub provider_invoice_id: String,
    pub created_at: String,
}

/// Payment status enumeration (provider-agnostic)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PaymentStatus {
    Pending,         // Payment initiated but not completed
    Completed,       // Payment successfully completed
    Failed,          // Payment failed
    Cancelled,       // Payment cancelled by user
    Expired,         // Payment session expired
    Refunded,        // Payment was refunded
    Unknown(String), // Status cannot be determined
}

impl PaymentStatus {
    pub fn as_str(&self) -> String {
        match self {
            PaymentStatus::Pending => "pending".to_string(),
            PaymentStatus::Completed => "completed".to_string(),
            PaymentStatus::Failed => "failed".to_string(),
            PaymentStatus::Cancelled => "cancelled".to_string(),
            PaymentStatus::Expired => "expired".to_string(),
            PaymentStatus::Refunded => "refunded".to_string(),
            PaymentStatus::Unknown(status) => status.clone(),
        }
    }
}

/// Domain struct for payment status request (provider-agnostic)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainGetPaymentStatusRequest {
    pub payment_id: String,                // session_id or payment_id
    pub provider: Option<PaymentProvider>, // Optional - can auto-detect from ID format
}

/// Domain struct for payment status response (provider-agnostic)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainGetPaymentStatusResponse {
    pub payment_id: String,        // session_id or payment_id
    pub status: PaymentStatus,     // Unified enum
    pub amount_total: Option<u64>, // Total amount in smallest currency unit
    pub currency: Option<String>,  // Currency code (USD, etc.)
    pub provider: PaymentProvider, // Which provider handled this payment
    pub raw_provider_data: String, // Raw provider response for debugging
    pub order_id: Option<String>,  // Associated order ID if available
    pub customer_email: Option<String>, // Customer email if available
                                   // pub invoice_id: Option<String>,     // Invoice ID if available
}

/// Trait for payment service abstraction
#[async_trait]
pub trait PaymentService {
    /// Create an invoice with the specified provider
    async fn create_invoice(
        &self,
        request: DomainCreateInvoiceRequest,
    ) -> Result<DomainCreateInvoiceResponse, PaymentServiceError>;

    /// Get payment status for a payment ID (auto-detects provider if not specified)
    async fn get_payment_status(
        &self,
        request: DomainGetPaymentStatusRequest,
    ) -> Result<DomainGetPaymentStatusResponse, PaymentServiceError>;
}

/// Payment service errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaymentServiceError {
    ProviderError(String),
    InvalidRequest(String),
    ConversionError(String),
    NetworkError(String),
}

impl std::fmt::Display for PaymentServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaymentServiceError::ProviderError(msg) => write!(f, "Provider error: {}", msg),
            PaymentServiceError::InvalidRequest(msg) => write!(f, "Invalid request: {}", msg),
            PaymentServiceError::ConversionError(msg) => write!(f, "Conversion error: {}", msg),
            PaymentServiceError::NetworkError(msg) => write!(f, "Network error: {}", msg),
        }
    }
}

impl std::error::Error for PaymentServiceError {}

/// Convert domain request to provider-specific request
impl From<DomainCreateInvoiceRequest> for CreateInvoiceRequest {
    fn from(domain_req: DomainCreateInvoiceRequest) -> Self {
        CreateInvoiceRequest {
            price_amount: domain_req.price_amount,
            price_currency: domain_req.price_currency,
            order_id: domain_req.order_id,
            order_description: domain_req.order_description,
            ipn_callback_url: domain_req.callback_urls.ipn_callback_url,
            success_url: domain_req.callback_urls.success_url,
            cancel_url: domain_req.callback_urls.cancel_url,
            partially_paid_url: domain_req.callback_urls.partially_paid_url,
            is_fixed_rate: domain_req.payment_settings.is_fixed_rate,
            is_fee_paid_by_user: domain_req.payment_settings.is_fee_paid_by_user,
        }
    }
}

/// Convert provider response to domain response
impl DomainCreateInvoiceResponse {
    pub fn from_provider_response(
        provider_response: CreateInvoiceResponse,
        provider: PaymentProvider,
    ) -> Self {
        Self {
            invoice_id: provider_response.id.clone(),
            payment_url: provider_response.invoice_url.clone(),
            order_id: provider_response.order_id.clone(),
            provider: provider.clone(),
            provider_response: ProviderResponseData {
                raw_response: serde_json::to_string(&provider_response)
                    .unwrap_or_else(|_| "Failed to serialize response".to_string()),
                provider_invoice_id: provider_response.id,
                created_at: provider_response.created_at,
            },
        }
    }
}
