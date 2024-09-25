use crate::{error_template::{AppError, ErrorTemplate}, page::location_search::SearchCtx};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use crate::page::RootPage;

#[component]
fn NotFound() -> impl IntoView {
    let mut outside_errors = Errors::default();
    outside_errors.insert_with_default_key(AppError::NotFound);
    view! { <ErrorTemplate outside_errors /> }
}


#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    provide_context(SearchCtx::default());
    view! {
        <Stylesheet id="leptos" href="/pkg/estate-fe.css" />

        // sets the document title
        <Title text="Estate DAO" />

        // content for this welcome page
        <Router fallback=|| { view! { <NotFound /> }.into_view() }>
            <main>
                <Routes>
                    <Route path="" view=RootPage />
                // <Route path="/hotel-details" view=HotelDetailsPage/>
                </Routes>
            </main>
        </Router>
    }
}
