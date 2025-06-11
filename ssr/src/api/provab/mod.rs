pub mod a00_search;

pub mod a01_hotel_info;
pub mod a02_get_room;

pub mod a03_block_room;
mod a04_book_room;
mod a05_hotel_booking_detail;

pub use a03_block_room::{BlockRoomRequest, BlockRoomResponse};

pub use a04_book_room::{
    BookRoomRequest, BookRoomResponse, BookingDetails, BookingDetailsContainer, BookingStatus,
    FailureBookRoomResponse, PassengerDetail, PaxType, RoomDetail, SuccessBookRoomResponse,
};

mod client;
pub use client::{DeserializableInput, ProvabReq, ProvabReqMeta};

cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {

        pub use a00_search::{
            HotelResult, HotelSearchRequest, HotelSearchResponse, HotelSearchResult, Price, RoomGuest,
            Search,
        };

        pub use a01_hotel_info::{HotelInfoRequest, HotelInfoResponse};
        pub use a01_hotel_info::hotel_info;
        pub use a02_get_room::{ HotelRoomDetail, HotelRoomRequest, HotelRoomResponse};
        // pub use a02_get_room::get_room;
        // pub use a03_block_room::block_room;

        pub use a04_book_room::{
            // book_room,
            _default_passenger_age, create_backend_book_room_response,
            user_details_to_passenger_details,
        };


        pub use a05_hotel_booking_detail::{
            // get_hotel_booking_detail_from_travel_provider_v2,
        };

        pub use a05_hotel_booking_detail::{HotelBookingDetailRequest,HotelBookingDetailResponse};

        pub use client::*;

        mod retry;

}
}
