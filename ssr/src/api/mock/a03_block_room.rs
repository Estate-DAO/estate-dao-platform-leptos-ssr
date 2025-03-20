use crate::api::a03_block_room::{
    BlockRoomContainer, BlockRoomResponse, BlockRoomResult, FailureBlockRoomResponse,
    HotelRoomDetail, SuccessBlockRoomResponse,
};
use crate::api::mock::mock_utils::MockableResponse;
use fake::{Dummy, Fake, Faker};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

impl MockableResponse for BlockRoomResponse {
    fn should_simulate_failure() -> bool {
        cfg_if::cfg_if! {
            if #[cfg(feature = "mock-block-room-fail")] {
                true
            } else {
                false
            }
        }
    }

    fn generate_failure_response() -> Self {
        BlockRoomResponse::Failure(FailureBlockRoomResponse {
            status: 400,
            message: Some("Room is no longer available".to_string()),
        })
    }

    fn generate_success_response() -> Self {
        BlockRoomResponse::Success(SuccessBlockRoomResponse {
            status: 200,
            message: Some("Success".to_string()),
            block_room: BlockRoomContainer {
                block_room_result: BlockRoomResult {
                    is_price_changed: false,
                    is_cancellation_policy_changed: false,
                    block_room_id: "TEST-BLOCK_ROOM_ID-123".to_string(),
                    hotel_rooms_details: vec![HotelRoomDetail::dummy(&Faker)],
                },
            },
        })
    }
}
