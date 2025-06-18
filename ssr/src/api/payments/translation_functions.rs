use crate::api::payments::domain::{
    DomainCallbackUrls, DomainCreateInvoiceRequest, DomainPaymentSettings,
};

/// Helper function to create a domain request from UI parameters
/// This can be used by UI components to construct the request
pub fn create_domain_request(
    price_amount: u32,
    price_currency: String,
    order_id: String,
    order_description: String,
    customer_email: String,
    ipn_callback_url: String,
    success_url: String,
    cancel_url: String,
    partially_paid_url: String,
    is_fixed_rate: bool,
    is_fee_paid_by_user: bool,
    provider: crate::api::payments::domain::PaymentProvider,
) -> DomainCreateInvoiceRequest {
    use crate::api::payments::domain::{DomainCallbackUrls, DomainPaymentSettings};

    DomainCreateInvoiceRequest {
        price_amount,
        price_currency,
        order_id,
        order_description,
        customer_email,
        callback_urls: DomainCallbackUrls {
            ipn_callback_url,
            success_url,
            cancel_url,
            partially_paid_url,
        },
        payment_settings: DomainPaymentSettings {
            is_fixed_rate,
            is_fee_paid_by_user,
        },
        provider,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::payments::domain::PaymentProvider;

    #[test]
    fn test_create_domain_request() {
        let request = create_domain_request(
            10000,
            "USD".to_string(),
            "test_order".to_string(),
            "Test booking".to_string(),
            "customer@example.com".to_string(),
            "https://example.com/ipn".to_string(),
            "https://example.com/success".to_string(),
            "https://example.com/cancel".to_string(),
            "https://example.com/partial".to_string(),
            false,
            false,
            PaymentProvider::NowPayments,
        );

        assert_eq!(request.price_amount, 10000);
        assert_eq!(request.provider, PaymentProvider::NowPayments);
        assert_eq!(request.order_id, "test_order");
        assert_eq!(request.customer_email, "customer@example.com");
    }

    #[test]
    fn test_request_serialization() {
        let request = create_domain_request(
            5000,
            "USD".to_string(),
            "order_123".to_string(),
            "Hotel booking".to_string(),
            "test@example.com".to_string(),
            "https://example.com/ipn".to_string(),
            "https://example.com/success".to_string(),
            "https://example.com/cancel".to_string(),
            "https://example.com/partial".to_string(),
            true,
            false,
            PaymentProvider::Stripe,
        );

        let json_str = serde_json::to_string(&request).expect("Should serialize");
        let parsed: DomainCreateInvoiceRequest =
            serde_json::from_str(&json_str).expect("Should deserialize");

        assert_eq!(parsed.price_amount, request.price_amount);
        assert_eq!(parsed.provider, request.provider);
        assert_eq!(parsed.customer_email, "test@example.com");
    }
}
