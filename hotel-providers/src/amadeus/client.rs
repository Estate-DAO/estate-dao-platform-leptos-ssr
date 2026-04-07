use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::de::DeserializeOwned;
use serde::Deserialize;

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

#[derive(Debug, Deserialize)]
struct AmadeusErrorEnvelope {
    errors: Vec<AmadeusErrorItem>,
}

#[derive(Debug, Deserialize)]
struct AmadeusErrorItem {
    title: Option<String>,
    detail: Option<String>,
    source: Option<AmadeusErrorSource>,
}

#[derive(Debug, Deserialize)]
struct AmadeusErrorSource {
    parameter: Option<String>,
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

        Self::parse_json_response(response, ProviderSteps::HotelSearch, "hotel list").await
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

        Self::parse_json_response(response, ProviderSteps::HotelSearch, "hotel offers").await
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

        Self::parse_json_response(response, ProviderSteps::HotelDetails, "hotel details").await
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

    async fn parse_json_response<T>(
        response: reqwest::Response,
        step: ProviderSteps,
        context: &'static str,
    ) -> Result<T, ProviderError>
    where
        T: DeserializeOwned,
    {
        let status = response.status();

        if !status.is_success() {
            let body = response.text().await.ok();
            return Err(Self::map_http_error(status, step, context, body.as_deref()));
        }

        response.json::<T>().await.map_err(|error| {
            ProviderError::parse_error(
                ProviderNames::Amadeus,
                step,
                format!("Failed to parse Amadeus {context} response: {error}"),
            )
        })
    }

    fn map_http_error(
        status: reqwest::StatusCode,
        step: ProviderSteps,
        context: &'static str,
        body: Option<&str>,
    ) -> ProviderError {
        let status_message = Self::format_error_body(body);
        let message = if status_message.is_empty() {
            format!("Amadeus {context} request failed with status {status}")
        } else {
            format!("Amadeus {context} request failed with status {status}: {status_message}")
        };

        match status {
            reqwest::StatusCode::BAD_REQUEST => ProviderError::new(
                ProviderNames::Amadeus,
                ProviderErrorKind::InvalidRequest,
                step,
                message,
            ),
            reqwest::StatusCode::UNAUTHORIZED | reqwest::StatusCode::FORBIDDEN => {
                ProviderError::new(
                    ProviderNames::Amadeus,
                    ProviderErrorKind::Auth,
                    step,
                    message,
                )
            }
            reqwest::StatusCode::NOT_FOUND => {
                ProviderError::not_found(ProviderNames::Amadeus, step, message)
            }
            reqwest::StatusCode::TOO_MANY_REQUESTS => ProviderError::new(
                ProviderNames::Amadeus,
                ProviderErrorKind::RateLimited,
                step,
                message,
            ),
            _ if status.is_server_error() => {
                ProviderError::service_unavailable(ProviderNames::Amadeus, step, message)
            }
            _ => ProviderError::other(ProviderNames::Amadeus, step, message),
        }
    }

