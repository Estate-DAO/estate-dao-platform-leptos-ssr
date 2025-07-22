use crate::api::auth::{YralAccessTokenClaims, YralIdTokenClaims};
use crate::view_state_layer::user_auth_state::{DelegatedIdentityWire, UserInfo};
use anyhow::{anyhow, Result};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde_json::Value;

/// YRAL's public key for JWT verification (ES256)
const YRAL_PUBLIC_KEY_PEM: &str = r#"-----BEGIN PUBLIC KEY-----
MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEoqN3/0RNfrnrnYGxKBgy/qHnmITr
+6ucjxStx7tjA30QJZlWzo0atxmY8y9dUR+eKQI0SnbQds4xLEU8+JGm8Q==
-----END PUBLIC KEY-----"#;

/// Decode and verify YRAL access token
pub fn decode_access_token(token: &str) -> Result<YralAccessTokenClaims> {
    let key = DecodingKey::from_ec_pem(YRAL_PUBLIC_KEY_PEM.as_bytes())?;
    let validation = Validation::new(Algorithm::ES256);

    let token_data = decode::<YralAccessTokenClaims>(token, &key, &validation)?;
    Ok(token_data.claims)
}

/// Decode and verify YRAL ID token
pub fn decode_id_token(token: &str) -> Result<YralIdTokenClaims> {
    let key = DecodingKey::from_ec_pem(YRAL_PUBLIC_KEY_PEM.as_bytes())?;
    let validation = Validation::new(Algorithm::ES256);

    let token_data = decode::<YralIdTokenClaims>(token, &key, &validation)?;
    Ok(token_data.claims)
}

/// Extract user info from ID token
pub fn extract_user_info_from_id_token(id_token: &str) -> Result<UserInfo> {
    let claims = decode_id_token(id_token)?;

    // Parse delegated identity from the claims
    let delegated_identity = parse_delegated_identity(&claims.ext_delegated_identity)?;

    Ok(UserInfo {
        principal: claims.sub,
        is_anonymous: claims.ext_is_anonymous,
        delegated_identity: Some(delegated_identity),
        email: None, // YRAL doesn't provide email in basic flow
        name: None,  // YRAL doesn't provide name in basic flow
    })
}

/// Parse DelegatedIdentityWire from JSON value
fn parse_delegated_identity(value: &Value) -> Result<DelegatedIdentityWire> {
    let from_key = value
        .get("from_key")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow!("Missing or invalid from_key"))?
        .iter()
        .map(|v| v.as_u64().map(|n| n as u8))
        .collect::<Option<Vec<u8>>>()
        .ok_or_else(|| anyhow!("Invalid from_key format"))?;

    let to_secret = value
        .get("to_secret")
        .ok_or_else(|| anyhow!("Missing to_secret"))?
        .clone();

    let delegation_chain = value
        .get("delegation_chain")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow!("Missing or invalid delegation_chain"))?
        .iter()
        .cloned()
        .collect();

    Ok(DelegatedIdentityWire {
        from_key,
        to_secret,
        delegation_chain,
    })
}

/// Verify if a JWT token is expired
pub fn is_token_expired(token: &str) -> bool {
    match decode_access_token(token) {
        Ok(claims) => {
            let now = chrono::Utc::now().timestamp();
            now >= claims.exp
        }
        Err(_) => true, // If we can't decode, consider it expired
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delegated_identity_parsing() {
        let test_json = serde_json::json!({
            "from_key": [1, 2, 3, 4, 5],
            "to_secret": {
                "kty": "EC",
                "crv": "secp256k1"
            },
            "delegation_chain": [
                {"signature": "test"},
                {"delegation": "test2"}
            ]
        });

        let result = parse_delegated_identity(&test_json);
        assert!(result.is_ok());

        let identity = result.unwrap();
        assert_eq!(identity.from_key, vec![1, 2, 3, 4, 5]);
        assert_eq!(identity.delegation_chain.len(), 2);
    }
}
