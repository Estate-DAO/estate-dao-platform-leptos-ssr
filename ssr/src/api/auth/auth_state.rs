use candid::Principal;
use codee::string::FromToStringCodec;
// use yral_canisters_common::{utils::time::current_epoch, Canisters, CanistersAuthWire};
use leptos::*;
use leptos_use::{use_cookie_with_options, UseCookieOptions};
// use leptos_reactive::Resource;

use crate::{
    api::{
        auth::{
            canisters::{do_canister_auth, AuthCansResource, Canisters, CanistersAuthWire},
            extract_identity_impl::extract_identity,
            types::NewIdentity,
        },
        consts::yral_auth::{
            ACCOUNT_CONNECTED_STORE, AUTH_UTIL_COOKIES_MAX_AGE_MS, REFRESH_MAX_AGE,
            USER_PRINCIPAL_STORE,
        },
    },
    send_wrap,
    utils::parent_resource::{MockPartialEq, ParentResource},
};

pub fn auth_state() -> AuthState {
    expect_context()
}

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




#[derive(Clone)]
pub struct AuthState {
    // _temp_identity_resource: OnceResource<Option<AnonymousIdentity>>,
    // _temp_id_cookie_resource: LocalResource<()>,
    // pub referrer_store: Signal<Option<Principal>>,
    is_logged_in_with_oauth: (Signal<Option<bool>>, WriteSignal<Option<bool>>),
    pub new_identity_setter: RwSignal<Option<NewIdentity>>,
    // canisters_resource: AuthCansResource,
    // // pub user_canister: Resource<Result<Principal, ServerFnError>>,
    // // user_canister_id_cookie: (Signal<Option<Principal>>, WriteSignal<Option<Principal>>),
    pub user_principal: RwSignal<Option<Principal>>,
    pub canister_store: RwSignal<Option<Canisters<true>>>,
    user_principal_cookie: (Signal<Option<Principal>>, WriteSignal<Option<Principal>>),
    pub user_identity: RwSignal<Option<NewIdentity>>,
    pub new_cans_setter: RwSignal<Option<CanistersAuthWire>>,
}

