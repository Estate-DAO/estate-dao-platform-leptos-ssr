use leptos::ev;
use leptos::prelude::*;
use leptos_router::hooks::use_location;
use web_sys::js_sys;

use crate::logging::{is_browser, is_server};

const FACEBOOK_PIXEL_ID: &str = "1720214695361495";

#[component]
pub fn FacebookPixel() -> impl IntoView {
    // Signal to ensure initialization happens only once
    let (initialized, set_initialized) = signal(false);

    // Run once on browser
    Effect::new(move |_| {
        if is_browser() && !initialized.get() {
            let js_code = format!(
                r#"
                !function(f,b,e,v,n,t,s)
                {{
                    if(f.fbq) return;
                    n=f.fbq=function(){{n.callMethod?
                        n.callMethod.apply(n,arguments):n.queue.push(arguments)}};
                    if(!f._fbq) f._fbq=n;
                    n.push=n;
                    n.loaded=!0;
                    n.version='2.0';
                    n.queue=[];
                    t=b.createElement(e); t.async=!0;
                    t.src=v;
                    s=b.getElementsByTagName(e)[0];
                    s.parentNode.insertBefore(t,s);
                }}(window, document,'script','https://connect.facebook.net/en_US/fbevents.js');
                fbq('init', '{}');
                fbq('track', 'PageView');
                "#,
                FACEBOOK_PIXEL_ID
            );

            _ = js_sys::eval(&js_code).ok();
            set_initialized.set(true);
        }
    });

    // Track route changes
    #[cfg(feature = "hydrate")]
    {
        Effect::new(move |_| {
            let location = use_location();
            _ = location.pathname.get();

            if is_browser() {
                _ = js_sys::eval("window.fbq && fbq('track', 'PageView');").ok();
            }
        });
    }

    // Return meta tag only; no separate script tag needed
    view! {
        <meta name="facebook-domain-verification" content="mpv8l4xgm70yb70avrye8pggdegv3r" />
    }
}
