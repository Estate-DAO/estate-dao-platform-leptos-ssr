use std::collections::HashMap;

use crate::amadeus::models::booking::AmadeusBlockRoomSnapshot;
use crate::amadeus::models::hotel_details::AmadeusHotelDetailsResponse;
use crate::amadeus::models::search::{
    AmadeusAddress, AmadeusHotelListResponse, AmadeusHotelOfferResponse,
    AmadeusHotelOffersResponse, AmadeusOffer,
};
use crate::domain::{
    DomainBlockRoomRequest, DomainBlockRoomResponse, DomainBlockedRoom, DomainDetailedPrice,
    DomainGroupedRoomRates, DomainHotelAfterSearch, DomainHotelListAfterSearch,
    DomainHotelStaticDetails, DomainLocation, DomainPaginationParams, DomainPrice,
    DomainStaticRoom,
};
use crate::ports::{ProviderError, ProviderErrorKind, ProviderNames, ProviderSteps};
use base64::Engine;
use hotel_types::{DomainRoomGroup, DomainRoomVariant, GroupedTaxItem};

const AMADEUS_BLOCK_ID_PREFIX: &str = "amadeus-block:";
const AMADEUS_PLACEHOLDER_IMAGE_PATH: &str = "/img/hotel-placeholder.svg";

#[derive(Clone, Debug, Default)]
pub struct AmadeusMapper;

impl AmadeusMapper {
    pub fn map_search_to_domain(
        hotel_list: AmadeusHotelListResponse,
        offers: AmadeusHotelOffersResponse,
        pagination_params: &Option<DomainPaginationParams>,
    ) -> DomainHotelListAfterSearch {
        let total_results = hotel_list
            .meta
            .as_ref()
            .and_then(|meta| meta.count)
            .or(Some(hotel_list.data.len() as i32));
        let hotels_by_id = hotel_list
            .data
            .into_iter()
            .map(|hotel| (hotel.hotel_id.clone(), hotel))
            .collect::<HashMap<_, _>>();

        let hotel_results = offers
            .data
            .into_iter()
            .filter_map(|offer_bundle| {
                let cheapest = Self::cheapest_price(&offer_bundle.offers)?;
                let hotel = hotels_by_id.get(&offer_bundle.hotel.hotel_id);

                let (hotel_name, hotel_address, location, distance_from_center_km) =
                    if let Some(hotel) = hotel {
                        (
                            hotel.name.clone(),
                            Self::format_address(hotel.address.as_ref()),
                            hotel.geo_code.as_ref().map(|geo| DomainLocation {
                                latitude: geo.latitude,
                                longitude: geo.longitude,
                            }),
                            hotel.distance.as_ref().map(|distance| distance.value),
                        )
                    } else {
                        (
                            offer_bundle.hotel.name.clone(),
                            None,
                            offer_bundle
                                .hotel
                                .latitude
                                .zip(offer_bundle.hotel.longitude)
                                .map(|(latitude, longitude)| DomainLocation {
                                    latitude,
                                    longitude,
                                }),
                            None,
                        )
                    };

                Some(DomainHotelAfterSearch {
                    hotel_code: offer_bundle.hotel.hotel_id.clone(),
                    hotel_name,
                    hotel_category: "Hotel".to_string(),
                    star_rating: 0,
                    price: Some(cheapest),
                    hotel_picture: AMADEUS_PLACEHOLDER_IMAGE_PATH.to_string(),
                    amenities: Vec::new(),
                    property_type: None,
                    result_token: offer_bundle.hotel.hotel_id,
                    hotel_address,
                    distance_from_center_km,
                    location,
                })
            })
            .collect();

        DomainHotelListAfterSearch {
            hotel_results,
            pagination: Some(Self::build_pagination_meta(
                total_results,
                pagination_params,
            )),
            provider: Some(ProviderNames::Amadeus.to_string()),
        }
    }

