use std::{env, sync::Arc};

use crate::api::consts::AGENT_URL;
use candid::Principal;
use ic_agent::{agent::AgentBuilder, Agent, Identity};

#[derive(Clone)]
pub struct AgentWrapper(Agent);

impl AgentWrapper {
    pub fn build(builder_func: impl FnOnce(AgentBuilder) -> AgentBuilder) -> Self {
        let agent_url = AGENT_URL;
        let mut builder = Agent::builder().with_url(agent_url);
        builder = builder_func(builder);
        Self(builder.build().unwrap())
    }

    pub async fn get_agent(&self) -> &Agent {
        let agent = &self.0;
        #[cfg(any(feature = "local-bin", feature = "local-lib"))]
        {
            agent
                .fetch_root_key()
                .await
                .expect("AGENT: fetch_root_key failed");
        }
        agent
    }

    pub fn set_arc_id(&mut self, id: Arc<impl Identity + 'static>) {
        self.0.set_arc_identity(id);
    }

    pub fn principal(&self) -> Result<Principal, String> {
        self.0.get_principal()
    }
}
