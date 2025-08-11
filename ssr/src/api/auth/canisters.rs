use std::sync::Arc;
use yral_types::delegated_identity::DelegatedIdentityWire;

use crate::{
    canister::BACKEND_ID,
    utils::{ic::AgentWrapper, parent_resource::MockPartialEq, parent_resource::ParentResource},
};
use candid::Principal;
use ic_agent::{identity::DelegatedIdentity, Identity};
use leptos::*;
use serde::{Deserialize, Serialize};

use crate::canister::backend::Backend;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CanistersAuthWire {
    pub id: DelegatedIdentityWire,
    pub user_principal: Principal,
    expiry: u64,
    pub backend_principal: Principal,
    // profile_details: ProfileDetails,
}

impl CanistersAuthWire {
    pub fn canisters(self) -> Result<Canisters<true>, k256::elliptic_curve::Error> {
        let unauth = unauth_canisters();

        let id: DelegatedIdentity = self.id.try_into()?;
        let arc_id = Arc::new(id);

        let mut agent = unauth.agent.clone();
        agent.set_arc_id(arc_id.clone());

        Ok(Canisters {
            agent,
            id: Some(arc_id),
            user_principal: self.user_principal,
            expiry: self.expiry,
            backend_principal: BACKEND_ID,
            // profile_details: Some(self.profile_details),
        })
    }
}

#[derive(Clone)]
pub struct Canisters<const AUTH: bool> {
    agent: AgentWrapper,
    id: Option<Arc<DelegatedIdentity>>,
    user_principal: Principal,
    expiry: u64,
    backend_principal: Principal,
    // profile_details: Option<ProfileDetails>,
}

impl Default for Canisters<false> {
    fn default() -> Self {
        Self {
            agent: AgentWrapper::build(|b| b),
            id: None,
            user_principal: Principal::anonymous(),
            expiry: 0,
            backend_principal: BACKEND_ID,
            // profile_details: None,
        }
    }
}

impl Canisters<true> {
    pub fn authenticated(id: DelegatedIdentity) -> Canisters<true> {
        let expiry = id
            .delegation_chain()
            .iter()
            .fold(u64::MAX, |prev_expiry, del| {
                del.delegation.expiration.min(prev_expiry)
            });
        let id = Arc::new(id);

        Canisters {
            agent: AgentWrapper::build(|b| b.with_arc_identity(id.clone())),
            id: Some(id),
            user_principal: Principal::anonymous(),
            expiry,
            backend_principal: BACKEND_ID,
            // profile_details: None,
        }
    }

    pub fn expiry_ns(&self) -> u64 {
        self.expiry
    }

    pub fn identity(&self) -> &DelegatedIdentity {
        self.id
            .as_ref()
            .expect("Authenticated canisters must have an identity")
    }

    // pub fn profile_details(&self) -> ProfileDetails {
    //     self.profile_details
    //         .clone()
    //         .expect("Authenticated canisters must have profile details")
    // }

    pub fn user_principal(&self) -> Principal {
        self.identity()
            .sender()
            .expect("expect principal to be present")
    }

    pub async fn backend_canister(&self) -> Backend<'_> {
        self.backend().await
    }
}

pub fn unauth_canisters() -> Canisters<false> {
    expect_context()
}

impl<const A: bool> Canisters<A> {
    pub async fn backend(&self) -> Backend<'_> {
        let agent = self.agent.get_agent().await;
        #[cfg(any(feature = "local-bin", feature = "local-lib"))]
        {
            agent
                .fetch_root_key()
                .await
                .expect("AGENT: fetch_root_key failed");
        }
        Backend(self.backend_principal, agent)
    }
}

pub type AuthCansResource = ParentResource<
    MockPartialEq<Option<DelegatedIdentityWire>>,
    Result<CanistersAuthWire, ServerFnError>,
>;

/// The Authenticated Canisters helper resource
/// prefer using helpers from [crate::component::canisters_prov]
/// instead
pub fn authenticated_canisters() -> AuthCansResource {
    expect_context()
}

/// The store for Authenticated canisters
/// Do not use this for anything other than analytics
pub fn auth_canisters_store() -> RwSignal<Option<Canisters<true>>> {
    expect_context()
}

pub async fn do_canister_auth(
    auth: DelegatedIdentityWire,
) -> Result<CanistersAuthWire, ServerFnError> {
    let id = auth.clone().try_into()?;
    let canisters = Canisters::<true>::authenticated(id);

    // let user = canisters.authenticated_user().await;

    // let profile_details = user.get_profile_details().await?.into();

    let cans_wire = CanistersAuthWire {
        id: auth,
        user_principal: canisters.user_principal,
        expiry: canisters.expiry,
        backend_principal: BACKEND_ID,
    };

    Ok(cans_wire)
}
