use cfg_if::cfg_if;

mod a00_search;
pub use a00_search::{search_hotel, HotelSearchRequest, HotelSearchResponse};

mod a01_hotel_info;
pub use a01_hotel_info::{HotelInfoRequest, HotelInfoResponse};

mod a02_get_room;
pub use a02_get_room::{HotelRoomRequest, RoomList};

// cfg_if! {
//     if #[cfg(feature = "ssr")] {
mod client;
pub use client::*;
// }
// }

mod types;
pub use types::*;

pub mod consts;
