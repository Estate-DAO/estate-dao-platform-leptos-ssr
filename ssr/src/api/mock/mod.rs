cfg_if::cfg_if! {

    if #[cfg(feature = "mock-provab")] {
        mod a02_get_room;
        pub use a02_get_room::*;

        mod a01_hotel_info;
        pub use a01_hotel_info::*;

        mod a00_search;
        pub use a00_search::*;
    }
}