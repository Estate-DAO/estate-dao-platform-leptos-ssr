use codee::string::FromToStringCodec;
use leptos::*;
use leptos_use::{use_cookie_with_options, UseCookieOptions};

use crate::api::{auth::auth_state::auth_state, consts::yral_auth::ACCOUNT_CONNECTED_STORE};

pub fn go_to_home() {
    let path = "/";
    #[cfg(feature = "hydrate")]
    {
        let nav = leptos_router::use_navigate();
        nav(path, Default::default());
    }
    #[cfg(not(feature = "hydrate"))]
    {
        use leptos_axum::redirect;
        redirect(path);
    }
}

#[component]
pub fn LogoutHandler() -> impl IntoView {
    create_effect(move |_| {
        // <!-- Clear user identity and cookies -->
        let auth = auth_state();
        auth.reset_user_identity();

        // <!-- Clear ACCOUNT_CONNECTED_STORE cookie -->
        let (_, set_account_connected) = use_cookie_with_options::<bool, FromToStringCodec>(
            ACCOUNT_CONNECTED_STORE,
            UseCookieOptions::default()
                .path("/")
                .same_site(leptos_use::SameSite::Lax),
        );
        set_account_connected.set(None);

        // <!-- Redirect to home after a short delay -->
        #[cfg(feature = "hydrate")]
        {
            use leptos::set_timeout;
            use std::time::Duration;
            set_timeout(
                move || {
                    go_to_home();
                },
                Duration::from_millis(1500),
            );
        }
        #[cfg(not(feature = "hydrate"))]
        {
            go_to_home();
        }
    });

    view! {
        <div class="flex flex-col gap-10 justify-center items-center bg-gray-50 h-screen w-full">
            <div class="flex flex-col items-center gap-6">
                <div class="w-16 h-16 border-4 border-blue-500 border-t-transparent rounded-full animate-spin"></div>
                <h1 class="text-2xl font-semibold text-gray-700">Logging Out...</h1>
                <p class="text-gray-500 text-center max-w-md">
                    Please wait while we securely log you out of your account.
                </p>
            </div>
        </div>
    }
}

#[component]
pub fn LogoutPage() -> impl IntoView {
    let logout_resource = create_local_resource(
        || (),
        |_| async move {
            // <!-- Just return success immediately -->
            Ok::<(), String>(())
        },
    );

    view! {
        <Suspense fallback=move || {
            view! {
                <div class="flex flex-col gap-10 justify-center items-center bg-gray-50 h-screen w-full">
                    <div class="flex flex-col items-center gap-6">
                        <div class="w-16 h-16 border-4 border-blue-500 border-t-transparent rounded-full animate-spin"></div>
                        <h1 class="text-2xl font-semibold text-gray-700">Logging Out...</h1>
                        <p class="text-gray-500 text-center max-w-md">
                            Please wait while we securely log you out of your account.
                        </p>
                    </div>
                </div>
            }
        }>
            {move || {
                let _logout_res = logout_resource.get();
                view! { <LogoutHandler /> }
            }}
        </Suspense>
    }
}
