use crate::component::{Destination, GuestSelection};
use crate::log;
use crate::{component::SelectedDateRange, state::search_state::SearchCtx};
use std::collections::HashMap;

cfg_if::cfg_if! {
    if #[cfg(feature = "mock-provab")] {
        // fake imports
        use fake::{Dummy, Fake, Faker};
        use rand::rngs::StdRng;
        use rand::SeedableRng;
    }
}
// These are the types that should be used for representing state in the UI and managing that state.

//
// SEARCH CORE TYPES
//
#[derive(Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct RoomGuestCore {
    pub no_of_adults: u32,
    pub no_of_child: u32,
    pub child_age: Option<Vec<String>>,
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct PriceCore {
    //
    // published_price: f64,
    //
    // published_price_rounded_off: u64,
    //
    // offered_price: f64,
    //
    // offered_price_rounded_off: u64,
    pub room_price: f64,
    //
    // tax: f64,
    //
    // extra_guest_charge: f64,
    //
    // child_charge: f64,
    //
    // other_charges: f64,
    //
    // discount: f64,
    //
    // agent_commission: f64,
    //
    // agent_mark_up: f64,
    //
    // service_tax: f64,
    //
    // tds: f64,
    //
    // room_price_wo_gst: f64,
    //
    // gst_price: f64,
    pub currency_code: String,
}

#[derive(Clone, Debug)]
pub struct HotelResultCore {
    // result_index: i32,
    pub hotel_code: String,
    pub hotel_name: String,
    pub hotel_category: String,
    pub star_rating: u8,
    // hotel_description: String,
    // hotel_policy: String,
    // hotel_promotion_content: String,
    // hotel_promotion: i32,
    pub price: PriceCore,
    pub hotel_picture: String,
    // image_order: i32,
    // hotel_address: String,
    // hotel_address: String,
    //
    // hotel_contact_no: String,
    //
    // hotel_map: String,
    //
    // latitude: String,
    //
    // longitude: String,
    // hotel_location: String,
    // supplier_price: String,
    // room_details: Vec<String>,
    pub result_token: String,
    // hotel_amenities: Vec<String>,
    // free_cancel_date: String,
    // trip_adv_url: String,
    // trip_rating: f64,
    // trip_reviews: u64,
    // trip_review_url: String,
    // web_reviews_url: String,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct HotelSearchResultCore {
    pub hotel_results: Vec<HotelResultCore>,
}

#[derive(Clone, Debug)]
pub struct SearchCore {
    pub hotel_search_result: HotelSearchResultCore,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct HotelSearchRequestCore {
    check_in_date: String,
    no_of_nights: u32,
    country_code: String,
    city_id: u32,
    guest_nationality: String,
    no_of_rooms: u32,
    room_guests: Vec<RoomGuestCore>,
}

#[derive(Clone, Debug)]
pub struct HotelSearchResponseCore {
    pub status: i32,

    pub message: String,

    pub search: Option<SearchCore>,
}

//
// Hotel Details  CORE TYPES
//

#[derive(Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct FirstRoomDetails {
    price: Price,
    //
    // cancellation_policy: Vec<CancellationPolicy>,
    room_name: String,

