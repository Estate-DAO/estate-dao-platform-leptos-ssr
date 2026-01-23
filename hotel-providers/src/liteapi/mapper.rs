use crate::domain::*;
use crate::liteapi::models::booking::*;
use crate::liteapi::models::hotel_details::*;
use crate::liteapi::models::places::*;
use crate::liteapi::models::search::*;
use crate::ports::ProviderError;
use crate::ports::UISearchFilters;

const PAGINATION_LIMIT: i32 = 100; // Default limit (LiteAPI may have its own max)

pub struct LiteApiMapper;

impl LiteApiMapper {
    pub fn map_domain_search_to_liteapi(
        domain_criteria: &DomainHotelSearchCriteria,
        _ui_filters: &UISearchFilters,
    ) -> LiteApiHotelSearchRequest {
        let (offset, limit) = Self::calculate_offset_limit(&domain_criteria.pagination);

        // Check if this is a coordinate-based search (from "Search this area" feature)
        // or if we should use the place_id
        let is_coordinate_search = domain_criteria.place_id.starts_with("custom_")
            || (domain_criteria.latitude.is_some()
                && domain_criteria.longitude.is_some()
                && domain_criteria.place_id.is_empty());

        if is_coordinate_search {
            // Use latitude/longitude for coordinate-based searches
            tracing::debug!(
                target: "hotel_providers::liteapi",
                latitude = ?domain_criteria.latitude,
                longitude = ?domain_criteria.longitude,
                "Coordinate-based search (Search this area)"
            );
            LiteApiHotelSearchRequest {
                place_id: None,
                latitude: domain_criteria.latitude,
                longitude: domain_criteria.longitude,
                distance: Some(100000), // 100km distance for coordinate searches
                limit: Some(limit),
                offset: Some(offset),
                country_code: None,
                city_name: None,
                hotel_name: None,
            }
        } else {
            // Use place_id for normal searches
            LiteApiHotelSearchRequest {
                place_id: Some(domain_criteria.place_id.clone()),
                latitude: None,
                longitude: None,
                distance: Some(100000), // Default distance from legacy code
                limit: Some(limit),
                offset: Some(offset),
                country_code: None,
                city_name: None,
                hotel_name: None,
            }
        }
    }

    pub fn map_liteapi_search_response_to_domain(
        response: LiteApiHotelSearchResponse,
        pagination_params: &Option<DomainPaginationParams>,
    ) -> DomainHotelListAfterSearch {
        let hotel_results: Vec<DomainHotelAfterSearch> = response
            .data
            .into_iter()
            .map(|h| Self::map_liteapi_hotel_to_domain(h))
            .collect();

        // Compute pagination metadata from response total and request params
        let pagination =
            Self::compute_pagination_meta(response.total, pagination_params, hotel_results.len());

        DomainHotelListAfterSearch {
            hotel_results,
            pagination,
        }
    }

    /// Compute pagination metadata from API response
    fn compute_pagination_meta(
        total: Option<i32>,
        pagination_params: &Option<DomainPaginationParams>,
        result_count: usize,
    ) -> Option<DomainPaginationMeta> {
        let (page, page_size) = match pagination_params {
            Some(params) => {
                let page = params.page.unwrap_or(1).max(1);
                let page_size = params
                    .page_size
                    .unwrap_or(PAGINATION_LIMIT as u32)
                    .clamp(1, PAGINATION_LIMIT as u32);
                (page, page_size)
            }
            None => (1, PAGINATION_LIMIT as u32),
        };

        // Calculate has_next_page based on total or result_count
        let has_next_page = if let Some(total_results) = total {
            let total_results = total_results.max(0) as u32;
            let offset = (page - 1) * page_size;
            offset + (result_count as u32) < total_results
        } else {
            // Fallback: if we got a full page of results, assume there might be more
            result_count >= page_size as usize
        };

        Some(DomainPaginationMeta {
            page,
            page_size,
            total_results: total,
            has_next_page,
            has_previous_page: page > 1,
        })
    }

    pub fn map_liteapi_hotel_to_domain(hotel: LiteApiHotelResult) -> DomainHotelAfterSearch {
        let amenities: Vec<String> = hotel
            .facility_ids
            .iter()
            .map(|&id| crate::liteapi::map_facility_id::map_facility_id_to_name(id))
            .collect();

        DomainHotelAfterSearch {
            hotel_code: hotel.id.clone(),
            hotel_name: hotel.name,
            hotel_category: format!("{} Star", hotel.stars.unwrap_or(0.0)),
            star_rating: hotel.stars.unwrap_or(0.0) as u8,
            price: Some(DomainPrice {
                room_price: 0.0,
                currency_code: hotel.currency,
            }),
            hotel_picture: hotel.main_photo,
            amenities,
            property_type: hotel.hotel_type_id.map(Self::map_hotel_type_id_to_label),
            result_token: hotel.id,
            hotel_address: Some(hotel.address),
            distance_from_center_km: None,
            location: Some(DomainLocation {
                latitude: hotel.latitude,
                longitude: hotel.longitude,
            }),
        }
    }

