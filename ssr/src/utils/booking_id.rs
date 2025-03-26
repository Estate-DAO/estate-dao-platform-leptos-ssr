// use crate::canister::backend::BookingId;

use crate::utils::app_reference::BookingId;

#[derive(Debug, Clone)]
pub struct PaymentIdentifiers {
    /// given to us by payment provider
    pub payment_id: Option<String>,

    /// we generate and give it to payment provider
    /// see to_order_id for more details
    pub order_id: String,

    /// we generate and give it to booking provider
    pub app_reference: String,
}

impl BookingId {
    /// identifier for the payment provider, for now, it is static
    /// todo (payment): make this configurable to support any payment provider
    const PREFIX: &'static str = "NP";

    /// Convert to order ID with RESP-inspired length prefixes
    pub fn to_order_id(&self) -> String {
        // Format: "NP${app_ref.len}:${app_ref}${email.len}:${email}"
        format!(
            "{}${}:{}${}:{}",
            Self::PREFIX,
            self.app_reference.len(),
            self.app_reference,
            self.email.len(),
            self.email
        )
    }

    /// Parse using length prefixes, inspired by RESP protocol
    pub fn from_order_id(order_id: &str) -> Option<Self> {
        // Check prefix
        if !order_id.starts_with(Self::PREFIX) {
            return None;
        }

        // Skip prefix
        let mut remaining = &order_id[Self::PREFIX.len()..];

        // Parse app_reference with length prefix
        let app_reference = Self::extract_length_prefixed_string(&mut remaining)?;

        // Parse email with length prefix
        let email = Self::extract_length_prefixed_string(&mut remaining)?;

        Some(BookingId {
            app_reference,
            email,
        })
    }

    /// Helper method to extract length-prefixed strings
    fn extract_length_prefixed_string(input: &mut &str) -> Option<String> {
        // Check for $ prefix
        if !input.starts_with('$') {
            return None;
        }

        // Skip the $ character
        *input = &input[1..];

        // Find the : separator after the length
        let colon_pos = input.find(':')?;

        // Parse the length
        let length_str = &input[..colon_pos];
        let length = length_str.parse::<usize>().ok()?;

        // Move past the colon
        *input = &input[colon_pos + 1..];

        // Check if there are enough characters left
        if input.len() < length {
            return None;
        }

        // Extract the string according to the specified length
        let result = input[..length].to_string();

        // Update the remaining input
        *input = &input[length..];

        Some(result)
    }

    /// Create payment identifiers for this booking
    pub fn create_payment_identifiers(&self) -> PaymentIdentifiers {
        PaymentIdentifiers {
            payment_id: None, // This will be set by payment provider
            order_id: self.to_order_id(),
            app_reference: self.app_reference.clone(),
        }
    }
}

impl PaymentIdentifiers {
    /// Create from app_reference and email
    pub fn from_booking_id(booking_id: &BookingId) -> Self {
        booking_id.create_payment_identifiers()
    }

    /// Create from order_id
    pub fn from_order_id(order_id: &str) -> Option<Self> {
        BookingId::from_order_id(order_id).map(|booking_id| booking_id.create_payment_identifiers())
    }

    /// Update with payment_id from provider
    pub fn with_payment_id(mut self, payment_id: String) -> Self {
        self.payment_id = Some(payment_id);
        self
    }

    /// Get app_reference from order_id - order_id is what we send to NowPayments
    /// it is RESP-like protocol encoded Example:  NP$5:HB-14$10:ab@def.com
    ///
    pub fn app_reference_from_order_id(order_id: &str) -> Option<String> {
        Self::from_order_id(order_id).map(|ids| ids.app_reference)
    }

