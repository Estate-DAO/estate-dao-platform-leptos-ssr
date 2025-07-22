use crate::api::auth::{jwt_utils::*, pkce::*, yral_types::*};
use crate::api::consts::EnvVarConfig;
use crate::utils::route::join_base_and_path_url;
use crate::view_state_layer::user_auth_state::UserInfo;
use anyhow::{anyhow, Result};
use reqwest::Client;
use url::Url;

const YRAL_BASE_URL: &str = "https://auth.yral.com";
const YRAL_AUTH_ENDPOINT: &str = "/oauth/auth";
const YRAL_TOKEN_ENDPOINT: &str = "/oauth/token";

pub struct YralOAuthClient {
    client: Client,
    config: EnvVarConfig,
}

impl YralOAuthClient {
    pub fn new(config: EnvVarConfig) -> Result<Self> {
        let client = Client::new();
        Ok(Self { client, config })
    }

    /// Generate authorization URL for YRAL OAuth flow
    /// Returns (auth_url, code_verifier, state)
    pub fn get_authorization_url(&self) -> (String, String, String) {
        let (code_verifier, code_challenge) = generate_pkce_challenge();
        let state = generate_state();

        let auth_url_str = join_base_and_path_url(YRAL_BASE_URL, YRAL_AUTH_ENDPOINT)
            .expect("Valid YRAL auth URL construction");
        let mut url = Url::parse(&auth_url_str).expect("Valid YRAL auth URL");

        url.query_pairs_mut()
            .append_pair("response_type", "code")
            .append_pair("response_mode", "query")
            .append_pair("client_id", &self.config.yral_client_id)
            .append_pair("redirect_uri", &self.config.yral_redirect_uri)
            .append_pair("scope", "openid")
            .append_pair("state", &state)
            .append_pair("code_challenge", &code_challenge)
            .append_pair("code_challenge_method", "S256")
            .append_pair("provider", "google"); // Force Google provider

        (url.to_string(), code_verifier, state)
    }

    /// Exchange authorization code for tokens
    pub async fn exchange_code_for_tokens(
        &self,
        code: String,
        code_verifier: String,
    ) -> Result<YralTokenResponse> {
        let token_url = join_base_and_path_url(YRAL_BASE_URL, YRAL_TOKEN_ENDPOINT)
            .map_err(|e| anyhow!("Failed to construct token URL: {}", e))?;

        let request_body = YralTokenRequest {
            grant_type: "authorization_code".to_string(),
            code,
            redirect_uri: self.config.yral_redirect_uri.clone(),
            client_id: self.config.yral_client_id.clone(),
            client_secret: self.config.yral_client_secret.clone(),
            code_verifier,
        };

        let response = self
            .client
            .post(&token_url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Token exchange failed: {}", error_text));
        }

        let token_response: YralTokenResponse = response.json().await?;
        Ok(token_response)
    }

    /// Refresh access token using refresh token
    pub async fn refresh_token(&self, refresh_token: String) -> Result<YralTokenResponse> {
        let token_url = join_base_and_path_url(YRAL_BASE_URL, YRAL_TOKEN_ENDPOINT)
            .map_err(|e| anyhow!("Failed to construct token URL: {}", e))?;

        let request_body = YralRefreshTokenRequest {
            grant_type: "refresh_token".to_string(),
            refresh_token,
            client_id: self.config.yral_client_id.clone(),
            client_secret: self.config.yral_client_secret.clone(),
        };

        let response = self
            .client
            .post(&token_url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Token refresh failed: {}", error_text));
        }

        let token_response: YralTokenResponse = response.json().await?;
        Ok(token_response)
    }

    /// Extract user info from token response
    pub fn extract_user_info(&self, token_response: &YralTokenResponse) -> Result<UserInfo> {
        extract_user_info_from_id_token(&token_response.id_token)
    }

    /// Validate access token and get claims
    pub fn validate_access_token(&self, access_token: &str) -> Result<YralAccessTokenClaims> {
        decode_access_token(access_token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authorization_url_generation() {
        let config = EnvVarConfig::for_testing();
        let client = YralOAuthClient::new(config).unwrap();

        let (auth_url, code_verifier, state) = client.get_authorization_url();

        // Verify URL contains required parameters
        assert!(auth_url.contains("response_type=code"));
        assert!(auth_url.contains("client_id=test_client_id"));
        assert!(auth_url.contains("provider=google"));
        assert!(auth_url.contains("code_challenge_method=S256"));

        // Verify code verifier and state are not empty
        assert!(!code_verifier.is_empty());
        assert!(!state.is_empty());
    }
}