    fn map_hotel_type_id_to_label(type_id: i32) -> String {
        match type_id {
            0 => "Not Available".to_string(),
            201 => "Apartments".to_string(),
            203 => "Hostels".to_string(),
            204 => "Hotels".to_string(),
            205 => "Motels".to_string(),
            206 => "Resorts".to_string(),
            207 => "Residences".to_string(),
            208 => "Bed and breakfasts".to_string(),
            209 => "Ryokans".to_string(),
            210 => "Farm stays".to_string(),
            212 => "Holiday parks".to_string(),
            213 => "Villas".to_string(),
            214 => "Campsites".to_string(),
            215 => "Boats".to_string(),
            216 => "Guest houses".to_string(),
            218 => "Inns".to_string(),
            219 => "Aparthotels".to_string(),
            220 => "Holiday homes".to_string(),
            221 => "Lodges".to_string(),
            222 => "Homestays".to_string(),
            223 => "Country houses".to_string(),
            224 => "Luxury tents".to_string(),
            225 => "Capsule hotels".to_string(),
            226 => "Love hotels".to_string(),
            227 => "Riads".to_string(),
            228 => "Chalets".to_string(),
            229 => "Condos".to_string(),
            230 => "Cottages".to_string(),
            231 => "Economy hotels".to_string(),
            232 => "Gites".to_string(),
            233 => "Health resorts".to_string(),
            234 => "Cruises".to_string(),
            235 => "Student accommodation".to_string(),
            243 => "Tree house property".to_string(),
            247 => "Pension".to_string(),
            250 => "Private vacation home".to_string(),
            251 => "Pousada".to_string(),
            252 => "Country house".to_string(),
            254 => "Campsite".to_string(),
            257 => "Cabin".to_string(),
            258 => "Holiday park".to_string(),
            262 => "Affittacamere".to_string(),
            264 => "Hostel/Backpacker accommodation".to_string(),
            265 => "Houseboat".to_string(),
            268 => "Ranch".to_string(),
            271 => "Agritourism property".to_string(),
            272 => "Mobile home".to_string(),
            273 => "Safari/Tentalow".to_string(),
            274 => "All-inclusive property".to_string(),
            276 => "Castle".to_string(),
            277 => "Property".to_string(),
            278 => "Palace".to_string(),
            _ => format!("Type {}", type_id),
        }
    }

    pub fn map_places_domain_to_liteapi(
        payload: DomainPlacesSearchPayload,
    ) -> LiteApiGetPlacesRequest {
        LiteApiGetPlacesRequest {
            text_query: payload.text_query,
        }
    }

    pub fn map_places_id_domain_to_liteapi(
        payload: DomainPlaceDetailsPayload,
    ) -> LiteApiGetPlaceRequest {
        LiteApiGetPlaceRequest {
            place_id: payload.place_id,
        }
    }

    pub fn map_liteapi_places_response_to_domain(
        payload: LiteApiGetPlacesResponse,
    ) -> DomainPlacesResponse {
        DomainPlacesResponse {
            data: payload
                .data
                .into_iter()
                .map(|place| DomainPlace {
                    place_id: place.place_id,
                    display_name: place.display_name,
                    formatted_address: place.formatted_address,
                })
                .collect(),
        }
    }

    pub fn map_liteapi_place_details_response_to_domain(
        payload: LiteApiGetPlaceResponse,
    ) -> DomainPlaceDetails {
        DomainPlaceDetails {
            data: DomainPlaceData {
                address_components: payload
                    .data
                    .address_components
                    .into_iter()
                    .map(|ac| DomainAddressComponent {
                        language_code: ac.language_code,
                        long_text: ac.long_text,
                        short_text: ac.short_text,
                        types: ac.types,
                    })
                    .collect(),
                location: DomainLocation {
                    latitude: payload.data.location.latitude,
                    longitude: payload.data.location.longitude,
                },
                viewport: DomainViewport {
                    high: DomainHigh {
                        latitude: payload.data.viewport.high.latitude,
                        longitude: payload.data.viewport.high.longitude,
                    },
                    low: DomainLow {
                        latitude: payload.data.viewport.low.latitude,
                        longitude: payload.data.viewport.low.longitude,
                    },
                },
            },
        }
    }

