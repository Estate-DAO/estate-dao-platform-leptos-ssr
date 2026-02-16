use crate::booking::models::{
    AccommodationsAvailabilityInput, AccommodationsAvailabilityOutput,
    AccommodationsBulkAvailabilityInput, AccommodationsBulkAvailabilityOutput,
    AccommodationsDetailsInput, AccommodationsDetailsOutput, AccommodationsSearchInput,
    AccommodationsSearchOutput, AccommodationDetails, AvailabilityData, AvailabilityProduct,
    CancellationPolicy, CancellationSchedule, LocationCoordinates, LocationInfo, MealPlanPolicy,
    OrdersDetailsAccommodationsInput, OrdersDetailsAccommodationsOutput,
    OrdersDetailsAccommodationOutput, OrdersPreviewAccommodationOutput, OrdersPreviewData,
    OrdersPreviewInput, OrdersPreviewOutput, OrdersPreviewProductOutput, OrderCreateInput,
    OrderCreateOutput, OrderCreateDataOutput, OrderCreateAccommodationOutput, PriceCurrency,
    PreviewProductPrice, ProductPolicies, ReviewInfo, RoomInfo, SearchPrice, SearchProduct,
    TranslatedString,
};
use crate::ports::{ProviderError, ProviderErrorKind, ProviderSteps};
use reqwest::{Client, StatusCode};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct BookingClient {
    client: Client,
    base_url: String,
    token: String,
    affiliate_id: i64,
    currency: String,
}

impl BookingClient {
    pub fn new(token: String, affiliate_id: i64, base_url: String, currency: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            token,
            affiliate_id,
            currency,
        }
    }

    pub fn currency(&self) -> &str {
        &self.currency
    }

    async fn post<T, R>(&self, path: &str, body: &T, step: ProviderSteps) -> Result<R, ProviderError>
    where
        T: Serialize,
        R: DeserializeOwned,
    {
        let url = self.build_url(path);
        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("X-Affiliate-Id", self.affiliate_id.to_string())
            .json(body)
            .send()
            .await
            .map_err(|e| ProviderError::network("Booking.com", step.clone(), e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(self.map_api_error(status, text, step));
        }

        resp.json::<R>().await.map_err(|e| {
            ProviderError::parse_error("Booking.com", step, e.to_string())
        })
    }

    fn build_url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    fn map_api_error(
        &self,
        status: StatusCode,
        text: String,
        step: ProviderSteps,
    ) -> ProviderError {
        let msg = format!("Booking.com Error {}: {}", status, text);
        match status {
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                ProviderError::new("Booking.com", ProviderErrorKind::Auth, step, msg)
            }
            StatusCode::NOT_FOUND => ProviderError::not_found("Booking.com", step, msg),
            StatusCode::TOO_MANY_REQUESTS => {
                ProviderError::new("Booking.com", ProviderErrorKind::RateLimited, step, msg)
            }
            StatusCode::INTERNAL_SERVER_ERROR
            | StatusCode::BAD_GATEWAY
            | StatusCode::SERVICE_UNAVAILABLE => {
                ProviderError::service_unavailable("Booking.com", step, msg)
            }
            _ => ProviderError::new("Booking.com", ProviderErrorKind::Other, step, msg),
        }
    }

    pub async fn search_accommodations(
        &self,
        req: &AccommodationsSearchInput,
    ) -> Result<AccommodationsSearchOutput, ProviderError> {
        self.post("/accommodations/search", req, ProviderSteps::HotelSearch)
            .await
    }

    pub async fn get_accommodation_details(
        &self,
        req: &AccommodationsDetailsInput,
    ) -> Result<AccommodationsDetailsOutput, ProviderError> {
        self.post("/accommodations/details", req, ProviderSteps::HotelDetails)
            .await
    }

    pub async fn get_availability(
        &self,
        req: &AccommodationsAvailabilityInput,
    ) -> Result<AccommodationsAvailabilityOutput, ProviderError> {
        self.post("/accommodations/availability", req, ProviderSteps::HotelRate)
            .await
    }

    pub async fn get_bulk_availability(
        &self,
        req: &AccommodationsBulkAvailabilityInput,
    ) -> Result<AccommodationsBulkAvailabilityOutput, ProviderError> {
        self.post(
            "/accommodations/bulk-availability",
            req,
            ProviderSteps::HotelRate,
        )
        .await
    }

    pub async fn orders_preview(
        &self,
        req: &OrdersPreviewInput,
    ) -> Result<OrdersPreviewOutput, ProviderError> {
        self.post("/orders/preview", req, ProviderSteps::HotelBlockRoom)
            .await
    }

    pub async fn orders_create(
        &self,
        req: &OrderCreateInput,
    ) -> Result<OrderCreateOutput, ProviderError> {
        self.post("/orders/create", req, ProviderSteps::HotelBookRoom)
            .await
    }

    pub async fn orders_details_accommodations(
        &self,
        req: &OrdersDetailsAccommodationsInput,
    ) -> Result<OrdersDetailsAccommodationsOutput, ProviderError> {
        self.post(
            "/orders/details/accommodations",
            req,
            ProviderSteps::GetBookingDetails,
        )
        .await
    }
}

