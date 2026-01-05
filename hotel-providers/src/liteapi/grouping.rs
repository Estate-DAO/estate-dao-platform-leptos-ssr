use std::collections::{BTreeMap, HashMap, HashSet};

use hotel_types::{
    DomainGroupedRoomRates, DomainRoomGroup, DomainRoomOption, DomainRoomVariant, DomainStaticRoom,
    GroupedTaxItem,
};

fn format_room_names(names: &[String]) -> String {
    let mut counts = BTreeMap::new();
    for name in names {
        *counts.entry(name.to_string()).or_insert(0) += 1;
    }

    let mut result = Vec::new();

    for (name, count) in counts {
        if count > 1 {
            result.push(format!("{} x {}", count, name));
        } else {
            result.push(name);
        }
    }
    result.join(" + ")
}

use std::cmp::Ordering;

// --- Helper Structs for Grouping ---

#[derive(Eq, PartialEq, Hash)]
struct OfferRoomSignature {
    mapped_room_id: String,
    occupancy_number: u32, // Used to distinguish between room configurations
    meal_plan: String,
    price_bits: u64, // For float comparison
}

struct OfferGroup {
    offer_id: String,
    mapped_room_id: Option<String>, // Only set if all rooms in offer have same mapped_id
    rates: Vec<DomainRoomOption>,
    room_names: Vec<String>,
}

