use crate::{
    error_template::{AppError, ErrorTemplate},
    state::search_state::SearchCtx,
};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use crate::page::HotelDetailsPage;
use crate::page::HotelListPage;
use crate::page::RootPage;

#[component]
fn NotFound() -> impl IntoView {
    let mut outside_errors = Errors::default();
    outside_errors.insert_with_default_key(AppError::NotFound);
    view! { <ErrorTemplate outside_errors /> }
}

// ---------
// ROUTES
// ---------

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppRoutes {
    Root,
    HotelList,
    HotelDetails,
}

impl AppRoutes {
    pub fn to_string(&self) -> &'static str {
        match self {
            AppRoutes::Root => "",
            AppRoutes::HotelList => "/hotel-list",
            AppRoutes::HotelDetails => "/hotel-details",
        }
    }
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

        // import Figtree font
        <Link rel="preconnect" href="https://fonts.googleapis.com" />
        <Link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="true" />
        <Link
            href="https://fonts.googleapis.com/css2?family=Figtree:ital,wght@0,300..900;1,300..900&display=swap"
            rel="stylesheet"
        />

        // content for this welcome page
        <Router fallback=|| { view! { <NotFound /> }.into_view() }>
            <main>
                <Routes>
                    <Route path=AppRoutes::Root.to_string() view=RootPage />
                    <Route path=AppRoutes::HotelList.to_string() view=HotelListPage />
                    <Route path=AppRoutes::HotelDetails.to_string() view=HotelDetailsPage />
                </Routes>
            </main>
        </Router>
    }
}
