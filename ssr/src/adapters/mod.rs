cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {
        pub mod liteapi_adapter;
        pub use liteapi_adapter::LiteApiAdapter;

        pub mod provider_bridge;
        pub use provider_bridge::LiteApiProviderBridge;
    }
}
