use crate::{
    api::{consts::EnvVarConfig, payments::ports::GetPaymentStatusResponse},
    component::{
        ErrorPopup, GoogleTagManagerIFrame, GoogleTagManagerScriptAsync, NotificationExample,
        NotificationState,
    },
    error_template::{AppError, ErrorTemplate},
    page::{
        BlockRoomPage, ConfirmationPage, HotelDetailsPage, HotelListPage,
        PaymentBookingStatusUpdates, RootPage, SSEBookingStatusUpdates, SSEConfirmationPage,
    },
    state::{
        api_error_state::ApiErrorState,
        confirmation_results_state::ConfirmationResultsState,
        hotel_details_state::PricingBookNowState,
        input_group_state::InputGroupState,
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
    Notifications,
}

impl AppRoutes {
    pub fn to_string(&self) -> &'static str {
        match self {
            AppRoutes::Root => "",
            AppRoutes::HotelList => "/hotel-list",
            AppRoutes::HotelDetails => "/hotel-details",
            AppRoutes::BlockRoom => "/block_room",
            AppRoutes::Confirmation => "/confirmation",
            AppRoutes::Notifications => "/notifications",
        }
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    // fallible component in app startup
    // -> if environment variables are not defined, panic!
    // provide_context(EnvVarConfig::try_from_env());

    provide_context(InputGroupState::default());

    provide_context(SearchCtx::default());
    provide_context(SearchListResults::default());

    provide_context(HotelInfoCtx::default());
    provide_context(HotelInfoResults::default());
    // NEW booking context for hotel_details page
    provide_context(PricingBookNowState::default());

    provide_context(BlockRoomCtx::default());
    provide_context(BlockRoomResults::default());

    provide_context(ConfirmationResults::default());
    provide_context(ConfirmationResultsState::default());

    provide_context(PaymentBookingStatusUpdates::default());
    provide_context(SSEBookingStatusUpdates::default());

    provide_context(ApiErrorState::default());

    provide_context(NotificationState::default());

    // Provides Query Client for entire app.
    // leptos_query::provide_query_client();
    provide_query_client_with_options_and_persister(
        Default::default(),
        query_persister::LocalStoragePersister,
    );

    view! {
        <Stylesheet id="leptos" href="/pkg/estate-fe.css" />

        // sets the document title
        <Title text="NoFeeBooking" />

        // import Figtree font
        <Link rel="preconnect" href="https://fonts.googleapis.com" />
        <Link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="true" />
        <Link
            href="https://fonts.googleapis.com/css2?family=Figtree:ital,wght@0,300..900;1,300..900&display=swap"
            rel="stylesheet"
        />
        <LeptosQueryDevtools />

        <GoogleTagManagerScriptAsync />

        // <Body>
        <main>
            <GoogleTagManagerIFrame />

        // content for this welcome page
        <Router fallback=|| { view! { <NotFound /> }.into_view() }>
                <Routes>
                    <Route path=AppRoutes::Root.to_string() view=RootPage />
                    <Route path=AppRoutes::HotelList.to_string() view=HotelListPage />
                    <Route path=AppRoutes::HotelDetails.to_string() view=HotelDetailsPage />
                    <Route path=AppRoutes::BlockRoom.to_string() view=BlockRoomPage />
                    <Route path=AppRoutes::Confirmation.to_string() view=SSEConfirmationPage />
                    // <Route path=AppRoutes::Confirmation.to_string() view=ConfirmationPage />
                    <Route path=AppRoutes::Notifications.to_string() view=NotificationExample />
                </Routes>
        </Router>
        </main>

        // </Body>

    }
}
