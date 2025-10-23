/// **Client-side Domain Normalization**
///
/// **Purpose**: Ensures consistent subdomain usage on the client side as a fallback
/// **Usage**: Call this early in app initialization or in specific components that handle payment redirects
/// **Benefits**: Works even if server-side middleware is bypassed

pub fn ensure_canonical_domain() {
    #[cfg(feature = "hydrate")]
    {
        use crate::api::consts::APP_URL;
        use leptos_use::use_window;

        if let Some(window) = use_window().as_ref() {
            let location = window.location();
            if let Ok(hostname) = location.hostname() {
                // Extract canonical domain from APP_URL configuration
                let app_url = APP_URL.as_str();
                let canonical_domain = extract_canonical_domain(app_url);

                // Check if current hostname is a www subdomain of the canonical domain
                if let Some(canonical) = canonical_domain {
                    let www_variant = format!("www.{}", canonical);

                    if hostname == www_variant {
                        // Get the current path and search params
                        let pathname = location.pathname().unwrap_or_default();
                        let search = location.search().unwrap_or_default();
                        let hash = location.hash().unwrap_or_default();

                        // Build the canonical URL with the same protocol as APP_URL
                        let protocol = if app_url.starts_with("https://") {
                            "https"
                        } else {
                            "http"
                        };
                        let canonical_url =
                            format!("{}://{}{}{}{}", protocol, canonical, pathname, search, hash);

                        leptos::logging::log!(
                            "Client-side domain normalization: Redirecting {} to {}",
                            hostname,
                            canonical_url
                        );

                        // Perform the redirect
                        let _ = location.replace(&canonical_url);
                    }
                }
            }
        }
    }

    #[cfg(not(feature = "hydrate"))]
    {
        // No-op on server side - middleware handles this
    }
}

/// Extract the canonical domain from APP_URL, handling different environments
/// Examples:
/// - "https://nofeebooking.com" -> Some("nofeebooking.com")
/// - "https://staging.example.com" -> Some("staging.example.com")
/// - "http://localhost:3002" -> None (localhost doesn't need www normalization)
fn extract_canonical_domain(app_url: &str) -> Option<&str> {
    // Remove protocol
    let without_protocol = app_url
        .strip_prefix("https://")
        .or_else(|| app_url.strip_prefix("http://"))
        .unwrap_or(app_url);

    // Remove trailing slash and port
    let domain = without_protocol
        .trim_end_matches('/')
        .split(':')
        .next()
        .unwrap_or(without_protocol);

    // Skip normalization for localhost and IP addresses
    if domain.starts_with("localhost") || domain.parse::<std::net::IpAddr>().is_ok() {
        return None;
    }

    Some(domain)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_canonical_domain() {
        // Production domains
        assert_eq!(
            extract_canonical_domain("https://nofeebooking.com"),
            Some("nofeebooking.com")
        );
        assert_eq!(
            extract_canonical_domain("https://nofeebooking.com/"),
            Some("nofeebooking.com")
        );

        // Staging domains
        assert_eq!(
            extract_canonical_domain("https://staging.example.com"),
            Some("staging.example.com")
        );
        assert_eq!(
            extract_canonical_domain("https://pr-111-estate.fly.dev"),
            Some("pr-111-estate.fly.dev")
        );

        // Localhost - should return None (no normalization needed)
        assert_eq!(extract_canonical_domain("http://localhost:3002"), None);
        assert_eq!(extract_canonical_domain("http://localhost:3002/"), None);
        assert_eq!(extract_canonical_domain("localhost:3002"), None);

        // IP addresses - should return None
        assert_eq!(extract_canonical_domain("http://127.0.0.1:3002"), None);
        assert_eq!(extract_canonical_domain("https://192.168.1.1"), None);
    }

    #[test]
    fn test_domain_normalization_logic() {
        // Test the logic that would be used in the normalization functions
        let test_cases = vec![
            ("www.nofeebooking.com", "nofeebooking.com", true),
            ("www.staging.example.com", "staging.example.com", true),
            ("nofeebooking.com", "nofeebooking.com", false),
            ("staging.example.com", "staging.example.com", false),
            ("localhost:3002", "localhost:3002", false),
        ];

        for (input_host, expected_canonical, should_redirect) in test_cases {
            let canonical = input_host.strip_prefix("www.");
            let needs_redirect = canonical.is_some();

            assert_eq!(needs_redirect, should_redirect, "Failed for {}", input_host);

            if let Some(canonical_domain) = canonical {
                assert_eq!(
                    canonical_domain, expected_canonical,
                    "Failed canonical extraction for {}",
                    input_host
                );
            }
        }
    }
}