    /// Get order_id from app_reference and email
    pub fn order_id_from_app_reference(app_reference: &str, email: &str) -> String {
        BookingId {
            app_reference: app_reference.to_string(),
            email: email.to_string(),
        }
        .to_order_id()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test data for various test cases
    fn get_test_cases() -> Vec<(&'static str, &'static str, &'static str)> {
        vec![
            // (app_reference, email, description)
            ("ABC123", "user@example.com", "basic case"),
            ("", "empty@example.com", "empty app reference"),
            (
                "special:chars::here",
                "special+chars@example.com",
                "special characters",
            ),
            ("üñîçødé-123", "unicode@例子.com", "unicode characters"),
            (
                "very-long-reference-1234567890-abcdefghijklmnopqrstuvwxyz",
                "long@example.com",
                "very long app reference",
            ),
            (
                "ABC:123::456",
                "user+tag@example.com",
                "app reference with colons",
            ),
        ]
    }

    /// Helper function to create a test order_id
    fn create_test_order_id(app_ref: &str, email: &str) -> String {
        BookingId {
            app_reference: app_ref.to_string(),
            email: email.to_string(),
        }
        .to_order_id()
    }

    #[test]
    /// Tests that BookingId is correctly converted to order_id
    fn test_to_order_id() {
        let test_cases = get_test_cases();

        for (app_ref, email, desc) in test_cases {
            let booking = BookingId {
                app_reference: app_ref.to_string(),
                email: email.to_string(),
            };

            let order_id = booking.to_order_id();
            let expected = format!("NP${}:{}${}:{}", app_ref.len(), app_ref, email.len(), email);

            assert_eq!(order_id, expected, "Failed for test case: {}", desc);
        }
    }

    #[test]
    /// Tests that a valid order_id can be parsed back to BookingId
    fn test_from_order_id_valid() {
        let test_cases = get_test_cases();

        for (app_ref, email, desc) in test_cases {
            let order_id = create_test_order_id(app_ref, email);
            let booking = BookingId::from_order_id(&order_id).unwrap();

            assert_eq!(
                booking.app_reference, app_ref,
                "App reference mismatch for: {}",
                desc
            );
            assert_eq!(booking.email, email, "Email mismatch for: {}", desc);
        }
    }

    #[test]
    /// Tests that invalid prefix in order_id is rejected
    fn test_from_order_id_invalid_prefix() {
        let order_id = "XX$6:ABC123$16:user@example.com";
        let result = BookingId::from_order_id(order_id);

        assert!(result.is_none(), "Should reject invalid prefix");
    }

    #[test]
    /// Tests that invalid length specification in order_id is rejected
    fn test_from_order_id_invalid_length() {
        let order_id = "NP$X:ABC123$16:user@example.com";
        let result = BookingId::from_order_id(order_id);

        assert!(result.is_none(), "Should reject invalid length");
    }

    #[test]
    /// Tests that missing colon in order_id is rejected
    fn test_from_order_id_missing_colon() {
        let order_id = "NP$6ABC123$16:user@example.com";
        let result = BookingId::from_order_id(order_id);

        assert!(result.is_none(), "Should reject missing colon");
    }

    #[test]
    /// Tests that truncated data in order_id is rejected
    fn test_from_order_id_truncated_data() {
        let order_id = "NP$6:ABC123$16:user@exam";
        let result = BookingId::from_order_id(order_id);

        assert!(result.is_none(), "Should reject truncated data");
    }

    #[test]
    /// Tests that extra data after valid content is ignored
    fn test_from_order_id_extra_data() {
        let order_id = "NP$6:ABC123$16:user@example.comEXTRA";
        let booking = BookingId::from_order_id(order_id).unwrap();

        assert_eq!(
            booking.app_reference, "ABC123",
            "Should extract correct app reference"
        );
        assert_eq!(
            booking.email, "user@example.com",
            "Should extract correct email"
        );
    }

    #[test]
    /// Tests the roundtrip property (encoding then decoding returns original)
    fn test_roundtrip_property() {
        let test_cases = get_test_cases();

        for (app_ref, email, desc) in test_cases {
            let original = BookingId {
                app_reference: app_ref.to_string(),
                email: email.to_string(),
            };

            let order_id = original.to_order_id();
            let decoded = BookingId::from_order_id(&order_id).unwrap();

            assert_eq!(
                decoded.app_reference, app_ref,
                "App reference roundtrip failed for: {}",
                desc
            );
            assert_eq!(decoded.email, email, "Email roundtrip failed for: {}", desc);
        }
    }

    #[test]
    /// Tests that PaymentIdentifiers are correctly created from BookingId
    fn test_create_payment_identifiers() {
        let test_cases = get_test_cases();

        for (app_ref, email, desc) in test_cases {
            let booking = BookingId {
                app_reference: app_ref.to_string(),
                email: email.to_string(),
            };

            let payment_ids = booking.create_payment_identifiers();

            assert_eq!(
                payment_ids.payment_id, None,
                "Payment ID should be None for: {}",
                desc
            );
            assert_eq!(
                payment_ids.order_id,
                booking.to_order_id(),
                "Order ID mismatch for: {}",
                desc
            );
            assert_eq!(
                payment_ids.app_reference, app_ref,
                "App reference mismatch for: {}",
                desc
            );
        }
    }
}
