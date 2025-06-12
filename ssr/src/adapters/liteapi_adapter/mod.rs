cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {
        pub mod impl_hotel_provider_trait;
    }
}

use std::sync::Arc;

use crate::api::api_client::ApiClient;
use crate::api::liteapi::{
    liteapi_hotel_rates, liteapi_hotel_search, liteapi_prebook, LiteApiHTTPClient,
    LiteApiHotelRatesRequest, LiteApiHotelRatesResponse, LiteApiHotelResult,
    LiteApiHotelSearchRequest, LiteApiHotelSearchResponse, LiteApiOccupancy, LiteApiPrebookRequest,
    LiteApiPrebookResponse,
};
use crate::utils;
use futures::future::{BoxFuture, FutureExt};

use crate::api::ApiError;
use crate::application_services::filter_types::UISearchFilters;
use crate::domain::{
    DomainBlockRoomRequest, DomainBlockRoomResponse, DomainBlockedRoom, DomainDetailedPrice,
    DomainFirstRoomDetails, DomainHotelAfterSearch, DomainHotelDetails, DomainHotelInfoCriteria,
    DomainHotelListAfterSearch, DomainHotelSearchCriteria, DomainPrice, DomainRoomData,
};
use crate::ports::hotel_provider_port::{ProviderError, ProviderErrorDetails, ProviderSteps};
use crate::ports::ProviderNames;
use crate::utils::date::date_tuple_to_dd_mm_yyyy;
use async_trait::async_trait;

#[derive(Clone)]
pub struct LiteApiAdapter {
    client: LiteApiHTTPClient,
}

impl LiteApiAdapter {
    pub fn new(client: LiteApiHTTPClient) -> Self {
        Self { client }
    }

    // --- Mapping functions ---
    fn map_domain_search_to_liteapi(
        domain_criteria: &DomainHotelSearchCriteria,
        ui_filters: UISearchFilters,
    ) -> LiteApiHotelSearchRequest {
        LiteApiHotelSearchRequest {
            country_code: domain_criteria.destination_country_code.clone(),
            city_name: domain_criteria.destination_city_name.clone(), // Assuming this field exists
            offset: 0,
            limit: 50,
        }
    }

    fn map_liteapi_search_to_domain(
        liteapi_response: LiteApiHotelSearchResponse,
    ) -> DomainHotelListAfterSearch {
        DomainHotelListAfterSearch {
            hotel_results: liteapi_response
                .data
                .into_iter()
                .map(|hotel| Self::map_liteapi_hotel_to_domain(hotel))
                .collect(),
        }
    }

    fn map_liteapi_hotel_to_domain(liteapi_hotel: LiteApiHotelResult) -> DomainHotelAfterSearch {
        let hotel_id = liteapi_hotel.id.clone();
        DomainHotelAfterSearch {
            hotel_code: hotel_id.clone(),
            hotel_name: liteapi_hotel.name,
            hotel_category: format!("{} Star", liteapi_hotel.stars),
            star_rating: liteapi_hotel.stars as u8,
            price: DomainPrice {
                room_price: 0.0, // LiteAPI doesn't provide price in search
                currency_code: liteapi_hotel.currency,
            },
            hotel_picture: liteapi_hotel.main_photo,
            result_token: hotel_id,
        }
    }

