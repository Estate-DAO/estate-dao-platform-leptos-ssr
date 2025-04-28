use crate::api::consts::EnvVarConfig;
use candid::types::principal;
use candid::Principal;
use ic_agent::identity::{self, BasicIdentity};
use ic_agent::{Agent, Identity};
use leptos::{expect_context, use_context};
use log::info;

use super::ic::AgentWrapper;
use crate::canister::backend::Backend;

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

    pub fn from_env_axum_ssr() -> Self {
        let key = create_identity_from_admin_principal_axum_ssr();
        Self::new(key)
    }

    pub async fn backend_canister(&self) -> Backend {
        let agent = self.agent.get_agent().await;
        let principal = crate::canister::BACKEND_ID;
        #[cfg(feature = "debug_log")]
        {
            let agent_principal = agent.get_principal().expect("Failed to get principal");
            // println!("agent_principal - {:#?}", agent_principal.to_text());
        }
        Backend(principal, agent)
    }
}

/// Must be run on server only
/// since EnvVarConfig is available in letpos server function context
fn create_identity_from_admin_principal() -> impl Identity {
    let config: EnvVarConfig = expect_context();

    let identity = ic_agent::identity::Secp256k1Identity::from_pem(
        stringreader::StringReader::new(config.admin_private_key.as_str()),
    )
    .unwrap();

    identity
}

fn create_identity_from_admin_principal_axum_ssr() -> impl Identity {
    let config = EnvVarConfig::try_from_env();

    let identity = ic_agent::identity::Secp256k1Identity::from_pem(
        stringreader::StringReader::new(config.admin_private_key.as_str()),
    )
    .unwrap();

    identity
}

// this works for both leptos context and axum ssr context
// returns AdminCanisters from leptos context if available
// otherwise returns AdminCanisters from env variables
pub fn admin_canister() -> AdminCanisters {
    let admin_canisters = use_context::<AdminCanisters>();
    match admin_canisters {
        Some(admin_canisters_leptos) => {
            info!("admin_canister: Using AdminCanisters from leptos context");
            admin_canisters_leptos
        }
        None => {
            info!("admin_canister: Using AdminCanisters from env/axum SSR");
            AdminCanisters::from_env_axum_ssr()
        }
    }
}
