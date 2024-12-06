use codee::string::JsonSerdeCodec;
use leptos::{Signal, WriteSignal};
use leptos_use::storage::use_local_storage;

use crate::api::{
    consts::{BOOK_ROOM_RESPONSE, PAYMENT_ID, PAYMENT_STATUS},
    BookRoomResponse,
};
use crate::{api::consts::BOOKING_ID, utils::app_reference::BookingId};

pub fn use_payment_store() -> (
    Signal<Option<u64>>,
    WriteSignal<Option<u64>>,
    impl Fn() + Clone,
) {
    use_local_storage::<Option<u64>, JsonSerdeCodec>(PAYMENT_ID)
}

pub fn use_payment_status_store() -> (
    Signal<Option<String>>,
    WriteSignal<Option<String>>,
    impl Fn() + Clone,
) {
    use_local_storage::<Option<String>, JsonSerdeCodec>(PAYMENT_STATUS)
}

pub fn use_booking_id_store() -> (
    Signal<Option<BookingId>>,
    WriteSignal<Option<BookingId>>,
    impl Fn() + Clone,
) {
    use_local_storage::<Option<BookingId>, JsonSerdeCodec>(BOOKING_ID)
}

pub fn use_booking_response_store() -> (
    Signal<Option<BookRoomResponse>>,
    WriteSignal<Option<BookRoomResponse>>,
    impl Fn() + Clone,
) {
    use_local_storage::<Option<BookRoomResponse>, JsonSerdeCodec>(BOOK_ROOM_RESPONSE)
}
