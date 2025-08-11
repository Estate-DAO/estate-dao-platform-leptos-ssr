use crate::log;
use crate::{
    api::{
        auth::{
            auth_state::AuthState,
            canisters::{do_canister_auth, AuthCansResource, Canisters, CanistersAuthWire},
            extract_identity_impl::extract_identity,
            types::NewIdentity,
        },
        consts::yral_auth::{AUTH_UTIL_COOKIES_MAX_AGE_MS, USER_PRINCIPAL_STORE},
    },
    send_wrap,
    utils::parent_resource::{MockPartialEq, ParentResource},
};
use candid::Principal;
use codee::string::FromToStringCodec;
use leptos::*;
use leptos_router::{use_navigate, Outlet};
use leptos_use::storage::use_local_storage;
use leptos_use::{use_cookie_with_options, use_event_listener, use_window, UseCookieOptions};

#[derive(Clone)]
pub struct Notification(pub RwSignal<Option<serde_json::Value>>);

#[component]
fn CtxProvider(children: Children) -> impl IntoView {
    let auth = AuthState::default();
    let auth_clone = auth.clone();
    let auth_clone2 = auth.clone();

    provide_context(auth.clone());

    let navigate = use_navigate();

    let canisters_action = create_action(move |new_identity: &NewIdentity| {
        let auth = auth.clone();
        let new_identity = new_identity.clone();
        async move {
            let id_wire = new_identity.id_wire.clone();
            match do_canister_auth(id_wire).await {
                Ok(cans_wire) => {
                    // auth.set_cans_auth_wire(cans_wire);
                    Ok(cans_wire)
                }
                Err(e) => {
                    log!("Failed to get canisters! {e}");
                    Err(e)
                }
            }
        }
    });

    //
    // IDENTITY RESOURCE
    //

    let user_identity_resource = Resource::new(
        move || MockPartialEq(auth_clone.new_identity_setter.get()),
        // move || MockPartialEq(auth.new_identity_setter.get()),
        move |auth_id| {
            let auth_clone = auth_clone.clone();
            async move {
                // let temp_identity = temp_identity_resource.await;

                // this early return prevents infinite loop
                if let Some(id_wire) = auth_id.0 {
                    return Ok::<_, ServerFnError>(id_wire);
                }

                // just implement the refresh token flow from server side
                let id_wire = match extract_identity().await {
                    Ok(Some(id_wire)) => id_wire,
                    Ok(None) => return Err(ServerFnError::new("No refresh cookie set?!")),
                    Err(e) => {
                        return Err(ServerFnError::new(e.to_string()));
                    }
                };

                let user_identity = NewIdentity {
                    id_wire,
                    // todo(2025-08-10) set a default username later.
                    fallback_username: None,
                };

                // set user_identity in state
                auth_clone.set_user_identity_with_cookie(user_identity.clone());

                // call canisters action
                let cans_auth_wire = canisters_action.dispatch(user_identity.clone());
                Ok(user_identity)
            }
        },
    );

    //
    // CANISTER RESOURCE
    //

    // let canisters_resource: AuthCansResource = ParentResource(create_resource(
    //     let canisters_resource: AuthCansResource = Resource::new(
    //     move || {
    //         user_identity_resource.track();
    //         MockPartialEq(auth.new_cans_setter.get())
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

    //             // todo(2025-08-10): should I also set this into canisters_store here?
    //             // auth.canister_store.set(Some(res.clone()));
    //             Ok(res)
    //         })
    //     },
    // );

    // taken from csr code.
    // let canisters_res: AuthCansResource = ParentResource(create_resource(
    //     move || MockPartialEq(auth.user_identity.get()),
    //     move |auth_id| {
    //         async move {
    //             if let Some(new_identity) = auth_id.0 {
    //                 return do_canister_auth(new_identity.id_wire).await;
    //             }

    //             let Some(jwk_key) = temp_identity else {
    //                 let identity = match  extract_identity().await {
    //                     Ok(id) => id,
    //                     Err(_) => return Err(ServerFnError::new("Failed to extract identity"))
    //                 };
    //                 let id_wire = identity.expect("No refresh cookie set?!");
    //                 return do_canister_auth(id_wire).await;
    //             };

    //             let key = k256::SecretKey::from_jwk(&jwk_key)?;
    //             let id = Secp256k1Identity::from_private_key(key);
    //             let id_wire = DelegatedIdentityWire::delegate(&id);

    //             do_canister_auth(id_wire).await
    //         }
    //     },
    // ));
    // provide_context(canisters_resource.clone());

    // Monitor auth errors and navigate to logout if needed

    // todo(2025-08-10): uncomment this for auto logout
    // create_effect(move |_| {
    //     if user_identity_resource.get().is_none() {
    //         // todo (2025-08-10) - do the navigation to logout - impl logout route
    //         navigate("/logout", Default::default());
    //     }
    // });

    //
    // USER PRINCIPAL RESOURCE
    //

    //  this is untracked cookie
    // let user_principal_cookie = auth.get_user_principal_cookie();
    // let user_principal = create_resource(
    //     move || {
    //         user_identity_resource.track();
    //         MockPartialEq(())
    //     },
    //     move |_| async move {
    //         if let Some(princ) = user_principal_cookie {
    //             return Ok(princ);
    //         }

    //         let id_wire = user_identity_resource.await?;
    //         let princ = Principal::self_authenticating(&id_wire.id_wire.from_key);
    //         auth.set_user_principal_cookie(princ);

    //         Ok(princ)
    //     },
    // );

    Effect::new(move |_| {
        // let pathname = location.pathname.get();
        let is_logged_in = auth_clone2.is_logged_in_with_oauth();
        let Some(principal) = auth_clone2.user_principal_if_available() else {
            return;
        };
    });

    children()
}

#[component]
pub fn BaseRoute() -> impl IntoView {
    view! {
        <CtxProvider>
            <Outlet />
        </CtxProvider>
    }
}
