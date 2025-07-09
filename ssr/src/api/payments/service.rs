use crate::api::payments::domain::{
    DomainCreateInvoiceRequest, DomainCreateInvoiceResponse, DomainGetPaymentStatusRequest,
    DomainGetPaymentStatusResponse, PaymentProvider, PaymentService, PaymentServiceError,
    PaymentStatus,
};
use crate::api::payments::ports::{CreateInvoiceRequest, PaymentGateway, PaymentGatewayParams};
use crate::api::payments::{NowPayments, StripeEstate};
use crate::{error, log, warn};
use async_trait::async_trait;

/// Main payment service implementation that routes to appropriate providers
#[derive(Debug, Clone)]
pub struct PaymentServiceImpl;

impl PaymentServiceImpl {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PaymentServiceImpl {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PaymentService for PaymentServiceImpl {
    async fn create_invoice(
        &self,
        request: DomainCreateInvoiceRequest,
    ) -> Result<DomainCreateInvoiceResponse, PaymentServiceError> {
        log!(
            "Creating invoice with provider: {:?}, order_id: {}",
            request.provider,
            request.order_id
        );

        match request.provider {
            PaymentProvider::NowPayments => self.create_nowpayments_invoice(request).await,
            PaymentProvider::Stripe => self.create_stripe_invoice(request).await,
        }
    }

    async fn get_payment_status(
        &self,
        request: DomainGetPaymentStatusRequest,
    ) -> Result<DomainGetPaymentStatusResponse, PaymentServiceError> {
        log!(
            "Getting payment status for payment_id: {}, provider: {:?}",
            request.payment_id,
            request.provider
        );

        // Auto-detect provider if not specified
        let provider = match request.provider {
            Some(p) => p,
            None => self.detect_provider_from_payment_id_instance(&request.payment_id)?,
        };

        match provider {
            PaymentProvider::NowPayments => self.get_nowpayments_status(request.payment_id).await,
            PaymentProvider::Stripe => self.get_stripe_status(request.payment_id).await,
        }
    }
}

impl PaymentServiceImpl {
    /// Create invoice using NowPayments
    async fn create_nowpayments_invoice(
        &self,
        domain_request: DomainCreateInvoiceRequest,
    ) -> Result<DomainCreateInvoiceResponse, PaymentServiceError> {
        log!("Processing NowPayments invoice creation");

        // Convert domain request to NowPayments request
        let provider_request: CreateInvoiceRequest = domain_request.clone().into();

        // Initialize NowPayments client
        let nowpayments = NowPayments::default();

        // Call NowPayments API
        match nowpayments.send(provider_request).await {
            Ok(provider_response) => {
                log!(
                    "NowPayments invoice created successfully: {}",
                    provider_response.id
                );

                let domain_response = DomainCreateInvoiceResponse::from_provider_response(
                    provider_response,
                    PaymentProvider::NowPayments,
                );

                Ok(domain_response)
            }
            Err(e) => {
                error!("NowPayments invoice creation failed: {}", e);
                Err(PaymentServiceError::ProviderError(format!(
                    "NowPayments error: {}",
                    e
                )))
            }
        }
    }

    /// Create invoice using Stripe
    async fn create_stripe_invoice(
        &self,
        domain_request: DomainCreateInvoiceRequest,
    ) -> Result<DomainCreateInvoiceResponse, PaymentServiceError> {
        log!("Processing Stripe invoice creation");

        // Convert domain request to Stripe request
        let stripe_request = self.convert_to_stripe_request(domain_request.clone())?;

        // Call Stripe API via the existing server function
        let request_json = serde_json::to_string(&stripe_request)
            .map_err(|e| PaymentServiceError::ConversionError(e.to_string()))?;

        match crate::api::payments::stripe_create_invoice(request_json).await {
            Ok(stripe_response) => {
                log!("Stripe checkout session created successfully");

                // Convert Stripe response to domain response
                let domain_response = self
                    .convert_stripe_response_to_domain(stripe_response, domain_request.order_id)?;

                Ok(domain_response)
            }
            Err(e) => {
                error!("Stripe checkout session creation failed: {}", e);
                Err(PaymentServiceError::ProviderError(format!(
                    "Stripe error: {}",
                    e
                )))
            }
        }
    }

