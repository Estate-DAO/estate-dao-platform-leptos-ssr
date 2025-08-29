use leptos::*;
use leptos_meta::Script;

use crate::api::consts::GTAG_MEASUREMENT_ID;

#[component]
pub fn GoogleTagManagerIFrame() -> impl IntoView {
    let gtag_measurement_id: &str = GTAG_MEASUREMENT_ID.as_ref();

    view! {
        <iframe
            src=format!("https://www.googletagmanager.com/ns.html?id={gtag_measurement_id}")
            height="0"
            width="0"
            style="display:none;visibility:hidden"
        ></iframe>
    }
}

// #[component]
// pub fn GoogleTagManagerScriptAsync() -> impl IntoView {
//     // Analytics
//     let enable_ga4_script = create_rw_signal(false);
//     #[cfg(feature = "ga4")]
//     {
//         enable_ga4_script.set(true);
//     }
//     view! {
//         // GA4 Script

//         <Show when=enable_ga4_script>
//             <Script
//                 async_="true"
//                 src=concat!("https://www.googletagmanager.com/gtag/js?id=", "G-BPRVSPTP2T")
//             />
//             <Script>
//                 {r#"
//                     window.dataLayer = window.dataLayer || [];
//                     function gtag(){dataLayer.push(arguments);}
//                     gtag('js', new Date());
//                     gtag('config', 'G-BPRVSPTP2T');
//                 "#}
//             </Script>
//         </Show>
//     }
// }

#[component]
pub fn GA4ScriptAsync() -> impl IntoView {
    // Analytics
    let enable_ga4_script = create_rw_signal(false);
    #[cfg(feature = "ga4")]
    {
        enable_ga4_script.set(true);
    }
    view! {
        // GA4 Script

        <Show when=enable_ga4_script>
            <Script
                async_="true"
                src=concat!("https://www.googletagmanager.com/gtag/js?id=", "G-BPRVSPTP2T")
            />
            <Script>
                {r#"
                    window.dataLayer = window.dataLayer || []; 
                    function gtag(){dataLayer.push(arguments);} 
                    gtag('js', new Date()); 
                    gtag('config', 'G-BPRVSPTP2T'); 
                "#}
            </Script>
        </Show>
    }
}
