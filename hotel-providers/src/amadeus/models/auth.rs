use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AmadeusAuthResponse {
    pub access_token: String,
    pub expires_in: u64,
    pub token_type: String,
}
