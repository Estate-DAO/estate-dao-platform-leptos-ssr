use cfg_if::cfg_if;

pub mod hotel_provider_port;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderNames {
    Provab,
    LiteApi,
}

cfg_if! {
    if #[cfg(feature = "ssr")]
    {
        pub mod traits;
    }
}
