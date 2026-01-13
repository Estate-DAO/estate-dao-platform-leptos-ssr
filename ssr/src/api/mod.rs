use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "ssr")] {
        pub mod api_client;
    }
}

pub mod client_side_api;

// pub mod provab;
// pub use provab::{DeserializableInput, ProvabReq, ProvabReqMeta};

cfg_if! {
    if #[cfg(feature = "mock-provab")]{
    // pub mod mock;

    }
}

// fake imports
cfg_if::cfg_if! {
    // todo: fix feature flags
    if #[cfg(feature = "mock-provab")] {
        // fake imports
        use fake::{Dummy, Fake, Faker};
        use rand::{rngs::StdRng, Rng, SeedableRng};

    /// Trait for generating mock responses with optional failure simulation.
    /// Implement this trait to provide custom failure behavior for specific response types.
    pub trait MockableResponse: Sized {
        /// Returns true if this response type should simulate a failure based on feature flags.
        fn should_simulate_failure() -> bool {
            false
        }

        /// Generate a failure response.
        fn generate_failure_response() -> Self;

        /// Generate a success response.
        fn generate_success_response() -> Self;
    }

    /// Helper trait to provide default mock response generation.
    /// This is automatically implemented for all types that implement MockableResponse.
    pub trait MockResponseGenerator: MockableResponse {
        /// Generate a mock response with given probability of failure.
        /// Uses a seeded RNG to ensure consistent behavior in tests.
        fn generate_mock_response(failure_probability: f64) -> Self {
            let mut rng = StdRng::seed_from_u64(42);
            if Self::should_simulate_failure() || rng.gen_bool(failure_probability) {
                Self::generate_failure_response()
            } else {
                Self::generate_success_response()
            }
        }
    }

    // Implement MockResponseGenerator for all types that implement MockableResponse
    impl<T: MockableResponse> MockResponseGenerator for T {}

    // Blanket implementation for all types that implement Dummy<Faker>
    // This means any type that can be faked automatically gets mock response support
    impl<T: Dummy<Faker>> MockableResponse for T {
        fn generate_failure_response() -> Self {
            Faker.fake()
        }
        fn generate_success_response() -> Self {
            Faker.fake()
        }
    }
}
 }

mod types;
pub use types::*;

pub mod consts;

pub mod payments;
pub use payments::ports::{FailureGetPaymentStatusResponse, SuccessGetPaymentStatusResponse};

pub mod canister;

pub mod auth;

cfg_if! {
    if #[cfg(feature = "ssr")] {
        // pub mod cities_api_provider;
        // pub use cities_api_provider::SsrCityApiProvider;
    }
}