    pub fn map_domain_info_to_liteapi_rates(
        domain_criteria: &DomainHotelInfoCriteria,
        currency: &str,
        room_mapping: bool,
    ) -> Result<LiteApiHotelRatesRequest, ProviderError> {
        let search_criteria = &domain_criteria.search_criteria;

        let room_count = if search_criteria.no_of_rooms == 0 {
            let derived = search_criteria.room_guests.len() as i32;
            derived.max(1)
        } else {
            search_criteria.no_of_rooms as i32
        };

        let map_guest_to_occupancy = |room_guest: &DomainRoomGuest| -> LiteApiOccupancy {
            let children: Vec<i32> = if room_guest.no_of_children > 0 {
                room_guest
                    .children_ages
                    .as_ref()
                    .map(|ages| {
                        ages.iter()
                            .filter_map(|age_str| age_str.parse::<i32>().ok())
                            .collect()
                    })
                    .unwrap_or_default()
            } else {
                Vec::new()
            };
            LiteApiOccupancy {
                adults: room_guest.no_of_adults as i32,
                children,
            }
        };

        let occupancies: Vec<LiteApiOccupancy> =
            if search_criteria.room_guests.len() as i32 == room_count {
                search_criteria
                    .room_guests
                    .iter()
                    .map(map_guest_to_occupancy)
                    .collect()
            } else {
                let total_adults: i32 = search_criteria
                    .room_guests
                    .iter()
                    .map(|rg| rg.no_of_adults as i32)
                    .sum();
                let total_children: i32 = search_criteria
                    .room_guests
                    .iter()
                    .map(|rg| rg.no_of_children as i32)
                    .sum();
                let children_ages: Vec<i32> = search_criteria
                    .room_guests
                    .iter()
                    .filter_map(|rg| rg.children_ages.as_ref())
                    .flat_map(|ages| ages.iter())
                    .filter_map(|age_str| age_str.parse::<i32>().ok())
                    .collect();

                let base_adults = total_adults / room_count;
                let mut extra_adults = total_adults % room_count;
                let base_children = total_children / room_count;
                let mut extra_children = total_children % room_count;
                let mut age_iter = children_ages.into_iter();

                (0..room_count)
                    .map(|_| {
                        let adults = base_adults
                            + if extra_adults > 0 {
                                extra_adults -= 1;
                                1
                            } else {
                                0
                            };
                        let children_cnt = base_children
                            + if extra_children > 0 {
                                extra_children -= 1;
                                1
                            } else {
                                0
                            };
                        let children: Vec<i32> = if children_cnt > 0 {
                            age_iter.by_ref().take(children_cnt as usize).collect()
                        } else {
                            Vec::new()
                        };
                        LiteApiOccupancy { adults, children }
                    })
                    .collect()
            };

        let checkin = Self::format_date(search_criteria.check_in_date);
        let checkout = Self::format_date(search_criteria.check_out_date);

        let hotel_ids = if !domain_criteria.hotel_ids.is_empty() {
            domain_criteria.hotel_ids.clone()
        } else if !domain_criteria.token.is_empty() {
            vec![domain_criteria.token.clone()]
        } else {
            return Err(ProviderError::new(
                "LiteAPI",
                crate::ports::ProviderErrorKind::InvalidRequest,
                crate::ports::ProviderSteps::HotelRate,
                "No hotel ID available".to_string(),
            ));
        };

        Ok(LiteApiHotelRatesRequest {
            hotel_ids,
            occupancies,
            checkin,
            checkout,
            guest_nationality: search_criteria.guest_nationality.clone(),
            currency: currency.to_string(),
            timeout: None,
            max_rates_per_hotel: None,
            board_type: None,
            refundable_rates_only: None,
            sort: None,
            room_mapping: if room_mapping { Some(true) } else { None },
            include_hotel_data: None,
        })
    }

    pub fn map_domain_search_to_liteapi_min_rates(
        domain_criteria: &DomainHotelSearchCriteria,
        hotel_ids: Vec<String>,
        currency: &str,
    ) -> Result<LiteApiMinRatesRequest, ProviderError> {
        let checkin = Self::format_date(domain_criteria.check_in_date);
        let checkout = Self::format_date(domain_criteria.check_out_date);

        // Reuse occupancy logic
        let room_count = if domain_criteria.no_of_rooms == 0 {
            let derived = domain_criteria.room_guests.len() as i32;
            derived.max(1)
        } else {
            domain_criteria.no_of_rooms as i32
        };

        let map_guest_to_occupancy = |room_guest: &DomainRoomGuest| -> LiteApiOccupancy {
            let children: Vec<i32> = if room_guest.no_of_children > 0 {
                room_guest
                    .children_ages
                    .as_ref()
                    .map(|ages| {
                        ages.iter()
                            .filter_map(|age_str| age_str.parse::<i32>().ok())
                            .collect()
                    })
                    .unwrap_or_default()
            } else {
                Vec::new()
            };
            LiteApiOccupancy {
                adults: room_guest.no_of_adults as i32,
                children,
            }
        };

        let occupancies: Vec<LiteApiOccupancy> =
            if domain_criteria.room_guests.len() as i32 == room_count {
                domain_criteria
                    .room_guests
                    .iter()
                    .map(map_guest_to_occupancy)
                    .collect()
            } else {
                let total_adults: i32 = domain_criteria
                    .room_guests
                    .iter()
                    .map(|rg| rg.no_of_adults as i32)
                    .sum();
                let total_children: i32 = domain_criteria
                    .room_guests
                    .iter()
                    .map(|rg| rg.no_of_children as i32)
                    .sum();
                let children_ages: Vec<i32> = domain_criteria
                    .room_guests
                    .iter()
                    .flat_map(|rg| {
                        rg.children_ages
                            .as_ref()
                            .map(|ages| {
                                ages.iter()
                                    .filter_map(|age_str| age_str.parse::<i32>().ok())
                                    .collect::<Vec<i32>>()
                            })
                            .unwrap_or_default()
                    })
                    .collect();

                let base_adults = total_adults / room_count;
                let mut extra_adults = total_adults % room_count;
                let base_children = total_children / room_count;
                let mut extra_children = total_children % room_count;

                let mut age_iter = children_ages.into_iter();

                (0..room_count)
                    .map(|_| {
                        let adults = base_adults
                            + if extra_adults > 0 {
                                extra_adults -= 1;
                                1
                            } else {
                                0
                            };
                        let children_cnt = base_children
                            + if extra_children > 0 {
                                extra_children -= 1;
                                1
                            } else {
                                0
                            };
                        let children: Vec<i32> = if children_cnt > 0 {
                            age_iter.by_ref().take(children_cnt as usize).collect()
                        } else {
                            Vec::new()
                        };

                        LiteApiOccupancy { adults, children }
                    })
                    .collect()
            };

        Ok(LiteApiMinRatesRequest {
            hotel_ids,
            occupancies,
            currency: currency.to_string(),
            guest_nationality: domain_criteria.guest_nationality.clone(),
            checkin,
            checkout,
            timeout: Some(4), // 4 seconds timeout for min rates
        })
    }