    // Convert domain search criteria to LiteAPI hotel rates request
    fn map_domain_info_to_liteapi_rates(
        domain_criteria: &DomainHotelInfoCriteria,
    ) -> Result<LiteApiHotelRatesRequest, ProviderError> {
        let search_criteria = &domain_criteria.search_criteria;

        // Convert room guests to occupancies
        let occupancies: Vec<LiteApiOccupancy> = search_criteria
            .room_guests
            .iter()
            .map(|room_guest| {
                let children = if room_guest.no_of_children > 0 {
                    room_guest.children_ages.as_ref().map(|ages| {
                        ages.iter()
                            .filter_map(|age_str| age_str.parse::<u32>().ok())
                            .collect()
                    })
                } else {
                    None
                };

                LiteApiOccupancy {
                    adults: room_guest.no_of_adults,
                    children,
                }
            })
            .collect();

        // Format dates as YYYY-MM-DD
        let checkin = utils::date::date_tuple_to_yyyy_mm_dd(search_criteria.check_in_date.clone());
        let checkout =
            utils::date::date_tuple_to_yyyy_mm_dd(search_criteria.check_out_date.clone());

        // Get hotel IDs - use provided hotel_ids or fall back to token
        let hotel_ids = if !domain_criteria.hotel_ids.is_empty() {
            domain_criteria.hotel_ids.clone()
        } else if !domain_criteria.token.is_empty() {
            // For LiteAPI, the token is the hotel ID from search results
            vec![domain_criteria.token.clone()]
        } else {
            return Err(ProviderError(Arc::new(ProviderErrorDetails {
                provider_name: ProviderNames::LiteApi,
                api_error: ApiError::Other(
                    "No hotel ID available. Either hotel_ids or token must be provided."
                        .to_string(),
                ),
                error_step: ProviderSteps::HotelDetails,
            })));
        };

        Ok(LiteApiHotelRatesRequest {
            hotel_ids,
            occupancies,
            currency: "USD".to_string(), // Could be configurable
            guest_nationality: search_criteria.guest_nationality.clone(),
            checkin,
            checkout,
        })
    }

    // Map LiteAPI rates response to domain hotel details
    fn map_liteapi_rates_to_domain_details(
        liteapi_response: LiteApiHotelRatesResponse,
        search_criteria: &DomainHotelSearchCriteria,
    ) -> Result<DomainHotelDetails, ProviderError> {
        // Check if we have any data
        if liteapi_response.data.is_empty() {
            return Err(ProviderError(Arc::new(ProviderErrorDetails {
                provider_name: ProviderNames::LiteApi,
                api_error: ApiError::Other(
                    "Empty response from LiteAPI rates endpoint. Hotel may not be available for the selected dates.".to_string()
                ),
                error_step: ProviderSteps::HotelDetails,
            })));
        }

        // Get first hotel data
        let hotel_data = liteapi_response.get_first_hotel_data().ok_or_else(|| {
            ProviderError(Arc::new(ProviderErrorDetails {
                provider_name: ProviderNames::LiteApi,
                api_error: ApiError::Other("No hotel data found in rates response".to_string()),
                error_step: ProviderSteps::HotelDetails,
            }))
        })?;

        // Get first room type and rate
        let room_type = liteapi_response.get_first_room_type().ok_or_else(|| {
            ProviderError(Arc::new(ProviderErrorDetails {
                provider_name: ProviderNames::LiteApi,
                api_error: ApiError::Other(
                    "No room types available for this hotel. It may be fully booked for the selected dates.".to_string()
                ),
                error_step: ProviderSteps::HotelDetails,
            }))
        })?;

        let rate = liteapi_response.get_first_rate().ok_or_else(|| {
            ProviderError(Arc::new(ProviderErrorDetails {
                provider_name: ProviderNames::LiteApi,
                api_error: ApiError::Other(
                    "No rates available for this room type. Please try different dates or occupancy.".to_string()
                ),
                error_step: ProviderSteps::HotelDetails,
            }))
        })?;

        // Extract price from first rate
        let room_price = rate
            .retail_rate
            .total
            .first()
            .map(|amount| amount.amount)
            .unwrap_or(0.0);

        let currency_code = rate
            .retail_rate
            .total
            .first()
            .map(|amount| amount.currency.clone())
            .unwrap_or_else(|| "USD".to_string());

        // Build domain hotel details
        Ok(DomainHotelDetails {
            checkin: format!(
                "{}-{:02}-{:02}",
                search_criteria.check_in_date.0,
                search_criteria.check_in_date.1,
                search_criteria.check_in_date.2
            ),
            checkout: format!(
                "{}-{:02}-{:02}",
                search_criteria.check_out_date.0,
                search_criteria.check_out_date.1,
                search_criteria.check_out_date.2
            ),
            hotel_name: search_criteria.destination_city_name.clone(), // Would need to get from search results
            hotel_code: hotel_data.hotel_id.clone(),
            star_rating: 0,             // Would need to get from search results
            description: String::new(), // LiteAPI doesn't provide in rates endpoint
            hotel_facilities: vec![],
            address: String::new(), // Would need to get from search results
            images: vec![],         // Would need to get from search results
            first_room_details: DomainFirstRoomDetails {
                price: DomainDetailedPrice {
                    published_price: room_price,
                    published_price_rounded_off: room_price,
                    offered_price: room_price,
                    offered_price_rounded_off: room_price,
                    room_price,
                    tax: 0.0,
                    extra_guest_charge: 0.0,
                    child_charge: 0.0,
                    other_charges: 0.0,
                    currency_code: currency_code.clone(),
                },
                room_data: DomainRoomData {
                    room_name: rate.name.clone(),
                    room_unique_id: room_type.room_type_id.clone(),
                    rate_key: rate.rate_id.clone(),
                    offer_id: room_type.offer_id.clone(),
                },
            },
            amenities: vec![],
        })
    }