    /// Convert domain request to Stripe-specific request
    fn convert_to_stripe_request(
        &self,
        domain_request: DomainCreateInvoiceRequest,
    ) -> Result<
        crate::api::payments::stripe_service::StripeCreateCheckoutSession,
        PaymentServiceError,
    > {
        use crate::api::payments::stripe_service::{
            StripeCreateCheckoutSession, StripeLineItem, StripeMetadata, StripeUIModeEnum,
        };
        use std::collections::HashMap;

        warn!("Converting domain request to Stripe request - using placeholder data for demo");

        // Create line item for the booking
        let line_item = StripeLineItem {
            price_data: crate::api::payments::stripe_service::StripePriceData {
                currency: domain_request.price_currency.clone(),
                product_data: crate::api::payments::stripe_service::StripeProductData {
                    name: domain_request.order_description.clone(),
                    description: Some("Hotel Room Booking".to_string()),
                    metadata: None,
                },
                unit_amount: domain_request.price_amount * 100, // stripe api exepects price in cents
            },
            quantity: 1,
        };

        // Create metadata
        let mut metadata_map = HashMap::new();
        metadata_map.insert("order_id".to_string(), domain_request.order_id.clone());
        metadata_map.insert("booking_type".to_string(), "hotel".to_string());
        let metadata = StripeMetadata::new(metadata_map)
            .map_err(|e| PaymentServiceError::InvalidRequest(format!("Invalid metadata: {}", e)))?;

        let stripe_request = StripeCreateCheckoutSession::new(
            domain_request.callback_urls.success_url,
            domain_request.callback_urls.cancel_url,
            vec![line_item],
            "payment".to_string(),
            Some(metadata),
            domain_request.order_id,
            domain_request.customer_email.clone(),
            StripeUIModeEnum::Hosted,
        );

        Ok(stripe_request)
    }

    /// Convert Stripe response to domain response
    fn convert_stripe_response_to_domain(
        &self,
        stripe_response: crate::api::payments::stripe_service::StripeCreateCheckoutSessionResponse,
        order_id: String,
    ) -> Result<DomainCreateInvoiceResponse, PaymentServiceError> {
        use crate::api::payments::domain::ProviderResponseData;

        let domain_response = DomainCreateInvoiceResponse::from_provider_response(
            stripe_response.into(),
            PaymentProvider::Stripe,
        );

        Ok(domain_response)
    }

    /// Auto-detect payment provider from payment ID format
    /// Public static method to detect payment provider from payment ID
    pub fn detect_provider_from_payment_id(
        payment_id: &str,
    ) -> Result<PaymentProvider, PaymentServiceError> {
        if payment_id.starts_with("cs_") {
            // Stripe checkout session IDs start with 'cs_'
            Ok(PaymentProvider::Stripe)
        } else if payment_id.chars().all(|c| c.is_ascii_digit()) {
            // NowPayments payment IDs are numeric
            Ok(PaymentProvider::NowPayments)
        } else {
            Err(PaymentServiceError::InvalidRequest(format!(
                "Cannot detect provider from payment ID: {}",
                payment_id
            )))
        }
    }

    fn detect_provider_from_payment_id_instance(
        &self,
        payment_id: &str,
    ) -> Result<PaymentProvider, PaymentServiceError> {
        Self::detect_provider_from_payment_id(payment_id)
    }

    /// Get payment status from NowPayments
    async fn get_nowpayments_status(
        &self,
        payment_id: String,
    ) -> Result<DomainGetPaymentStatusResponse, PaymentServiceError> {
        log!("Getting NowPayments status for payment_id: {}", payment_id);

        // Parse payment_id to u64 for NowPayments API
        let payment_id_u64 = payment_id.parse::<u64>().map_err(|_| {
            PaymentServiceError::InvalidRequest(format!(
                "Invalid NowPayments payment ID format: {}",
                payment_id
            ))
        })?;

        // Initialize NowPayments client
        let nowpayments = NowPayments::default();

        // Create the request using the existing ports::GetPaymentStatusRequest
        let nowpayments_request = crate::api::payments::ports::GetPaymentStatusRequest {
            payment_id: payment_id_u64,
        };

        match nowpayments.send(nowpayments_request).await {
            Ok(nowpayments_response) => {
                log!(
                    "NowPayments status retrieved successfully for payment_id: {}",
                    payment_id
                );

                // Handle both Success and Failure response variants
                match nowpayments_response {
                    crate::api::payments::ports::GetPaymentStatusResponse::Success(
                        success_response,
                    ) => {
                        let status = match success_response.payment_status.as_str() {
                            "waiting" | "confirming" | "confirmed" => PaymentStatus::Pending,

                            // only 'finished' is considered the terminal success step in nowpayments
                            "finished" => PaymentStatus::Completed,
                            "failed" => PaymentStatus::Failed,
                            "refunded" => PaymentStatus::Refunded,
                            "expired" => PaymentStatus::Expired,
                            other_status => PaymentStatus::Unknown(String::from(other_status)),
                        };

                        let domain_response = DomainGetPaymentStatusResponse {
                            payment_id: payment_id.clone(),
                            status,
                            amount_total: Some(success_response.price_amount),
                            currency: Some(success_response.price_currency.clone()),
                            provider: PaymentProvider::NowPayments,
                            raw_provider_data: serde_json::to_string(&success_response)
                                .unwrap_or_else(|_| "Failed to serialize response".to_string()),
                            order_id: Some(success_response.order_id.clone()),
                            customer_email: None, // NowPayments doesn't return customer email
                        };

                        Ok(domain_response)
                    }
                    crate::api::payments::ports::GetPaymentStatusResponse::Failure(
                        failure_response,
                    ) => {
                        error!("NowPayments API returned failure: {:?}", failure_response);

                        let domain_response = DomainGetPaymentStatusResponse {
                            payment_id: payment_id.clone(),
                            status: PaymentStatus::Failed,
                            amount_total: None,
                            currency: None,
                            provider: PaymentProvider::NowPayments,
                            raw_provider_data: serde_json::to_string(&failure_response)
                                .unwrap_or_else(|_| "Failed to serialize response".to_string()),
                            order_id: None,
                            customer_email: None,
                        };

                        Ok(domain_response)
                    }
                }
            }
            Err(e) => {
                error!("NowPayments status check failed: {}", e);
                Err(PaymentServiceError::ProviderError(format!(
                    "NowPayments error: {}",
                    e
                )))
            }
        }
    }