    fn build_pagination_meta(
        total_results: Option<i32>,
        pagination_params: &Option<DomainPaginationParams>,
    ) -> hotel_types::DomainPaginationMeta {
        let page = pagination_params
            .as_ref()
            .and_then(|params| params.page)
            .unwrap_or(1)
            .max(1);
        let page_size = pagination_params
            .as_ref()
            .and_then(|params| params.page_size)
            .unwrap_or(25)
            .max(1);

        hotel_types::DomainPaginationMeta {
            page,
            page_size,
            total_results,
            has_next_page: false,
            has_previous_page: page > 1,
        }
    }

    fn cheapest_price(offers: &[AmadeusOffer]) -> Option<DomainPrice> {
        offers
            .iter()
            .filter_map(|offer| {
                offer
                    .price
                    .total
                    .parse::<f64>()
                    .ok()
                    .map(|amount| DomainPrice {
                        room_price: amount,
                        currency_code: offer.price.currency.clone(),
                    })
            })
            .min_by(|left, right| left.room_price.total_cmp(&right.room_price))
    }

    fn format_address(address: Option<&AmadeusAddress>) -> Option<String> {
        let address = address?;
        let mut parts = Vec::new();

        if let Some(lines) = &address.lines {
            let joined = lines
                .iter()
                .map(String::as_str)
                .filter(|line| !line.trim().is_empty())
                .collect::<Vec<_>>()
                .join(", ");
            if !joined.is_empty() {
                parts.push(joined);
            }
        }

        if let Some(city_name) = &address.city_name {
            if !city_name.trim().is_empty() {
                parts.push(city_name.clone());
            }
        }

        if let Some(country_code) = &address.country_code {
            if !country_code.trim().is_empty() {
                parts.push(country_code.clone());
            }
        }

        if parts.is_empty() {
            None
        } else {
            Some(parts.join(", "))
        }
    }

    pub fn map_hotel_details_to_domain(
        response: AmadeusHotelDetailsResponse,
    ) -> Option<DomainHotelStaticDetails> {
        let hotel = response.data.into_iter().next()?;

        Some(DomainHotelStaticDetails {
            hotel_name: hotel.name,
            hotel_code: hotel.hotel_id,
            star_rating: 0,
            rating: None,
            review_count: None,
            categories: Vec::new(),
            description: String::new(),
            hotel_facilities: Vec::new(),
            address: Self::format_address(hotel.address.as_ref()).unwrap_or_default(),
            images: vec![AMADEUS_PLACEHOLDER_IMAGE_PATH.to_string()],
            amenities: Vec::new(),
            rooms: Vec::<DomainStaticRoom>::new(),
            location: hotel.geo_code.map(|geo| DomainLocation {
                latitude: geo.latitude,
                longitude: geo.longitude,
            }),
            checkin_checkout_times: None,
            policies: Vec::new(),
            provider: Some(ProviderNames::Amadeus.to_string()),
        })
    }