    fn format_error_body(body: Option<&str>) -> String {
        let Some(body) = body.map(str::trim).filter(|body| !body.is_empty()) else {
            return String::new();
        };

        if let Ok(envelope) = serde_json::from_str::<AmadeusErrorEnvelope>(body) {
            if let Some(first) = envelope.errors.into_iter().next() {
                let mut parts = Vec::new();

                if let Some(title) = first.title.filter(|title| !title.trim().is_empty()) {
                    parts.push(title);
                }

                if let Some(detail) = first.detail.filter(|detail| !detail.trim().is_empty()) {
                    parts.push(detail);
                }

                if let Some(parameter) = first
                    .source
                    .and_then(|source| source.parameter)
                    .filter(|parameter| !parameter.trim().is_empty())
                {
                    parts.push(format!("parameter: {parameter}"));
                }

                if !parts.is_empty() {
                    return parts.join(" | ");
                }
            }
        }

        body.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    fn format_date(date: (u32, u32, u32)) -> String {
        format!("{:04}-{:02}-{:02}", date.0, date.1, date.2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::thread::JoinHandle;

    use crate::ports::{ProviderErrorKind, ProviderSteps};

    struct TestHttpResponse {
        method: &'static str,
        path: &'static str,
        status: u16,
        body: &'static str,
    }

    fn spawn_test_server(responses: Vec<TestHttpResponse>) -> (String, JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
        let addr = listener.local_addr().expect("test server addr");

        let handle = std::thread::spawn(move || {
            for response in responses {
                let (mut stream, _) = listener.accept().expect("accept test request");
                stream
                    .set_read_timeout(Some(std::time::Duration::from_secs(2)))
                    .expect("set read timeout");

                let request = read_http_request(&mut stream);
                let request_line = request.lines().next().expect("request line");
                assert!(
                    request_line.starts_with(&format!("{} {} ", response.method, response.path)),
                    "unexpected request line: {request_line}"
                );

                write_http_response(&mut stream, response.status, response.body);
            }
        });

        (format!("http://{}", addr), handle)
    }

    fn read_http_request(stream: &mut TcpStream) -> String {
        let mut buffer = Vec::new();
        let mut chunk = [0_u8; 1024];
        let mut header_end = None;
        let mut content_length = 0usize;

        loop {
            let bytes_read = stream.read(&mut chunk).expect("read request bytes");
            if bytes_read == 0 {
                break;
            }
            buffer.extend_from_slice(&chunk[..bytes_read]);

            if header_end.is_none() {
                if let Some(idx) = buffer.windows(4).position(|w| w == b"\r\n\r\n") {
                    header_end = Some(idx + 4);
                    let headers = String::from_utf8_lossy(&buffer[..idx + 4]);
                    content_length = headers
                        .lines()
                        .find_map(|line| {
                            let lower = line.to_ascii_lowercase();
                            lower
                                .strip_prefix("content-length:")
                                .and_then(|value| value.trim().parse::<usize>().ok())
                        })
                        .unwrap_or(0);
                }
            }

            if let Some(end) = header_end {
                if buffer.len() >= end + content_length {
                    break;
                }
            }
        }

        String::from_utf8(buffer).expect("request utf8")
    }

    fn write_http_response(stream: &mut TcpStream, status: u16, body: &str) {
        let reason = match status {
            200 => "OK",
            400 => "Bad Request",
            401 => "Unauthorized",
            403 => "Forbidden",
            404 => "Not Found",
            429 => "Too Many Requests",
            500 => "Internal Server Error",
            503 => "Service Unavailable",
            _ => "Test Response",
        };

        let response = format!(
            "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream
            .write_all(response.as_bytes())
            .expect("write test response");
        stream.flush().expect("flush test response");
    }

    #[tokio::test]
    async fn client_reuses_unexpired_access_token() {
        let client = AmadeusClient::new_mock();

        let first = client.access_token().await.unwrap();
        let second = client.access_token().await.unwrap();

        assert_eq!(first, second);
    }

    #[tokio::test]
    async fn geocode_400_maps_to_invalid_request_error() {
        let (base_url, handle) = spawn_test_server(vec![
            TestHttpResponse {
                method: "POST",
                path: "/v1/security/oauth2/token",
                status: 200,
                body: r#"{"type":"amadeusOAuth2Token","username":"test","application_name":"app","client_id":"id","token_type":"Bearer","access_token":"token","expires_in":1799,"state":"approved","scope":""}"#,
            },
            TestHttpResponse {
                method: "GET",
                path: "/v1/reference-data/locations/hotels/by-geocode?latitude=25.2048493&longitude=55.2707828",
                status: 400,
                body: r#"{"errors":[{"status":400,"code":477,"title":"INVALID FORMAT","detail":"invalid query parameter format","source":{"parameter":"longitude"}}]}"#,
            },
        ]);

        let client = AmadeusClient::new(
            "client-id".to_string(),
            "client-secret".to_string(),
            Some(base_url.clone()),
            Some(base_url),
        );

        let error = client
            .get_hotels_by_geocode(25.2048493, 55.2707828)
            .await
            .expect_err("400 should map to provider error");

        handle.join().expect("join test server");

        assert_eq!(error.kind(), &ProviderErrorKind::InvalidRequest);
        assert_eq!(error.step(), &ProviderSteps::HotelSearch);
        assert!(error.message().contains("INVALID FORMAT"));
        assert!(error.message().contains("longitude"));
    }

    #[tokio::test]
    async fn hotel_details_404_maps_to_not_found_error() {
        let (base_url, handle) = spawn_test_server(vec![
            TestHttpResponse {
                method: "POST",
                path: "/v1/security/oauth2/token",
                status: 200,
                body: r#"{"type":"amadeusOAuth2Token","username":"test","application_name":"app","client_id":"id","token_type":"Bearer","access_token":"token","expires_in":1799,"state":"approved","scope":""}"#,
            },
            TestHttpResponse {
                method: "GET",
                path: "/v1/reference-data/locations/hotels/by-hotels?hotelIds=HIDXB891",
                status: 404,
                body: r#"{"errors":[{"status":404,"code":1797,"title":"NOT FOUND","detail":"requested property was not found"}]}"#,
            },
        ]);

        let client = AmadeusClient::new(
            "client-id".to_string(),
            "client-secret".to_string(),
            Some(base_url.clone()),
            Some(base_url),
        );

        let error = client
            .get_hotels_by_ids(&["HIDXB891".to_string()])
            .await
            .expect_err("404 should map to provider error");

        handle.join().expect("join test server");

        assert_eq!(error.kind(), &ProviderErrorKind::NotFound);
        assert_eq!(error.step(), &ProviderSteps::HotelDetails);
        assert!(error.message().contains("NOT FOUND"));
    }
}
