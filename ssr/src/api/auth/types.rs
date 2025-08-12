use serde::{Deserialize, Serialize};
use yral_types::delegated_identity::DelegatedIdentityWire;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct YralAuthAdditionalTokenClaims {
    pub ext_is_anonymous: bool,
    pub ext_delegated_identity: DelegatedIdentityWire,
}

pub type YralAuthMessage = Result<NewIdentity, String>;

cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {
        use openidconnect::{
            core::{
                CoreAuthDisplay, CoreAuthPrompt, CoreAuthenticationFlow, CoreErrorResponseType,
                CoreGenderClaim, CoreIdTokenVerifier, CoreJsonWebKey, CoreJsonWebKeyType,
                CoreJsonWebKeyUse, CoreJweContentEncryptionAlgorithm, CoreJwsSigningAlgorithm,
                CoreRevocableToken, CoreRevocationErrorResponse, CoreTokenIntrospectionResponse,
                CoreTokenType,
            },
            reqwest::async_http_client,
            AdditionalClaims, AuthorizationCode, CsrfToken, EmptyExtraTokenFields, IdTokenFields,
            LoginHint, Nonce, OAuth2TokenResponse, PkceCodeChallenge, PkceCodeVerifier, Scope,
            StandardErrorResponse, StandardTokenResponse,
        };

        pub type YralOAuthClient = openidconnect::Client<
            YralAuthAdditionalTokenClaims,
            CoreAuthDisplay,
            CoreGenderClaim,
            CoreJweContentEncryptionAlgorithm,
            CoreJwsSigningAlgorithm,
            CoreJsonWebKeyType,
            CoreJsonWebKeyUse,
            CoreJsonWebKey,
            CoreAuthPrompt,
            StandardErrorResponse<CoreErrorResponseType>,
            StandardTokenResponse<
                IdTokenFields<
                    YralAuthAdditionalTokenClaims,
                    EmptyExtraTokenFields,
                    CoreGenderClaim,
                    CoreJweContentEncryptionAlgorithm,
                    CoreJwsSigningAlgorithm,
                    CoreJsonWebKeyType,
                >,
                CoreTokenType,
            >,
            CoreTokenType,
            CoreTokenIntrospectionResponse,
            CoreRevocableToken,
            CoreRevocationErrorResponse,
        >;
    impl AdditionalClaims for YralAuthAdditionalTokenClaims {}

    }
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct YralTokenResponse {
//     pub access_token: String,
//     pub refresh_token: String,
//     pub expires_in: u32,
//     pub token_type: String,
// }

// /// Check if a JWT token is expired
// pub fn is_token_expired(token: &str) -> bool {
//     use jsonwebtoken::{decode, decode_header, DecodingKey, Validation, Algorithm};
//     use crate::api::consts::yral_auth::YRAL_AUTH_TRUSTED_KEY;

//     // Decode the token header to get the algorithm
//     let header = match decode_header(token) {
//         Ok(header) => header,
//         Err(_) => return true, // If we can't decode the header, consider it expired
//     };

//     // Set up validation
//     let mut validation = Validation::new(Algorithm::ES256);
//     validation.validate_exp = true;

//     // Try to decode and validate the token
//     match decode::<YralAuthAdditionalTokenClaims>(
//         token,
//         &YRAL_AUTH_TRUSTED_KEY,
//         &validation,
//     ) {
//         Ok(_) => false, // Token is valid and not expired
//         Err(_) => true, // Token is invalid or expired
//     }
// }

#[derive(Debug, Clone, Serialize, Default, Deserialize, PartialEq)]
pub enum LoginProvider {
    #[default]
    Google,
    Apple,
    Any,
}
pub const PKCE_VERIFIER_COOKIE: &str = "google-pkce-verifier";
pub const CSRF_TOKEN_COOKIE: &str = "google-csrf-token";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProviderKind {
    YralAuth,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NewIdentity {
    pub id_wire: DelegatedIdentityWire,
    pub fallback_username: Option<String>,
    pub email: Option<String>,
}

impl NewIdentity {
    pub fn new_without_username(id: DelegatedIdentityWire) -> Self {
        Self {
            id_wire: id,
            fallback_username: None,
            email: None,
        }
    }
}