    pub fn map_offers_to_grouped_rates(
        offers_response: AmadeusHotelOffersResponse,
    ) -> DomainGroupedRoomRates {
        let mut groups = HashMap::<String, DomainRoomGroup>::new();

        for hotel_offer in offers_response.data {
            for offer in hotel_offer.offers {
                let room_name = Self::room_name(&offer);
                let mapped_room_id = format!(
                    "{}:{}",
                    hotel_offer.hotel.hotel_id,
                    Self::normalized_room_key(&room_name)
                );
                let currency_code = offer.price.currency.clone();
                let total_price = Self::parse_amount(&offer.price.total);
                let base_price = offer
                    .price
                    .base
                    .as_deref()
                    .map(Self::parse_amount)
                    .unwrap_or(total_price);
                let tax_amount = (total_price - base_price).max(0.0);
                let bed_types = offer
                    .room
                    .as_ref()
                    .and_then(|room| room.type_estimated.as_ref())
                    .and_then(|estimated| estimated.bed_type.clone())
                    .map(|bed_type| vec![bed_type])
                    .unwrap_or_default();

                let variant = DomainRoomVariant {
                    offer_id: offer.id.clone(),
                    rate_key: offer.id.clone(),
                    room_name: room_name.clone(),
                    mapped_room_id: mapped_room_id.clone(),
                    room_count: 1,
                    room_unique_id: offer.id.clone(),
                    occupancy_number: None,
                    meal_plan: None,
                    total_price_for_all_rooms: total_price,
                    total_price_for_one_room: total_price,
                    price_per_room_excluding_taxes: base_price,
                    currency_code: currency_code.clone(),
                    tax_breakdown: if tax_amount > 0.0 {
                        vec![GroupedTaxItem {
                            description: "Taxes and fees".to_string(),
                            amount: tax_amount,
                            currency_code: currency_code.clone(),
                            included: true,
                        }]
                    } else {
                        Vec::new()
                    },
                    occupancy_info: None,
                    cancellation_info: None,
                    perks: Vec::new(),
                    original_price: None,
                    board_type_code: None,
                    remarks: None,
                };

                groups
                    .entry(room_name.clone())
                    .and_modify(|group| {
                        group.min_price = group.min_price.min(total_price);
                        if group.currency_code.is_empty() {
                            group.currency_code = currency_code.clone();
                        }
                        for bed_type in &bed_types {
                            if !group.bed_types.contains(bed_type) {
                                group.bed_types.push(bed_type.clone());
                            }
                        }
                        group.room_types.push(variant.clone());
                    })
                    .or_insert_with(|| DomainRoomGroup {
                        name: room_name.clone(),
                        mapped_room_id: Some(mapped_room_id),
                        min_price: total_price,
                        currency_code: currency_code.clone(),
                        images: Vec::new(),
                        amenities: Vec::new(),
                        bed_types,
                        room_types: vec![variant],
                    });
            }
        }

        let mut room_groups = groups.into_values().collect::<Vec<_>>();
        for group in &mut room_groups {
            group.room_types.sort_by(|left, right| {
                left.total_price_for_all_rooms
                    .total_cmp(&right.total_price_for_all_rooms)
            });
        }
        room_groups.sort_by(|left, right| left.min_price.total_cmp(&right.min_price));

        DomainGroupedRoomRates {
            room_groups,
            provider: Some(ProviderNames::Amadeus.to_string()),
        }
    }

    pub fn map_offer_revalidation_to_domain_block(
        offer_response: AmadeusHotelOfferResponse,
        original_request: &DomainBlockRoomRequest,
    ) -> Result<DomainBlockRoomResponse, ProviderError> {
        let offer_id = Self::selected_offer_id(original_request)?;
        let hotel_offer = offer_response.data;
        let offer = hotel_offer
            .offers
            .into_iter()
            .find(|offer| offer.id == offer_id)
            .ok_or_else(|| {
                ProviderError::not_found(
                    ProviderNames::Amadeus,
                    ProviderSteps::HotelBlockRoom,
                    format!("Amadeus offer {offer_id} was not present in the lookup response"),
                )
            })?;

        let room_name = Self::room_name(&offer);
        let currency_code = offer.price.currency.clone();
        let total_price = Self::parse_amount(&offer.price.total);
        let base_price = offer
            .price
            .base
            .as_deref()
            .map(Self::parse_amount)
            .unwrap_or(total_price);
        let cancellation_policy = Self::cancellation_policy_summary(&offer);
        let snapshot = AmadeusBlockRoomSnapshot {
            offer_id: offer.id.clone(),
            hotel_id: hotel_offer.hotel.hotel_id.clone(),
            room_name: room_name.clone(),
            total_price,
            currency_code: currency_code.clone(),
            cancellation_policy: cancellation_policy.clone(),
        };
        let provider_data = serde_json::to_string(&snapshot).map_err(|error| {
            ProviderError::parse_error(
                ProviderNames::Amadeus,
                ProviderSteps::HotelBlockRoom,
                format!("Failed to serialize Amadeus block-room snapshot: {error}"),
            )
        })?;
        let block_id = Self::encode_block_id(&snapshot)?;
        let requested_total = Self::requested_total_price(original_request);

        let blocked_room = DomainBlockedRoom {
            room_code: if original_request.selected_room.room_unique_id.is_empty() {
                offer.id.clone()
            } else {
                original_request.selected_room.room_unique_id.clone()
            },
            room_name,
            room_type_code: Some(original_request.selected_room.mapped_room_id.clone()),
            price: Self::detailed_price(total_price, base_price, currency_code.clone()),
            cancellation_policy,
            meal_plan: None,
        };

        Ok(DomainBlockRoomResponse {
            block_id,
            is_price_changed: (requested_total - total_price).abs() > 0.01,
            is_cancellation_policy_changed: false,
            blocked_rooms: vec![blocked_room],
            total_price: Self::detailed_price(total_price, base_price, currency_code),
            provider_data: Some(provider_data),
            provider: Some(ProviderNames::Amadeus.to_string()),
        })
    }

