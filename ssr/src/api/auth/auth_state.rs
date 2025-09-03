use candid::Principal;
use codee::string::{FromToStringCodec, JsonSerdeCodec};
// use yral_canisters_common::{utils::time::current_epoch, Canisters, CanistersAuthWire};
use leptos::*;
use leptos_use::{use_cookie_with_options, SameSite, UseCookieOptions};
// use leptos_reactive::Resource;

use crate::{
    api::{
        auth::types::OidcUser, consts::{yral_auth::{
            ACCOUNT_CONNECTED_STORE, AUTH_UTIL_COOKIES_MAX_AGE_MS, REFRESH_MAX_AGE,
            USER_PRINCIPAL_STORE,
        }, USER_EMAIL_MAPPING_SYNCED, USER_IDENTITY}
    },
    send_wrap,
    utils::parent_resource::{MockPartialEq, ParentResource},
};

#[derive(Copy, Clone)]
pub struct AuthStateSignal(RwSignal<AuthState>);

impl Default for AuthStateSignal {
    fn default() -> Self {
        Self(RwSignal::new(AuthState::default()))
    }
}

use std::ops::Deref;

impl Deref for AuthStateSignal {
    type Target = RwSignal<AuthState>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Default, PartialEq, Debug, serde::Deserialize, serde::Serialize)]
pub struct AuthState {
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub name: Option<String>,
    pub picture: Option<String>,
}

impl AuthState {
    pub fn is_authenticated(&self) -> bool {
        self.email.is_some()
    }
}


impl From<OidcUser> for AuthState {
    fn from(user: OidcUser) -> Self {
        Self {
            email: user.email,
            email_verified: user.email_verified,
            name: user.name,
            picture: user.picture,
        }
    }
}


impl AuthStateSignal {
    pub fn init() -> Self {
        let this = Self::default();
        provide_context(this.clone());
        this
    }

    pub fn get() -> Self {
        use_context::<Self>().unwrap_or_else(AuthStateSignal::init)
    }

    pub fn auth_state() -> RwSignal<AuthState> {
        Self::get().0
    }

    pub fn set(state: AuthState) {
        let auth_state = Self::get();
        auth_state.set(state);
    }
}
