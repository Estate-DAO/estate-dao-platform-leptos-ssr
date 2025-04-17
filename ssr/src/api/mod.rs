use cfg_if::cfg_if;

mod a00_search;
pub use a00_search::{
    search_hotel, HotelResult, HotelSearchRequest, HotelSearchResponse, HotelSearchResult, Price,
    RoomGuest, Search,
};

mod a01_hotel_info;
pub use a01_hotel_info::{hotel_info, HotelInfoRequest, HotelInfoResponse};

mod a02_get_room;
pub use a02_get_room::{get_room, HotelRoomDetail, HotelRoomRequest, HotelRoomResponse};

mod a03_block_room;
pub use a03_block_room::{block_room, BlockRoomRequest, BlockRoomResponse};

mod retry;
pub use retry::RetryableRequest;

mod a04_book_room;
pub use a04_book_room::{
    book_room, BookRoomRequest, BookRoomResponse, BookingDetails, BookingDetailsContainer,
    BookingStatus, FailureBookRoomResponse, PassengerDetail, PaxType, RoomDetail,
    SuccessBookRoomResponse, _default_passenger_age, create_backend_book_room_response,
    user_details_to_passenger_details,
};

mod a05_hotel_booking_detail;
pub use a05_hotel_booking_detail::{
    get_hotel_booking_detail_from_travel_provider_v2, HotelBookingDetailRequest,
    HotelBookingDetailResponse,
};

// pub mod tower_client;
// cfg_if! {
//     if #[cfg(feature = "ssr")] {
mod client;
pub use client::*;
// }
// }

cfg_if! {
    if #[cfg(feature = "mock-provab")]{
pub mod mock;
    }
}

mod types;
pub use types::*;

pub mod consts;

pub mod payments;
pub use payments::ports::{FailureGetPaymentStatusResponse, SuccessGetPaymentStatusResponse};

pub mod canister;