    pub fn map_liteapi_min_rates_response_to_domain(
        response: LiteApiMinRatesResponse,
        currency_code: &str,
    ) -> std::collections::HashMap<String, DomainPrice> {
        let mut map = std::collections::HashMap::new();
        if let Some(data) = response.data {
            for hotel in data {
                // Use suggested_selling_price as the price
                // The min-rates endpoint returns net rates (price) and suggested selling price
                // We should display suggested selling price
                map.insert(
                    hotel.hotel_id,
                    DomainPrice {
                        room_price: hotel.suggested_selling_price,
                        currency_code: currency_code.to_string(),
                    },
                );
            }
        }
        map
    }

    fn format_date(date: (u32, u32, u32)) -> String {
        format!("{}-{:02}-{:02}", date.0, date.1, date.2)
    }

    pub fn map_liteapi_rates_response_to_domain(
        response: LiteApiHotelRatesResponse,
    ) -> Vec<DomainRoomOption> {
        let hotel_rate_info = match response.data.and_then(|v| v.into_iter().next()) {
            Some(data) => data,
            None => return Vec::new(),
        };

        hotel_rate_info
            .room_types
            .into_iter()
            .flat_map(|rt| {
                // In LiteAPI search response, offerRetailRate is sometimes at roomType level or invoked differently?
                // The structure shows `roomTypes` -> `rates`.
                // In generic mapping we iterate through rates.
                let room_type_id = rt.room_type_id.clone();
                let offer_id = rt.offer_id.clone();
                rt.rates.into_iter().map(move |r| {
                    // Note: The original SSR code referenced `rt.offer_retail_rate`.
                    // But checking `LiteApiSearchRoomType` in `search.rs`, it ONLY has `roomTypeId`, `offerId`, `rates`.
                    // It does NOT seem to have `offerRetailRate`.
                    // The `LiteApiRate` inside has `retailRate`.
                    // So we might not need `offer_retail_rate` from parent?
                    // In SSR code, `LiteApiSearchRoomType` HAD `offer_retail_rate`.
                    // Let's verify `search.rs` definition again...
                    // StartLine 60: public struct LiteApiSearchRoomType { room_type_id, offer_id, rates }
                    // So no offer_retail_rate at room level.
                    // The rate itself has `retail_rate`.
                    // So we pass None for offer_retail_rate equivalent or extract it from rate if needed.
                    Self::map_liteapi_room_to_domain(r, room_type_id.clone(), offer_id.clone())
                })
            })
            .collect()
    }

