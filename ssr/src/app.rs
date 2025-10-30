use crate::api::auth::auth_state::AuthStateSignal;
use crate::facebook::FacebookPixel;
use crate::page::{AboutUsPage, MyAccountPage};
use crate::{
    api::{
        consts::{EnvVarConfig, APP_URL},
        payments::ports::GetPaymentStatusResponse,
    },
    component::{
        DataTableCtx, ErrorPopup, GA4ScriptAsync, GoogleTagManagerIFrame, NotificationState,
    },
    error_template::{AppError, ErrorTemplate},
    page::{
        AdminEditPanel, AdminPanelPage, BlockRoomPage, BlockRoomV1Page, ConfirmationPage,
        /* ConfirmationPageV1, */ ConfirmationPageV2, HotelDetailsPage, HotelDetailsV1Page,
        HotelListPage, MyBookingsPage, PreviousSearchContext, RootPage, SSEConfirmationPage,
        WishlistPage,
    },
    view_state_layer::{
        api_error_state::ApiErrorState,
        booking_context_state::BookingContextState,
        booking_id_state::BookingIdState,
        confirmation_results_state::ConfirmationResultsState,
        cookie_booking_context_state::CookieBookingContextState,
        email_verification_state::EmailVerificationState,
        hotel_details_state::PricingBookNowState,
        input_group_state::InputGroupState,
        my_bookings_state::MyBookingsState,
        // search_state::{
        //     BlockRoomResults, ConfirmationResults, HotelInfoResults, SearchCtx, SearchListResults,
        // },
        ui_block_room::BlockRoomUIState,
        ui_confirmation_page::ConfirmationPageUIState,
        ui_confirmation_page_v2::ConfirmationPageState,
        ui_hotel_details::HotelDetailsUIState,
        ui_search_state::{SearchListResults, UIPaginationState, UISearchCtx},
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
    MyBookings,
    MyAccount, // Notifications
    Wishlist,
    AboutUs,
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
            AppRoutes::MyBookings => "/my-bookings",
            AppRoutes::MyAccount => "/account",
            AppRoutes::Wishlist => "/wishlist",
            AppRoutes::AboutUs => "/about-us",
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
            Self::MyBookings,
            Self::Wishlist,
            Self::AboutUs,
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

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {


                // <script
                //     inner_html=r#"
                //         window.addEventListener('load', function() {
                //         !function(f,b,e,v,n,t,s)
                //         {if(f.fbq)return;n=f.fbq=function(){n.callMethod?
                //         n.callMethod.apply(n,arguments):n.queue.push(arguments)};
                //         if(!f._fbq)f._fbq=n;n.push=n;n.loaded=!0;n.version='2.0';
                //         n.queue=[];t=b.createElement(e);t.async=!0;
                //         t.src=v;s=b.getElementsByTagName(e)[0];
                //         s.parentNode.insertBefore(t,s)}(window, document,'script',
                //         'https://connect.facebook.net/en_US/fbevents.js');
                //         fbq('init', '1720214695361495');
                //         fbq('track', 'PageView');
                //         });
                //         "#
                // ></script>


            <App />
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    // fallible component in app startup
    // -> if environment variables are not defined, panic!
    // provide_context(get_yral_oauth_client());

    let _ = AuthStateSignal::init();

    // Ensure canonical domain (redirect www.nofeebooking.com -> nofeebooking.com)
    crate::utils::domain_normalize::ensure_canonical_domain();

    provide_context(InputGroupState::default());

    provide_context(UIPaginationState::default());

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
    provide_context(CookieBookingContextState::default());

    // provide_context(PaymentBookingStatusUpdates::default());
    // provide_context(SSEBookingStatusUpdates::default());

    provide_context(ApiErrorState::default());

    // Booking ID state
    provide_context(BookingIdState::default());

    // Email verification state
    provide_context(EmailVerificationState::default());

    // My Bookings state
    provide_context(MyBookingsState::default());

    provide_context(DataTableCtx::default());
    // provide_context(NotificationState::default());

    // Provides Query Client for entire app.
    // leptos_query::provide_query_client();
    provide_query_client_with_options_and_persister(
        Default::default(),
        query_persister::LocalStoragePersister,
    );

    view! {

        <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />

                <Meta property="og:title" content="NoFeeBooking - Book Hotels Without Hidden Fees" />
                <Meta property="og:image" content="/img/logo_white.svg" />

                <Stylesheet id="leptos" href="/pkg/estate-fe.css" />
                <Link rel="preconnect" href="https://fonts.googleapis.com" />
                <Link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="true" />
                <Link
                    href="https://fonts.googleapis.com/css2?family=Figtree:ital,wght@0,300..900;1,300..900&display=swap"
                    rel="stylesheet"
                />

                <FacebookPixel />
                 <noscript>
                    <img
                        height="1"
                        width="1"
                        style="display:none"
                        src="https://www.facebook.com/tr?id=1720214695361495&ev=PageView&noscript=1"
                    />
                </noscript>

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
                        <Route path=AppRoutes::MyBookings.to_string() view=MyBookingsPage />
                        <Route path=AppRoutes::MyAccount.to_string() view=MyAccountPage />
                        <Route path=AppRoutes::Wishlist.to_string() view=WishlistPage />
                        <Route path=AppRoutes::AboutUs.to_string() view=AboutUsPage />
                        // <Route path=AppRoutes::Confirmation.to_string() view=ConfirmationPage />
                        // <Route path=AppRoutes::Notifications.to_string() view=NotificationExample />
                </Routes>
        </Router>
        </main>

        // </Body>

    }
}
