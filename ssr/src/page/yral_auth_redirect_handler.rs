use leptos::*;

use crate::api::{auth::types::YralAuthMessage, client_side_api::ClientSideApiClient};
use crate::log;
use leptos_router::{use_query, Params};
use serde::{Deserialize, Serialize};

#[derive(Params, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct OAuthQuery {
    pub code: String,
    pub state: String,
}

pub fn go_to_root() {
    let path = "/";
    #[cfg(feature = "hydrate")]
    {
        let nav = leptos_router::use_navigate();
        nav(path, Default::default());
    }
    #[cfg(not(feature = "hydrate"))]
    {
        use leptos_axum::redirect;
        redirect(&path);
    }
}

#[component]
pub fn IdentitySender(identity_res: Option<YralAuthMessage>) -> impl IntoView {
    create_effect(move |_| {
        if let None = &identity_res {
            log!("IdentitySender - identity_res: None");
            return;
        }

        let _id = identity_res.as_ref().unwrap();
        #[cfg(feature = "hydrate")]
        {
            use web_sys::Window;

            let win = window();
            let origin = win.origin();
            let opener = win.opener().expect("window opener value ");
            if opener.is_null() {
                go_to_root();
            }
            let opener = Window::from(opener);
            log!("IdentitySender - opener: {:#?}", opener);
            let msg = serde_json::to_string(&_id).expect("serde_json::to_string failed to unwrap");
            log!("IdentitySender - msg: {:#?}", msg);
            _ = opener.post_message(&msg.into(), &origin);
        }
    });

    view! {
        <div class="flex flex-col gap-10 justify-center items-center bg-black h-dvh w-dvw">
            <img class="object-contain w-56 h-56 animate-pulse" src="/img/yral/logo.webp" />
            <span class="text-2xl text-white/60">Good things come to those who wait...</span>
        </div>
    }
}

#[component]
pub fn YralAuthRedirectHandlerPage() -> impl IntoView {
    let query = use_query::<OAuthQuery>();
    log!("oauth query: {:#?}", query);
    let identity_resource = create_local_resource(query, |query_res| async move {
        let Ok(oauth_query) = query_res else {
            return Err("Invalid query".to_string());
        };

        handle_oauth_query(oauth_query).await
    });

    view! {
    <Suspense>
        {move || {
            let identity_res = identity_resource.get();
            log!("identity_res: {:#?}", identity_res);
            view! { <IdentitySender identity_res /> }
        }}
    </Suspense>
    }
}

async fn handle_oauth_query(oauth_query: OAuthQuery) -> YralAuthMessage {
    tracing::info!(
        "Received OAuth callback: code={}, state={}",
        oauth_query.code,
        oauth_query.state
    );
    let client = ClientSideApiClient::default();
    let delegated = client
        .perform_yral_oauth(oauth_query)
        .await
        .map_err(|e| e.to_string())?;
    Ok(delegated)
}
