// export root page as flattened
mod root;
pub use root::*;

mod search_params;
pub use search_params::*;

mod hotel_details;
pub use hotel_details::*;

mod hotel_details_v1;
pub use hotel_details_v1::*;

mod hotel_list;
pub use hotel_list::*;

mod hotel_list_params;
pub use hotel_list_params::*;

mod hotel_details_params;
pub use hotel_details_params::*;

mod block_room;
pub use block_room::*;

mod block_room_v1;
pub use block_room_v1::*;

mod confirm_booking;
pub use confirm_booking::*;

mod cancel_page;
pub use cancel_page::*;

mod confirmation_page_v1;
pub use confirmation_page_v1::*;

mod confirmation_page_v2;
pub use confirmation_page_v2::*;

mod input_group_mobile;
pub use input_group_mobile::*;

mod input_group_container;
pub use input_group_container::*;

mod admin_panel;
pub use admin_panel::*;

mod admin_edit_panel;
pub use admin_edit_panel::*;

mod my_bookings;
pub use my_bookings::*;

mod my_accounts;
pub use my_accounts::*;

mod wishlist;
pub use wishlist::*;

mod about_us;
pub use about_us::*;
