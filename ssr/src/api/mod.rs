use cfg_if::cfg_if;

mod a00_search;
pub use a00_search::{search_hotel, HotelSearchRequest, HotelSearchResponse, RoomGuest};

mod a01_hotel_info;
pub use a01_hotel_info::{hotel_info, HotelInfoRequest, HotelInfoResponse};

mod a02_get_room;
pub use a02_get_room::{get_room, HotelRoomRequest, HotelRoomResponse};

mod a03_block_room;
pub use a03_block_room::{BlockRoomRequest, BlockRoomResponse};

mod a04_book_room;
pub use a04_book_room::{book_room,
BookRoomRequest, BookRoomResponse, RoomDetail, 
PassengerDetail, PaxType, BookingStatus};

mod a05_hotel_booking_detail;
pub use a05_hotel_booking_detail::{HotelBookingDetailRequest, HotelBookingDetailResponse};

// cfg_if! {
//     if #[cfg(feature = "ssr")] {
mod client;
pub use client::*;
// }
// }

mod types;
pub use types::*;

pub mod consts;
