use codee::string::FromToStringCodec;
use ic_agent::identity::DelegatedIdentity;
use leptos::*;
// use leptos::{ev, prelude::*, component, view, server, ServerFnError, expect_context, create_action, window, IntoView, Children, Oco};
use leptos_use::{storage::use_local_storage, use_event_listener, use_interval_fn, use_window};
use web_sys::Window;

use crate::api::{
    auth::types::{LoginProvider, NewIdentity, ProviderKind},
    client_side_api::ClientSideApiClient,
};

// use super::auth::{NewIdentity, LoginProvider, ProviderKind, YralAuthMessage, YralOAuthClient, yral_auth_url_impl};
// use super::{LoginProvButton, LoginProvCtx, ProviderKind};

#[derive(Clone, Copy)]
pub struct LoginProvCtx {
    /// Setting processing should only be done on login cancellation
    /// and inside [LoginProvButton]
    /// stores the current provider handling the login
    pub processing: RwSignal<Option<ProviderKind>>,
    pub set_processing: RwSignal<Option<ProviderKind>>,
    pub login_complete: RwSignal<Option<NewIdentity>>,
}

impl Default for LoginProvCtx {
    fn default() -> Self {
        Self {
            processing: create_rw_signal(None),
            set_processing: create_rw_signal(None),
            login_complete: create_rw_signal(None),
        }
    }
}

/// Login providers must use this button to trigger the login action
/// automatically sets the processing state to true
#[component]
fn LoginProvButton<Cb: Fn(ev::MouseEvent) + 'static>(
    prov: ProviderKind,
    #[prop(into)] class: Oco<'static, str>,
    on_click: Cb,
    #[prop(optional, into)] disabled: Signal<bool>,
    children: Children,
) -> impl IntoView {
    let ctx: LoginProvCtx = expect_context();

    // let click_action = Action::new(move |()| async move {
    //     // LoginMethodSelected.send_event(prov);
    // });

    view! {
        <button
            disabled=move || ctx.processing.get().is_some() || disabled()
            class=class
            on:click=move |ev| {
                ctx.set_processing.set(Some(prov));
                on_click(ev);
                // click_action.dispatch(());
            }
        >

            {children()}
        </button>
    }
}

// #[server]
// async fn yral_auth_login_url(
//     login_hint: String,
//     provider: LoginProvider,
// ) -> Result<String, ServerFnError> {
//     let oauth2: YralOAuthClient = expect_context();
//     let url = yral_auth_url_impl(oauth2, login_hint, provider, None).await?;
//     Ok(url)
// }

#[component]
pub fn YralAuthProvider() -> impl IntoView {
    let ctx: LoginProvCtx = expect_context();
    let signing_in = move || ctx.processing.get() == Some(ProviderKind::YralAuth);
    let signing_in_provider = create_rw_signal(LoginProvider::Google);
    let done_guard = create_rw_signal(false);
    // let close_popup_store = StoredValue::new(None::<Callback<()>>);
    // let close_popup =
    //     move || _ = close_popup_store.with_value(|cb| cb.as_ref().map(|close_cb| close_cb.run(())));
    // let (_, set_notifs_enabled, _) =
    //     use_local_storage::<bool, FromToStringCodec>(NOTIFICATIONS_ENABLED_STORE);

    // let auth = auth_state();

    // let open_yral_auth = Action::new_unsync_local(
    let open_yral_auth = create_action(
        move |(target, origin, provider): &(Window, String, LoginProvider)| {
            let target = target.clone();
            let origin = origin.clone();
            let provider = provider.clone();

            let url_fut = async move {
                // let id = auth.user_identity.await?;
                // let id = DelegatedIdentity::try_from(id.id_wire)?;
                // let login_hint = yral_auth_login_hint(&id)?;
                // let login_hint = "".to_string();
                let client = ClientSideApiClient::new();
                client.yral_auth_login_url(provider).await

                // yral_auth_login_url(provider).await
            };

            async move {
                let url = match url_fut.await {
                    Ok(url) => url,
                    Err(e) => {
                        format!("{origin}/error?err={e}")
                    }
                };
                target
                    .location()
                    .replace(&url)
                    .expect("Failed to open Yral Auth?!");
            }
        },
    );

    let on_click = move |provider: LoginProvider, auth_journey: &str| {
        let window = window();
        let origin = window.origin();

        // if let Some(global) = MixpanelGlobalProps::from_ev_ctx(auth.event_ctx()) {
        //     MixPanelEvent::track_auth_initiated(global, auth_journey.to_string());
        // }
        // open a target window
        let target = window.open().transpose().and_then(|w| w.ok()).unwrap();

        // load yral auth url in background
        open_yral_auth.dispatch((target.clone(), origin.clone(), provider));

        // Check if the target window was closed by the user
        let target_c = target.clone();
        let pause = use_interval_fn(
            move || {
                // Target window was closed by user
                if target_c.closed().unwrap_or_default() && !done_guard.try_get().unwrap_or(true) {
                    ctx.set_processing.try_set(None);
                }
            },
            500,
        );

        // todo can we remove this event_listener?
        // _ = use_event_listener(use_window(), ev::message, move |msg| {
        //     if msg.origin() != origin {
        //         return;
        //     }

        //     let Some(data) = msg.data().as_string() else {
        //         log::warn!("received invalid message: {:?}", msg.data());
        //         return;
        //     };
        //     let res = match serde_json::from_str::<YralAuthMessage>(&data)
        //         .map_err(|e| e.to_string())
        //         .and_then(|r| r)
        //     {
        //         Ok(res) => res,
        //         Err(e) => {
        //             log::warn!("error processing {e:?}. msg {data}");
        //             // close_popup();
        //             return;
        //         }
        //     };
        //     done_guard.set(true);
        //     (pause.pause)();
        //     _ = target.close();
        //     ctx.set_processing.set(None);
        //     // set_notifs_enabled.set(false);
        //     ctx.login_complete.set(Some(res));
        // });
    };

    view! {
        <LoginProvButton
            prov=ProviderKind::YralAuth
            class="flex gap-3 justify-center items-center p-3 w-full font-bold text-black bg-white rounded-md hover:bg-white/95"
            on_click=move |ev| {
                ev.stop_propagation();
                signing_in_provider.set(LoginProvider::Google);
                // MixpanelGlobalProps::set_auth_journey("google".to_string());
                on_click(signing_in_provider.get(), "google");
            }
        >
            <img class="size-5" src="/img/common/google.svg" />
            <span>
                {format!(
                    "{}Google",
                    if signing_in() && signing_in_provider.get() == LoginProvider::Google {
                        "Logging in with "
                    } else {
                        "Login with "
                    },
                )}
            </span>
        </LoginProvButton>
        <LoginProvButton
            prov=ProviderKind::YralAuth
            class="flex gap-3 justify-center items-center py-3 w-full font-bold text-black bg-white rounded-md hover:bg-white/95"
            on_click=move |ev| {
                ev.stop_propagation();
                signing_in_provider.set(LoginProvider::Apple);
                // MixpanelGlobalProps::set_auth_journey("apple".to_string());
                on_click(signing_in_provider.get(), "apple");
            }
        >
            <img class="size-5" src="/img/common/apple.svg" />
            <span>
                {format!(
                    "{}Apple",
                    if signing_in() && signing_in_provider.get() == LoginProvider::Apple {
                        "Logging in with "
                    } else {
                        "Login with "
                    },
                )}
            </span>
        </LoginProvButton>
    }
}
