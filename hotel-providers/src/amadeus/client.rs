use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::amadeus::models::auth::AmadeusAuthResponse;
use crate::ports::{ProviderError, ProviderErrorKind, ProviderNames, ProviderSteps};

const DEFAULT_AUTH_BASE_URL: &str = "https://test.api.amadeus.com";
const DEFAULT_API_BASE_URL: &str = "https://test.api.amadeus.com";
const TOKEN_REFRESH_BUFFER_SECS: u64 = 30;

#[derive(Clone, Debug)]
pub struct AmadeusClient {
    http: reqwest::Client,
    auth_base_url: String,
    api_base_url: String,
    client_id: String,
    client_secret: String,
    token_cache: Arc<Mutex<Option<CachedToken>>>,
    mock_state: Option<Arc<MockState>>,
}

#[derive(Clone, Debug)]
struct CachedToken {
    access_token: String,
    expires_at: Instant,
}

#[derive(Debug, Default)]
struct MockState {
    token_fetch_count: AtomicUsize,
}

impl Default for AmadeusClient {
    fn default() -> Self {
        Self::new_mock()
    }
}

impl AmadeusClient {
    pub fn new(
        client_id: String,
        client_secret: String,
        auth_base_url: Option<String>,
        api_base_url: Option<String>,
    ) -> Self {
        Self {
            http: reqwest::Client::new(),
            auth_base_url: auth_base_url.unwrap_or_else(|| DEFAULT_AUTH_BASE_URL.to_string()),
            api_base_url: api_base_url.unwrap_or_else(|| DEFAULT_API_BASE_URL.to_string()),
            client_id,
            client_secret,
            token_cache: Arc::new(Mutex::new(None)),
            mock_state: None,
        }
    }

    pub fn new_mock() -> Self {
        Self {
            http: reqwest::Client::new(),
            auth_base_url: DEFAULT_AUTH_BASE_URL.to_string(),
            api_base_url: DEFAULT_API_BASE_URL.to_string(),
            client_id: String::new(),
            client_secret: String::new(),
            token_cache: Arc::new(Mutex::new(None)),
            mock_state: Some(Arc::new(MockState::default())),
        }
    }

    pub fn is_mock(&self) -> bool {
        self.mock_state.is_some()
    }

    pub fn api_base_url(&self) -> &str {
        &self.api_base_url
    }

    pub async fn access_token(&self) -> Result<String, ProviderError> {
        if let Some(cached) = self.cached_access_token() {
            return Ok(cached);
        }

        let fresh_token = if self.is_mock() {
            self.fetch_mock_token()
        } else {
            self.fetch_real_token().await?
        };
        let access_token = fresh_token.access_token.clone();

        let mut cache = self.token_cache.lock().expect("token cache lock poisoned");
        *cache = Some(fresh_token);

        Ok(access_token)
    }

    fn cached_access_token(&self) -> Option<String> {
        let cache = self.token_cache.lock().expect("token cache lock poisoned");
        let cached = cache.as_ref()?;
        let refresh_buffer = Duration::from_secs(TOKEN_REFRESH_BUFFER_SECS);

        if cached.expires_at > Instant::now() + refresh_buffer {
            Some(cached.access_token.clone())
        } else {
            None
        }
    }

    fn fetch_mock_token(&self) -> CachedToken {
        let mock_state = self
            .mock_state
            .as_ref()
            .expect("mock state should exist for mock clients");
        let token_index = mock_state.token_fetch_count.fetch_add(1, Ordering::SeqCst) + 1;

        CachedToken {
            access_token: format!("mock-access-token-{token_index}"),
            expires_at: Instant::now() + Duration::from_secs(3600),
        }
    }

    async fn fetch_real_token(&self) -> Result<CachedToken, ProviderError> {
        let auth_url = format!(
            "{}/v1/security/oauth2/token",
            self.auth_base_url.trim_end_matches('/')
        );

        let response = self
            .http
            .post(auth_url)
            .form(&[
                ("grant_type", "client_credentials"),
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
            ])
            .send()
            .await
            .map_err(|error| {
                ProviderError::new(
                    ProviderNames::Amadeus,
                    ProviderErrorKind::Auth,
                    ProviderSteps::HotelSearch,
                    format!("Amadeus auth request failed: {error}"),
                )
            })?;

        let status = response.status();
        if !status.is_success() {
            return Err(ProviderError::new(
                ProviderNames::Amadeus,
                ProviderErrorKind::Auth,
                ProviderSteps::HotelSearch,
                format!("Amadeus auth request failed with status {status}"),
            ));
        }

        let payload = response
            .json::<AmadeusAuthResponse>()
            .await
            .map_err(|error| {
                ProviderError::parse_error(
                    ProviderNames::Amadeus,
                    ProviderSteps::HotelSearch,
                    format!("Failed to parse Amadeus auth response: {error}"),
                )
            })?;

        Ok(CachedToken {
            access_token: payload.access_token,
            expires_at: Instant::now() + Duration::from_secs(payload.expires_in.max(1)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn client_reuses_unexpired_access_token() {
        let client = AmadeusClient::new_mock();

        let first = client.access_token().await.unwrap();
        let second = client.access_token().await.unwrap();

        assert_eq!(first, second);
    }
}
