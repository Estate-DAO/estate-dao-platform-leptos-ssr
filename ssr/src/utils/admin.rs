use crate::api::consts::EnvVarConfig;
use candid::types::principal;
use candid::Principal;
use ic_agent::identity::{self, BasicIdentity};
use ic_agent::{Agent, Identity};
// use leptos::tracing::{error as tracing_error, info as tracing_info};
use super::ic::AgentWrapper;
use crate::canister::backend::Backend;
use crate::log;
use leptos::prelude::{expect_context, use_context};

#[derive(Clone)]
pub struct AdminCanisters {
    pub agent: AgentWrapper,
}

impl AdminCanisters {
    pub fn new(key: impl Identity + 'static) -> Self {
        Self {
            agent: AgentWrapper::build(|b| b.with_identity(key)),
        }
    }

    pub fn from_env() -> Self {
        let key = create_identity_from_admin_principal();
        Self::new(key)
    }

    // pub fn from_env_axum_ssr() -> Self {
    //     let key = create_identity_from_admin_principal_axum_ssr();
    //     Self::new(key)
    // }

    pub async fn backend_canister(&self) -> Backend<'_> {
        let agent = self.agent.get_agent().await;
        let principal = crate::canister::BACKEND_ID;
        #[cfg(feature = "debug_log")]
        {
            let agent_principal = agent.get_principal().expect("Failed to get principal");
            log!(
                "ADMIN_CANISTER: Agent principal: {}",
                agent_principal.to_text()
            );
        }

        log!("ADMIN_CANISTER: Creating Backend canister instance");
        Backend(principal, agent)
    }
}

/// Must be run on server only
/// since EnvVarConfig is available in letpos server function context
fn create_identity_from_admin_principal() -> impl Identity {
    // tracing_info!("ADMIN_CANISTER: Loading admin private key from environment config");
    let config = EnvVarConfig::expect_context_or_try_from_env();

    // tracing_info!("ADMIN_CANISTER: Creating Secp256k1Identity from PEM");
    let identity = ic_agent::identity::Secp256k1Identity::from_pem(
        stringreader::StringReader::new(config.admin_private_key.as_str()),
    )
    .map_err(|e| {
        // tracing_error!("ADMIN_CANISTER: Failed to create identity from PEM: {}", e);
        e
    })
    .unwrap();

    // tracing_info!("ADMIN_CANISTER: Admin identity created from PEM successfully");
    identity
}

// this works for both leptos context and axum ssr context
// returns AdminCanisters from leptos context if available
// otherwise returns AdminCanisters from env variables
pub fn admin_canister() -> AdminCanisters {
    // tracing_info!("ADMIN_CANISTER: Getting admin canister instance");
    let admin_canisters = use_context::<AdminCanisters>();
    match admin_canisters {
        Some(admin_canisters_leptos) => {
            // tracing_info!("ADMIN_CANISTER: Using AdminCanisters from leptos context");
            admin_canisters_leptos
        }
        None => {
            // tracing_info!("ADMIN_CANISTER: Using AdminCanisters from env/axum SSR");
            AdminCanisters::from_env()
        }
    }
}
