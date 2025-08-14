/// Browser-specific utilities that are only available in hydrated client environments
use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "hydrate")] {
        use wasm_bindgen::JsCast;
        use web_sys::HtmlDocument;
        use crate::log;

        /// Scrolls the window to the top of the page
        pub fn scroll_to_top() {
            if let Some(window) = web_sys::window() {
                window.scroll_to_with_x_and_y(0.0, 0.0);
            }
        }


        /// Parses a Set-Cookie header value and extracts the cookie value suitable for document.cookie
        fn parse_cookie_for_document(set_cookie_header: &str) -> Option<String> {
            // Set-Cookie format: name=value; attribute1=value1; attribute2=value2
            // For document.cookie we only need: name=value

            let parts: Vec<&str> = set_cookie_header.split(';').collect();
            if let Some(name_value_pair) = parts.first() {
                let trimmed = name_value_pair.trim();
                if !trimmed.is_empty() {
                    log!("[COOKIE_UTIL] Parsed cookie name=value: {}", trimmed);
                    return Some(trimmed.to_string());
                }
            }

            log!("[COOKIE_UTIL] Failed to parse set-cookie header: {}", set_cookie_header);
            None
        }

        /// Verifies that cookies were successfully set by reading document.cookie
        fn verify_cookies_set(document: &HtmlDocument) {
            match document.cookie() {
                Ok(cookie_string) => {
                    log!("[COOKIE_UTIL] Current document cookies: {}", cookie_string);

                    // Check for specific expected cookies
                    if cookie_string.contains("google-csrf-token") {
                        log!("[COOKIE_UTIL] ✓ google-csrf-token cookie verified");
                    } else {
                        log!("[COOKIE_UTIL] ⚠ google-csrf-token cookie not found");
                    }

                    if cookie_string.contains("google-pkce-verifier") {
                        log!("[COOKIE_UTIL] ✓ google-pkce-verifier cookie verified");
                    } else {
                        log!("[COOKIE_UTIL] ⚠ google-pkce-verifier cookie not found");
                    }
                }
                Err(e) => {
                    log!("[COOKIE_UTIL] Failed to read document.cookie: {:?}", e);
                }
            }
        }

        /// Test function to manually set cookies for demonstration
        pub fn test_cookie_setting() {
            log!("[COOKIE_UTIL] Testing manual cookie setting...");

            let Some(window) = web_sys::window() else {
                log!("[COOKIE_UTIL] No window available for testing");
                return;
            };

            let Some(document) = window.document() else {
                log!("[COOKIE_UTIL] No document available for testing");
                return;
            };

            let Ok(html_document) = document.dyn_into::<HtmlDocument>() else {
                log!("[COOKIE_UTIL] Could not cast document to HtmlDocument");
                return;
            };

            // Test setting some cookies manually
            let test_cookies = vec![
                "google-csrf-token=test123456789",
                "google-pkce-verifier=verifier123456789",
            ];

            for cookie in test_cookies {
                log!("[COOKIE_UTIL] Setting test cookie: {}", cookie);
                if let Err(e) = html_document.set_cookie(cookie) {
                    log!("[COOKIE_UTIL] Failed to set test cookie: {:?}", e);
                }
            }

            // Verify cookies were set
            verify_cookies_set(&html_document);
        }

        /// Comprehensive cookie application function that takes response headers and applies all cookies
        pub fn apply_cookies_from_reqwest_response(response: &reqwest::Response) {
            let headers = response.headers();
            let mut cookie_headers = Vec::new();

            // Extract set-cookie headers from reqwest response
            for (name, value) in headers.iter() {
                if name.as_str().to_lowercase() == "set-cookie" {
                    if let Ok(value_str) = value.to_str() {
                        cookie_headers.push(value_str.to_string());
                        log!("[COOKIE_UTIL] Found set-cookie header: {}", value_str);
                    }
                }
            }

            let Some(window) = web_sys::window() else {
                log!("[COOKIE_UTIL] No window available for cookie setting");
                return;
            };

            let Some(document) = window.document() else {
                log!("[COOKIE_UTIL] No document available for cookie setting");
                return;
            };

            let Ok(html_document) = document.dyn_into::<HtmlDocument>() else {
                log!("[COOKIE_UTIL] Could not cast document to HtmlDocument");
                return;
            };

            // Apply each cookie
            for cookie_header in cookie_headers {
                if let Some(cookie_value) = parse_cookie_for_document(&cookie_header) {
                    log!("[COOKIE_UTIL] Setting cookie from reqwest response: {}", cookie_value);
                    if let Err(e) = html_document.set_cookie(&cookie_value) {
                        log!("[COOKIE_UTIL] Failed to set cookie from reqwest: {:?}", e);
                    }
                }
            }

            // Verify cookies were set
            verify_cookies_set(&html_document);
        }

    } else {
        /// No-op version for SSR - does nothing
        pub fn scroll_to_top() {
            // No-op on server side
        }

        /// No-op version for SSR - does nothing
        pub fn test_cookie_setting() {
            // No-op on server side
        }

        /// No-op version for SSR - does nothing
        pub fn apply_cookies_from_reqwest_response(_response: &reqwest::Response) {
            // No-op on server side
        }
    }
}
