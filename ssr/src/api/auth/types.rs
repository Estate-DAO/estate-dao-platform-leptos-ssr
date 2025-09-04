use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct OidcUser {
    pub sub: String,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub name: Option<String>,
    pub picture: Option<String>,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct AppUrl {
    pub env_url: String,
    pub const_url: String,
}
