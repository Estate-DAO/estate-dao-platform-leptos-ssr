use ::codee::string::{FromToStringCodec, JsonSerdeCodec};
use candid::Principal;
// use yral_canisters_common::{utils::time::current_epoch, Canisters, CanistersAuthWire};
use leptos::prelude::*;
use leptos_use::{use_cookie_with_options, SameSite, UseCookieOptions};
// use leptos_reactive::Resource;

use crate::{
    api::{
        auth::types::OidcUser,
        consts::{USER_EMAIL_MAPPING_SYNCED, USER_IDENTITY},
    },
    send_wrap,
};

#[derive(Copy, Clone, Default)]
pub struct AuthStateSignal {
    auth: RwSignal<AuthState>,
    wishlist: RwSignal<Option<Vec<String>>>,
}

use std::ops::Deref;

// impl Deref for AuthStateSignal {
//     type Target = RwSignal<AuthState>;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

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
        Self::get().auth
    }

    pub fn wishlist_state() -> RwSignal<Option<Vec<String>>> {
        Self::get().wishlist
    }

    pub fn wishlist_count() -> Option<usize> {
        Self::get().wishlist.get().map(|wishlist| wishlist.len())
    }

    pub fn wishlist_set(wishlist: Option<Vec<String>>) {
        let this = Self::get();
        if this.auth.get_untracked().is_authenticated() {
            this.wishlist.set(wishlist);
        }
    }

    pub fn toggle_wishlish(wish: String) {
        let this = Self::get();
        let mut wishlist = this.wishlist.get_untracked().unwrap_or_default();
        if !Self::check_if_added_to_wishlist_untracked(&wish) {
            wishlist.push(wish);
        } else {
            wishlist.retain(|x| x != &wish);
        }
        Self::wishlist_set(Some(wishlist));
    }

    pub fn wishlist_hotel_codes() -> Vec<String> {
        let this = Self::get();
        this.wishlist.get().unwrap_or_default()
    }

    // Check if already added to wishlist
    pub fn check_if_added_to_wishlist_untracked(wish: &String) -> bool {
        let this = Self::get();
        this.wishlist
            .get_untracked()
            .map_or(false, |wishlist| wishlist.contains(wish))
    }

    // Check if already added to wishlist
    pub fn check_if_added_to_wishlist(wish: &String) -> bool {
        let this = Self::get();
        this.wishlist
            .get()
            .map_or(false, |wishlist| wishlist.contains(wish))
    }

    pub fn auth_set(state: AuthState) {
        let auth_state = Self::get();
        auth_state.auth.set(state);
    }
}
