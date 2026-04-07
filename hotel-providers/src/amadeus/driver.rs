use async_trait::async_trait;
use std::collections::HashMap;

use crate::amadeus::client::AmadeusClient;
use crate::amadeus::mapper::AmadeusMapper;
use crate::domain::{
    DomainBlockRoomRequest, DomainBlockRoomResponse, DomainBookRoomRequest, DomainBookRoomResponse,
    DomainGetBookingRequest, DomainGetBookingResponse, DomainGroupedRoomRates,
    DomainHotelInfoCriteria, DomainHotelListAfterSearch, DomainHotelSearchCriteria,
    DomainHotelStaticDetails, DomainPrice,
};
use crate::ports::{
    HotelProviderPort, ProviderError, ProviderErrorKind, ProviderKeys, ProviderNames,
    ProviderSteps, UISearchFilters,
};

#[derive(Clone, Debug, Default)]
pub struct AmadeusDriver {
    client: AmadeusClient,
}

impl AmadeusDriver {
    pub fn new(client: AmadeusClient) -> Self {
        Self { client }
    }

    pub fn new_mock() -> Self {
        Self::new(AmadeusClient::new_mock())
    }

    pub fn client(&self) -> &AmadeusClient {
        &self.client
    }
}

#[async_trait]
impl HotelProviderPort for AmadeusDriver {
    fn key(&self) -> &'static str {
        ProviderKeys::Amadeus
    }

    fn name(&self) -> &'static str {
        ProviderNames::Amadeus
    }

    async fn search_hotels(
        &self,
        criteria: DomainHotelSearchCriteria,
        _ui_filters: UISearchFilters,
    ) -> Result<DomainHotelListAfterSearch, ProviderError> {
        let (latitude, longitude) = criteria.latitude.zip(criteria.longitude).ok_or_else(|| {
            ProviderError::new(
                self.name(),
                ProviderErrorKind::InvalidRequest,
                ProviderSteps::HotelSearch,
                "Amadeus search requires latitude and longitude",
            )
        })?;

        let hotel_list = self
            .client
            .get_hotels_by_geocode(latitude, longitude)
            .await?;
        if hotel_list.data.is_empty() {
            return Ok(DomainHotelListAfterSearch {
                hotel_results: Vec::new(),
                pagination: None,
                provider: Some(self.name().to_string()),
            });
        }

        let hotel_ids = hotel_list
            .data
            .iter()
            .map(|hotel| hotel.hotel_id.clone())
            .collect::<Vec<_>>();
        let offers = self.client.get_hotel_offers(&hotel_ids, &criteria).await?;

        Ok(AmadeusMapper::map_search_to_domain(
            hotel_list,
            offers,
            &criteria.pagination,
        ))
    }

    async fn get_hotel_static_details(
        &self,
        hotel_id: &str,
    ) -> Result<DomainHotelStaticDetails, ProviderError> {
        let response = self
            .client
            .get_hotels_by_ids(&[hotel_id.to_string()])
            .await?;

        AmadeusMapper::map_hotel_details_to_domain(response).ok_or_else(|| {
            ProviderError::not_found(
                self.name(),
                ProviderSteps::HotelDetails,
                "Amadeus hotel not found",
            )
        })
    }

    async fn get_hotel_rates(
        &self,
        criteria: DomainHotelInfoCriteria,
    ) -> Result<DomainGroupedRoomRates, ProviderError> {
        if criteria.hotel_ids.is_empty() {
            return Ok(DomainGroupedRoomRates {
                room_groups: Vec::new(),
                provider: Some(self.name().to_string()),
            });
        }

        let offers = self
            .client
            .get_hotel_offers(&criteria.hotel_ids, &criteria.search_criteria)
            .await?;

        Ok(AmadeusMapper::map_offers_to_grouped_rates(offers))
    }

    async fn get_min_rates(
        &self,
        _criteria: DomainHotelSearchCriteria,
        _hotel_ids: Vec<String>,
    ) -> Result<HashMap<String, DomainPrice>, ProviderError> {
        Err(ProviderError::other(
            self.name(),
            ProviderSteps::HotelRate,
            "Amadeus min rates are not implemented yet",
        ))
    }

    async fn block_room(
        &self,
        block_request: DomainBlockRoomRequest,
    ) -> Result<DomainBlockRoomResponse, ProviderError> {
        let offer_id = if !block_request.selected_room.offer_id.trim().is_empty() {
            block_request.selected_room.offer_id.clone()
        } else {
            block_request.selected_room.rate_key.clone()
        };
        let response = self.client.get_hotel_offer_by_id(&offer_id).await?;

        AmadeusMapper::map_offer_revalidation_to_domain_block(response, &block_request)
    }

    async fn book_room(
        &self,
        _book_request: DomainBookRoomRequest,
    ) -> Result<DomainBookRoomResponse, ProviderError> {
        Err(ProviderError::new(
            self.name(),
            ProviderErrorKind::InvalidRequest,
            ProviderSteps::HotelBookRoom,
            "Amadeus booking requires payment card details that are not present in DomainBookRoomRequest",
        ))
    }

    async fn get_booking_details(
        &self,
        _request: DomainGetBookingRequest,
    ) -> Result<DomainGetBookingResponse, ProviderError> {
        Err(ProviderError::other(
            self.name(),
            ProviderSteps::GetBookingDetails,
            "Amadeus booking lookup is not implemented yet",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::amadeus::models::booking::AmadeusBlockRoomSnapshot;
    use crate::amadeus::models::search::{
        AmadeusCancellationPolicy, AmadeusHotelOffer, AmadeusHotelOfferResponse, AmadeusOffer,
        AmadeusOfferHotel, AmadeusOfferPolicies, AmadeusOfferPrice, AmadeusOfferRoom,
        AmadeusRoomDescription, AmadeusRoomTypeEstimated,
    };
    use crate::domain::{
        DomainAdultDetail, DomainBlockRoomRequest, DomainBookRoomRequest, DomainBookingContext,
        DomainBookingGuest, DomainBookingHolder, DomainHotelInfoCriteria, DomainOriginalSearchInfo,
        DomainPaymentInfo, DomainPaymentMethod, DomainRoomData, DomainRoomGuest,
        DomainRoomOccupancyForBooking, DomainSelectedRoomWithQuantity, DomainUserDetails,
    };
    use crate::ports::{HotelProviderPort, ProviderErrorKind, ProviderSteps};

    fn sample_search_criteria() -> DomainHotelSearchCriteria {
        DomainHotelSearchCriteria {
            place_id: "ChIJLU7jZClu5kcR4PcOOO6p3I0".to_string(),
            check_in_date: (2026, 5, 1),
            check_out_date: (2026, 5, 2),
            no_of_nights: 1,
            no_of_rooms: 1,
            room_guests: vec![DomainRoomGuest {
                no_of_adults: 1,
                no_of_children: 0,
                children_ages: None,
            }],
            guest_nationality: "IN".to_string(),
            pagination: None,
            latitude: Some(48.8757),
            longitude: Some(2.32553),
            provider: None,
        }
    }

    fn sample_offer_lookup_response(total: &str) -> AmadeusHotelOfferResponse {
        AmadeusHotelOfferResponse {
            data: AmadeusHotelOffer {
                hotel: AmadeusOfferHotel {
                    hotel_id: "HLPAR266".to_string(),
                    name: "Hilton Paris Opera".to_string(),
                    city_code: Some("PAR".to_string()),
                    latitude: Some(48.8757),
                    longitude: Some(2.32553),
                },
                available: true,
                offers: vec![AmadeusOffer {
                    id: "ZBC0IYFMFW".to_string(),
                    check_in_date: "2026-05-01".to_string(),
                    check_out_date: "2026-05-02".to_string(),
                    room: Some(AmadeusOfferRoom {
                        description: Some(AmadeusRoomDescription {
                            text: "Deluxe King Room".to_string(),
                            lang: Some("EN".to_string()),
                        }),
                        type_estimated: Some(AmadeusRoomTypeEstimated {
                            category: Some("DELUXE_ROOM".to_string()),
                            beds: Some(1),
                            bed_type: Some("KING".to_string()),
                        }),
                    }),
                    price: AmadeusOfferPrice {
                        currency: "EUR".to_string(),
                        total: total.to_string(),
                        base: Some("200.00".to_string()),
                    },
                    policies: Some(AmadeusOfferPolicies {
                        cancellations: Some(vec![AmadeusCancellationPolicy {
                            amount: Some("0.00".to_string()),
                            deadline: Some("2026-04-30T23:59:00Z".to_string()),
                            description: Some(AmadeusRoomDescription {
                                text: "Free cancellation until the day before arrival".to_string(),
                                lang: Some("EN".to_string()),
                            }),
                        }]),
                    }),
                }],
            },
        }
    }

    fn sample_block_request() -> DomainBlockRoomRequest {
        DomainBlockRoomRequest {
            hotel_info_criteria: DomainHotelInfoCriteria {
                token: "search-token".to_string(),
                hotel_ids: vec!["HLPAR266".to_string()],
                search_criteria: sample_search_criteria(),
                provider: Some("amadeus".to_string()),
            },
            user_details: DomainUserDetails {
                children: Vec::new(),
                adults: vec![DomainAdultDetail {
                    email: Some("bob.smith@email.com".to_string()),
                    first_name: "Bob".to_string(),
                    last_name: Some("Smith".to_string()),
                    phone: Some("+33679278416".to_string()),
                }],
            },
            selected_rooms: vec![DomainSelectedRoomWithQuantity {
                room_data: DomainRoomData {
                    mapped_room_id: "HLPAR266:deluxe-king-room".to_string(),
                    occupancy_number: Some(1),
                    room_name: "Deluxe King Room".to_string(),
                    room_unique_id: "room-1".to_string(),
                    rate_key: "ZBC0IYFMFW".to_string(),
                    offer_id: "ZBC0IYFMFW".to_string(),
                },
                quantity: 1,
                price_per_night: 225.0,
            }],
            selected_room: DomainRoomData {
                mapped_room_id: "HLPAR266:deluxe-king-room".to_string(),
                occupancy_number: Some(1),
                room_name: "Deluxe King Room".to_string(),
                room_unique_id: "room-1".to_string(),
                rate_key: "ZBC0IYFMFW".to_string(),
                offer_id: "ZBC0IYFMFW".to_string(),
            },
            total_guests: 1,
            special_requests: Some("I will arrive at midnight".to_string()),
        }
    }

    fn sample_book_request(block_id: String) -> DomainBookRoomRequest {
        DomainBookRoomRequest {
            block_id,
            provider: Some(ProviderNames::Amadeus.to_string()),
            holder: DomainBookingHolder {
                first_name: "Bob".to_string(),
                last_name: "Smith".to_string(),
                email: "bob.smith@email.com".to_string(),
                phone: "+33679278416".to_string(),
            },
            guests: vec![DomainBookingGuest {
                occupancy_number: 1,
                first_name: "Bob".to_string(),
                last_name: "Smith".to_string(),
                email: "bob.smith@email.com".to_string(),
                phone: "+33679278416".to_string(),
                remarks: None,
            }],
            payment: DomainPaymentInfo {
                method: DomainPaymentMethod::AccCreditCard,
            },
            guest_payment: None,
            special_requests: Some("I will arrive at midnight".to_string()),
            booking_context: DomainBookingContext {
                number_of_rooms: 1,
                room_occupancies: vec![DomainRoomOccupancyForBooking {
                    room_number: 1,
                    adults: 1,
                    children: 0,
                    children_ages: Vec::new(),
                }],
                total_guests: 1,
                original_search_criteria: Some(DomainOriginalSearchInfo {
                    hotel_id: "HLPAR266".to_string(),
                    checkin_date: "2026-05-01".to_string(),
                    checkout_date: "2026-05-02".to_string(),
                    guest_nationality: Some("IN".to_string()),
                }),
            },
            client_reference: Some("app-ref-123".to_string()),
        }
    }

    #[tokio::test]
    async fn block_room_revalidates_offer_and_returns_provider_snapshot() {
        let client = AmadeusClient::new_mock()
            .with_mock_offer_lookup_response(sample_offer_lookup_response("225.00"));
        let driver = AmadeusDriver::new(client);

        let response = driver.block_room(sample_block_request()).await.unwrap();

        assert_eq!(response.provider.as_deref(), Some("Amadeus"));
        assert!(response.block_id.starts_with("amadeus-block:"));
        assert!(!response.is_price_changed);
        assert!(!response.is_cancellation_policy_changed);
        assert_eq!(response.total_price.room_price, 225.0);
        assert_eq!(response.total_price.currency_code, "EUR");
        assert_eq!(response.blocked_rooms.len(), 1);
        assert_eq!(response.blocked_rooms[0].room_name, "Deluxe King Room");

        let snapshot: AmadeusBlockRoomSnapshot =
            serde_json::from_str(response.provider_data.as_deref().unwrap()).unwrap();
        assert_eq!(snapshot.offer_id, "ZBC0IYFMFW");
        assert_eq!(snapshot.hotel_id, "HLPAR266");
        assert_eq!(snapshot.room_name, "Deluxe King Room");
        assert_eq!(snapshot.total_price, 225.0);
        assert_eq!(snapshot.currency_code, "EUR");
    }

    #[tokio::test]
    async fn book_room_returns_invalid_request_when_payment_card_data_is_unavailable() {
        let driver = AmadeusDriver::new_mock();

        let error = driver
            .book_room(sample_book_request("amadeus-block:opaque".to_string()))
            .await
            .unwrap_err();

        assert_eq!(error.kind(), &ProviderErrorKind::InvalidRequest);
        assert_eq!(error.step(), &ProviderSteps::HotelBookRoom);
        assert!(error.message().contains("payment card details"));
    }
}
