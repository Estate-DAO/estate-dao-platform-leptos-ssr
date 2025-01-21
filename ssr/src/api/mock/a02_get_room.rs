use crate::api::a02_get_room::{HotelRoomResponse, RoomList};

// fake imports
use fake::{Dummy, Fake, Faker};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

impl Dummy<Faker> for HotelRoomResponse {
    fn dummy_with_rng<R: Rng + ?Sized>(_: &Faker, _rng: &mut R) -> Self {
        Self {
            status: 200,
            message: "Success".to_string(),
            room_list: Some(RoomList::dummy(&Faker)),
        }
    }
}