    // Get payment status from Stripe
    async fn get_stripe_status(
        &self,
        session_id: String,
    ) -> Result<DomainGetPaymentStatusResponse, PaymentServiceError> {
        log!("Getting Stripe status for session_id: {}", session_id);

        // Call the Stripe server function
        match crate::api::payments::stripe_service::stripe_get_session_status(session_id.clone())
            .await
        {
            Ok(stripe_response) => {
                log!(
                    "Stripe status retrieved successfully for session_id: {}",
                    session_id
                );

                // Convert Stripe response to domain response
                // Priority: Use payment_status over session status for completed payments
                let status = if stripe_response.payment_status
                    == crate::api::payments::stripe_service::StripePaymentStatusEnum::Paid
                {
                    PaymentStatus::Completed
                } else {
                    stripe_response.status.clone().into()
                };

                let domain_response = DomainGetPaymentStatusResponse {
                    payment_id: session_id.clone(),
                    status,
                    amount_total: stripe_response.amount_total.map(|amt| amt as u64),
                    currency: stripe_response.currency.clone(),
                    provider: PaymentProvider::Stripe,
                    raw_provider_data: serde_json::to_string(&stripe_response)
                        .unwrap_or_else(|_| "Failed to serialize response".to_string()),
                    order_id: stripe_response.client_reference_id.clone(),
                    customer_email: stripe_response.customer_email.clone(),
                };

                Ok(domain_response)
            }
            Err(e) => {
                error!("Stripe status check failed: {}", e);
                Err(PaymentServiceError::ProviderError(format!(
                    "Stripe error: {}",
                    e
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::payments::domain::{DomainCallbackUrls, DomainPaymentSettings};

    fn create_test_domain_request(provider: PaymentProvider) -> DomainCreateInvoiceRequest {
        DomainCreateInvoiceRequest {
            price_amount: 10000, // $100.00
            price_currency: "USD".to_string(),
            order_id: "test_order_123".to_string(),
            order_description: "Test Hotel Booking".to_string(),
            customer_email: "test@example.com".to_string(),
            callback_urls: DomainCallbackUrls {
                ipn_callback_url: "https://example.com/ipn".to_string(),
                success_url: "https://example.com/success".to_string(),
                cancel_url: "https://example.com/cancel".to_string(),
                partially_paid_url: "https://example.com/partial".to_string(),
            },
            payment_settings: DomainPaymentSettings {
                is_fixed_rate: false,
                is_fee_paid_by_user: false,
            },
            provider,
        }
    }

    #[test]
    fn test_service_creation() {
        let service = PaymentServiceImpl::new();
        assert_eq!(std::mem::size_of_val(&service), 0); // Zero-sized struct
    }

    #[test]
    fn test_domain_request_creation() {
        let request = create_test_domain_request(PaymentProvider::NowPayments);
        assert_eq!(request.provider, PaymentProvider::NowPayments);
        assert_eq!(request.price_amount, 10000);
        assert_eq!(request.order_id, "test_order_123");
    }
}