    // --- Block Room Mapping Functions ---

    // Map domain block room request to LiteAPI prebook request
    fn map_domain_block_to_liteapi_prebook(
        domain_request: &DomainBlockRoomRequest,
    ) -> Result<LiteApiPrebookRequest, ProviderError> {
        // Extract offer ID from selected room
        let offer_id = &domain_request.selected_room.offer_id;

        if offer_id.is_empty() {
            return Err(ProviderError(Arc::new(ProviderErrorDetails {
                provider_name: ProviderNames::LiteApi,
                api_error: ApiError::Other(
                    "Offer ID is required for LiteAPI prebook. Ensure hotel details were fetched correctly.".to_string(),
                ),
                error_step: ProviderSteps::HotelBlockRoom,
            })));
        }

        Ok(LiteApiPrebookRequest {
            offer_id: offer_id.clone(),
            use_payment_sdk: true, // Always true for our use case
        })
    }

    // Map LiteAPI prebook response to domain block room response
    fn map_liteapi_prebook_to_domain_block(
        liteapi_response: LiteApiPrebookResponse,
        original_request: &DomainBlockRoomRequest,
    ) -> DomainBlockRoomResponse {
        // Extract price information from prebook response
        let total_price = liteapi_response.data.price;
        let suggested_selling_price = liteapi_response.data.suggested_selling_price;
        let currency = liteapi_response.data.currency.clone();

        // Get room details from first room type and rate if available
        let (room_name, cancellation_policy, meal_plan) =
            if let Some(room_type) = liteapi_response.data.room_types.first() {
                if let Some(rate) = room_type.rates.first() {
                    (
                        rate.name.clone(),
                        Some(rate.cancellation_policies.refundable_tag.clone()),
                        Some(format!("{} - {}", rate.board_type, rate.board_name)),
                    )
                } else {
                    (original_request.selected_room.room_name.clone(), None, None)
                }
            } else {
                (original_request.selected_room.room_name.clone(), None, None)
            };

        // Create detailed price from prebook response
        let detailed_price = DomainDetailedPrice {
            published_price: suggested_selling_price,
            published_price_rounded_off: suggested_selling_price.round(),
            offered_price: total_price,
            offered_price_rounded_off: total_price.round(),
            room_price: total_price,
            tax: 0.0, // Could extract from taxes_and_fees if needed
            extra_guest_charge: 0.0,
            child_charge: 0.0,
            other_charges: 0.0,
            currency_code: currency.clone(),
        };

        // Create a blocked room entry from the prebook response
        let blocked_room = DomainBlockedRoom {
            room_code: original_request.selected_room.room_unique_id.clone(),
            room_name,
            room_type_code: Some(original_request.selected_room.room_unique_id.clone()),
            price: detailed_price.clone(),
            cancellation_policy,
            meal_plan,
        };

        DomainBlockRoomResponse {
            block_id: liteapi_response.data.prebook_id.clone(),
            is_price_changed: liteapi_response.data.price_difference_percent != 0.0,
            is_cancellation_policy_changed: liteapi_response.data.cancellation_changed,
            blocked_rooms: vec![blocked_room],
            total_price: detailed_price,
            provider_data: Some(serde_json::to_string(&liteapi_response).unwrap_or_default()),
        }
    }

