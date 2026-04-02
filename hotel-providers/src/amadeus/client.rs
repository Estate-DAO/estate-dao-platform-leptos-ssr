use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::amadeus::models::auth::AmadeusAuthResponse;
use crate::amadeus::models::hotel_details::AmadeusHotelDetailsResponse;
use crate::amadeus::models::search::{
    AmadeusHotelListResponse, AmadeusHotelOfferResponse, AmadeusHotelOffersResponse,
};
use crate::domain::DomainHotelSearchCriteria;
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
    offer_lookup_response: Mutex<Option<AmadeusHotelOfferResponse>>,
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

    pub fn with_mock_offer_lookup_response(self, response: AmadeusHotelOfferResponse) -> Self {
        if let Some(mock_state) = &self.mock_state {
            let mut slot = mock_state
                .offer_lookup_response
                .lock()
                .expect("mock offer lookup lock poisoned");
            *slot = Some(response);
        }
        self
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

    pub async fn get_hotels_by_geocode(
        &self,
        latitude: f64,
        longitude: f64,
    ) -> Result<AmadeusHotelListResponse, ProviderError> {
        if self.is_mock() {
            return Ok(AmadeusHotelListResponse::default());
        }

        let access_token = self.access_token().await?;
        let url = format!(
            "{}/v1/reference-data/locations/hotels/by-geocode",
            self.api_base_url.trim_end_matches('/')
        );

        let response = self
            .http
            .get(url)
            .bearer_auth(access_token)
            .query(&[
                ("latitude", latitude.to_string()),
                ("longitude", longitude.to_string()),
            ])
            .send()
            .await
            .map_err(|error| {
                ProviderError::network(
                    ProviderNames::Amadeus,
                    ProviderSteps::HotelSearch,
                    format!("Amadeus hotel list request failed: {error}"),
                )
            })?;

        response
            .json::<AmadeusHotelListResponse>()
            .await
            .map_err(|error| {
                ProviderError::parse_error(
                    ProviderNames::Amadeus,
                    ProviderSteps::HotelSearch,
                    format!("Failed to parse Amadeus hotel list response: {error}"),
                )
            })
    }

    pub async fn get_hotel_offers(
        &self,
        hotel_ids: &[String],
        criteria: &DomainHotelSearchCriteria,
    ) -> Result<AmadeusHotelOffersResponse, ProviderError> {
        if self.is_mock() {
            return Ok(AmadeusHotelOffersResponse::default());
        }

        let access_token = self.access_token().await?;
        let url = format!(
            "{}/v3/shopping/hotel-offers",
            self.api_base_url.trim_end_matches('/')
        );
        let hotel_ids = hotel_ids.join(",");
        let adults = criteria
            .room_guests
            .iter()
            .map(|guest| guest.no_of_adults)
            .sum::<u32>()
            .max(1)
            .to_string();

        let response = self
            .http
            .get(url)
            .bearer_auth(access_token)
            .query(&[
                ("hotelIds", hotel_ids),
                ("adults", adults),
                ("checkInDate", Self::format_date(criteria.check_in_date)),
                ("checkOutDate", Self::format_date(criteria.check_out_date)),
                ("roomQuantity", criteria.no_of_rooms.max(1).to_string()),
            ])
            .send()
            .await
            .map_err(|error| {
                ProviderError::network(
                    ProviderNames::Amadeus,
                    ProviderSteps::HotelSearch,
                    format!("Amadeus hotel offers request failed: {error}"),
                )
            })?;

        response
            .json::<AmadeusHotelOffersResponse>()
            .await
            .map_err(|error| {
                ProviderError::parse_error(
                    ProviderNames::Amadeus,
                    ProviderSteps::HotelSearch,
                    format!("Failed to parse Amadeus hotel offers response: {error}"),
                )
            })
    }

    pub async fn get_hotel_offer_by_id(
        &self,
        hotel_offer_id: &str,
    ) -> Result<AmadeusHotelOfferResponse, ProviderError> {
        if self.is_mock() {
            let mock_state = self
                .mock_state
                .as_ref()
                .expect("mock state should exist for mock clients");
            let response = mock_state
                .offer_lookup_response
                .lock()
                .expect("mock offer lookup lock poisoned")
                .clone();

            return response.ok_or_else(|| {
                ProviderError::not_found(
                    ProviderNames::Amadeus,
                    ProviderSteps::HotelBlockRoom,
                    format!("Mock Amadeus offer lookup not configured for {hotel_offer_id}"),
                )
            });
        }

        let access_token = self.access_token().await?;
        let url = format!(
            "{}/v3/shopping/hotel-offers/{}",
            self.api_base_url.trim_end_matches('/'),
            hotel_offer_id
        );

        let response = self
            .http
            .get(url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|error| {
                ProviderError::network(
                    ProviderNames::Amadeus,
                    ProviderSteps::HotelBlockRoom,
                    format!("Amadeus hotel offer lookup failed: {error}"),
                )
            })?;

        let status = response.status();
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(ProviderError::not_found(
                ProviderNames::Amadeus,
                ProviderSteps::HotelBlockRoom,
                format!("Amadeus hotel offer {hotel_offer_id} was not found"),
            ));
        }

        response
            .json::<AmadeusHotelOfferResponse>()
            .await
            .map_err(|error| {
                ProviderError::parse_error(
                    ProviderNames::Amadeus,
                    ProviderSteps::HotelBlockRoom,
                    format!("Failed to parse Amadeus hotel offer response: {error}"),
                )
            })
    }

    pub async fn get_hotels_by_ids(
        &self,
        hotel_ids: &[String],
    ) -> Result<AmadeusHotelDetailsResponse, ProviderError> {
        if self.is_mock() {
            return Ok(AmadeusHotelDetailsResponse::default());
        }

        let access_token = self.access_token().await?;
        let url = format!(
            "{}/v1/reference-data/locations/hotels/by-hotels",
            self.api_base_url.trim_end_matches('/')
        );

        let response = self
            .http
            .get(url)
            .bearer_auth(access_token)
            .query(&[("hotelIds", hotel_ids.join(","))])
            .send()
            .await
            .map_err(|error| {
                ProviderError::network(
                    ProviderNames::Amadeus,
                    ProviderSteps::HotelDetails,
                    format!("Amadeus hotel details request failed: {error}"),
                )
            })?;

        response
            .json::<AmadeusHotelDetailsResponse>()
            .await
            .map_err(|error| {
                ProviderError::parse_error(
                    ProviderNames::Amadeus,
                    ProviderSteps::HotelDetails,
                    format!("Failed to parse Amadeus hotel details response: {error}"),
                )
            })
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

    fn format_date(date: (u32, u32, u32)) -> String {
        format!("{:04}-{:02}-{:02}", date.0, date.1, date.2)
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
