use crate::{
    api::BookRoomResponse,
    component::{Divider, Navbar, SpinnerFit, SpinnerGray},
    page::{
        block_room, confirm_booking::booking_handler::read_booking_details_from_local_storage,
        BookRoomHandler, BookingHandler, PaymentHandler,
    },
    state::{
        search_state::{ConfirmationResults, HotelInfoResults, SearchCtx},
        view_state::{BlockRoomCtx, HotelInfoCtx},
    },
};
use chrono::NaiveDate;
use leptos::*;
use log::log;

#[component]
pub fn ConfirmationPage() -> impl IntoView {
    view! {
        This is SSE Booking!
    }
}