    // --- Validation Functions ---

    // Validate block room request before processing
    fn validate_block_room_request(request: &DomainBlockRoomRequest) -> Result<(), ProviderError> {
        // Validate guest details match room occupancy
        let total_adults = request.user_details.adults.len() as u32;
        let total_children = request.user_details.children.len() as u32;
        let total_guests_from_details = total_adults + total_children;

        if total_guests_from_details != request.total_guests {
            return Err(ProviderError(Arc::new(ProviderErrorDetails {
                provider_name: ProviderNames::LiteApi,
                api_error: ApiError::Other(format!(
                    "Guest count mismatch. Expected {} guests but got {} in details",
                    request.total_guests, total_guests_from_details
                )),
                error_step: ProviderSteps::HotelBlockRoom,
            })));
        }

        // Validate essential fields are not empty
        if request.selected_room.room_unique_id.is_empty() {
            return Err(ProviderError(Arc::new(ProviderErrorDetails {
                provider_name: ProviderNames::LiteApi,
                api_error: ApiError::Other("Room unique ID cannot be empty".to_string()),
                error_step: ProviderSteps::HotelBlockRoom,
            })));
        }

        // Validate at least one adult is present
        if request.user_details.adults.is_empty() {
            return Err(ProviderError(Arc::new(ProviderErrorDetails {
                provider_name: ProviderNames::LiteApi,
                api_error: ApiError::Other("At least one adult guest is required".to_string()),
                error_step: ProviderSteps::HotelBlockRoom,
            })));
        }

        // Validate adult details
        for (i, adult) in request.user_details.adults.iter().enumerate() {
            if adult.first_name.trim().is_empty() {
                return Err(ProviderError(Arc::new(ProviderErrorDetails {
                    provider_name: ProviderNames::LiteApi,
                    api_error: ApiError::Other(format!(
                        "Adult {} first name cannot be empty",
                        i + 1
                    )),
                    error_step: ProviderSteps::HotelBlockRoom,
                })));
            }
        }

        // Validate children ages
        for child in &request.user_details.children {
            if child.age > 17 {
                return Err(ProviderError(Arc::new(ProviderErrorDetails {
                    provider_name: ProviderNames::LiteApi,
                    api_error: ApiError::Other(format!(
                        "Child age {} exceeds maximum allowed age of 17",
                        child.age
                    )),
                    error_step: ProviderSteps::HotelBlockRoom,
                })));
            }
        }

        Ok(())
    }

    // <!-- BOOKING MAPPING FUNCTIONS -->

