pub fn get_host() -> String {
    #[cfg(feature = "hydrate")]
    {
        use leptos::prelude::window;
        return window().location().host().unwrap().to_string();
    }

    #[cfg(not(feature = "hydrate"))]
    {
        use axum::http::request::Parts;
        use leptos::prelude::{expect_context, use_context};

        let parts: Option<Parts> = use_context();
        if parts.is_none() {
            return "".to_string();
        }
        let headers = parts.unwrap().headers;
        headers.get("Host").unwrap().to_str().unwrap().to_string()
    }
}
