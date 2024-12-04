use crate::canister::backend::{
    BackendPaymentStatus, Booking, HotelDetails, HotelRoomDetails, PaymentDetails,
    SelectedDateRange, UserDetails,
};
use crate::state::search_state::{
    BlockRoomResults, ConfirmationResults, HotelInfoResults, SearchCtx, SearchListResults,
};
use crate::state::view_state::{BlockRoomCtx, HotelInfoCtx};
use crate::utils::admin::admin_canister;
use leptos::logging::log;
use leptos::*;

// impl Booking {
//     fn new(&self) -> Self {
//         let search_ctx: SearchCtx = expect_context();
//         let search_list_results: SearchListResults = expect_context();
//         let search_list_results = store_value(search_list_results); // Store it so we can clone inside closure
//         let hotel_info_results: HotelInfoResults = expect_context();
//         let hotel_info_ctx: HotelInfoCtx = expect_context();
//         let confirmation_results: ConfirmationResults = expect_context();
//         let block_room_ctx: BlockRoomCtx = expect_context();
//         let block_room_results_context: BlockRoomResults = expect_context();

//         let user_selected_hotel_room_details = HotelRoomDetails{
//             destination: search_ctx.destination.get(),
//             requested_payment_amount: f64,
//             date_range: search_ctx.date_range.get(),
//             room_details: Vec<RoomDetails>,
//             hotel_details: HotelDetails{hotel_code: hotel_info_ctx.hotel_code.get().unwrap(), hotel_name: hotel_info_ctx.selected_hotel_name.get()},
//         };
//         let guests = UserDetails::default();
//         let booking_id = ("".to_string(), "".to_string());
//         let book_room_status = None;
//         let payment_details =  PaymentDetails::default();
//     }
// }

impl Default for Booking {
    fn default() -> Self {
        Self {
            user_selected_hotel_room_details: HotelRoomDetails::default(),
            guests: UserDetails::default(),
            booking_id: ("".to_string(), "".to_string()),
            book_room_status: None,
            payment_details: PaymentDetails::default(),
        }
    }
}

impl Default for PaymentDetails {
    fn default() -> Self {
        Self {
            payment_status: BackendPaymentStatus::Unpaid(None), // Provide a default BackendPaymentStatus
            booking_id: (String::default(), String::default()),
        }
    }
}

impl Default for UserDetails {
    fn default() -> Self {
        Self {
            children: Vec::new(),
            adults: Vec::new(),
        }
    }
}

impl Default for HotelRoomDetails {
    fn default() -> Self {
        Self {
            destination: None,
            requested_payment_amount: 0.0,
            date_range: SelectedDateRange::default(), // Requires SelectedDateRange to also implement Default
            room_details: Vec::new(),
            hotel_details: HotelDetails::default(), // Requires HotelDetails to also implement Default
        }
    }
}

impl Default for HotelDetails {
    fn default() -> Self {
        Self {
            hotel_code: String::default(),
            hotel_name: String::default(),
        }
    }
}

impl Default for SelectedDateRange {
    fn default() -> Self {
        Self {
            start: (0, 0, 0),
            end: (0, 0, 0),
        }
    }
}

#[server(GreetBackend)]
pub async fn add_booking_backend(email: String, booking: Booking) -> Result<String, ServerFnError> {
    log!("\nHeres your booking{:?}", booking);
    let adm_cans = admin_canister();
    log!("\nHeres your booking now {:?}", booking);

    let backend_cans = adm_cans.backend_canister().await;
    log!("\nHeres your booking again now{:?}", booking);

    let result = backend_cans.add_booking(email, booking).await;

    log!("{result:?}");

    Ok("Got it!".into())
}
