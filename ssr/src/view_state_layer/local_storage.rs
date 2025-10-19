use ::codee::string::{JsonSerdeCodec, OptionCodec};
use leptos::prelude::*;
use leptos_use::storage::{use_local_storage, use_local_storage_with_options, UseStorageOptions};

use crate::api::{
    consts::{BOOK_ROOM_RESPONSE, PAYMENT_ID, PAYMENT_STATUS},
    // BookRoomResponse,
};
use crate::{api::consts::BOOKING_ID, utils::app_reference::BookingId};

pub fn use_payment_store() -> (
    Signal<Option<u64>>,
    WriteSignal<Option<u64>>,
    impl Fn() + Clone,
) {
    // use_local_storage::<Option<u64>, JsonSerdeCodec>(PAYMENT_ID)
    use_local_storage_with_options::<Option<u64>, OptionCodec<JsonSerdeCodec>>(
        PAYMENT_ID,
        UseStorageOptions::default().delay_during_hydration(true),
    )
}

pub fn use_payment_status_store() -> (
    Signal<Option<String>>,
    WriteSignal<Option<String>>,
    impl Fn() + Clone,
) {
    // use_local_storage::<Option<String>, JsonSerdeCodec>(PAYMENT_STATUS)
    use_local_storage_with_options::<Option<String>, OptionCodec<JsonSerdeCodec>>(
        PAYMENT_STATUS,
        UseStorageOptions::default().delay_during_hydration(true),
    )
}

pub fn use_booking_id_store() -> (
    Signal<Option<BookingId>>,
    WriteSignal<Option<BookingId>>,
    impl Fn() + Clone,
) {
    use_local_storage_with_options::<Option<BookingId>, OptionCodec<JsonSerdeCodec>>(
        BOOKING_ID,
        UseStorageOptions::default().delay_during_hydration(true),
    )
}

// pub fn use_booking_response_store() -> (
//     Signal<Option<BookRoomResponse>>,
//     WriteSignal<Option<BookRoomResponse>>,
//     impl Fn() + Clone,
// ) {
//     // use_local_storage::<Option<BookRoomResponse>, JsonSerdeCodec>(BOOK_ROOM_RESPONSE)
//     use_local_storage_with_options::<Option<BookRoomResponse>, JsonSerdeCodec>(
//         BOOK_ROOM_RESPONSE,
//         UseStorageOptions::default().delay_during_hydration(true),
//     )
// }
