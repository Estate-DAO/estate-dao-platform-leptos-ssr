// cfg_if::cfg_if! {
//     if #[cfg(feature = "ssr")] {
//         use crate::api::provab::Provab;
//         use crate::adapters::ProvabAdapter;
//     }
// }

use crate::{
    api::{
        consts::{EnvVarConfig, APP_URL},
        payments::ports::GetPaymentStatusResponse,
    },
    // application_services::hotel_service::HotelService,
    component::{
        DataTableCtx, ErrorPopup, GA4ScriptAsync, GoogleTagManagerIFrame, NotificationExample,
        NotificationState,
    },
    error_template::{AppError, ErrorTemplate},
    page::{
        AdminEditPanel, AdminPanelPage, BlockRoomPage, BlockRoomV1Page, ConfirmationPage,
        ConfirmationPageV1, ConfirmationPageV2, HotelDetailsPage, HotelDetailsV1Page,
        HotelListPage, PreviousSearchContext, RootPage, SSEConfirmationPage,
    },
    view_state_layer::{
        api_error_state::ApiErrorState,
        booking_context_state::BookingContextState,
        confirmation_results_state::ConfirmationResultsState,
        hotel_details_state::PricingBookNowState,
        input_group_state::InputGroupState,
        // search_state::{
        //     BlockRoomResults, ConfirmationResults, HotelInfoResults, SearchCtx, SearchListResults,
        // },
        ui_block_room::BlockRoomUIState,
        ui_confirmation_page::ConfirmationPageUIState,
        ui_confirmation_page_v2::ConfirmationPageState,
        ui_hotel_details::HotelDetailsUIState,
        ui_search_state::{SearchListResults, UISearchCtx},
        view_state::{BlockRoomCtx, HotelInfoCtx},
    },
};
use chrono::prelude::*;
use leptos::*;
use leptos_meta::*;
use leptos_query::{query_persister, *};
use leptos_query_devtools::LeptosQueryDevtools;
use leptos_router::*;
use sitewriter::{ChangeFreq, UrlEntry};
use std::sync::{Arc, OnceLock};

static SITEMAP: OnceLock<String> = OnceLock::new();

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
    // ConfirmationV1,
    // ConfirmationV2,
    AdminPanel,
    AdminEditPanel,
    // Notifications
}

impl AppRoutes {
    pub fn to_string(&self) -> &'static str {
        match self {
            AppRoutes::Root => "",
            AppRoutes::HotelList => "/hotel-list",
            AppRoutes::HotelDetails => "/hotel-details",
            AppRoutes::BlockRoom => "/block_room",
            AppRoutes::Confirmation => "/confirmation",
            // AppRoutes::ConfirmationV1 => "/confirmation-v1",
            // AppRoutes::ConfirmationV2 => "/confirmation-v2",
            AppRoutes::AdminPanel => "/admin-panel",
            AppRoutes::AdminEditPanel => "/admin-edit-panel",
            // AppRoutes::Notifications => "/notifications"
        }
    }

    pub fn all() -> impl Iterator<Item = Self> {
        [
            Self::Root,
            Self::HotelList,
            Self::HotelDetails,
            Self::BlockRoom,
            Self::Confirmation,
            // Self::ConfirmationV1,
            // Self::ConfirmationV2,
            Self::AdminPanel,
            Self::AdminEditPanel,
            // Self::Notifications,
        ]
        .into_iter()
    }

    fn get_priority(&self) -> f32 {
        match self {
            AppRoutes::Root => 1.0,
            AppRoutes::HotelList => 0.9,
            AppRoutes::HotelDetails => 0.8,
            AppRoutes::BlockRoom => 0.7,
            AppRoutes::Confirmation => 0.6,
            // AppRoutes::ConfirmationV1 => 0.6,
            // AppRoutes::ConfirmationV2 => 0.6,
            _ => 0.1,
            // AppRoutes::Notifications => 0.5,
        }
    }

    fn get_change_freq(&self) -> ChangeFreq {
        match self {
            // AppRoutes::Notifications => ChangeFreq::Weekly,
            _ => ChangeFreq::Weekly,
        }
    }

    pub fn generate_sitemap() -> &'static str {
        SITEMAP.get_or_init(|| {
            let urls: Vec<UrlEntry> = Self::all()
                .map(|route| UrlEntry {
                    loc: format!("{}{}", APP_URL.as_str(), route.to_string())
                        .parse()
                        .unwrap(),
                    changefreq: Some(route.get_change_freq()),
                    priority: Some(route.get_priority()),
                    lastmod: Some(Utc::now()),
                })
                .collect();

            sitewriter::generate_str(&urls)
        })
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

    provide_context(PreviousSearchContext::default());
    provide_context(UISearchCtx::default());
    provide_context(SearchListResults::default());

    provide_context(HotelInfoCtx::default());
    provide_context(HotelDetailsUIState::default());
    // provide_context(HotelInfoResults::default());
    // NEW booking context for hotel_details page
    provide_context(PricingBookNowState::default());

    provide_context(BlockRoomCtx::default());
    provide_context(BlockRoomUIState::default());
    // provide_context(BlockRoomResults::default());

    // provide_context(ConfirmationResults::default());
    provide_context(ConfirmationResultsState::default());
    provide_context(ConfirmationPageUIState::default());
    provide_context(ConfirmationPageState::default());
    provide_context(BookingContextState::default());

    // provide_context(PaymentBookingStatusUpdates::default());
    // provide_context(SSEBookingStatusUpdates::default());

    provide_context(ApiErrorState::default());

    provide_context(DataTableCtx::default());
    // provide_context(NotificationState::default());

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
        <meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no" />

        // import Figtree font
        <Link rel="preconnect" href="https://fonts.googleapis.com" />
        <Link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="true" />
        <Link
            href="https://fonts.googleapis.com/css2?family=Figtree:ital,wght@0,300..900;1,300..900&display=swap"
            rel="stylesheet"
        />
        <LeptosQueryDevtools />

        <GA4ScriptAsync />

        // <Body>
        <main>
            // <GoogleTagManagerIFrame />

        // content for this welcome page
        <Router fallback=|| { view! { <NotFound /> }.into_view() }>
                <Routes>
                    <Route path=AppRoutes::Root.to_string() view=RootPage />
                    <Route path=AppRoutes::HotelList.to_string() view=HotelListPage />
                    <Route path=AppRoutes::HotelDetails.to_string() view=HotelDetailsV1Page />
                    <Route path=AppRoutes::BlockRoom.to_string() view=BlockRoomV1Page />
                    <Route path=AppRoutes::Confirmation.to_string() view=ConfirmationPageV2 />
                    // <Route path=AppRoutes::ConfirmationV1.to_string() view=ConfirmationPageV1 />
                    // <Route path=AppRoutes::ConfirmationV2.to_string() view=ConfirmationPageV2 />
                    <Route path=AppRoutes::AdminPanel.to_string() view=AdminPanelPage />
                    <Route path=AppRoutes::AdminEditPanel.to_string() view=AdminEditPanel />
                    // <Route path=AppRoutes::Confirmation.to_string() view=ConfirmationPage />
                    // <Route path=AppRoutes::Notifications.to_string() view=NotificationExample />
                </Routes>
        </Router>
        </main>

        // </Body>

    }
}
