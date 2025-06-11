use crate::api::provab::a00_search::{
    HotelResult, HotelSearchResponse, HotelSearchResult, Price, Search,
};

cfg_if::cfg_if! {

    if #[cfg(feature = "mock-provab")]{

        // fake imports
        use fake::{Dummy, Fake, Faker};
        use rand::rngs::StdRng;
        use rand::{Rng, SeedableRng};

        impl Dummy<Faker> for HotelSearchResponse {
            fn dummy_with_rng<R: Rng + ?Sized>(f: &Faker, rng: &mut R) -> Self {
                let mut search_result = HotelSearchResult::dummy(f);
                search_result.hotel_results = (0..10).map(|_| HotelResult::dummy(f)).collect();

                let search = Search {
                    hotel_search_result: search_result,
                };

                Self {
                    status: 200,
                    message: "Success".to_string(),
                    search: Some(search),
                }
            }
        }

        impl Dummy<Faker> for HotelResult {
            fn dummy_with_rng<R: Rng + ?Sized>(f: &Faker, rng: &mut R) -> Self {
                let price = rng.gen_range(100.0..1000.0);

                Self {
                    hotel_code: format!("HOTEL{}", rng.gen_range(1000..9999)),
                    hotel_name: "DUMMY HOTEL NAME".into(),
                    hotel_category: "Hotel".to_string(),
                    star_rating: rng.gen_range(1..6),
                    price: Price {
                        currency_code: "USD".to_string(),
                        room_price: price,
                    },
                    hotel_picture: "/img/home.webp".to_string(),
                    result_token: format!("TOKEN{}", rng.gen_range(1000..9999)),
                }
            }
        }

    }


}
