use crate::api::payments::domain::{
    DomainCreateInvoiceRequest, DomainCreateInvoiceResponse, PaymentProvider, PaymentService,
    PaymentServiceError,
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
