cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {
        pub mod hotel_service;
        pub mod places_service;
        pub use hotel_service::HotelService;
        pub use places_service::PlaceService;
    }
}

// BookingService is available on both client and server since it handles client-side calls to server functions
pub mod booking_service;
pub use booking_service::BookingService;

pub use filter_types::{DomainSortDirection, DomainSortField, UISearchFilters, UISortOptions};
pub mod filter_types;