#[derive(Clone, Debug)]
pub struct BookingMockClient {
    currency: String,
}

impl BookingMockClient {
    pub fn new(currency: String) -> Self {
        Self { currency }
    }

    pub fn currency(&self) -> &str {
        &self.currency
    }

    pub async fn search_accommodations(
        &self,
        _req: &AccommodationsSearchInput,
    ) -> Result<AccommodationsSearchOutput, ProviderError> {
        Ok(AccommodationsSearchOutput {
            request_id: Some("mock-search".to_string()),
            data: vec![
                Self::mock_search_data(10101, 120.0),
                Self::mock_search_data(20202, 155.0),
                Self::mock_search_data(30303, 98.0),
            ],
            next_page: None,
        })
    }

    pub async fn get_accommodation_details(
        &self,
        req: &AccommodationsDetailsInput,
    ) -> Result<AccommodationsDetailsOutput, ProviderError> {
        let ids = req.accommodations.clone().unwrap_or_default();
        let data = if ids.is_empty() {
            vec![Self::mock_details(10101)]
        } else {
            ids.into_iter().map(Self::mock_details).collect()
        };
        Ok(AccommodationsDetailsOutput {
            request_id: Some("mock-details".to_string()),
            data,
        })
    }

    pub async fn get_availability(
        &self,
        req: &AccommodationsAvailabilityInput,
    ) -> Result<AccommodationsAvailabilityOutput, ProviderError> {
        let products = vec![
            Self::mock_availability_product(format!("{}_P1", req.accommodation), 110.0),
            Self::mock_availability_product(format!("{}_P2", req.accommodation), 145.0),
        ];
        Ok(AccommodationsAvailabilityOutput {
            request_id: Some("mock-availability".to_string()),
            data: AvailabilityData {
                id: req.accommodation,
                currency: Some(self.currency.clone()),
                products: Some(products),
            },
        })
    }

    pub async fn get_bulk_availability(
        &self,
        req: &AccommodationsBulkAvailabilityInput,
    ) -> Result<AccommodationsBulkAvailabilityOutput, ProviderError> {
        let data = req
            .accommodations
            .iter()
            .map(|id| AvailabilityData {
                id: *id,
                currency: Some(self.currency.clone()),
                products: Some(vec![Self::mock_availability_product(
                    format!("{id}_P1"),
                    125.0,
                )]),
            })
            .collect();
        Ok(AccommodationsBulkAvailabilityOutput {
            request_id: Some("mock-bulk-availability".to_string()),
            data,
        })
    }

