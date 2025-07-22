use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use sha2::{Digest, Sha256};

/// Generate PKCE code verifier and challenge for OAuth2 flow
/// Returns (code_verifier, code_challenge)
pub fn generate_pkce_challenge() -> (String, String) {
    // Generate code verifier (43-128 chars, URL-safe base64)
    let random_bytes = uuid::Uuid::new_v4().as_bytes().to_vec();
    let mut extended_bytes = random_bytes;

    // Extend to make it longer (closer to 128 chars when base64 encoded)
    for _ in 0..3 {
        extended_bytes.extend_from_slice(uuid::Uuid::new_v4().as_bytes());
    }

    let code_verifier = URL_SAFE_NO_PAD
        .encode(&extended_bytes)
        .chars()
        .take(128)
        .collect::<String>();

    // Generate code challenge (SHA256 hash of verifier, base64url encoded)
    let mut hasher = Sha256::new();
    hasher.update(code_verifier.as_bytes());
    let code_challenge = URL_SAFE_NO_PAD.encode(hasher.finalize());

    (code_verifier, code_challenge)
}

/// Generate a random state parameter for CSRF protection
pub fn generate_state() -> String {
    uuid::Uuid::new_v4().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pkce_generation() {
        let (verifier, challenge) = generate_pkce_challenge();

        // Verify verifier length
        assert!(verifier.len() >= 43);
        assert!(verifier.len() <= 128);

        // Verify challenge is different from verifier
        assert_ne!(verifier, challenge);

        // Verify both are URL-safe base64
        assert!(verifier
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_'));
        assert!(challenge
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_'));
    }

    #[test]
    fn test_state_generation() {
        let state1 = generate_state();
        let state2 = generate_state();

        // States should be different
        assert_ne!(state1, state2);

        // Should be valid UUIDs
        assert!(uuid::Uuid::parse_str(&state1).is_ok());
        assert!(uuid::Uuid::parse_str(&state2).is_ok());
    }
}