    fn room_name(offer: &AmadeusOffer) -> String {
        offer
            .room
            .as_ref()
            .and_then(|room| room.description.as_ref())
            .map(|description| description.text.trim().to_string())
            .filter(|description| !description.is_empty())
            .or_else(|| {
                offer
                    .room
                    .as_ref()
                    .and_then(|room| room.type_estimated.as_ref())
                    .and_then(|estimated| estimated.category.as_ref())
                    .map(|category| Self::humanize_category(category))
            })
            .unwrap_or_else(|| "Room".to_string())
    }

    fn normalized_room_key(value: &str) -> String {
        value
            .to_lowercase()
            .chars()
            .map(|ch| if ch.is_alphanumeric() { ch } else { ' ' })
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join("-")
    }

    fn humanize_category(category: &str) -> String {
        category
            .split('_')
            .filter(|segment| !segment.is_empty())
            .map(|segment| {
                let mut chars = segment.chars();
                match chars.next() {
                    Some(first) => {
                        let mut humanized = first.to_uppercase().collect::<String>();
                        humanized.push_str(&chars.as_str().to_lowercase());
                        humanized
                    }
                    None => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn parse_amount(raw_amount: &str) -> f64 {
        raw_amount.parse::<f64>().unwrap_or_default()
    }

    fn selected_offer_id(request: &DomainBlockRoomRequest) -> Result<&str, ProviderError> {
        if !request.selected_room.offer_id.trim().is_empty() {
            return Ok(request.selected_room.offer_id.as_str());
        }

        if !request.selected_room.rate_key.trim().is_empty() {
            return Ok(request.selected_room.rate_key.as_str());
        }

        Err(ProviderError::new(
            ProviderNames::Amadeus,
            ProviderErrorKind::InvalidRequest,
            ProviderSteps::HotelBlockRoom,
            "Amadeus block room requires an offer_id or rate_key on the selected room",
        ))
    }

    fn requested_total_price(request: &DomainBlockRoomRequest) -> f64 {
        let nights = request
            .hotel_info_criteria
            .search_criteria
            .no_of_nights
            .max(1) as f64;
        request
            .selected_rooms
            .iter()
            .map(|room| room.price_per_night * room.quantity as f64 * nights)
            .sum()
    }

    fn detailed_price(
        total_price: f64,
        base_price: f64,
        currency_code: String,
    ) -> DomainDetailedPrice {
        let tax_amount = (total_price - base_price).max(0.0);

        DomainDetailedPrice {
            published_price: total_price,
            published_price_rounded_off: total_price.round(),
            offered_price: total_price,
            offered_price_rounded_off: total_price.round(),
            suggested_selling_price: total_price,
            suggested_selling_price_rounded_off: total_price.round(),
            room_price: total_price,
            tax: tax_amount,
            extra_guest_charge: 0.0,
            child_charge: 0.0,
            other_charges: 0.0,
            currency_code,
        }
    }

    fn cancellation_policy_summary(offer: &AmadeusOffer) -> Option<String> {
        offer
            .policies
            .as_ref()
            .and_then(|policies| policies.cancellations.as_ref())
            .and_then(|cancellations| cancellations.first())
            .and_then(|policy| {
                policy
                    .description
                    .as_ref()
                    .map(|description| description.text.trim().to_string())
                    .filter(|description| !description.is_empty())
                    .or_else(|| policy.deadline.clone())
                    .or_else(|| policy.amount.clone())
            })
    }

    fn encode_block_id(snapshot: &AmadeusBlockRoomSnapshot) -> Result<String, ProviderError> {
        let payload = serde_json::to_vec(snapshot).map_err(|error| {
            ProviderError::parse_error(
                ProviderNames::Amadeus,
                ProviderSteps::HotelBlockRoom,
                format!("Failed to encode Amadeus block-room payload: {error}"),
            )
        })?;
        let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(payload);
        Ok(format!("{AMADEUS_BLOCK_ID_PREFIX}{encoded}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::amadeus::models::hotel_details::{
        AmadeusHotelDetailsEntry, AmadeusHotelDetailsResponse,
    };
    use crate::amadeus::models::search::{
        AmadeusAddress, AmadeusGeoCode, AmadeusHotelListEntry, AmadeusHotelListMeta,
        AmadeusHotelListResponse, AmadeusHotelOffer, AmadeusHotelOffersResponse, AmadeusOffer,
        AmadeusOfferHotel, AmadeusOfferPolicies, AmadeusOfferPrice, AmadeusOfferRoom,
        AmadeusRoomDescription, AmadeusRoomTypeEstimated,
    };

    #[test]
    fn maps_amadeus_geocode_and_offers_to_domain_hotel_list() {
        let hotel_list = AmadeusHotelListResponse {
            data: vec![AmadeusHotelListEntry {
                hotel_id: "ACPARH29".to_string(),
                name: "Acropolis Hotel Paris Boulogne".to_string(),
                geo_code: Some(AmadeusGeoCode {
                    latitude: 48.83593,
                    longitude: 2.24922,
                }),
                address: Some(AmadeusAddress {
                    lines: None,
                    city_name: Some("PARIS".to_string()),
                    country_code: Some("FR".to_string()),
                }),
                distance: None,
                chain_code: Some("AC".to_string()),
                iata_code: Some("PAR".to_string()),
            }],
            meta: Some(AmadeusHotelListMeta { count: Some(1) }),
        };

        let offers = AmadeusHotelOffersResponse {
            data: vec![AmadeusHotelOffer {
                hotel: AmadeusOfferHotel {
                    hotel_id: "ACPARH29".to_string(),
                    name: "Acropolis Hotel Paris Boulogne".to_string(),
                    city_code: Some("PAR".to_string()),
                    latitude: Some(48.83593),
                    longitude: Some(2.24922),
                },
                available: true,
                offers: vec![
                    AmadeusOffer {
                        id: "OFFER-1".to_string(),
                        check_in_date: "2026-05-01".to_string(),
                        check_out_date: "2026-05-03".to_string(),
                        room: None,
                        price: AmadeusOfferPrice {
                            currency: "EUR".to_string(),
                            total: "215.00".to_string(),
                            base: Some("200.00".to_string()),
                        },
                        policies: None,
                    },
                    AmadeusOffer {
                        id: "OFFER-2".to_string(),
                        check_in_date: "2026-05-01".to_string(),
                        check_out_date: "2026-05-03".to_string(),
                        room: None,
                        price: AmadeusOfferPrice {
                            currency: "EUR".to_string(),
                            total: "175.50".to_string(),
                            base: Some("150.00".to_string()),
                        },
                        policies: None,
                    },
                ],
            }],
        };

        let result = AmadeusMapper::map_search_to_domain(
            hotel_list,
            offers,
            &Some(DomainPaginationParams {
                page: Some(1),
                page_size: Some(25),
            }),
        );

        assert_eq!(result.provider.as_deref(), Some("Amadeus"));
        assert_eq!(result.hotel_results.len(), 1);
        assert!(result.pagination.is_some());
        let pagination = result.pagination.unwrap();
        assert_eq!(pagination.page, 1);
        assert_eq!(pagination.page_size, 25);
        assert_eq!(pagination.total_results, Some(1));
        assert!(!pagination.has_next_page);
        assert!(!pagination.has_previous_page);

        let hotel = &result.hotel_results[0];
        assert_eq!(hotel.hotel_code, "ACPARH29");
        assert_eq!(hotel.hotel_name, "Acropolis Hotel Paris Boulogne");
        assert_eq!(hotel.result_token, "ACPARH29");
        assert_eq!(hotel.hotel_picture, "/img/hotel-placeholder.svg");
        assert_eq!(hotel.price.as_ref().unwrap().room_price, 175.5);
        assert_eq!(hotel.price.as_ref().unwrap().currency_code, "EUR");
        assert_eq!(hotel.location.as_ref().unwrap().latitude, 48.83593);
        assert_eq!(hotel.location.as_ref().unwrap().longitude, 2.24922);
    }

    #[test]
    fn maps_amadeus_hotel_details_to_domain_static_details() {
        let response = AmadeusHotelDetailsResponse {
            data: vec![AmadeusHotelDetailsEntry {
                hotel_id: "HLPAR266".to_string(),
                name: "Hilton Paris Opera".to_string(),
                geo_code: Some(AmadeusGeoCode {
                    latitude: 48.8757,
                    longitude: 2.32553,
                }),
                address: Some(AmadeusAddress {
                    lines: Some(vec!["108 Rue Saint-Lazare".to_string()]),
                    city_name: Some("Paris".to_string()),
                    country_code: Some("FR".to_string()),
                }),
                chain_code: Some("HL".to_string()),
                iata_code: Some("PAR".to_string()),
            }],
        };

        let result = AmadeusMapper::map_hotel_details_to_domain(response).unwrap();

        assert_eq!(result.hotel_code, "HLPAR266");
        assert_eq!(result.hotel_name, "Hilton Paris Opera");
        assert_eq!(result.address, "108 Rue Saint-Lazare, Paris, FR");
        assert_eq!(
            result.images,
            vec!["/img/hotel-placeholder.svg".to_string()]
        );
        assert_eq!(result.provider.as_deref(), Some("Amadeus"));
        assert_eq!(result.location.as_ref().unwrap().latitude, 48.8757);
    }

    #[test]
    fn maps_amadeus_offers_to_grouped_room_rates() {
        let offers = AmadeusHotelOffersResponse {
            data: vec![AmadeusHotelOffer {
                hotel: AmadeusOfferHotel {
                    hotel_id: "HLPAR266".to_string(),
                    name: "Hilton Paris Opera".to_string(),
                    city_code: Some("PAR".to_string()),
                    latitude: Some(48.8757),
                    longitude: Some(2.32553),
                },
                available: true,
                offers: vec![
                    AmadeusOffer {
                        id: "ZBC0IYFMFV".to_string(),
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
                            total: "245.00".to_string(),
                            base: Some("220.00".to_string()),
                        },
                        policies: Some(AmadeusOfferPolicies::default()),
                    },
                    AmadeusOffer {
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
                            total: "225.00".to_string(),
                            base: Some("200.00".to_string()),
                        },
                        policies: Some(AmadeusOfferPolicies::default()),
                    },
                ],
            }],
        };

        let result = AmadeusMapper::map_offers_to_grouped_rates(offers);

        assert_eq!(result.provider.as_deref(), Some("Amadeus"));
        assert_eq!(result.room_groups.len(), 1);
        let room_group = &result.room_groups[0];
        assert_eq!(room_group.name, "Deluxe King Room");
        assert_eq!(room_group.min_price, 225.0);
        assert_eq!(room_group.currency_code, "EUR");
        assert_eq!(room_group.room_types.len(), 2);
        assert_eq!(room_group.room_types[0].offer_id, "ZBC0IYFMFW");
        assert_eq!(room_group.room_types[1].offer_id, "ZBC0IYFMFV");
    }
}
