use codee::string::JsonSerdeCodec;
use leptos::{Signal, WriteSignal};
use leptos_use::storage::use_local_storage;

use crate::{
    api::consts::{BOOKING_ID, PAYMENT_ID, PAYMENT_STATUS},
    utils::app_reference::BookingId,
};

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

// let (state, set_state, _) = use_local_storage::<BookingId, JsonSerdeCodec>("booking_id");
