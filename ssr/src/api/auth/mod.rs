cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {
        pub mod auth_url;
    }
}
pub mod auth_state;
pub mod canisters;
pub mod extract_identity_impl;
pub mod types;
