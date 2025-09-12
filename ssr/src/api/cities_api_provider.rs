// use crate::api::liteapi::{get_all_cities, AllCitiesIterator};
// use crate::api::ApiError;
// use bg_ractor::{City, CityApiProvider, CityIterator, Country, CountryCitiesResult};
// use std::future::Future;
// use std::pin::Pin;
// use tracing::instrument;

// /// Adapter that wraps the SSR AllCitiesIterator to work with bg-ractor's CityIterator trait
// pub struct SsrCityIteratorAdapter {
//     inner: AllCitiesIterator,
// }

// impl CityIterator for SsrCityIteratorAdapter {
//     fn next(&mut self) -> Pin<Box<dyn Future<Output = Option<CountryCitiesResult>> + Send + '_>> {
//         Box::pin(async move {
//             let span = tracing::debug_span!(
//                 "ssr_city_iterator_next",
//                 api_provider = "ssr_city_iterator",
//                 operation = "next_country_cities",
//                 service.name = "estate_fe_ssr",
//                 component = "cities_api_provider"
//             );
//             let _guard = span.enter();
//             let result = self.inner.next().await?;
//             drop(_guard);

//             match result {
//                 Ok((country, cities)) => {
//                     // Convert from SSR types to bg-ractor types
//                     let bg_country = Country {
//                         code: country.code,
//                         name: country.name,
//                     };
//                     let bg_cities: Vec<City> = cities
//                         .into_iter()
//                         .map(|city| City { city: city.city })
//                         .collect();

//                     Some(Ok((bg_country, bg_cities)))
//                 }
//                 Err((country, error)) => {
//                     let bg_country = Country {
//                         code: country.code,
//                         name: country.name,
//                     };
//                     Some(Err((
//                         bg_country,
//                         Box::new(error) as Box<dyn std::error::Error + Send + Sync>,
//                     )))
//                 }
//             }
//         })
//     }

//     fn progress(&self) -> (usize, usize) {
//         self.inner.progress()
//     }
// }

// /// API provider implementation for the SSR crate
// #[derive(Clone)]
// pub struct SsrCityApiProvider;

// #[async_trait::async_trait]
// impl CityApiProvider for SsrCityApiProvider {
//     type Iterator = SsrCityIteratorAdapter;
//     type Error = ApiError;

//     #[instrument(skip(self), fields(
//         api_provider = "ssr_city_api_provider",
//         operation = "get_all_cities",
//         service.name = "estate_fe_ssr",
//         component = "cities_api_provider"
//     ))]
//     async fn get_all_cities(&self) -> Result<Self::Iterator, Self::Error> {
//         let iterator = get_all_cities().await?;
//         Ok(SsrCityIteratorAdapter { inner: iterator })
//     }
// }

// impl SsrCityApiProvider {
//     pub fn new() -> Self {
//         Self
//     }
// }

// impl Default for SsrCityApiProvider {
//     fn default() -> Self {
//         Self::new()
//     }
// }
