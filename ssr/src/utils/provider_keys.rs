use hotel_types::ports::{ProviderKeys, ProviderNames};

pub fn normalize_hotel_provider_key(provider: &str) -> Option<&'static str> {
    let normalized = provider.trim().to_ascii_lowercase();

    match normalized.as_str() {
        "liteapi" => Some(ProviderKeys::LiteApi),
        "booking" | "booking.com" => Some(ProviderKeys::Booking),
        "amadeus" => Some(ProviderKeys::Amadeus),
        "composite" => Some(ProviderKeys::Composite),
        "mock" | "mockhotelprovider" => Some(ProviderKeys::Mock),
        _ if provider.eq_ignore_ascii_case(ProviderNames::LiteApi) => Some(ProviderKeys::LiteApi),
        _ if provider.eq_ignore_ascii_case(ProviderNames::Booking) => Some(ProviderKeys::Booking),
        _ if provider.eq_ignore_ascii_case(ProviderNames::Amadeus) => Some(ProviderKeys::Amadeus),
        _ => None,
    }
}

pub fn normalize_owned_hotel_provider_key(provider: Option<String>) -> Option<String> {
    provider
        .as_deref()
        .and_then(normalize_hotel_provider_key)
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::normalize_hotel_provider_key;
    use hotel_types::ports::ProviderKeys;

    #[test]
    fn normalizes_stable_keys_and_provider_names() {
        assert_eq!(
            normalize_hotel_provider_key("amadeus"),
            Some(ProviderKeys::Amadeus)
        );
        assert_eq!(
            normalize_hotel_provider_key("Amadeus"),
            Some(ProviderKeys::Amadeus)
        );
        assert_eq!(
            normalize_hotel_provider_key("LiteAPI"),
            Some(ProviderKeys::LiteApi)
        );
        assert_eq!(
            normalize_hotel_provider_key("Booking.com"),
            Some(ProviderKeys::Booking)
        );
    }
}
