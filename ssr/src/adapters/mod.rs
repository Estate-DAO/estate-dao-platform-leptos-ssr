cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {
        pub mod provab_adapter;
        pub use provab_adapter::ProvabAdapter;
        pub mod liteapi_adapter;
        pub use liteapi_adapter::LiteApiAdapter;
    }
}