    pub async fn orders_preview(
        &self,
        req: &OrdersPreviewInput,
    ) -> Result<OrdersPreviewOutput, ProviderError> {
        let products = req
            .accommodation
            .products
            .iter()
            .map(|p| OrdersPreviewProductOutput {
                id: p.id.clone(),
                price: Some(PreviewProductPrice {
                    base: Some(PriceCurrency {
                        accommodation_currency: Some(100.0),
                        booker_currency: None,
                    }),
                    total: Some(PriceCurrency {
                        accommodation_currency: Some(120.0),
                        booker_currency: None,
                    }),
                }),
                policies: Some(ProductPolicies {
                    cancellation: Some(CancellationPolicy {
                        schedule: Some(vec![CancellationSchedule {
                            from: Some("now".to_string()),
                            price: 0.0,
                        }]),
                        kind: Some("free_cancellation".to_string()),
                    }),
                    meal_plan: Some(MealPlanPolicy {
                        plan: Some("breakfast_included".to_string()),
                        meals: Some(vec!["breakfast".to_string()]),
                    }),
                    payment: None,
                }),
            })
            .collect();

        Ok(OrdersPreviewOutput {
            request_id: Some("mock-preview".to_string()),
            data: OrdersPreviewData {
                accommodation: Some(OrdersPreviewAccommodationOutput {
                    id: req.accommodation.id,
                    products: Some(products),
                }),
                order_token: format!("mock-token-{}", req.accommodation.id),
            },
        })
    }

    pub async fn orders_create(
        &self,
        req: &OrderCreateInput,
    ) -> Result<OrderCreateOutput, ProviderError> {
        Ok(OrderCreateOutput {
            request_id: Some("mock-create".to_string()),
            data: Some(OrderCreateDataOutput {
                accommodation: Some(OrderCreateAccommodationOutput {
                    order: Some(format!("order_{}", req.order_token)),
                    reservation: Some(5550001),
                    pincode: Some("1234".to_string()),
                    third_party_inventory: None,
                }),
            }),
        })
    }

    pub async fn orders_details_accommodations(
        &self,
        req: &OrdersDetailsAccommodationsInput,
    ) -> Result<OrdersDetailsAccommodationsOutput, ProviderError> {
        let mut data = Vec::new();
        if let Some(orders) = &req.orders {
            for order in orders {
                data.push(OrdersDetailsAccommodationOutput {
                    id: order.clone(),
                    accommodation: Some(10101),
                    accommodation_details: Some(crate::booking::models::OrderAccommodationDetailsOutput {
                        name: Some(format!("Mock Hotel for {}", order)),
                    }),
                });
            }
        }
        Ok(OrdersDetailsAccommodationsOutput {
            request_id: Some("mock-details-orders".to_string()),
            data,
        })
    }

    fn mock_search_data(id: i64, price: f64) -> crate::booking::models::AccommodationsSearchData {
        crate::booking::models::AccommodationsSearchData {
            id,
            currency: Some("USD".to_string()),
            price: Some(SearchPrice {
                base: Some(price),
                book: Some(price + 12.0),
                total: Some(price + 20.0),
                extra_charges: Some(crate::booking::models::ExtraChargesSimple {
                    included: Some(12.0),
                    excluded: Some(8.0),
                }),
            }),
            products: Some(vec![SearchProduct {
                id: format!("{id}_P1"),
            }]),
            deep_link_url: None,
        }
    }

    fn mock_details(id: i64) -> AccommodationDetails {
        let mut description: TranslatedString = HashMap::new();
        description.insert(
            "en-gb".to_string(),
            Some(format!("Mock description for accommodation {id}.")),
        );

        AccommodationDetails {
            id,
            name: Some(format!("Mock Hotel {}", id)),
            location: Some(LocationInfo {
                address: Some({
                    let mut addr: TranslatedString = HashMap::new();
                    addr.insert("en-gb".to_string(), Some("123 Mock Street".to_string()));
                    addr
                }),
                coordinates: Some(LocationCoordinates {
                    latitude: Some(40.7128),
                    longitude: Some(-74.0060),
                }),
                country: Some("us".to_string()),
            }),
            review: Some(ReviewInfo {
                review_score: Some(8.4),
                number_of_reviews: Some(120),
                stars: Some(4.0),
            }),
            photos: None,
            facilities: None,
            description: Some(description),
            rooms: Some(vec![RoomInfo {
                id: 1,
                name: Some("Mock Room".to_string()),
                description: None,
                photos: None,
                facilities: None,
            }]),
        }
    }