impl Default for AuthState {
    fn default() -> Self {
        // Super complex, don't mess with this.
        crate::log!("AUTH_FLOW: AuthState::default() called - initializing auth state");

        // let temp_identity_resource = OnceResource::new(async move {
        //     generate_anonymous_identity_if_required()
        //         .await
        //         .expect("Failed to generate anonymous identity?!")
        // });
        // let temp_id_cookie_resource = LocalResource::new(move || async move {
        //     let Some(temp_identity) = temp_identity_resource.await else {
        //         return;
        //     };
        //     if let Err(e) = set_anonymous_identity_cookie(temp_identity.refresh_token).await {
        //         log::error!("Failed to set anonymous identity as cookie?! err {e}");
        //     }
        // });

        // let (referrer_cookie, set_referrer_cookie) =
        //     use_cookie_with_options::<Principal, FromToStringCodec>(
        //         REFERRER_COOKIE,
        //         UseCookieOptions::default()
        //             .path("/")
        //             .max_age(AUTH_UTIL_COOKIES_MAX_AGE_MS),
        //     );
        // let referrer_query = use_query::<Referrer>();
        // let referrer_principal = Signal::derive(move || {
        //     let referrer_query_val = referrer_query()
        //         .ok()
        //         .and_then(|r| Principal::from_text(r.user_refer).ok());

        //     let referrer_cookie_val = referrer_cookie.get_untracked();
        //     if let Some(ref_princ) = referrer_query_val {
        //         set_referrer_cookie(Some(ref_princ));
        //         Some(ref_princ)
        //     } else {
        //         referrer_cookie_val
        //     }
        // });

        let is_logged_in_with_oauth = use_cookie_with_options::<bool, FromToStringCodec>(
            ACCOUNT_CONNECTED_STORE,
            UseCookieOptions::default()
                .path("/")
                .max_age(REFRESH_MAX_AGE.as_millis() as i64),
        );
        crate::log!("AUTH_FLOW: OAuth login state from cookie: {:?}", is_logged_in_with_oauth.0.get_untracked());

        let new_identity_setter = RwSignal::new(None::<NewIdentity>);

        // let user_identity_resource = Resource::new(
        //     move || MockPartialEq(new_identity_setter()),
        //     move |auth_id| async move {
        //         // let temp_identity = temp_identity_resource.await;

        //         if let Some(id_wire) = auth_id.0 {
        //             return Ok::<_, ServerFnError>(id_wire);
        //         }

        //         // just implement the refresh token flow from server side
        //         let id_wire = match extract_identity().await {
        //             Ok(Some(id_wire)) => id_wire,
        //             Ok(None) => return Err(ServerFnError::new("No refresh cookie set?!")),
        //             Err(e) => {
        //                 return Err(ServerFnError::new(e.to_string()));
        //             }
        //         };

        //         Ok(NewIdentity {
        //             id_wire,
        //             fallback_username: None,
        //         })
        //     },
        // );

        let new_cans_setter = RwSignal::new(None::<CanistersAuthWire>);

        // let canisters_resource: AuthCansResource = ParentResource(create_resource(
        //     move || {
        //         user_identity_resource.track();
        //         MockPartialEq(new_cans_setter())
        //     },
        //     move |new_cans| {
        //         send_wrap(async move {
        //             let new_id = user_identity_resource.await?;
        //             match new_cans.0 {
        //                 Some(cans) if cans.id.from_key == new_id.id_wire.from_key => {
        //                     return Ok::<_, ServerFnError>(cans);
        //                 }
        //                 // this means that the user did the following:
        //                 // 1. Changed their username, then
        //                 // 2. Logged in with oauth (or logged out)
        //                 _ => {}
        //             };

        //             let res: CanistersAuthWire = do_canister_auth(new_id.id_wire).await?;

        //             Ok::<_, ServerFnError>(res)
        //         })
        //     },
        // ));

        let user_principal_cookie = use_cookie_with_options::<Principal, FromToStringCodec>(
            USER_PRINCIPAL_STORE,
            UseCookieOptions::default()
                .path("/")
                .max_age(AUTH_UTIL_COOKIES_MAX_AGE_MS),
        );
        crate::log!("AUTH_FLOW: User principal from cookie: {:?}", user_principal_cookie.0.get_untracked());
        // let user_principal = create_resource(
        //     move || {
        //         user_identity_resource.track();
        //         MockPartialEq(())
        //     },
        //     move |_| async move {
        //         if let Some(princ) = user_principal_cookie.0.get_untracked() {
        //             return Ok(princ);
        //         }

        //         let id_wire = user_identity_resource.await?;
        //         let princ = Principal::self_authenticating(&id_wire.id_wire.from_key);
        //         user_principal_cookie.1.set(Some(princ));

        //         Ok(princ)
        //     },
        // );

        // let user_canister_id_cookie = use_cookie_with_options::<Principal, FromToStringCodec>(
        //     USER_CANISTER_ID_STORE,
        //     UseCookieOptions::default()
        //         .path("/")
        //         .max_age(AUTH_UTIL_COOKIES_MAX_AGE_MS),
        // );
        // let user_canister = Resource::new(
        //     move || {
        //         canisters_resource.track();
        //         MockPartialEq(())
        //     },
        //     move |_| async move {
        //         if let Some(canister_id) = user_canister_id_cookie.0.get_untracked() {
        //             return Ok(canister_id);
        //         }

        //         let cans_wire = canisters_resource.await?;

        //         let canister_id = cans_wire.user_canister;
        //         user_canister_id_cookie.1.set(Some(canister_id));

        //         Ok(canister_id)
        //     },
        // );

        // let event_ctx = EventCtx {
        //     is_connected: StoredValue::new(Box::new(move || {
        //         is_logged_in_with_oauth
        //             .0
        //             .get_untracked()
        //             .unwrap_or_default()
        //     })),
        //     user_details: StoredValue::new(Box::new(move || {
        //         canisters_resource
        //             .into_future()
        //             .now_or_never()
        //             .and_then(|c| {
        //                 let cans_wire = c.ok()?;
        //                 Some(EventUserDetails {
        //                     details: cans_wire.profile_details.clone(),
        //                     canister_id: cans_wire.user_canister,
        //                 })
        //             })
        //     })),
        // };

        let auth_state = Self {
            // _temp_identity_resource: temp_identity_resource,
            // _temp_id_cookie_resource: temp_id_cookie_resource,
            // referrer_store: referrer_principal,
            is_logged_in_with_oauth,
            new_identity_setter,
            // canisters_resource,
            user_principal: create_rw_signal(None::<Principal>),
            canister_store: create_rw_signal(None::<Canisters<true>>),
            user_principal_cookie,
            // user_canister,
            // user_canister_id_cookie,
            // event_ctx,
            user_identity: create_rw_signal(None::<NewIdentity>),
            new_cans_setter,
        };
        
        crate::log!("AUTH_FLOW: AuthState initialized - canister_store: {:?}, user_identity: {:?}", 
            auth_state.canister_store.get_untracked().is_some(), 
            auth_state.user_identity.get_untracked().is_some());
            
        auth_state
    }
}

impl AuthState {
    pub fn is_logged_in_with_oauth(&self) -> Signal<bool> {
        let logged_in = self.is_logged_in_with_oauth.0;
        Signal::derive(move || logged_in.get().unwrap_or_default())
    }

    pub fn get_user_principal(&self) -> Option<Principal> {
        self.user_principal.get_untracked()
    }

    fn get_user_principal_cookie(&self) -> Option<Principal> {
        self.user_principal_cookie.0.get_untracked()
    }

    fn set_user_principal_cookie(&self, principal: Principal) {
        self.user_principal_cookie.1.set(Some(principal));
    }

    pub fn set_user_identity_with_cookie(&self, user_identity: NewIdentity) {
        self.user_identity.set(Some(user_identity.clone()));
        let princ = Principal::self_authenticating(&user_identity.id_wire.from_key);
        self.set_user_principal_cookie(princ);
    }
    pub fn set_cans_auth_wire(&self, canisters: CanistersAuthWire) {
        self.new_cans_setter.set(Some(canisters));
    }

    pub fn user_principal_if_available(&self) -> Option<Principal> {
        self.user_principal_cookie.0.get_untracked()
    }

    pub fn reset_user_identity(&self) {
        self.user_identity.set(None);
        self.user_principal_cookie.1.set(None);
    }
}
// /// Prevents hydration bugs if the value in store is used to conditionally show views
// /// this is because the server will always get a `false` value and do rendering based on that
// pub fn account_connected_reader() -> (ReadSignal<bool>, Effect<()>) {
//     let (read_account_connected, _, _) =
//         use_local_storage::<bool, FromToStringCodec>(ACCOUNT_CONNECTED_STORE);
//     let (is_connected, set_is_connected) = create_signal(false);

//     (
//         is_connected,
//         create_effect(move |_| {
//             set_is_connected(read_account_connected());
//         }),
//     )
// }
