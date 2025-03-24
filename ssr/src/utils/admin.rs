use crate::api::consts::EnvVarConfig;
use candid::types::principal;
use candid::Principal;
use ic_agent::identity::{self, BasicIdentity};
use ic_agent::{Agent, Identity};
use leptos::expect_context;

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

    pub async fn backend_canister(&self) -> Backend {
        let agent = self.agent.get_agent().await;
        let principal = crate::canister::BACKEND_ID;
        // println!("{principal:#?}");
        Backend(principal, agent)
    }
}

/// Must be run on server only
/// since EnvVarConfig is available in server context
fn create_identity_from_admin_principal() -> impl Identity {
    let config: EnvVarConfig = expect_context();

    let identity = ic_agent::identity::Secp256k1Identity::from_pem(
        stringreader::StringReader::new(config.admin_private_key.as_str()),
    )
    .unwrap();

    identity
}

pub fn admin_canister() -> AdminCanisters {
    expect_context()
}
