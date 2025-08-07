cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {
        pub mod auth_url;
    }
}
pub mod types;
