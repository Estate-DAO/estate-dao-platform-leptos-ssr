cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {
        pub mod hotel_service;
        pub use hotel_service::HotelService;
    }
}

pub use filter_types::{DomainSortDirection, DomainSortField, UISearchFilters, UISortOptions};
pub mod filter_types;
