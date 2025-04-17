use super::{ApiClientResult, ApiError, Provab, ProvabReq, ProvabReqMeta};
use crate::log;
use async_trait::async_trait;
use error_stack::ResultExt;
use leptos::ServerFnError;
use reqwest::Method;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;
use std::time::Duration;
#[cfg(feature = "ssr")]
use tokio;

/// Trait for implementing retry functionality for API requests
#[async_trait]
pub trait RetryableRequest: ProvabReq + Serialize + Clone + 'static {
    #[cfg(feature = "ssr")]
    /// Retry the API call with exponential backoff
    async fn retry_with_backoff(self, retry_count: u8) -> Result<Self::Response, ServerFnError> {
        // let provab = Provab::default();
        // since this is working in SSR context, we need to get the provab instance from context

        use super::a04_book_room::from_leptos_context_or_axum_ssr;

        let provab: Provab = from_leptos_context_or_axum_ssr();

        let mut attempts = 0;
        let max_attempts = retry_count as usize + 1; // +1 because we count the initial attempt

        loop {
            attempts += 1;
            log!(
                "{} API attempt {}/{}",
                Self::path_suffix(),
                attempts,
                max_attempts
            );

            match provab.send(self.clone()).await {
                Ok(response) => {
                    log!(
                        "{} API succeeded on attempt {}",
                        Self::path_suffix(),
                        attempts
                    );
                    return Ok(response);
                }
                Err(e) => {
                    if attempts >= max_attempts {
                        log!(
                            "{} API failed after {} attempts. Error: {:?}",
                            Self::path_suffix(),
                            attempts,
                            e
                        );
                        return Err(ServerFnError::ServerError(e.to_string()));
                    } else {
                        // Calculate backoff time: 2^attempt * 100ms (100ms, 200ms, 400ms, etc.)
                        let backoff_ms = std::cmp::min(2_u64.pow(attempts as u32 - 1) * 100, 2000);
                        log!(
                            "Retrying {} after {}ms (simulated)...",
                            Self::path_suffix(),
                            backoff_ms
                        );

                        // Sleep before retrying
                        tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                    }
                }
            }
        }
    }

    #[cfg(not(feature = "ssr"))]
    async fn retry_with_backoff(self, retry_count: u8) -> Result<Self::Response, ServerFnError> {
        Err(ServerFnError::ServerError(
            "Environment Not supported".to_string(),
        ))
        // Ok(Response)
    }
}

// Implement RetryableRequest for any type that implements ProvabReq
impl<T> RetryableRequest for T where T: ProvabReq + Serialize + Clone + 'static {}