    /// Validates the domain book room request
    pub fn validate_book_room_request(
        request: &crate::domain::DomainBookRoomRequest,
    ) -> Result<(), ProviderError> {
        use crate::api::ApiError;
        use crate::ports::hotel_provider_port::{ProviderErrorDetails, ProviderSteps};
        use crate::ports::ProviderNames;
        use std::sync::Arc;

        // Validate block ID is present
        // this is called "prebookId" in the request

        if request.block_id.is_empty() {
            return Err(ProviderError(Arc::new(ProviderErrorDetails {
                provider_name: ProviderNames::LiteApi,
                api_error: ApiError::Other("Block ID cannot be empty".to_string()),
                error_step: ProviderSteps::HotelBookRoom,
            })));
        }

        // Validate holder information
        if request.holder.first_name.trim().is_empty() || request.holder.last_name.trim().is_empty()
        {
            return Err(ProviderError(Arc::new(ProviderErrorDetails {
                provider_name: ProviderNames::LiteApi,
                api_error: ApiError::Other(
                    "Holder first name and last name are required".to_string(),
                ),
                error_step: ProviderSteps::HotelBookRoom,
            })));
        }

        if request.holder.email.trim().is_empty() {
            return Err(ProviderError(Arc::new(ProviderErrorDetails {
                provider_name: ProviderNames::LiteApi,
                api_error: ApiError::Other("Holder email is required".to_string()),
                error_step: ProviderSteps::HotelBookRoom,
            })));
        }

        // Validate at least one guest
        if request.guests.is_empty() {
            return Err(ProviderError(Arc::new(ProviderErrorDetails {
                provider_name: ProviderNames::LiteApi,
                api_error: ApiError::Other("At least one guest is required".to_string()),
                error_step: ProviderSteps::HotelBookRoom,
            })));
        }

        // LITEAPI SPECIFIC VALIDATION: One guest per room requirement
        let number_of_rooms = request.booking_context.number_of_rooms;

        // Validate exactly one guest per room
        if request.guests.len() as u32 != number_of_rooms {
            return Err(ProviderError(Arc::new(ProviderErrorDetails {
                provider_name: ProviderNames::LiteApi,
                api_error: ApiError::Other(format!(
                    "LiteAPI requires exactly one guest per room. Expected {} guests for {} rooms, but got {}",
                    number_of_rooms, number_of_rooms, request.guests.len()
                )),
                error_step: ProviderSteps::HotelBookRoom,
            })));
        }

        // Validate occupancy numbers are sequential starting from 1
        let mut occupancy_numbers: Vec<u32> =
            request.guests.iter().map(|g| g.occupancy_number).collect();
        occupancy_numbers.sort();

        for (i, &occupancy_num) in occupancy_numbers.iter().enumerate() {
            let expected = (i + 1) as u32;
            if occupancy_num != expected {
                return Err(ProviderError(Arc::new(ProviderErrorDetails {
                    provider_name: ProviderNames::LiteApi,
                    api_error: ApiError::Other(format!(
                        "LiteAPI requires occupancy numbers to be sequential starting from 1. Expected {}, but found {}",
                        expected, occupancy_num
                    )),
                    error_step: ProviderSteps::HotelBookRoom,
                })));
            }
        }

        // Validate no duplicate occupancy numbers
        let unique_occupancy_count = occupancy_numbers
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len();
        if unique_occupancy_count != occupancy_numbers.len() {
            return Err(ProviderError(Arc::new(ProviderErrorDetails {
                provider_name: ProviderNames::LiteApi,
                api_error: ApiError::Other(
                    "LiteAPI requires unique occupancy numbers for each guest".to_string(),
                ),
                error_step: ProviderSteps::HotelBookRoom,
            })));
        }

        // Validate guest details (existing validation)
        for (i, guest) in request.guests.iter().enumerate() {
            if guest.first_name.trim().is_empty() || guest.last_name.trim().is_empty() {
                return Err(ProviderError(Arc::new(ProviderErrorDetails {
                    provider_name: ProviderNames::LiteApi,
                    api_error: ApiError::Other(format!(
                        "Guest {} first name and last name are required",
                        i + 1
                    )),
                    error_step: ProviderSteps::HotelBookRoom,
                })));
            }

            if guest.email.trim().is_empty() {
                return Err(ProviderError(Arc::new(ProviderErrorDetails {
                    provider_name: ProviderNames::LiteApi,
                    api_error: ApiError::Other(format!("Guest {} email is required", i + 1)),
                    error_step: ProviderSteps::HotelBookRoom,
                })));
            }

            if guest.occupancy_number == 0 {
                return Err(ProviderError(Arc::new(ProviderErrorDetails {
                    provider_name: ProviderNames::LiteApi,
                    api_error: ApiError::Other(format!(
                        "Guest {} occupancy number must be greater than 0",
                        i + 1
                    )),
                    error_step: ProviderSteps::HotelBookRoom,
                })));
            }

            // Validate occupancy number does not exceed number of rooms
            if guest.occupancy_number > number_of_rooms {
                return Err(ProviderError(Arc::new(ProviderErrorDetails {
                    provider_name: ProviderNames::LiteApi,
                    api_error: ApiError::Other(format!(
                        "Guest {} occupancy number {} exceeds the number of rooms being booked ({})",
                        i + 1, guest.occupancy_number, number_of_rooms
                    )),
                    error_step: ProviderSteps::HotelBookRoom,
                })));
            }
        }

        // Validate room occupancies consistency (if provided)
        if !request.booking_context.room_occupancies.is_empty() {
            if request.booking_context.room_occupancies.len() as u32 != number_of_rooms {
                return Err(ProviderError(Arc::new(ProviderErrorDetails {
                    provider_name: ProviderNames::LiteApi,
                    api_error: ApiError::Other(format!(
                        "Room occupancies count ({}) does not match number of rooms ({})",
                        request.booking_context.room_occupancies.len(),
                        number_of_rooms
                    )),
                    error_step: ProviderSteps::HotelBookRoom,
                })));
            }

            // Validate that each room in occupancies has a corresponding guest
            for occupancy in &request.booking_context.room_occupancies {
                let guest_exists = request
                    .guests
                    .iter()
                    .any(|g| g.occupancy_number == occupancy.room_number);
                if !guest_exists {
                    return Err(ProviderError(Arc::new(ProviderErrorDetails {
                        provider_name: ProviderNames::LiteApi,
                        api_error: ApiError::Other(format!(
                            "No guest provided for room {} as required by LiteAPI",
                            occupancy.room_number
                        )),
                        error_step: ProviderSteps::HotelBookRoom,
                    })));
                }
            }
        }

        Ok(())
    }