    fn map_liteapi_room_to_domain(
        rate: LiteApiRate,
        room_type_id: String,
        offer_id: String,
    ) -> DomainRoomOption {
        // Use retail_rate.total as the primary price source (Gross)
        // suggested_selling_price is mapped to its own field
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
            .unwrap_or_else(|| "SGD".to_string()); // Default to SGD if missing? Or USD?

        let meal_plan = {
            let board_name = rate.board_name.as_deref().unwrap_or("").trim();
            let board_type = rate.board_type.as_deref().unwrap_or("").trim();
            if board_name.is_empty() && board_type.is_empty() {
                None
            } else if board_type.is_empty() {
                Some(board_name.to_string())
            } else if board_name.is_empty() {
                Some(board_type.to_string())
            } else {
                Some(format!("{} ({})", board_name, board_type))
            }
        };

        let tax_entries = rate.retail_rate.taxes_and_fees.as_deref().unwrap_or(&[]);
        let tax_lines: Vec<DomainTaxLine> = tax_entries
            .iter()
            .filter(|tax| tax.amount > 0.0)
            .map(|tax| {
                let description = tax
                    .description
                    .as_ref()
                    .map(|desc| desc.trim())
                    .filter(|desc| !desc.is_empty())
                    .map(|desc| desc.to_string())
                    .unwrap_or_else(|| "Taxes & Fees".to_string());
                let currency = if tax.currency.is_empty() {
                    currency_code.clone()
                } else {
                    tax.currency.clone()
                };
                DomainTaxLine {
                    description,
                    amount: tax.amount,
                    currency_code: currency,
                    included: tax.included,
                }
            })
            .collect();

        let occupancy_info = Some(DomainRoomOccupancy {
            max_occupancy: Some(rate.max_occupancy as u32),
            adult_count: Some(rate.adult_count as u32),
            child_count: Some(rate.child_count as u32),
        });

        // Calculate included taxes FIRST (for fallback)
        let included_tax_total: f64 = tax_lines
            .iter()
            .filter(|line| line.included)
            .map(|line| line.amount)
            .sum();

        // Use suggested_selling_price if available, otherwise compute Net (room_price - included_taxes)
        let suggested_price_val = rate
            .retail_rate
            .suggested_selling_price
            .as_ref()
            .and_then(|v| v.first())
            .map(|amount| amount.amount)
            .unwrap_or_else(|| room_price - included_tax_total);

        let excluded_tax_total: f64 = tax_lines
            .iter()
            .filter(|line| !line.included)
            .map(|line| line.amount)
            .sum();

        let cancellation_policies = if let Some(cp) = rate.cancellation_policies {
            Some(DomainCancellationPolicies {
                cancel_policy_infos: cp
                    .cancel_policy_infos
                    .unwrap_or_default()
                    .into_iter()
                    .map(|policy| DomainCancelPolicyInfo {
                        cancel_time: policy.cancel_time.unwrap_or_default(),
                        amount: policy.amount,
                        policy_type: policy.r#type.unwrap_or_default(), // r#type from models?
                        timezone: "UTC".to_string(),                    // Default if missing?
                        currency: policy.currency,
                    })
                    .collect(),
                hotel_remarks: None, // Missing in Search LiteApiCancellationPolicies?
                refundable_tag: cp.refundable_tag.unwrap_or_default(),
            })
        } else {
            None
        };

        let mapped_room_id_val = rate.mapped_room_id.clone().unwrap_or_default();

        DomainRoomOption {
            mapped_room_id: mapped_room_id_val.clone(),
            price: DomainDetailedPrice {
                published_price: room_price,
                published_price_rounded_off: room_price, // Rounding logic can be added
                offered_price: room_price,
                offered_price_rounded_off: room_price,
                suggested_selling_price: suggested_price_val,
                suggested_selling_price_rounded_off: suggested_price_val.round(),
                room_price,
                tax: excluded_tax_total,
                extra_guest_charge: 0.0,
                child_charge: 0.0,
                other_charges: 0.0,
                currency_code: currency_code.clone(),
            },
            tax_lines,
            room_data: DomainRoomData {
                mapped_room_id: mapped_room_id_val,
                occupancy_number: Some(rate.occupancy_number as u32),
                room_name: rate.name,
                room_unique_id: room_type_id,
                rate_key: rate.rate_id,
                offer_id,
            },
            meal_plan,
            occupancy_info,
            offer_retail_rate: None,
            cancellation_policies,
            promotions: None,
            remarks: rate.remarks.clone(),
            // NEW: Enhanced rate information
            perks: rate
                .perks
                .iter()
                .filter_map(|p| {
                    p.name.clone().map(|name| DomainPerk {
                        name,
                        amount: p.amount,
                        currency: p.currency.clone(),
                    })
                })
                .collect(),
            original_price: rate
                .retail_rate
                .initial_price
                .as_ref()
                .and_then(|v| v.first())
                .map(|a| a.amount),
            board_type_code: rate.board_type.clone(),
            payment_types: rate.payment_types.clone(),
        }
    }

    pub fn map_liteapi_details_to_domain_static(
        details: LiteApiSingleHotelDetailData,
    ) -> DomainHotelStaticDetails {
        let sentiment_categories = details
            .sentiment_analysis
            .as_ref()
            .map(|s| s.categories.clone())
            .unwrap_or_default();
        let mut images = Vec::new();
        if !details.main_photo.is_empty() {
            images.push(details.main_photo.clone());
        }
        if let Some(thumbnail) = &details.thumbnail {
            if !thumbnail.is_empty() && thumbnail != &details.main_photo {
                images.push(thumbnail.clone());
            }
        }
        for hotel_image in &details.hotel_images {
            if !hotel_image.url.is_empty() && !images.contains(&hotel_image.url) {
                images.push(hotel_image.url.clone());
            }
            if !hotel_image.url_hd.is_empty()
                && hotel_image.url_hd != hotel_image.url
                && !images.contains(&hotel_image.url_hd)
            {
                images.push(hotel_image.url_hd.clone());
            }
        }

        let location = details.location;

        DomainHotelStaticDetails {
            hotel_code: details.id,
            hotel_name: details.name,
            star_rating: details.star_rating,
            rating: if details.rating > 0.0 {
                Some(details.rating)
            } else {
                None
            },
            review_count: if details.review_count > 0 {
                Some(details.review_count as u32)
            } else {
                None
            },
            categories: if !sentiment_categories.is_empty() {
                sentiment_categories
            } else {
                details.categories
            }
            .into_iter()
            .map(|c| DomainReviewCategory {
                name: c.name,
                rating: c.rating as f32,
                description: if c.description.trim().is_empty() {
                    None
                } else {
                    Some(c.description)
                },
            })
            .collect(),
            description: details.hotel_description,
            address: details.address,
            images,
            hotel_facilities: details.hotel_facilities,
            amenities: vec![],
            rooms: Self::map_room_details(details.rooms),
            location: location.map(|location| DomainLocation {
                latitude: location.latitude,
                longitude: location.longitude,
            }),
            checkin_checkout_times: Some(DomainCheckinCheckoutTimes {
                checkin: details.checkin_checkout_times.checkin,
                checkout: details.checkin_checkout_times.checkout,
            }),
            policies: details
                .policies
                .into_iter()
                .map(|p| DomainPolicy {
                    policy_type: if p.policy_type.is_empty() {
                        None
                    } else {
                        Some(p.policy_type)
                    },
                    name: p.name,
                    description: p.description,
                })
                .collect(),
        }
    }

