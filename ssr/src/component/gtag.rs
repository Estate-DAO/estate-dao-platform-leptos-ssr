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

#[component]
pub fn GoogleTagManagerScriptAsync() -> impl IntoView {
    // Analytics
    let enable_ga4_script = create_rw_signal(false);
    #[cfg(feature = "ga4")]
    {
        enable_ga4_script.set(true);
    }

    view! {
        // Google Tag Manager (GTM) Script
        <Show when=enable_ga4_script>
            <Script async_="true">
                {r#"
                (function(w,d,s,l,i){
                  w[l]=w[l]||[];
                  w[l].push({'gtm.start': new Date().getTime(),event:'gtm.js'});
                  var f=d.getElementsByTagName(s)[0],
                      j=d.createElement(s),dl=l!='dataLayer'?'&l='+l:'';
                  j.async=true;
                  j.src='https://www.googletagmanager.com/gtm.js?id='+i+dl;
                  f.parentNode.insertBefore(j,f);
                })(window,document,'script','dataLayer','GTM-PFCRL3ZG');
                "#}
            </Script>
        </Show>
    }
}