    /// Maps domain book room request to LiteAPI book request
    pub fn map_domain_book_to_liteapi_book(
        domain_request: &crate::domain::DomainBookRoomRequest,
    ) -> Result<crate::api::liteapi::LiteApiBookRequest, ProviderError> {
        use crate::api::liteapi::l03_book::LiteApiPaymentAddress;
        use crate::api::liteapi::{
            LiteApiBookGuest, LiteApiBookHolder, LiteApiBookMetadata, LiteApiBookRequest,
            LiteApiGuestPayment, LiteApiPayment,
        };
        use crate::domain::DomainPaymentMethod;

        // Map holder
        let holder = LiteApiBookHolder {
            first_name: domain_request.holder.first_name.clone(),
            last_name: domain_request.holder.last_name.clone(),
            email: domain_request.holder.email.clone(),
            phone: domain_request.holder.phone.clone(),
        };

        // Map payment method
        let payment = LiteApiPayment {
            method: match domain_request.payment.method {
                DomainPaymentMethod::AccCreditCard => "ACC_CREDIT_CARD".to_string(),
                DomainPaymentMethod::Wallet => "WALLET".to_string(),
            },
        };

        // Map guests
        let guests: Vec<LiteApiBookGuest> = domain_request
            .guests
            .iter()
            .map(|guest| LiteApiBookGuest {
                occupancy_number: guest.occupancy_number,
                first_name: guest.first_name.clone(),
                last_name: guest.last_name.clone(),
                email: guest.email.clone(),
                phone: guest.phone.clone(),
                remarks: guest.remarks.clone(),
            })
            .collect();

        // Map metadata if present
        // let metadata = domain_request.metadata.as_ref().map(|meta| LiteApiBookMetadata {
        //     ip: meta.ip.clone(),
        //     country: meta.country.clone(),
        //     language: meta.language.clone(),
        //     platform: meta.platform.clone(),
        //     device_id: meta.device_id.clone(),
        //     user_agent: meta.user_agent.clone(),
        //     utm_medium: meta.utm_medium.clone(),
        //     utm_source: meta.utm_source.clone(),
        //     utm_campaign: meta.utm_campaign.clone(),
        // });
        let metadata = None;

        // Map guest payment if present
        let guest_payment = domain_request
            .guest_payment
            .as_ref()
            .map(|gp| LiteApiGuestPayment {
                address: LiteApiPaymentAddress {
                    city: gp.address.city.clone(),
                    address: gp.address.address.clone(),
                    country: gp.address.country.clone(),
                    postal_code: gp.address.postal_code.clone(),
                },
                method: gp.method.clone(),
                phone: gp.phone.clone(),
                payee_first_name: gp.payee_first_name.clone(),
                payee_last_name: gp.payee_last_name.clone(),
                last_4_digits: gp.last_4_digits.clone(),
            });

        Ok(LiteApiBookRequest {
            holder,
            metadata,
            guest_payment,
            payment,
            prebook_id: domain_request.block_id.clone(),
            guests,
            client_reference: domain_request.client_reference.clone(),
        })
    }

