use crate::utils::admin::admin_canister;
// use leptos::logging::log;
use crate::log;
use leptos::*;

#[server(GreetBackend)]
pub async fn greet_backend(name: String) -> Result<String, ServerFnError> {
    let adm_cans = admin_canister();

    let backend_cans = adm_cans.backend_canister().await;

    let result = backend_cans.greet("STRINGGGGGGGG".into()).await;

    log!("{result:?}");

    Ok("Got it!".into())
}
