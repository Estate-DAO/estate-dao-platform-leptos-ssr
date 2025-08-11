use crate::api::auth::auth_state::AuthStateSignal;
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
        consts::USER_IDENTITY,
    },
    send_wrap,
    utils::parent_resource::{MockPartialEq, ParentResource},
};
use candid::Principal;
use codee::string::{FromToStringCodec, JsonSerdeCodec};
use leptos::SignalGetUntracked;
use leptos::*;
use leptos_router::{use_navigate, Outlet};
use leptos_use::storage::use_local_storage;
use leptos_use::{
    use_cookie_with_options, use_event_listener, use_window, SameSite, UseCookieOptions,
};

#[derive(Clone)]
pub struct Notification(pub RwSignal<Option<serde_json::Value>>);

#[component]
fn CtxProvider(children: Children) -> impl IntoView {
    log!("AUTH_FLOW: base_route - CtxProvider started");
    let auth_state_signal: AuthStateSignal = expect_context();

    let auth = auth_state_signal.get();
    log!(
        "AUTH_FLOW: base_route - Retrieved auth state - logged_in: {:?}",
        auth.is_logged_in_with_oauth().get_untracked()
    );

    provide_context(auth.clone());

    let navigate = use_navigate();

    let canisters_action = create_action(move |new_identity: &NewIdentity| {
        let auth = auth_state_signal.get();
        let new_identity = new_identity.clone();
        async move {
            log!("AUTH_FLOW: base_route - Canisters action triggered");
            let id_wire = new_identity.id_wire.clone();
            match do_canister_auth(id_wire).await {
                Ok(cans_wire) => {
                    log!(
                        "AUTH_FLOW: base_route - Successfully got canisters from do_canister_auth"
                    );

                    // IMPORTANT: Set the canisters in the store so other parts can use it
                    auth.set_cans_auth_wire(cans_wire.clone());
                    log!("AUTH_FLOW: base_route - Set cans_auth_wire in auth state");

                    // let canisters = cans_wire.clone().canisters().map_err(|e| {
                    //     log!(
                    //         "AUTH_FLOW: base_route - Failed to convert cans_wire to canisters: {}",
                    //         e
                    //     );
                    //     e
                    // })?;
                    // auth.canister_store.set(Some(canisters));
                    // log!("AUTH_FLOW: base_route - Set canisters in canister_store - success: {}",
                    //     auth.canister_store.get_untracked().is_some());
                    Ok(cans_wire)
                }
                Err(e) => {
                    log!("AUTH_FLOW: base_route - Failed to get canisters! {e}");
                    Err(e)
                }
            }
        }
    });

    //
    // IDENTITY RESOURCE
    //

    let user_identity_resource = Resource::new(
        move || {
            log!("AUTH_FLOW: base_route - user_identity_resource - User identity resource signal changed");
            MockPartialEq(auth.new_identity_setter.get())
        },
        // move || MockPartialEq(auth.new_identity_setter.get()),
        move |auth_id| {
            let auth = auth_state_signal.get();
            async move {
                log!("AUTH_FLOW: base_route - user_identity_resource - User identity resource loading - auth_id present: {}", auth_id.0.is_some());
                // let temp_identity = temp_identity_resource.await;

                // this early return prevents infinite loop
                if let Some(id_wire) = auth_id.0 {
                    log!("AUTH_FLOW: base_route - user_identity_resource - Using existing identity from auth_id");
                    return Ok::<_, ServerFnError>(id_wire);
                }

                // First try to get identity from frontend USER_IDENTITY cookie
                log!("AUTH_FLOW: base_route - user_identity_resource - Checking frontend USER_IDENTITY cookie");
                let (user_identity_cookie, _) =
                    use_cookie_with_options::<NewIdentity, JsonSerdeCodec>(
                        USER_IDENTITY,
                        UseCookieOptions::default()
                            .path("/")
                            .same_site(SameSite::Lax)
                            .http_only(false)
                            .secure(false),
                    );

                if let Some(stored_user_identity) = user_identity_cookie.get_untracked() {
                    log!("AUTH_FLOW: base_route - user_identity_resource - Found USER_IDENTITY cookie: {:?}", stored_user_identity);

                    // Set user_identity in state
                    let auth = auth_state_signal.get();
                    auth.set_user_identity_with_cookie(stored_user_identity.clone());
                    log!("AUTH_FLOW: base_route - user_identity_resource - Set user identity from frontend cookie");

                    // call canisters action to populate canister_store
                    log!("AUTH_FLOW: base_route - user_identity_resource - Dispatching canisters action from frontend cookie");
                    let _cans_auth_wire = canisters_action.dispatch(stored_user_identity.clone());
                    return Ok(stored_user_identity);
                }

                log!("AUTH_FLOW: base_route - user_identity_resource - No USER_IDENTITY cookie, trying server-side extraction");

                // Fallback: try to extract from server-side cookies
                log!("AUTH_FLOW: base_route - user_identity_resource - Attempting to extract identity from server cookies");
                let id_wire = match extract_identity().await {
                    Ok(Some(id_wire)) => {
                        log!("AUTH_FLOW: base_route - user_identity_resource - Successfully extracted identity from server cookies");
                        id_wire
                    }
                    Ok(None) => {
                        log!("AUTH_FLOW: base_route - user_identity_resource - No refresh cookie found");
                        return Err(ServerFnError::new("No refresh cookie set?!"));
                    }
                    Err(e) => {
                        log!("AUTH_FLOW: base_route - user_identity_resource - Failed to extract identity: {}", e);
                        return Err(ServerFnError::new(e.to_string()));
                    }
                };

                let user_identity = NewIdentity {
                    id_wire,
                    // todo(2025-08-10) set a default username later.
                    fallback_username: None,
                };

                // set user_identity in state
                log!("AUTH_FLOW: base_route - user_identity_resource - Setting user_identity to cookie: {user_identity:?}");

                let auth = auth_state_signal.get();
                auth.set_user_identity_with_cookie(user_identity.clone());
                log!("AUTH_FLOW: base_route - user_identity_resource - User identity set in auth state");

                // call canisters action
                log!(
                    "AUTH_FLOW: base_route - user_identity_resource - Dispatching canisters action"
                );
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

    // Effect to check USER_IDENTITY cookie and populate auth state on page load
    Effect::new(move |_| {
        let auth = auth_state_signal.get();

        // Check if auth state is already populated
        if auth.user_identity.get_untracked().is_some() {
            log!("AUTH_FLOW: base_route - Effect: Auth state already populated, skipping");
            return;
        }

        log!("AUTH_FLOW: base_route - Effect: Checking for USER_IDENTITY cookie to populate auth state");

        // Check USER_IDENTITY cookie
        let (user_identity_cookie, _) = use_cookie_with_options::<NewIdentity, JsonSerdeCodec>(
            USER_IDENTITY,
            UseCookieOptions::default()
                .path("/")
                .same_site(SameSite::Lax)
                .http_only(false)
                .secure(false),
        );

        if let Some(stored_user_identity) = user_identity_cookie.get_untracked() {
            log!("AUTH_FLOW: base_route - Effect: Found USER_IDENTITY cookie, populating auth state: {:?}", stored_user_identity);

            // Populate auth state
            auth.user_identity.set(Some(stored_user_identity.clone()));
            auth.set_user_identity_with_cookie(stored_user_identity.clone());

            // // Also set the new_identity_setter to trigger the resource
            // auth.new_identity_setter
            //     .set(Some(stored_user_identity.clone()));
            // log!("AUTH_FLOW: base_route - Effect: Set user identity in auth state and triggered resource");

            // Dispatch canisters action to populate canister_store
            log!("AUTH_FLOW: base_route - Effect: Dispatching canisters action from USER_IDENTITY cookie");
            canisters_action.dispatch(stored_user_identity.clone());
        } else {
            log!("AUTH_FLOW: base_route - Effect: No USER_IDENTITY cookie found");
        }
    });

    Effect::new(move |_| {
        let auth = auth_state_signal.get();
        // let pathname = location.pathname.get();
        let is_logged_in = auth.is_logged_in_with_oauth();
        let principal_available = auth.user_principal_if_available();
        log!(
            "AUTH_FLOW: base_route - Effect running - logged_in: {:?}, principal_available: {}",
            is_logged_in.get_untracked(),
            principal_available.is_some()
        );

        let Some(principal) = principal_available else {
            log!("AUTH_FLOW: base_route - No principal available, effect returning early");
            return;
        };
        log!("AUTH_FLOW: base_route - Principal available: {}", principal);
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
