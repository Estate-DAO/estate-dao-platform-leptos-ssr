use crate::api::{
    a01_hotel_info::{FirstRoomDetails, HotelDetailsLevel1, HotelDetailsLevel2},
    HotelInfoResponse,
};

cfg_if::cfg_if! {

    if #[cfg(feature = "mock-provab")]{

        // fake imports
        use fake::{Dummy, Fake, Faker};
        use rand::rngs::StdRng;
        use rand::{Rng, SeedableRng};


        impl Dummy<Faker> for HotelDetailsLevel2 {
            fn dummy_with_rng<R: Rng + ?Sized>(_: &Faker, _rng: &mut R) -> Self {
                Self {
                    checkin: "2024-07-20".to_string(),
                    checkout: "2024-07-21".to_string(),
                    hotel_name: "Test Hotel".to_string(),
                    hotel_code: "TH123".to_string(),
                    star_rating: 4,
                    description: "A nice test hotel".to_string(),
                    hotel_facilities: vec!["pool".to_string(), "gym".to_string()],
                    address: "123 Main St".to_string(),
                    images: vec![
                        "/img/home.webp".to_string(),
                        "/img/home.webp".to_string(),
                        "/img/home.webp".to_string(),
                        "/img/home.webp".to_string(),
                        "/img/home.webp".to_string(),
                        "/img/home.webp".to_string(),
                        "/img/home.webp".to_string(),
                        "/img/home.webp".to_string(),
                        "/img/home.webp".to_string()
                    ],
                    first_room_details: FirstRoomDetails::dummy(&Faker),
                    amenities: vec!["wifi".to_string(), "parking".to_string()],
                }
            }
            // fn dummy(config: &Faker) -> Self {
            //     let mut r = rand::thread_rng();
            //     Dummy::<Faker>::dummy_with_rng(config, &mut r)
            // }
        }


        impl Dummy<Faker> for HotelInfoResponse {
            fn dummy_with_rng<R: Rng + ?Sized>(_: &Faker, _rng: &mut R) -> Self {
                Self {
                    status: 200,
                    message: "Success".to_string(),
                    hotel_details: Some(HotelDetailsLevel1::dummy(&Faker)),
                }
            }
        }
    }


}
