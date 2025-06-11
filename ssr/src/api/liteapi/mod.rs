cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {
        pub mod client;
        pub use client::*;

        pub mod traits;
        pub use traits::*;


        pub use l00_hotel_search::search_hotels_from_destination;

        pub mod l00_hotel_search;
        pub use l00_hotel_search::{
            liteapi_hotel_search, LiteApiHotelResult, LiteApiHotelSearchRequest, LiteApiHotelSearchResponse,
        };

        pub use l01_get_hotel_info::liteapi_hotel_rates;
        pub mod l01_get_hotel_info;
        pub use l01_get_hotel_info::{
            LiteApiHotelData, LiteApiHotelRatesRequest, LiteApiHotelRatesResponse, LiteApiOccupancy,
            LiteApiRate, LiteApiRoomType,
        };

    }
}
