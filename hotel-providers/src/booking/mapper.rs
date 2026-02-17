use crate::booking::models::*;
use crate::domain::*;
use crate::ports::{
    ProviderError, ProviderErrorKind, ProviderNames, ProviderSteps, UISearchFilters,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

pub struct BookingMapper;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BookingBlockIdPayload {
    order_token: String,
    product_ids: Vec<String>,
}

const BOOKING_BLOCK_ID_PREFIX: &str = "booking:";

fn encode_booking_block_id(order_token: &str, product_ids: &[String]) -> String {
    let payload = BookingBlockIdPayload {
        order_token: order_token.to_string(),
        product_ids: product_ids.to_vec(),
    };
    let json = serde_json::to_vec(&payload).unwrap_or_default();
    let encoded = URL_SAFE_NO_PAD.encode(json);
    format!("{BOOKING_BLOCK_ID_PREFIX}{encoded}")
}

fn decode_booking_block_id(block_id: &str) -> Option<BookingBlockIdPayload> {
    let encoded = block_id.strip_prefix(BOOKING_BLOCK_ID_PREFIX)?;
    let bytes = URL_SAFE_NO_PAD.decode(encoded).ok()?;
    serde_json::from_slice::<BookingBlockIdPayload>(&bytes).ok()
}

impl BookingMapper {
    fn format_date(date: (u32, u32, u32)) -> String {
        format!("{:04}-{:02}-{:02}", date.0, date.1, date.2)
    }

    fn pick_translated_string(value: &Option<TranslatedString>) -> String {
        if let Some(map) = value {
            if let Some(Some(en)) = map.get("en-gb") {
                return en.clone();
            }
            if let Some(Some(en_us)) = map.get("en-us") {
                return en_us.clone();
            }
            if let Some(first) = map.values().find_map(|v| v.clone()) {
                return first;
            }
        }
        String::new()
    }

    fn build_booker(country: &str) -> BookerInput {
        BookerInput {
            country: country.to_lowercase(),
            platform: "desktop".to_string(),
            state: None,
            travel_purpose: None,
            user_groups: None,
        }
    }

    fn build_guests(criteria: &DomainHotelSearchCriteria) -> GuestsInput {
        let mut total_adults = 0u32;
        let mut children_ages: Vec<u8> = Vec::new();
        for guest in &criteria.room_guests {
            total_adults += guest.no_of_adults;
            if let Some(ages) = &guest.children_ages {
                for age in ages {
                    if let Ok(v) = age.parse::<u8>() {
                        children_ages.push(v);
                    }
                }
            }
        }

        GuestsInput {
            number_of_adults: total_adults.max(criteria.no_of_rooms),
            number_of_rooms: criteria.no_of_rooms.max(1),
            children: if children_ages.is_empty() {
                None
            } else {
                Some(children_ages)
            },
            allocation: None,
        }
    }

    fn build_allocations(
        total_adults: u32,
        children_ages: &[u8],
        rooms: u32,
    ) -> Vec<GuestAllocation> {
        if rooms == 0 {
            return Vec::new();
        }
        let adults = total_adults.max(rooms);
        let base = adults / rooms;
        let extra = adults % rooms;
        let mut allocations = Vec::new();
        let mut child_idx = 0usize;
        let children_per_room = if rooms > 0 {
            (children_ages.len() as u32 + rooms - 1) / rooms
        } else {
            0
        } as usize;

        for i in 0..rooms {
            let adults_for_room = base + if i < extra { 1 } else { 0 };
            let mut room_children = Vec::new();
            for _ in 0..children_per_room {
                if child_idx < children_ages.len() {
                    room_children.push(children_ages[child_idx]);
                    child_idx += 1;
                }
            }
            allocations.push(GuestAllocation {
                number_of_adults: adults_for_room.max(1),
                children: if room_children.is_empty() {
                    None
                } else {
                    Some(room_children)
                },
            });
        }
        allocations
    }

    pub fn map_domain_search_to_booking(
        criteria: &DomainHotelSearchCriteria,
        _ui_filters: &UISearchFilters,
        currency: &str,
    ) -> AccommodationsSearchInput {
        let guests = Self::build_guests(criteria);
        AccommodationsSearchInput {
            booker: Self::build_booker(&criteria.guest_nationality),
            checkin: Self::format_date(criteria.check_in_date),
            checkout: Self::format_date(criteria.check_out_date),
            guests,
            coordinates: criteria.latitude.zip(criteria.longitude).map(|(lat, lng)| {
                CoordinatesInput {
                    latitude: lat,
                    longitude: lng,
                    radius: 100.0,
                }
            }),
            currency: Some(currency.to_string()),
            extras: Some(vec!["products".to_string(), "extra_charges".to_string()]),
            page: None,
        }
    }

    pub fn map_booking_search_to_domain(
        response: AccommodationsSearchOutput,
        details_map: Option<HashMap<i64, AccommodationDetails>>,
    ) -> DomainHotelListAfterSearch {
        let mut hotel_results = Vec::new();

        for item in response.data {
            let details = details_map.as_ref().and_then(|map| map.get(&item.id));
            let (hotel_name, star_rating, hotel_picture, amenities, location, address) =
                if let Some(detail) = details {
                    let name = detail
                        .name
                        .clone()
                        .unwrap_or_else(|| format!("Accommodation {}", item.id));
                    let stars = detail
                        .review
                        .as_ref()
                        .and_then(|r| r.stars)
                        .unwrap_or(0.0)
                        .round() as u8;

                    let photo = detail
                        .photos
                        .as_ref()
                        .and_then(|photos| {
                            photos
                                .iter()
                                .find(|p| p.main_photo.unwrap_or(false))
                                .or_else(|| photos.first())
                        })
                        .and_then(|photo| photo.url.as_ref())
                        .and_then(|url| {
                            url.large
                                .clone()
                                .or(url.standard.clone())
                                .or(url.thumbnail.clone())
                        })
                        .unwrap_or_default();

                    let amenities = detail
                        .facilities
                        .as_ref()
                        .map(|f| f.iter().map(|fac| format!("facility_{}", fac.id)).collect())
                        .unwrap_or_else(Vec::new);

                    let location = detail.location.as_ref().and_then(|loc| {
                        loc.coordinates.as_ref().map(|coords| DomainLocation {
                            latitude: coords.latitude.unwrap_or_default(),
                            longitude: coords.longitude.unwrap_or_default(),
                        })
                    });

                    let address = Self::pick_translated_string(
                        &detail.location.as_ref().and_then(|l| l.address.clone()),
                    );

                    (name, stars, photo, amenities, location, address)
                } else {
                    (
                        format!("Accommodation {}", item.id),
                        0u8,
                        String::new(),
                        Vec::new(),
                        None,
                        String::new(),
                    )
                };

            let price = item.price.and_then(|p| {
                let amount = p.book.or(p.total).or(p.base).unwrap_or(0.0);
                let currency = item.currency.clone().unwrap_or_else(|| "USD".to_string());
                Some(DomainPrice {
                    room_price: amount,
                    currency_code: currency,
                })
            });

            hotel_results.push(DomainHotelAfterSearch {
                hotel_code: item.id.to_string(),
                hotel_name,
                hotel_category: if star_rating > 0 {
                    format!("{star_rating} Star")
                } else {
                    "Property".to_string()
                },
                star_rating,
                price,
                hotel_picture,
                amenities,
                property_type: None,
                result_token: item.id.to_string(),
                hotel_address: if address.is_empty() {
                    None
                } else {
                    Some(address)
                },
                distance_from_center_km: None,
                location,
            });
        }

        DomainHotelListAfterSearch {
            hotel_results,
            pagination: None,
            provider: Some(ProviderNames::Booking.to_string()),
        }
    }

    pub fn map_booking_details_to_domain_static(
        detail: AccommodationDetails,
    ) -> DomainHotelStaticDetails {
        let rating = detail
            .review
            .as_ref()
            .and_then(|r| r.review_score)
            .map(|v| v as f64);
        let review_count = detail.review.as_ref().and_then(|r| r.number_of_reviews);
        let star_rating = detail
            .review
            .as_ref()
            .and_then(|r| r.stars)
            .unwrap_or(0.0)
            .round() as i32;

        let images = detail
            .photos
            .as_ref()
            .map(|photos| {
                photos
                    .iter()
                    .filter_map(|p| {
                        p.url.as_ref().and_then(|u| {
                            u.large
                                .clone()
                                .or(u.standard.clone())
                                .or(u.thumbnail.clone())
                        })
                    })
                    .collect()
            })
            .unwrap_or_else(Vec::new);

        let amenities = detail
            .facilities
            .as_ref()
            .map(|f| f.iter().map(|fac| format!("facility_{}", fac.id)).collect())
            .unwrap_or_else(Vec::new);

        let rooms = detail
            .rooms
            .as_ref()
            .map(|rooms| {
                rooms
                    .iter()
                    .map(|room| DomainStaticRoom {
                        room_id: room.id.to_string(),
                        room_name: room
                            .name
                            .clone()
                            .unwrap_or_else(|| format!("Room {}", room.id)),
                        description: Self::pick_translated_string(&room.description.clone()),
                        room_size_square: None,
                        room_size_unit: None,
                        max_adults: None,
                        max_children: None,
                        max_occupancy: None,
                        amenities: room
                            .facilities
                            .as_ref()
                            .map(|f| f.iter().map(|fac| format!("facility_{}", fac.id)).collect())
                            .unwrap_or_else(Vec::new),
                        photos: room
                            .photos
                            .as_ref()
                            .map(|photos| {
                                photos
                                    .iter()
                                    .filter_map(|p| {
                                        p.url.as_ref().and_then(|u| {
                                            u.large
                                                .clone()
                                                .or(u.standard.clone())
                                                .or(u.thumbnail.clone())
                                        })
                                    })
                                    .collect()
                            })
                            .unwrap_or_else(Vec::new),
                        bed_types: Vec::new(),
                    })
                    .collect()
            })
            .unwrap_or_else(Vec::new);

        let address = detail
            .location
            .as_ref()
            .and_then(|loc| loc.address.clone())
            .map(|addr| Self::pick_translated_string(&Some(addr)))
            .unwrap_or_default();

        let location = detail.location.and_then(|loc| {
            loc.coordinates.map(|coords| DomainLocation {
                latitude: coords.latitude.unwrap_or_default(),
                longitude: coords.longitude.unwrap_or_default(),
            })
        });

        DomainHotelStaticDetails {
            hotel_name: detail
                .name
                .clone()
                .unwrap_or_else(|| format!("Accommodation {}", detail.id)),
            hotel_code: detail.id.to_string(),
            star_rating,
            rating,
            review_count,
            categories: Vec::new(),
            description: Self::pick_translated_string(&detail.description),
            hotel_facilities: amenities.clone(),
            address,
            images,
            amenities,
            rooms,
            location,
            checkin_checkout_times: None,
            policies: Vec::new(),
            provider: Some(ProviderNames::Booking.to_string()),
        }
    }

    pub fn map_domain_info_to_booking_availability(
        criteria: &DomainHotelInfoCriteria,
        currency: &str,
    ) -> Result<AccommodationsAvailabilityInput, ProviderError> {
        let accommodation = criteria.hotel_ids.first().ok_or_else(|| {
            ProviderError::other(
                "Booking.com",
                crate::ports::ProviderSteps::HotelRate,
                "Missing hotel id",
            )
        })?;

        let guests = Self::build_guests(&criteria.search_criteria);
        Ok(AccommodationsAvailabilityInput {
            accommodation: accommodation.parse::<i64>().unwrap_or_default(),
            booker: Self::build_booker(&criteria.search_criteria.guest_nationality),
            checkin: Self::format_date(criteria.search_criteria.check_in_date),
            checkout: Self::format_date(criteria.search_criteria.check_out_date),
            guests,
            currency: Some(currency.to_string()),
            extras: Some(vec!["extra_charges".to_string()]),
            products: None,
        })
    }

    pub fn map_booking_availability_to_grouped_rates(
        response: AccommodationsAvailabilityOutput,
    ) -> DomainGroupedRoomRates {
        let currency = response
            .data
            .currency
            .clone()
            .unwrap_or_else(|| "USD".to_string());
        let mut room_groups = Vec::new();

        if let Some(products) = response.data.products {
            for product in products {
                let price_amount = product
                    .price
                    .as_ref()
                    .and_then(|p| p.book.or(p.total).or(p.base))
                    .unwrap_or(0.0);

                let included_tax = product
                    .price
                    .as_ref()
                    .and_then(|p| p.extra_charges.as_ref())
                    .and_then(|e| e.included)
                    .unwrap_or(0.0);

                let excluded_tax = product
                    .price
                    .as_ref()
                    .and_then(|p| p.extra_charges.as_ref())
                    .and_then(|e| e.excluded)
                    .unwrap_or(0.0);

                let tax_breakdown = vec![
                    GroupedTaxItem {
                        description: "Included taxes".to_string(),
                        amount: included_tax,
                        currency_code: currency.clone(),
                        included: true,
                    },
                    GroupedTaxItem {
                        description: "Excluded taxes".to_string(),
                        amount: excluded_tax,
                        currency_code: currency.clone(),
                        included: false,
                    },
                ];

                let cancellation_info = product.policies.as_ref().and_then(|p| {
                    p.cancellation.as_ref().map(|c| DomainCancellationPolicies {
                        cancel_policy_infos: c
                            .schedule
                            .clone()
                            .unwrap_or_default()
                            .into_iter()
                            .map(|s| DomainCancelPolicyInfo {
                                cancel_time: s.from.unwrap_or_else(|| "now".to_string()),
                                amount: s.price,
                                policy_type: c.kind.clone().unwrap_or_default(),
                                timezone: "UTC".to_string(),
                                currency: currency.clone(),
                            })
                            .collect(),
                        hotel_remarks: None,
                        refundable_tag: c.kind.clone().unwrap_or_default(),
                    })
                });

                let meal_plan = product
                    .policies
                    .as_ref()
                    .and_then(|p| p.meal_plan.as_ref())
                    .and_then(|m| m.plan.clone());

                let occupancy_info = product.maximum_occupancy.as_ref().map(|o| {
                    let max_children = o
                        .children
                        .as_ref()
                        .and_then(|c| c.first())
                        .map(|c| c.total)
                        .unwrap_or(0);
                    DomainRoomOccupancy {
                        max_occupancy: Some(o.adults + max_children),
                        adult_count: Some(o.adults),
                        child_count: Some(max_children),
                    }
                });

                let room_name = format!("Room {}", product.id);
                let variant = DomainRoomVariant {
                    offer_id: product.id.clone(),
                    rate_key: product.id.clone(),
                    room_name: room_name.clone(),
                    mapped_room_id: product.id.clone(),
                    room_count: 1,
                    room_unique_id: product.id.clone(),
                    occupancy_number: Some(1),
                    meal_plan: meal_plan.clone(),
                    total_price_for_all_rooms: price_amount,
                    total_price_for_one_room: price_amount,
                    price_per_room_excluding_taxes: (price_amount - included_tax).max(0.0),
                    currency_code: currency.clone(),
                    tax_breakdown,
                    occupancy_info,
                    cancellation_info,
                    perks: Vec::new(),
                    original_price: None,
                    board_type_code: meal_plan,
                    remarks: None,
                };

                room_groups.push(DomainRoomGroup {
                    name: room_name,
                    mapped_room_id: None,
                    min_price: price_amount,
                    currency_code: currency.clone(),
                    images: Vec::new(),
                    amenities: Vec::new(),
                    bed_types: Vec::new(),
                    room_types: vec![variant],
                });
            }
        }

        DomainGroupedRoomRates {
            room_groups,
            provider: Some(ProviderNames::Booking.to_string()),
        }
    }

    pub fn map_domain_search_to_booking_bulk_availability(
        criteria: &DomainHotelSearchCriteria,
        hotel_ids: Vec<String>,
        currency: &str,
    ) -> Result<AccommodationsBulkAvailabilityInput, ProviderError> {
        let accommodations: Vec<i64> = hotel_ids
            .into_iter()
            .filter_map(|id| id.parse::<i64>().ok())
            .collect();
        Ok(AccommodationsBulkAvailabilityInput {
            accommodations,
            booker: Self::build_booker(&criteria.guest_nationality),
            checkin: Self::format_date(criteria.check_in_date),
            checkout: Self::format_date(criteria.check_out_date),
            guests: Self::build_guests(criteria),
            currency: Some(currency.to_string()),
            extras: Some(vec!["extra_charges".to_string()]),
        })
    }

    pub fn map_booking_bulk_availability_to_domain(
        response: AccommodationsBulkAvailabilityOutput,
        currency: &str,
    ) -> HashMap<String, DomainPrice> {
        let mut map = HashMap::new();
        for data in response.data {
            if let Some(products) = data.products {
                let mut min_price: Option<f64> = None;
                for product in products {
                    let price_amount = product
                        .price
                        .as_ref()
                        .and_then(|p| p.book.or(p.total).or(p.base))
                        .unwrap_or(0.0);
                    min_price = Some(min_price.map_or(price_amount, |cur| cur.min(price_amount)));
                }
                if let Some(price) = min_price {
                    map.insert(
                        data.id.to_string(),
                        DomainPrice {
                            room_price: price,
                            currency_code: currency.to_string(),
                        },
                    );
                }
            }
        }
        map
    }

    pub fn map_domain_block_to_booking_preview(
        request: &DomainBlockRoomRequest,
        currency: &str,
    ) -> Result<OrdersPreviewInput, ProviderError> {
        let criteria = &request.hotel_info_criteria.search_criteria;

        let total_adults = request.user_details.adults.len() as u32;
        let children_ages: Vec<u8> = request
            .user_details
            .children
            .iter()
            .map(|c| c.age)
            .collect();
        let rooms = request
            .selected_rooms
            .iter()
            .map(|r| r.quantity)
            .sum::<u32>()
            .max(1);

        let allocations = Self::build_allocations(total_adults, &children_ages, rooms);

        let mut products = Vec::new();
        let mut alloc_idx = 0usize;
        for selected in &request.selected_rooms {
            for _ in 0..selected.quantity {
                let allocation = allocations
                    .get(alloc_idx)
                    .cloned()
                    .unwrap_or(GuestAllocation {
                        number_of_adults: 1,
                        children: None,
                    });
                alloc_idx += 1;
                products.push(OrdersPreviewProductInput {
                    id: selected.room_data.offer_id.clone(),
                    allocation,
                });
            }
        }

        Ok(OrdersPreviewInput {
            booker: Self::build_booker(&criteria.guest_nationality),
            currency: Some(currency.to_string()),
            accommodation: OrdersPreviewAccommodationInput {
                id: request
                    .hotel_info_criteria
                    .hotel_ids
                    .first()
                    .and_then(|v| v.parse::<i64>().ok())
                    .unwrap_or_default(),
                checkin: Self::format_date(criteria.check_in_date),
                checkout: Self::format_date(criteria.check_out_date),
                products,
            },
        })
    }

    pub fn map_booking_preview_to_domain_block(
        preview: OrdersPreviewOutput,
    ) -> Result<DomainBlockRoomResponse, ProviderError> {
        let product_ids: Vec<String> = preview
            .data
            .accommodation
            .as_ref()
            .and_then(|accommodation| accommodation.products.as_ref())
            .map(|products| products.iter().map(|p| p.id.clone()).collect())
            .unwrap_or_default();

        let mut total_price = 0.0;
        if let Some(accommodation) = preview.data.accommodation.as_ref() {
            if let Some(products) = accommodation.products.as_ref() {
                for product in products {
                    if let Some(price) = product.price.as_ref().and_then(|p| p.total.as_ref()) {
                        if let Some(amount) = price.accommodation_currency {
                            total_price += amount;
                        }
                    }
                }
            }
        }

        let total_price = if total_price == 0.0 { 0.0 } else { total_price };
        let price = DomainDetailedPrice {
            published_price: total_price,
            published_price_rounded_off: total_price,
            offered_price: total_price,
            offered_price_rounded_off: total_price,
            suggested_selling_price: total_price,
            suggested_selling_price_rounded_off: total_price,
            room_price: total_price,
            tax: 0.0,
            extra_guest_charge: 0.0,
            child_charge: 0.0,
            other_charges: 0.0,
            currency_code: "USD".to_string(),
        };

        let order_token = preview.data.order_token.clone();
        let block_id = if product_ids.is_empty() {
            order_token.clone()
        } else {
            encode_booking_block_id(&order_token, &product_ids)
        };

        Ok(DomainBlockRoomResponse {
            block_id,
            is_price_changed: false,
            is_cancellation_policy_changed: false,
            blocked_rooms: Vec::new(),
            total_price: price,
            provider_data: Some(json!(preview).to_string()),
            provider: Some(ProviderNames::Booking.to_string()),
        })
    }

    pub fn map_domain_book_to_booking_create(
        request: &DomainBookRoomRequest,
    ) -> Result<OrderCreateInput, ProviderError> {
        let (order_token, product_ids) = decode_booking_block_id(&request.block_id)
            .map(|payload| (payload.order_token, payload.product_ids))
            .unwrap_or_else(|| (request.block_id.clone(), Vec::new()));

        if product_ids.is_empty() {
            return Err(ProviderError::new(
                ProviderNames::Booking,
                ProviderErrorKind::InvalidRequest,
                ProviderSteps::HotelBookRoom,
                "Booking.com block_id is missing product IDs from preview. Ensure Booking preview product IDs are encoded into block_id.".to_string(),
            ));
        }

        let address = request
            .guest_payment
            .as_ref()
            .map(|p| OrderCreateAddressInput {
                address_line: p.address.address.clone(),
                city: p.address.city.clone(),
                country: p.address.country.clone(),
                post_code: p.address.postal_code.clone(),
            })
            .unwrap_or(OrderCreateAddressInput {
                address_line: "N/A".to_string(),
                city: "N/A".to_string(),
                country: "us".to_string(),
                post_code: "00000".to_string(),
            });

        let booker = OrderCreateBookerInput {
            address,
            email: request.holder.email.clone(),
            name: OrderCreateBookerNameInput {
                first_name: request.holder.first_name.clone(),
                last_name: request.holder.last_name.clone(),
            },
            telephone: request.holder.phone.clone(),
            language: Some("en-us".to_string()),
        };

        let mut products = Vec::with_capacity(product_ids.len());
        for (idx, product_id) in product_ids.iter().enumerate() {
            let guest = request.guests.get(idx).or_else(|| request.guests.first());

            let guests = guest.map(|g| {
                vec![OrderCreateGuestInput {
                    email: g.email.clone(),
                    name: format!("{} {}", g.first_name, g.last_name),
                }]
            });

            products.push(OrderCreateProductInput {
                id: product_id.clone(),
                guests,
            });
        }

        Ok(OrderCreateInput {
            order_token,
            booker,
            accommodation: OrderCreateAccommodationInput {
                label: None,
                products,
                remarks: request
                    .special_requests
                    .as_ref()
                    .map(|s| OrderCreateRemarksInput {
                        special_requests: Some(s.clone()),
                    }),
            },
        })
    }

    pub fn map_booking_create_to_domain_book(
        response: OrderCreateOutput,
        request: &DomainBookRoomRequest,
    ) -> Result<DomainBookRoomResponse, ProviderError> {
        let accommodation = response
            .data
            .as_ref()
            .and_then(|d| d.accommodation.as_ref());

        let booking_id = accommodation
            .and_then(|a| a.order.clone())
            .unwrap_or_else(|| "booking_order".to_string());
        let reservation_id = accommodation
            .and_then(|a| a.reservation)
            .map(|v| v.to_string())
            .unwrap_or_default();
        let confirmation_code = accommodation
            .and_then(|a| a.pincode.clone())
            .unwrap_or_default();

        Ok(DomainBookRoomResponse {
            booking_id: booking_id.clone(),
            client_reference: request
                .client_reference
                .clone()
                .unwrap_or_else(|| booking_id.clone()),
            supplier_booking_id: reservation_id.clone(),
            supplier_booking_name: booking_id.clone(),
            supplier: "Booking.com".to_string(),
            supplier_id: 0,
            status: DomainBookingStatus::Confirmed,
            hotel_confirmation_code: confirmation_code,
            checkin: request
                .booking_context
                .original_search_criteria
                .as_ref()
                .map(|c| c.checkin_date.clone())
                .unwrap_or_default(),
            checkout: request
                .booking_context
                .original_search_criteria
                .as_ref()
                .map(|c| c.checkout_date.clone())
                .unwrap_or_default(),
            hotel: DomainBookedHotel {
                hotel_id: request
                    .booking_context
                    .original_search_criteria
                    .as_ref()
                    .map(|c| c.hotel_id.clone())
                    .unwrap_or_default(),
                name: "Booking.com Accommodation".to_string(),
            },
            booked_rooms: Vec::new(),
            holder: request.holder.clone(),
            created_at: String::new(),
            cancellation_policies: DomainCancellationPolicies {
                cancel_policy_infos: Vec::new(),
                hotel_remarks: None,
                refundable_tag: String::new(),
            },
            price: 0.0,
            commission: 0.0,
            currency: "USD".to_string(),
            special_remarks: None,
            optional_fees: None,
            mandatory_fees: None,
            know_before_you_go: None,
            remarks: None,
            guest_id: None,
            provider: Some(ProviderNames::Booking.to_string()),
        })
    }

    pub fn map_domain_get_booking_to_booking_details(
        request: &DomainGetBookingRequest,
        currency: &str,
    ) -> OrdersDetailsAccommodationsInput {
        let orders = request.client_reference.clone().map(|v| vec![v]);
        let reservations = request
            .guest_id
            .as_ref()
            .and_then(|v| v.parse::<i64>().ok())
            .map(|v| vec![v]);

        OrdersDetailsAccommodationsInput {
            orders,
            reservations,
            currency: Some(currency.to_string()),
            extras: Some(vec!["accommodation_details".to_string()]),
        }
    }

    pub fn map_booking_details_to_domain(
        response: OrdersDetailsAccommodationsOutput,
    ) -> DomainGetBookingResponse {
        let mut bookings = Vec::new();
        for item in response.data {
            bookings.push(DomainBookingDetails {
                booking_id: item.id.clone(),
                client_reference: Some(item.id.clone()),
                status: "confirmed".to_string(),
                hotel: DomainBookingHotelInfo {
                    hotel_id: item
                        .accommodation
                        .map(|v| v.to_string())
                        .unwrap_or_default(),
                    name: item
                        .accommodation_details
                        .as_ref()
                        .and_then(|d| d.name.clone())
                        .unwrap_or_else(|| "Booking.com Accommodation".to_string()),
                },
                holder: DomainBookingHolder {
                    first_name: String::new(),
                    last_name: String::new(),
                    email: String::new(),
                    phone: String::new(),
                },
                price: 0.0,
                currency: "USD".to_string(),
            });
        }

        DomainGetBookingResponse {
            bookings,
            provider: Some(ProviderNames::Booking.to_string()),
        }
    }
}