    fn map_room_details(rooms: Option<Vec<LiteApiRoom>>) -> Vec<DomainStaticRoom> {
        rooms
            .unwrap_or_default()
            .into_iter()
            .map(|room| {
                let amenities = room.room_amenities.into_iter().map(|a| a.name).collect();
                let photos = room
                    .photos
                    .into_iter()
                    .filter_map(|photo| {
                        if !photo.hd_url.trim().is_empty() {
                            Some(photo.hd_url)
                        } else if !photo.url.trim().is_empty() {
                            Some(photo.url)
                        } else {
                            None
                        }
                    })
                    .collect();
                let bed_types = room
                    .bed_types
                    .into_iter()
                    .map(|bed| {
                        let mut label = format!("{} {}", bed.quantity, bed.bed_type);
                        if !bed.bed_size.trim().is_empty() {
                            label.push_str(&format!(" ({})", bed.bed_size));
                        }
                        label
                    })
                    .collect();

                DomainStaticRoom {
                    room_id: room.id.to_string(),
                    room_name: room.room_name,
                    description: room.description,
                    room_size_square: room.room_size_square,
                    room_size_unit: room.room_size_unit,
                    max_adults: Some(room.max_adults as u32),
                    max_children: Some(room.max_children as u32),
                    max_occupancy: Some(room.max_occupancy as u32),
                    amenities,
                    photos,
                    bed_types,
                }
            })
            .collect()
    }

    pub fn map_liteapi_get_booking_to_domain(
        response: LiteApiGetBookingResponse,
    ) -> Result<DomainGetBookingResponse, ProviderError> {
        let bookings = response
            .data
            .into_iter()
            .map(|liteapi_booking| {
                let hotel = DomainBookingHotelInfo {
                    hotel_id: liteapi_booking.hotel.hotel_id,
                    name: liteapi_booking.hotel.name,
                };

                let holder = if let Some(h) = liteapi_booking.holder {
                    DomainBookingHolder {
                        first_name: h.first_name,
                        last_name: h.last_name,
                        email: h.email,
                        phone: h.phone.unwrap_or_default(),
                    }
                } else {
                    DomainBookingHolder {
                        first_name: "".to_string(),
                        last_name: "".to_string(),
                        email: "".to_string(),
                        phone: "".to_string(),
                    }
                };

                DomainBookingDetails {
                    booking_id: liteapi_booking.booking_id,
                    client_reference: Some(liteapi_booking.client_reference),
                    status: liteapi_booking.status,
                    hotel,
                    holder,
                    price: liteapi_booking.price,
                    currency: liteapi_booking.currency,
                }
            })
            .collect();

        Ok(DomainGetBookingResponse { bookings })
    }

    pub fn map_domain_get_booking_to_liteapi(
        request: &DomainGetBookingRequest,
    ) -> Result<LiteApiGetBookingRequest, ProviderError> {
        Ok(LiteApiGetBookingRequest {
            client_reference: request.client_reference.clone(),
            guest_id: request.guest_id.clone(),
            timeout: None,
        })
    }

    fn calculate_offset_limit(pagination: &Option<DomainPaginationParams>) -> (i32, i32) {
        match pagination {
            Some(params) => {
                let page = params.page.unwrap_or(1).max(1);
                let page_size = params
                    .page_size
                    .unwrap_or(PAGINATION_LIMIT as u32)
                    .clamp(1, PAGINATION_LIMIT as u32);
                let offset = (page - 1) * page_size;
                (offset as i32, page_size as i32)
            }
            None => (0, PAGINATION_LIMIT),
        }
    }
    pub fn map_domain_block_to_liteapi_prebook(
        domain_request: &crate::domain::DomainBlockRoomRequest,
    ) -> Result<LiteApiPrebookRequest, ProviderError> {
        let offer_id = &domain_request.selected_room.offer_id;

        if offer_id.is_empty() {
            return Err(ProviderError::new(
                "LiteAPI",
                crate::ports::ProviderErrorKind::InvalidRequest,
                crate::ports::ProviderSteps::HotelBlockRoom,
                "Offer ID is required for LiteAPI prebook.".to_string(),
            ));
        }

        Ok(LiteApiPrebookRequest {
            offer_id: offer_id.clone(),
            use_payment_sdk: false,
            addons: None,
            include_credit_balance: None,
        })
    }

