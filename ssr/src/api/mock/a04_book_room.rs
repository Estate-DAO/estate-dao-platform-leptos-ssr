use fake::{Dummy, Fake, Faker};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::api::a04_book_room::{
    BookRoomResponse, FailureBookRoomResponse, SuccessBookRoomResponse,
};
use crate::api::mock::mock_utils::MockableResponse;

impl MockableResponse for BookRoomResponse {
    fn should_simulate_failure() -> bool {
        // cfg_if::cfg_if! {
        //     if #[cfg(feature = "mock-book-room-fail")] {
        //         true
        //     } else {
        //         false
        //     }
        // }
        false
    }

    fn generate_failure_response() -> Self {
        BookRoomResponse::Failure(FailureBookRoomResponse {
            status: 400,
            message: "Room booking failed. Please try again.".to_string(),
        })
    }

    fn generate_success_response() -> Self {
        BookRoomResponse::Success(SuccessBookRoomResponse {
            status: crate::api::a04_book_room::BookingStatus::Confirmed,
            message: "Booking successful".to_string(),
            commit_booking: crate::api::a04_book_room::BookingDetailsContainer {
                booking_details: crate::api::a04_book_room::BookingDetails::dummy(&Faker),
            },
        })
    }
}
