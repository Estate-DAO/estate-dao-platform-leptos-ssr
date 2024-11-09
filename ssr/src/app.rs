use crate::{
    api::consts::EnvVarConfig,
    error_template::{AppError, ErrorTemplate},
    page::{BlockRoomPage, ConfirmationPage, HotelDetailsPage, HotelListPage, RootPage},
    state::{
        search_state::{
            BlockRoomResults, ConfirmationResults, HotelInfoResults, SearchCtx, SearchListResults,
        },
        view_state::{BlockRoomCtx, HotelInfoCtx},
    },
};
use leptos::*;
use leptos_meta::*;
use leptos_query::{query_persister, *};
use leptos_query_devtools::LeptosQueryDevtools;
use leptos_router::*;

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
    BlockRoom,
    Confirmation,
}

impl AppRoutes {
    pub fn to_string(&self) -> &'static str {
        match self {
            AppRoutes::Root => "",
            AppRoutes::HotelList => "/hotel-list",
            AppRoutes::HotelDetails => "/hotel-details",
            AppRoutes::BlockRoom => "/block_room",
            Self::Confirmation => "/confirmation",
        }
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    // fallible component in app startup
    // -> if environment variables are not defined, panic!
    provide_context(EnvVarConfig::try_from_env());

    provide_context(SearchCtx::default());
    provide_context(SearchListResults::default());

    provide_context(HotelInfoCtx::default());
    provide_context(HotelInfoResults::default());

    provide_context(BlockRoomCtx::default());
    provide_context(BlockRoomResults::default());

    provide_context(ConfirmationResults::default()); // Add this line

    // Provides Query Client for entire app.
    // leptos_query::provide_query_client();
    provide_query_client_with_options_and_persister(
        Default::default(),
        query_persister::LocalStoragePersister,
    );

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
        <LeptosQueryDevtools/>
        // content for this welcome page
        <Router fallback=|| { view! { <NotFound /> }.into_view() }>
            <main>
                <Routes>
                    <Route path=AppRoutes::Root.to_string() view=RootPage />
                    <Route path=AppRoutes::HotelList.to_string() view=HotelListPage />
                    <Route path=AppRoutes::HotelDetails.to_string() view=HotelDetailsPage />
                    <Route path=AppRoutes::BlockRoom.to_string() view=BlockRoomPage />
                    <Route path=AppRoutes::Confirmation.to_string() view=ConfirmationPage />
                </Routes>
            </main>
        </Router>
    }
}
