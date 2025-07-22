use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct YralTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub refresh_token: String,
    pub id_token: String,
    pub scope: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct YralAccessTokenClaims {
    pub aud: String, // Client ID
    pub exp: i64,    // Expiry timestamp
    pub iat: i64,    // Issued at timestamp
    pub iss: String, // Issuer
    pub sub: String, // Subject (user principal)
    pub nonce: Option<String>,
    pub ext_is_anonymous: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct YralIdTokenClaims {
    pub aud: String, // Client ID
    pub exp: i64,    // Expiry timestamp
    pub iat: i64,    // Issued at timestamp
    pub iss: String, // Issuer
    pub sub: String, // Subject (user principal)
    pub nonce: Option<String>,
    pub ext_is_anonymous: bool,
    pub ext_delegated_identity: serde_json::Value, // DelegatedIdentityWire
}

#[derive(Debug, Serialize)]
pub struct YralTokenRequest {
    pub grant_type: String,
    pub code: String,
    pub redirect_uri: String,
    pub client_id: String,
    pub client_secret: String,
    pub code_verifier: String,
}

#[derive(Debug, Serialize)]
pub struct YralRefreshTokenRequest {
    pub grant_type: String,
    pub refresh_token: String,
    pub client_id: String,
    pub client_secret: String,
}

/// Error response from YRAL OAuth endpoints
#[derive(Debug, Deserialize)]
pub struct YralErrorResponse {
    pub error: String,
    pub error_description: Option<String>,
}