    /// Maps LiteAPI book response to domain book room response
    pub fn map_liteapi_book_to_domain_book(
        liteapi_response: crate::api::liteapi::LiteApiBookResponse,
        _original_request: &crate::domain::DomainBookRoomRequest,
    ) -> crate::domain::DomainBookRoomResponse {
        use crate::domain::{
            DomainBookRoomResponse, DomainBookedHotel, DomainBookedPrice, DomainBookedRetailRate,
            DomainBookedRoom, DomainBookedRoomRate, DomainBookingHolder, DomainBookingStatus,
            DomainCancelPolicyInfo, DomainCancellationPolicies, DomainRoomTypeInfo,
        };

        let data = liteapi_response.data;

        // Map booking status
        let status = match data.status.as_str() {
            "CONFIRMED" => DomainBookingStatus::Confirmed,
            "PENDING" => DomainBookingStatus::Pending,
            "FAILED" => DomainBookingStatus::Failed,
            "CANCELLED" => DomainBookingStatus::Cancelled,
            _ => DomainBookingStatus::Pending, // Default to pending for unknown statuses
        };

        // Map hotel
        let hotel = DomainBookedHotel {
            hotel_id: data.hotel.hotel_id,
            name: data.hotel.name,
        };

        // Map booked rooms
        let booked_rooms: Vec<DomainBookedRoom> = data
            .booked_rooms
            .into_iter()
            .map(|room| DomainBookedRoom {
                room_type: DomainRoomTypeInfo {
                    name: room.room_type.name,
                },
                board_type: room.board_type,
                board_name: room.board_name,
                adults: room.adults,
                children: room.children,
                rate: DomainBookedRoomRate {
                    retail_rate: DomainBookedRetailRate {
                        total: DomainBookedPrice {
                            amount: room.rate.retail_rate.total.amount,
                            currency: room.rate.retail_rate.total.currency,
                        },
                    },
                },
                first_name: room.first_name,
                last_name: room.last_name,
                mapped_room_id: room.mapped_room_id,
            })
            .collect();

        // Map holder
        let holder = DomainBookingHolder {
            first_name: data.holder.first_name,
            last_name: data.holder.last_name,
            email: data.holder.email,
            phone: data.holder.phone,
        };

        // Map cancellation policies
        let cancellation_policies = DomainCancellationPolicies {
            cancel_policy_infos: data
                .cancellation_policies
                .cancel_policy_infos
                .into_iter()
                .map(|policy| DomainCancelPolicyInfo {
                    cancel_time: policy.cancel_time,
                    amount: policy.amount,
                    policy_type: policy.policy_type,
                    timezone: policy.timezone,
                    currency: policy.currency,
                })
                .collect(),
            hotel_remarks: data.cancellation_policies.hotel_remarks,
            refundable_tag: data.cancellation_policies.refundable_tag,
        };

        DomainBookRoomResponse {
            booking_id: data.booking_id,
            client_reference: data.client_reference,
            supplier_booking_id: data.supplier_booking_id,
            supplier_booking_name: data.supplier_booking_name,
            supplier: data.supplier,
            supplier_id: data.supplier_id,
            status,
            hotel_confirmation_code: data.hotel_confirmation_code,
            checkin: data.checkin,
            checkout: data.checkout,
            hotel,
            booked_rooms,
            holder,
            created_at: data.created_at,
            cancellation_policies,
            price: data.price,
            commission: data.commission,
            currency: data.currency,
            special_remarks: data.special_remarks,
            optional_fees: data.optional_fees,
            mandatory_fees: data.mandatory_fees,
            know_before_you_go: data.know_before_you_go,
            remarks: data.remarks,
            guest_id: data.guest_id,
        }
    }
}