    pub fn map_liteapi_prebook_to_domain_block(
        liteapi_response: crate::liteapi::models::booking::LiteApiPrebookResponse,
        original_request: &crate::domain::DomainBlockRoomRequest,
    ) -> crate::domain::DomainBlockRoomResponse {
        let provider_data_json = serde_json::to_string(&liteapi_response).unwrap_or_default();

        let data = match liteapi_response.data {
            Some(d) => d,
            None => {
                return crate::domain::DomainBlockRoomResponse {
                    block_id: "".to_string(),
                    is_price_changed: false,
                    is_cancellation_policy_changed: false,
                    blocked_rooms: vec![],
                    total_price: crate::domain::DomainDetailedPrice {
                        published_price: 0.0,
                        published_price_rounded_off: 0.0,
                        offered_price: 0.0,
                        offered_price_rounded_off: 0.0,
                        suggested_selling_price: 0.0,
                        suggested_selling_price_rounded_off: 0.0,
                        room_price: 0.0,
                        tax: 0.0,
                        extra_guest_charge: 0.0,
                        child_charge: 0.0,
                        other_charges: 0.0,
                        currency_code: "".to_string(),
                    },
                    provider_data: Some(provider_data_json),
                };
            }
        };

        let mut flattened_rates = Vec::new();
        for room_type in &data.room_types {
            flattened_rates.extend(room_type.rates.clone());
        }

        let currency = data.currency.clone().unwrap_or_default();
        let mut blocked_rooms = Vec::new();

        for rate in &flattened_rates {
            let room_price_amount = rate
                .retail_rate
                .total
                .first()
                .map(|a| a.amount)
                .unwrap_or(data.price.unwrap_or_default());
            let room_currency = rate
                .retail_rate
                .total
                .first()
                .map(|a| a.currency.clone())
                .unwrap_or_else(|| currency.clone());

            // Note: In refined models, retail_rate has 'total' which is Vec<LiteApiPrice>
            // We need to confirm if retail_rate in PREBOOK response matches LiteApiRetailRate from search.
            // LiteApiRate is shared, so it should be same.

            let suggested_price_amount = rate
                .retail_rate
                .suggested_selling_price
                .as_ref()
                .and_then(|v| v.first())
                .map(|a| a.amount)
                .unwrap_or(room_price_amount);

            let detailed_price = crate::domain::DomainDetailedPrice {
                published_price: room_price_amount,
                published_price_rounded_off: room_price_amount.round(),
                offered_price: room_price_amount,
                offered_price_rounded_off: room_price_amount.round(),
                suggested_selling_price: suggested_price_amount,
                suggested_selling_price_rounded_off: suggested_price_amount.round(),
                room_price: room_price_amount,
                tax: 0.0,
                extra_guest_charge: 0.0,
                child_charge: 0.0,
                other_charges: 0.0,
                currency_code: room_currency,
            };

            let room_name = if !rate.name.is_empty() {
                rate.name.clone()
            } else {
                original_request.selected_room.room_name.clone()
            };

            blocked_rooms.push(crate::domain::DomainBlockedRoom {
                room_code: original_request.selected_room.room_unique_id.clone(),
                room_name,
                room_type_code: Some(original_request.selected_room.room_unique_id.clone()),
                price: detailed_price,
                cancellation_policy: rate
                    .cancellation_policies
                    .as_ref()
                    .and_then(|cp| cp.refundable_tag.clone()),
                meal_plan: Some(format!(
                    "{} - {}",
                    rate.board_type.clone().unwrap_or_default(),
                    rate.board_name.clone().unwrap_or_default()
                )),
            });
        }

        // Use suggested price from data level
        // Currently LiteApiPrebookData has price: Option<f64>
        // We might need to adjust models if suggested_selling_price is available at top level in prebook response,
        // but looking at `booking.rs`, `LiteApiPrebookData` only has `price`.
        // Legacy code used `data.suggested_selling_price`.
        // Let's check `booking.rs` model again.

        let total_amount = data.price.unwrap_or_default();

        // Manual construction of DomainDetailedPrice since Default is not implemented
        let total_price = crate::domain::DomainDetailedPrice {
            published_price: total_amount,
            published_price_rounded_off: total_amount.round(),
            offered_price: total_amount,
            offered_price_rounded_off: total_amount.round(),
            suggested_selling_price: total_amount, // Fallback as field is missing in current model
            suggested_selling_price_rounded_off: total_amount.round(),
            room_price: total_amount,
            tax: 0.0,
            extra_guest_charge: 0.0,
            child_charge: 0.0,
            other_charges: 0.0,
            currency_code: currency.clone(),
        };

        crate::domain::DomainBlockRoomResponse {
            block_id: data.prebook_id.clone(),
            is_price_changed: false, // Legacy used explicit field, check if we need to add it to model
            is_cancellation_policy_changed: false,
            blocked_rooms,
            total_price,
            provider_data: Some(provider_data_json),
        }
    }
    pub fn map_domain_book_to_liteapi_book(
        domain_request: &crate::domain::DomainBookRoomRequest,
    ) -> Result<LiteApiBookRequest, ProviderError> {
        let holder = LiteApiHolder {
            first_name: domain_request.holder.first_name.clone(),
            last_name: domain_request.holder.last_name.clone(),
            email: domain_request.holder.email.clone(),
            phone: Some(domain_request.holder.phone.clone()),
        };

        let payment = LiteApiPayment {
            method: match domain_request.payment.method {
                crate::domain::DomainPaymentMethod::AccCreditCard => "ACC_CREDIT_CARD".to_string(),
                crate::domain::DomainPaymentMethod::Wallet => "WALLET".to_string(),
            },
            transaction_id: None,
        };

        let guests: Vec<LiteApiGuest> = domain_request
            .guests
            .iter()
            .map(|guest| {
                // Use holder's email/phone as fallback if guest has empty values
                let email = if guest.email.is_empty() {
                    domain_request.holder.email.clone()
                } else {
                    guest.email.clone()
                };
                let phone = if guest.phone.is_empty() {
                    domain_request.holder.phone.clone()
                } else {
                    guest.phone.clone()
                };
                LiteApiGuest {
                    occupancy_number: guest.occupancy_number as i32,
                    first_name: guest.first_name.clone(),
                    last_name: guest.last_name.clone(),
                    email,
                    phone,
                    remarks: guest.remarks.clone(),
                }
            })
            .collect();

        Ok(LiteApiBookRequest {
            holder,
            payment,
            prebook_id: domain_request.block_id.clone(),
            guests,
            client_reference: domain_request.client_reference.clone(),
        })
    }