    fn mock_availability_product(id: String, price: f64) -> AvailabilityProduct {
        AvailabilityProduct {
            id,
            number_available_at_this_price: Some(5),
            maximum_occupancy: None,
            price: Some(SearchPrice {
                base: Some(price),
                book: Some(price + 10.0),
                total: Some(price + 18.0),
                extra_charges: Some(crate::booking::models::ExtraChargesSimple {
                    included: Some(10.0),
                    excluded: Some(8.0),
                }),
            }),
            policies: Some(ProductPolicies {
                cancellation: Some(CancellationPolicy {
                    schedule: Some(vec![CancellationSchedule {
                        from: Some("now".to_string()),
                        price: 0.0,
                    }]),
                    kind: Some("free_cancellation".to_string()),
                }),
                meal_plan: Some(MealPlanPolicy {
                    plan: Some("breakfast_included".to_string()),
                    meals: Some(vec!["breakfast".to_string()]),
                }),
                payment: None,
            }),
        }
    }
}

#[derive(Clone, Debug)]
pub enum BookingApiClient {
    Real(BookingClient),
    Mock(BookingMockClient),
}

impl BookingApiClient {
    pub fn currency(&self) -> &str {
        match self {
            BookingApiClient::Real(client) => client.currency(),
            BookingApiClient::Mock(client) => client.currency(),
        }
    }

    pub async fn search_accommodations(
        &self,
        req: &AccommodationsSearchInput,
    ) -> Result<AccommodationsSearchOutput, ProviderError> {
        match self {
            BookingApiClient::Real(client) => client.search_accommodations(req).await,
            BookingApiClient::Mock(client) => client.search_accommodations(req).await,
        }
    }

    pub async fn get_accommodation_details(
        &self,
        req: &AccommodationsDetailsInput,
    ) -> Result<AccommodationsDetailsOutput, ProviderError> {
        match self {
            BookingApiClient::Real(client) => client.get_accommodation_details(req).await,
            BookingApiClient::Mock(client) => client.get_accommodation_details(req).await,
        }
    }

    pub async fn get_availability(
        &self,
        req: &AccommodationsAvailabilityInput,
    ) -> Result<AccommodationsAvailabilityOutput, ProviderError> {
        match self {
            BookingApiClient::Real(client) => client.get_availability(req).await,
            BookingApiClient::Mock(client) => client.get_availability(req).await,
        }
    }

    pub async fn get_bulk_availability(
        &self,
        req: &AccommodationsBulkAvailabilityInput,
    ) -> Result<AccommodationsBulkAvailabilityOutput, ProviderError> {
        match self {
            BookingApiClient::Real(client) => client.get_bulk_availability(req).await,
            BookingApiClient::Mock(client) => client.get_bulk_availability(req).await,
        }
    }

    pub async fn orders_preview(
        &self,
        req: &OrdersPreviewInput,
    ) -> Result<OrdersPreviewOutput, ProviderError> {
        match self {
            BookingApiClient::Real(client) => client.orders_preview(req).await,
            BookingApiClient::Mock(client) => client.orders_preview(req).await,
        }
    }

    pub async fn orders_create(
        &self,
        req: &OrderCreateInput,
    ) -> Result<OrderCreateOutput, ProviderError> {
        match self {
            BookingApiClient::Real(client) => client.orders_create(req).await,
            BookingApiClient::Mock(client) => client.orders_create(req).await,
        }
    }

    pub async fn orders_details_accommodations(
        &self,
        req: &OrdersDetailsAccommodationsInput,
    ) -> Result<OrdersDetailsAccommodationsOutput, ProviderError> {
        match self {
            BookingApiClient::Real(client) => client.orders_details_accommodations(req).await,
            BookingApiClient::Mock(client) => client.orders_details_accommodations(req).await,
        }
    }
}