    room_data: RoomData,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct Price {
    published_price: f64,

    published_price_rounded_off: f64,

    offered_price: f64,

    offered_price_rounded_off: f64,

    room_price: f64,

    tax: f64,

    extra_guest_charge: f64,

    child_charge: f64,

    other_charges: f64,
    //
    // discount: f64,
    //
    // agent_commission: f64,
    //
    // agent_mark_up: f64,
    //
    // service_tax: f64,
    //
    // tds: f64,
    //
    // room_price_wo_gst: f64,
    //
    // gst_price: f64,
    currency_code: String,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct CancellationPolicy {
    charge: f64,

    charge_type: i32,

    currency: String,

    from_date: String,

    to_date: String,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct RoomData {
    room_unique_id: String,

    rate_key: String,
    //
    // group_code: String,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct HotelDetailsLevel1 {
    hotel_info_result: HotelInfoResult,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct HotelInfoResult {
    hotel_details: HotelDetailsLevel2,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "mock-provab", derive(Dummy))]
pub struct HotelInfoRequest {
    pub token: String,
}

#[derive(Clone, Debug)]
// #[display("Status: {}, Message: {}", status, message)]
pub struct HotelInfoResponse {
    pub status: i32,

    pub message: String,

    pub hotel_details: Option<HotelDetailsLevel1>,
}

#[derive(Clone, Debug)]
pub struct HotelDetailsLevel2 {
    pub checkin: String,
    pub checkout: String,

    pub hotel_name: String,

    pub hotel_code: String,

    pub star_rating: i32,

    pub description: String,
    //
    // attractions: Vec<String>,
    //
    // hotel_policy: String,
    pub hotel_facilities: Vec<String>,

    pub address: String,
    //
    // latitude: f64,
    //
    // longitude: f64,
    pub images: Vec<String>,
    pub first_room_details: FirstRoomDetails,
    // first_rm_cancel_date: String,
    // cancel_date: String,
    pub amenities: Vec<String>,
    // trip_adv_url: String,
    // trip_rating: String,
}

// <!-- Implementation for HotelSearchRequestCore -->
impl Default for HotelSearchRequestCore {
    fn default() -> Self {
        Self {
            check_in_date: "11-11-2024".into(),
            no_of_nights: 1,
            country_code: "IN".into(),
            city_id: 1254,
            guest_nationality: "IN".into(),
            no_of_rooms: 1,
            room_guests: vec![RoomGuestCore {
                no_of_adults: 1,
                no_of_child: 0,
                child_age: None,
            }],
        }
    }
}

impl HotelSearchRequestCore {
    fn get_room_guests_from_ctx(search_ctx: &SearchCtx) -> Vec<RoomGuestCore> {
        let guest_selection = search_ctx.guests;

        let no_of_adults = guest_selection.get_untracked().adults.get_untracked();
        let no_of_child = guest_selection.get_untracked().children.get_untracked();
        let children_ages: Vec<String> = guest_selection
            .get_untracked()
            .children_ages
            .get_untracked()
            .iter()
            .map(|age| age.to_string())
            .collect();

        let child_age = if no_of_child > 0 {
            Some(children_ages)
        } else {
            None
        };

        vec![RoomGuestCore {
            no_of_adults,
            no_of_child,
            child_age,
        }]
    }
}

impl From<SearchCtx> for HotelSearchRequestCore {
    fn from(ctx: SearchCtx) -> Self {
        let check_in_date = SelectedDateRange::format_date(ctx.date_range.get_untracked().start);
        let no_of_nights = ctx.date_range.get_untracked().no_of_nights();
        let request = HotelSearchRequestCore {
            check_in_date,
            no_of_nights,
            room_guests: Self::get_room_guests_from_ctx(&ctx),
            country_code: Destination::get_country_code(&ctx),
            city_id: Destination::get_city_id(&ctx),
            ..Default::default()
        };

        log!("HotelSearchRequestCore: {request:?}");

        request
    }
}

// <!-- Implementation for HotelSearchResponseCore -->
impl HotelSearchResponseCore {
    pub fn get_results_token_map(&self) -> HashMap<String, String> {
        let mut hotel_map = HashMap::new();

        if let Some(search) = self.search.clone() {
            for hotel in search.hotel_search_result.hotel_results {
                hotel_map.insert(hotel.hotel_code, hotel.result_token);
            }
        }

        hotel_map
    }

    pub fn hotel_results(&self) -> Vec<HotelResultCore> {
        self.search
            .clone()
            .map_or_else(Vec::new, |search| search.hotel_search_result.hotel_results)
    }
}

// <!-- Implementation for HotelInfoResponse -->
impl HotelInfoResponse {
    pub fn get_address(&self) -> String {
        self.hotel_details.as_ref().map_or_else(
            || "".to_owned(),
            |details| details.hotel_info_result.hotel_details.address.clone(),
        )
    }

    pub fn get_description(&self) -> String {
        self.hotel_details.as_ref().map_or_else(
            || "".to_string(),
            |details| details.hotel_info_result.hotel_details.description.clone(),
        )
    }

    pub fn get_amenities(&self) -> Vec<String> {
        self.hotel_details.as_ref().map_or_else(
            || vec![],
            |details| details.hotel_info_result.hotel_details.amenities.clone(),
        )
    }

    pub fn get_images(&self) -> Vec<String> {
        self.hotel_details.as_ref().map_or_else(
            || vec![],
            |details| details.hotel_info_result.hotel_details.images.clone(),
        )
    }

    pub fn get_hotel_name(&self) -> String {
        self.hotel_details.as_ref().map_or_else(
            || "".to_owned(),
            |details| details.hotel_info_result.hotel_details.hotel_name.clone(),
        )
    }

    pub fn get_star_rating(&self) -> i32 {
        self.hotel_details.as_ref().map_or_else(
            || 0,
            |details| details.hotel_info_result.hotel_details.star_rating,
        )
    }

    pub fn get_room_price(&self) -> f64 {
        self.hotel_details.as_ref().map_or_else(
            || 0.0,
            |details| {
                details
                    .hotel_info_result
                    .hotel_details
                    .first_room_details
                    .price
                    .room_price
            },
        )
    }

    pub fn get_location(&self) -> String {
        self.hotel_details.as_ref().map_or_else(
            || "".to_owned(),
            |details| details.hotel_info_result.hotel_details.address.clone(),
        )
    }
}