    pub fn map_liteapi_book_to_domain_book(
        liteapi_response: crate::liteapi::models::booking::LiteApiBookResponse,
        original_request: &crate::domain::DomainBookRoomRequest,
    ) -> crate::domain::DomainBookRoomResponse {
        let _provider_data_json = serde_json::to_string(&liteapi_response).unwrap_or_default();

        let data = match liteapi_response.data {
            Some(d) => d,
            None => {
                return crate::domain::DomainBookRoomResponse {
                    booking_id: "".to_string(),
                    client_reference: original_request
                        .client_reference
                        .clone()
                        .unwrap_or_default(),
                    supplier_booking_id: "".to_string(),
                    supplier_booking_name: "".to_string(),
                    supplier: "".to_string(),
                    supplier_id: 0,
                    status: crate::domain::DomainBookingStatus::Failed,
                    hotel_confirmation_code: "".to_string(),
                    checkin: "".to_string(),
                    checkout: "".to_string(),
                    hotel: crate::domain::DomainBookedHotel {
                        hotel_id: "".to_string(),
                        name: "".to_string(),
                    },
                    booked_rooms: vec![],
                    holder: original_request.holder.clone(),
                    created_at: "".to_string(),
                    cancellation_policies: crate::domain::DomainCancellationPolicies {
                        cancel_policy_infos: vec![],
                        hotel_remarks: None,
                        refundable_tag: "".to_string(),
                    },
                    price: 0.0,
                    commission: 0.0,
                    currency: "".to_string(),
                    special_remarks: None,
                    optional_fees: None,
                    mandatory_fees: None,
                    know_before_you_go: None,
                    remarks: None,
                    guest_id: None,
                };
            }
        };

        let status = match data.status.as_str() {
            "CONFIRMED" => crate::domain::DomainBookingStatus::Confirmed,
            "PENDING" => crate::domain::DomainBookingStatus::Pending,
            "FAILED" => crate::domain::DomainBookingStatus::Failed,
            "CANCELLED" => crate::domain::DomainBookingStatus::Cancelled,
            _ => crate::domain::DomainBookingStatus::Pending,
        };

        let hotel = crate::domain::DomainBookedHotel {
            hotel_id: data.hotel.hotel_id,
            name: data.hotel.name,
        };

        let booked_rooms = data
            .booked_rooms
            .into_iter()
            .map(|room| crate::domain::DomainBookedRoom {
                room_type: crate::domain::DomainRoomTypeInfo {
                    name: room.room_type.name,
                },
                board_type: room.board_type,
                board_name: room.board_name,
                adults: room.adults,
                children: room.children,
                rate: crate::domain::DomainBookedRoomRate {
                    retail_rate: crate::domain::DomainBookedRetailRate {
                        total: crate::domain::DomainBookedPrice {
                            amount: room.flattened_rate.amount,
                            currency: room.flattened_rate.currency,
                        },
                    },
                },
                first_name: room.first_name,
                last_name: room.last_name,
                mapped_room_id: room.mapped_room_id,
            })
            .collect();

        let holder = crate::domain::DomainBookingHolder {
            first_name: data.holder.first_name,
            last_name: data.holder.last_name,
            email: data.holder.email,
            phone: data.holder.phone.unwrap_or_default(),
        };

        let cancellation_policies = crate::domain::DomainCancellationPolicies {
            cancel_policy_infos: data
                .cancellation_policies
                .cancel_policy_infos
                .unwrap_or_default()
                .into_iter()
                .map(|policy| crate::domain::DomainCancelPolicyInfo {
                    cancel_time: policy.cancel_time,
                    amount: policy.amount,
                    policy_type: policy.policy_type,
                    timezone: policy.timezone,
                    currency: policy.currency,
                })
                .collect(),
            hotel_remarks: data
                .cancellation_policies
                .hotel_remarks
                .as_ref()
                .filter(|remarks| !remarks.is_empty())
                .map(|remarks| remarks.join(". ")),
            refundable_tag: data.cancellation_policies.refundable_tag,
        };

        crate::domain::DomainBookRoomResponse {
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
