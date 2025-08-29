use crate::view_state_layer::GlobalStateForLeptos;
use leptos::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegatedIdentityWire {
    pub from_key: Vec<u8>,
    pub to_secret: serde_json::Value, // JWK
    pub delegation_chain: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub principal: String,
    pub is_anonymous: bool,
    pub delegated_identity: Option<DelegatedIdentityWire>,
    pub email: Option<String>,
    pub name: Option<String>,
}

#[derive(Clone, Default, Debug)]
pub struct UserAuthState {
    pub is_authenticated: RwSignal<bool>,
    pub user_info: RwSignal<Option<UserInfo>>,
    pub access_token: RwSignal<Option<String>>,
    pub refresh_token: RwSignal<Option<String>>,
    pub token_expires_at: RwSignal<Option<i64>>,
    pub auth_loading: RwSignal<bool>,
}

impl GlobalStateForLeptos for UserAuthState {}

impl UserAuthState {
    // Static methods following established patterns in codebase
    pub fn is_authenticated() -> bool {
        Self::get().is_authenticated.get()
    }

    pub fn get_user_principal() -> Option<String> {
        Self::get().user_info.get().map(|u| u.principal)
    }

    pub fn get_user_info() -> Option<UserInfo> {
        Self::get().user_info.get()
    }

    pub fn is_loading() -> bool {
        Self::get().auth_loading.get()
    }

    pub fn set_loading(loading: bool) {
        Self::get().auth_loading.set(loading);
    }

    pub fn set_authenticated_user(
        user_info: UserInfo,
        access_token: String,
        refresh_token: String,
        expires_at: i64,
    ) {
        let state = Self::get();
        state.is_authenticated.set(true);
        state.user_info.set(Some(user_info));
        state.access_token.set(Some(access_token));
        state.refresh_token.set(Some(refresh_token));
        state.token_expires_at.set(Some(expires_at));
        state.auth_loading.set(false);
    }

    pub fn clear_auth() {
        let state = Self::get();
        state.is_authenticated.set(false);
        state.user_info.set(None);
        state.access_token.set(None);
        state.refresh_token.set(None);
        state.token_expires_at.set(None);
        state.auth_loading.set(false);
    }

    pub fn get_access_token() -> Option<String> {
        Self::get().access_token.get()
    }

    pub fn is_token_expired() -> bool {
        if let Some(expires_at) = Self::get().token_expires_at.get() {
            let now = chrono::Utc::now().timestamp();
            now >= expires_at
        } else {
            true
        }
    }

    /// Check if user is authenticated and token is not expired
    pub fn is_authenticated_and_valid() -> bool {
        Self::is_authenticated() && !Self::is_token_expired()
    }
}
