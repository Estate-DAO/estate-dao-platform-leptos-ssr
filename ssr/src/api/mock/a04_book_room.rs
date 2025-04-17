use fake::{Dummy, Fake, Faker};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::api::a04_book_room::{
    BookRoomResponse, FailureBookRoomResponse, SuccessBookRoomResponse,
};
use crate::api::mock::mock_utils::MockableResponse;
use crate::api::{BookingDetails, BookingDetailsContainer, BookingStatus};

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
        // randomly pick one of the three booking statuses
        let mut rng = rand::thread_rng();

        let statuses = ["BOOKING_CONFIRMED", "BOOKING_FAILED"];
        // let statuses = ["BOOKING_CONFIRMED", "BOOKING_FAILED", "BOOKING_HOLD"];
        let idx = rng.gen_range(0..statuses.len());
        generate_success_response_with_status(statuses[idx])
    }
}

/// Helper to generate a Success response with a specific booking_status string.
/// booking_status should be one of: "BOOKING_CONFIRMED", "BOOKING_FAILED", "BOOKING_HOLD"
pub fn generate_success_response_with_status(status: &str) -> BookRoomResponse {
    BookRoomResponse::Success(SuccessBookRoomResponse {
        status: BookingStatus::Confirmed, // You can adjust this if needed
        message: "Booking successful".to_string(),
        commit_booking: BookingDetailsContainer {
            booking_details: BookingDetails {
                travelomatrix_id: "TID123".to_string(),
                booking_ref_no: "BRN456".to_string(),
                confirmation_no: "CONF789".to_string(),
                booking_status: status.to_string(),
            },
        },
    })
}