pub fn group_liteapi_rates(
    rates: Vec<DomainRoomOption>,
    static_rooms: Option<&[DomainStaticRoom]>,
) -> DomainGroupedRoomRates {
    let mut grouped_by_offer: BTreeMap<String, OfferGroup> = BTreeMap::new();
    let mut offer_room_signatures: HashMap<String, HashSet<OfferRoomSignature>> = HashMap::new();
    let mut seen_rate_keys = HashSet::new();

    // 1. Group raw rates by offer_id
    for rate in rates {
        if !seen_rate_keys.insert(rate.room_data.rate_key.clone()) {
            continue;
        }

        let offer_key = rate.room_data.offer_id.clone();

        // Track unique signatures per offer if needed for validation
        let signature = OfferRoomSignature {
            mapped_room_id: rate.mapped_room_id.clone(),
            occupancy_number: rate.room_data.occupancy_number.unwrap_or(0),
            meal_plan: rate.meal_plan.clone().unwrap_or_default(),
            price_bits: rate.price.room_price.to_bits(),
        };
        offer_room_signatures
            .entry(offer_key.clone())
            .or_default()
            .insert(signature);

        grouped_by_offer
            .entry(offer_key.clone())
            .and_modify(|entry| {
                // If we encounter a different mapped_id (and it's not empty), then it's a mixed offer (or multi-room)
                if let Some(current_id) = &entry.mapped_room_id {
                    if !rate.mapped_room_id.is_empty() && rate.mapped_room_id != *current_id {
                        entry.mapped_room_id = None;
                    }
                } else if entry.mapped_room_id.is_none() && !rate.mapped_room_id.is_empty() {
                    // Logic: if it's currently None and we verified it's the first item logic in separate block
                    if entry.rates.is_empty() {
                        entry.mapped_room_id = Some(rate.mapped_room_id.clone());
                    }
                }

                entry.rates.push(rate.clone());
                entry.room_names.push(rate.room_data.room_name.clone());
            })
            .or_insert_with(|| OfferGroup {
                offer_id: offer_key.clone(),
                mapped_room_id: if !rate.mapped_room_id.is_empty() {
                    Some(rate.mapped_room_id.clone())
                } else {
                    None
                },
                rates: vec![rate.clone()],
                room_names: vec![rate.room_data.room_name.clone()],
            });
    }

    // 2. Convert OfferGroups to DomainRoomGroups
    // Map: (MappedID OR CombinedString) -> DomainRoomGroup
    let mut card_map: HashMap<String, DomainRoomGroup> = HashMap::new();

    for offer in grouped_by_offer.into_values() {
        if offer.rates.is_empty() {
            continue;
        }

        let first_rate = &offer.rates[0];
        let currency_code = first_rate.price.currency_code.clone();

        let formatted_name = format_room_names(&offer.room_names);

        // Determine Group Key (Card Identity)
        // If single mapped_id (Type A), use it. Else use combined names.
        let group_key = if let Some(mid) = &offer.mapped_room_id {
            format!("MAPPED_{}", mid)
        } else {
            let mut names = offer.room_names.clone();
            names.sort();
            names.join(" + ")
        };

        // Calculate Totals for this Offer Variant
        let count = offer.rates.len() as f64;
        let total_room_price: f64 = offer
            .rates
            .iter()
            .map(|r| r.price_excluding_included_taxes())
            .sum();
        let total_tax: f64 = offer.rates.iter().map(|r| r.price.tax).sum();
        let avg_price_per_room = if count > 0.0 {
            total_room_price / count
        } else {
            0.0
        };

        // Build Tax Items
        let mut tax_breakdown = Vec::new();
        // Just transforming the first room's taxes for the structure.
        for tax in &first_rate.tax_lines {
            tax_breakdown.push(GroupedTaxItem {
                description: tax.description.clone(),
                amount: tax.amount, // Note: this is per-room tax usually
                currency_code: tax.currency_code.clone(),
                included: tax.included,
            });
        }

        // Construct Variant
        let variant = DomainRoomVariant {
            offer_id: offer.offer_id.clone(),
            rate_key: first_rate.room_data.rate_key.clone(),
            room_name: formatted_name.clone(), // Combined Name
            mapped_room_id: first_rate.mapped_room_id.clone(),
            room_count: count as u32,
            room_unique_id: first_rate.room_data.room_unique_id.clone(),
            occupancy_number: first_rate.room_data.occupancy_number,
            meal_plan: first_rate.meal_plan.clone(),
            total_price_for_all_rooms: total_room_price,
            total_price_for_one_room: avg_price_per_room + (total_tax / count), // Gross estimated
            price_per_room_excluding_taxes: avg_price_per_room,
            currency_code: currency_code.clone(),
            tax_breakdown,
            occupancy_info: first_rate.occupancy_info.clone(),
            cancellation_info: first_rate.cancellation_policies.clone(),
        };

        // Add to existing group or create new
        card_map
            .entry(group_key.clone())
            .and_modify(|group| {
                // Update min price
                if avg_price_per_room < group.min_price {
                    group.min_price = avg_price_per_room;
                }
                group.room_types.push(variant.clone());
            })
            .or_insert_with(|| {
                // Initialize new group
                let mid_val = offer.mapped_room_id.clone();

                let card_name = if let Some(ref mid_s) = mid_val {
                    // Try to find static name
                    if let Some(static_list) = static_rooms {
                        if let Some(room_def) = static_list.iter().find(|r| r.room_id == *mid_s) {
                            room_def.room_name.clone()
                        } else {
                            formatted_name.clone()
                        }
                    } else {
                        formatted_name.clone()
                    }
                } else {
                    formatted_name.clone()
                };

                // Static Assets
                let mut images = Vec::new();
                let mut amenities = Vec::new();
                let mut bed_types = Vec::new();

                if let Some(ref mid_s) = mid_val {
                    if let Some(static_list) = static_rooms {
                        if let Some(room_def) = static_list.iter().find(|r| r.room_id == *mid_s) {
                            images = room_def.photos.clone();
                            amenities = room_def.amenities.clone();
                            bed_types = room_def.bed_types.clone();
                        }
                    }
                }

                DomainRoomGroup {
                    name: card_name,
                    mapped_room_id: offer.mapped_room_id.clone(),
                    min_price: avg_price_per_room, // Initial min
                    currency_code: currency_code,
                    images,
                    amenities,
                    bed_types,
                    room_types: vec![variant],
                }
            });
    }

    let mut groups: Vec<DomainRoomGroup> = card_map.into_values().collect();

    // Sort groups by min_price
    groups.sort_by(|a, b| {
        a.min_price
            .partial_cmp(&b.min_price)
            .unwrap_or(Ordering::Equal)
    });

    // Sort variants within groups by price
    for group in &mut groups {
        group.room_types.sort_by(|a, b| {
            a.price_per_room_excluding_taxes
                .partial_cmp(&b.price_per_room_excluding_taxes)
                .unwrap_or(Ordering::Equal)
        });
    }

    DomainGroupedRoomRates {
        room_groups: groups,
    }
}
