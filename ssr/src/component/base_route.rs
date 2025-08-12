use crate::api::auth::auth_state::AuthStateSignal;
use crate::api::canister::update_user_principal_email::update_user_principal_email_mapping_in_canister;
use crate::log;
use crate::{
    api::{
        auth::{
            auth_state::AuthState,
            canisters::{do_canister_auth, AuthCansResource, Canisters, CanistersAuthWire},
            extract_identity_impl::extract_new_identity,
            types::NewIdentity,
        },
        client_side_api::ClientSideApiClient,
        consts::yral_auth::{AUTH_UTIL_COOKIES_MAX_AGE_MS, USER_PRINCIPAL_STORE},
        consts::USER_EMAIL_MAPPING_SYNCED,
    },
    send_wrap,
    utils::parent_resource::{MockPartialEq, ParentResource},
};
use candid::Principal;
use leptos::SignalGetUntracked;
use leptos::*;
use leptos_router::{use_location, use_navigate, Outlet};
use leptos_use::storage::use_local_storage;
use leptos_use::{use_event_listener, use_window};

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
    let location = use_location();

    // Define protected routes that require authentication
    let protected_routes = vec!["/my-bookings"];

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
                let auth = auth_state_signal.get();

                if let Some(stored_user_identity) = auth.get_user_identity_cookie() {
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

                // Fallback: try to extract full NewIdentity (with username/email) from server-side cookies
                log!("AUTH_FLOW: base_route - user_identity_resource - Attempting to extract identity from server cookies");
                let user_identity: NewIdentity = match extract_new_identity().await {
                    Ok(Some(new_identity)) => {
                        log!("AUTH_FLOW: base_route - user_identity_resource - Successfully extracted NewIdentity from server cookies with username: {:?}, email: {:?}", 
                            new_identity.fallback_username, new_identity.email);
                        new_identity
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

    // Create a blocking resource for auth state initialization
    // This runs synchronously on SSR and reactively on client
    let auth_init_resource = create_resource(
        move || {
            // Track location changes to handle route protection
            (location.pathname.get(), ())
        },
        move |(current_path, _)| {
            let navigate = navigate.clone();
            let protected_routes = protected_routes.clone();
            async move {
                let auth = auth_state_signal.get();
                log!("AUTH_FLOW: base_route - Blocking resource: Initializing auth state for path: {}", current_path);

                // Check if auth state is already initialized
                if auth.user_identity.get_untracked().is_some() {
                    log!("AUTH_FLOW: base_route - Blocking resource: Auth state already populated");
                    return Ok::<_, ServerFnError>(true);
                }

                log!("AUTH_FLOW: base_route - Blocking resource: Checking USER_IDENTITY cookie");

                // Check USER_IDENTITY cookie using AuthState
                if let Some(stored_user_identity) = auth.get_user_identity_cookie() {
                    log!("AUTH_FLOW: base_route - Blocking resource: Found USER_IDENTITY cookie, initializing auth state: {:?}", stored_user_identity);

                    // Initialize auth state
                    auth.set_user_identity_with_cookie(stored_user_identity.clone());

                    // Dispatch canisters action to populate canister_store
                    log!("AUTH_FLOW: base_route - Blocking resource: Dispatching canisters action");
                    canisters_action.dispatch(stored_user_identity.clone());

                    Ok(true)
                } else {
                    log!(
                        "AUTH_FLOW: base_route - Blocking resource: No USER_IDENTITY cookie found"
                    );

                    // Check if current route requires authentication
                    if protected_routes.contains(&current_path.as_str()) {
                        log!("AUTH_FLOW: base_route - Blocking resource: Protected route accessed without authentication, redirecting");
                        navigate("/", Default::default());
                    }

                    Ok(false)
                }
            }
        },
    );

    //
    // USER EMAIL MAPPING SYNC RESOURCE
    //

    let user_email_sync_resource = create_resource(
        move || {
            log!("USER_EMAIL_SYNC: Signal tracker called");

            // Track multiple signals that could trigger email sync
            let auth = auth_state_signal.get();
            let auth_cans_setter = auth.new_cans_setter.get();
            // let user_identity = auth.user_identity.get(); // Track user_identity signal
            let auth_init_status = auth_init_resource.get(); // Track auth_init_resource

            // log!(
            //     "USER_EMAIL_SYNC: Tracking signals - user_identity.is_some: {}, auth_init_status.is_some: {}",
            //     user_identity.is_some(),
            //     auth_init_status.is_some()
            // );

            log!("USER_EMAIL_SYNC: Tracking signals - auth_cans_setter: {}, auth_init_resource: {:?}", auth_cans_setter.is_some(), auth_init_status.is_some());

            // Return a tuple wrapped in MockPartialEq to track both signals
            MockPartialEq((auth_cans_setter, auth_init_status))
        },
        move |data| {
            let (_tracker_cans, auth_init_status) = data.0.clone();
            let auth = auth_state_signal.get();
            let user_identity = auth.user_identity.get();
            async move {
                log!("USER_EMAIL_SYNC: Fetcher executing");

                // Check if we have user identity and auth is initialized
                let (Some(identity), Some(Ok(true))) = (user_identity, auth_init_status) else {
                    log!("USER_EMAIL_SYNC: Conditions not met - returning early");
                    return Ok::<_, String>("Auth not ready".to_string());
                };

                let Some(email) = identity.email.clone() else {
                    log!("USER_EMAIL_SYNC: User identity found but no email available");
                    return Ok("No email in identity".to_string());
                };

                log!("USER_EMAIL_SYNC: Found email to sync: {}", email);

                // Check if we already synced this email using AuthState
                let auth = auth_state_signal.get();
                if auth.is_email_mapping_synced(&email) {
                    log!("USER_EMAIL_SYNC: Email already synced, skipping: {}", email);
                    return Ok("Email already synced".to_string());
                }

                log!("USER_EMAIL_SYNC: Email not yet synced, calling backend API");

                // Call the canister to sync email mapping
                match update_user_principal_email_mapping_in_canister(email.clone()).await {
                    Ok(result) => {
                        log!(
                            "USER_EMAIL_SYNC: Successfully synced email with backend - result: {}",
                            result
                        );
                        // Mark email as synced in AuthState
                        auth.mark_email_mapping_synced(email.clone());
                        log!(
                            "USER_EMAIL_SYNC: Marked email as synced in AuthState: {}",
                            email
                        );
                        Ok(result)
                    }
                    Err(e) => {
                        log!(
                            "USER_EMAIL_SYNC: Failed to sync email with backend - error: {}",
                            e
                        );
                        Err(e.to_string())
                    }
                }
            }
        },
    );

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

    // Store children in StoredValue to make it accessible in the closure
    // let child_view = store_value(children());

    view! {
        <div>
            <Suspense>
                {move || {
                    let email_sync_status = user_email_sync_resource.get();
                    log!(
                        "USER_EMAIL_SYNC: Suspense check - resource status: {:?}", email_sync_status.is_some()
                    );
                    match email_sync_status {
                        Some(Ok(result)) => {
                            log!(
                                "USER_EMAIL_SYNC: Suspense - email sync completed successfully: {}", result
                            );

                            // Ensure the cookie is set after successful sync
                            let auth = auth_state_signal.get();
                            if let Some(identity) = auth.user_identity.get() {
                                if let Some(email) = identity.email {
                                    if !auth.is_email_mapping_synced(&email) {
                                        log!("USER_EMAIL_SYNC: Cookie not set, setting it now for email: {}", email);
                                        auth.mark_email_mapping_synced(email);
                                    } else {
                                        log!("USER_EMAIL_SYNC: Cookie already set for email: {}", email);
                                    }
                                }
                            }
                        }
                        Some(Err(e)) => {
                            log!("USER_EMAIL_SYNC: Suspense - email sync failed: {}", e);
                        }
                        None => {
                            log!("USER_EMAIL_SYNC: Suspense - email sync resource still loading");
                        }
                    }
                    view! { <></> }
                }}
            </Suspense>
            <Suspense fallback=move || {
                view! {
                    <div class="min-h-screen bg-gray-50 flex items-center justify-center">
                        <div class="text-center">
                            <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto mb-4"></div>
                            <p class="text-gray-600">Initializing authentication...</p>
                        </div>
                    </div>
                }
            }>
                {move || {
                    match auth_init_resource.get() {
                        Some(Ok(auth_initialized)) => {
                            log!(
                                "AUTH_FLOW: base_route - Suspense: Auth resource loaded successfully, authenticated: {}", auth_initialized
                            );
                            // child_view.with_value(|child| child.clone()).into_view()
                            view! { <></> }
                                .into_view()
                        }
                        Some(Err(e)) => {
                            log!("AUTH_FLOW: base_route - Suspense: Auth resource error: {}", e);
                            view! {
                                <div class="min-h-screen bg-gray-50 flex items-center justify-center">
                                    <div class="text-center">
                                        <p class="text-red-600 mb-2">Authentication Error</p>
                                        <p class="text-gray-500 text-sm">{e.to_string()}</p>
                                    </div>
                                </div>
                            }
                                .into_view()
                        }
                        None => {
                            log!("AUTH_FLOW: base_route - Suspense: Auth resource still loading");
                            view! { <></> }.into_view()
                        }
                    }
                }}
            </Suspense>

            {children()}
        </div>
    }
}

#[component]
pub fn BaseRoute() -> impl IntoView {
    view! {
        <CtxProvider>
            <Outlet />
        </CtxProvider>
    }
}
