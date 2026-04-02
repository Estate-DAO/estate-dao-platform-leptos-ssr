use std::collections::HashMap;

use crate::amadeus::models::search::{
    AmadeusAddress, AmadeusHotelListResponse, AmadeusHotelOffersResponse, AmadeusOffer,
};
use crate::domain::{DomainHotelAfterSearch, DomainHotelListAfterSearch, DomainLocation, DomainPaginationParams, DomainPrice};
use crate::ports::ProviderNames;

#[derive(Clone, Debug, Default)]
pub struct AmadeusMapper;

impl AmadeusMapper {
    pub fn map_search_to_domain(
        hotel_list: AmadeusHotelListResponse,
        offers: AmadeusHotelOffersResponse,
        _pagination_params: &Option<DomainPaginationParams>,
    ) -> DomainHotelListAfterSearch {
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
                            offer_bundle.hotel.latitude.zip(offer_bundle.hotel.longitude).map(
                                |(latitude, longitude)| DomainLocation {
                                    latitude,
                                    longitude,
                                },
                            ),
                            None,
                        )
                    };

                Some(DomainHotelAfterSearch {
                    hotel_code: offer_bundle.hotel.hotel_id.clone(),
                    hotel_name,
                    hotel_category: "Hotel".to_string(),
                    star_rating: 0,
                    price: Some(cheapest),
                    hotel_picture: String::new(),
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
            pagination: None,
            provider: Some(ProviderNames::Amadeus.to_string()),
        }
    }

    fn cheapest_price(offers: &[AmadeusOffer]) -> Option<DomainPrice> {
        offers
            .iter()
            .filter_map(|offer| {
                offer.price.total.parse::<f64>().ok().map(|amount| DomainPrice {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::amadeus::models::search::{
        AmadeusAddress, AmadeusGeoCode, AmadeusHotelListEntry, AmadeusHotelListResponse,
        AmadeusHotelOffer, AmadeusHotelOffersResponse, AmadeusOffer, AmadeusOfferHotel,
        AmadeusOfferPrice,
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
                        price: AmadeusOfferPrice {
                            currency: "EUR".to_string(),
                            total: "215.00".to_string(),
                            base: Some("200.00".to_string()),
                        },
                    },
                    AmadeusOffer {
                        id: "OFFER-2".to_string(),
                        check_in_date: "2026-05-01".to_string(),
                        check_out_date: "2026-05-03".to_string(),
                        price: AmadeusOfferPrice {
                            currency: "EUR".to_string(),
                            total: "175.50".to_string(),
                            base: Some("150.00".to_string()),
                        },
                    },
                ],
            }],
        };

        let result = AmadeusMapper::map_search_to_domain(hotel_list, offers, &None);

        assert_eq!(result.provider.as_deref(), Some("Amadeus"));
        assert_eq!(result.hotel_results.len(), 1);

        let hotel = &result.hotel_results[0];
        assert_eq!(hotel.hotel_code, "ACPARH29");
        assert_eq!(hotel.hotel_name, "Acropolis Hotel Paris Boulogne");
        assert_eq!(hotel.result_token, "ACPARH29");
        assert_eq!(hotel.price.as_ref().unwrap().room_price, 175.5);
        assert_eq!(hotel.price.as_ref().unwrap().currency_code, "EUR");
        assert_eq!(hotel.location.as_ref().unwrap().latitude, 48.83593);
        assert_eq!(hotel.location.as_ref().unwrap().longitude, 2.24922);
    }
}
