use leptos::*;
use leptos_router::*;

use crate::{
    api::auth::auth_state::AuthStateSignal,
    app::AppRoutes,
    component::{my_account_tabs::*, FaqView, Navbar, SocialLinks},
};

#[derive(Copy, Clone, PartialEq)]
pub enum AccountTabs {
    PersonalInfo,
    Wallet,
    Wishlist,
    Support,
    Terms,
    Privacy,
    Faq,
}

impl AccountTabs {
    pub fn from_str(s: &String) -> Self {
        match s.as_str() {
            "wallet" => AccountTabs::Wallet,
            "wishlist" => AccountTabs::Wishlist,
            "support" => AccountTabs::Support,
            "tnc" => AccountTabs::Terms,
            "privacy_policy" => AccountTabs::Privacy,
            "faq" => AccountTabs::Faq,
            _ => AccountTabs::PersonalInfo,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            AccountTabs::PersonalInfo => "personal",
            AccountTabs::Wallet => "wallet",
            AccountTabs::Wishlist => "wishlist",
            AccountTabs::Support => "support",
            AccountTabs::Terms => "tnc",
            AccountTabs::Privacy => "privacy_policy",
            AccountTabs::Faq => "faq",
        }
    }

    pub fn as_route(&self) -> String {
        format!(
            "{}?page={}",
            AppRoutes::MyAccount.to_string(),
            self.as_str()
        )
    }

    pub fn disabled(&self) -> bool {
        matches!(
            self,
            AccountTabs::PersonalInfo
                | AccountTabs::Wallet
                | AccountTabs::Wishlist
                | AccountTabs::Support
        )
    }

    pub fn label(&self) -> &'static str {
        match self {
            AccountTabs::PersonalInfo => "Personal Information",
            AccountTabs::Wallet => "Wallet",
            AccountTabs::Wishlist => "Wishlist",
            AccountTabs::Support => "Support",
            AccountTabs::Terms => "Terms & Conditions",
            AccountTabs::Privacy => "Privacy Policy",
            AccountTabs::Faq => "FAQ's",
        }
    }
}
#[component]
pub fn MyAccountPage() -> impl IntoView {
    let query = use_query_map();
    let active_tab = move || {
        query.with(|q| {
            q.get("page")
                .map(AccountTabs::from_str)
                .unwrap_or(AccountTabs::PersonalInfo)
        })
    };

    let (sidebar_open, set_sidebar_open) = create_signal(false);

    view! {
        {/* Navbar */}
        <div class="sticky top-0 lg:z-40 bg-white"><Navbar /></div>

        <div class="min-h-screen bg-gray-50 flex flex-col">
            <main class="flex-1 container mx-auto px-4 py-8 flex gap-6">
                {/* Sidebar toggle (mobile only) */}
                <button
                    class="lg:hidden p-2 bg-white rounded-md border h-10"
                    on:click=move |_| set_sidebar_open.set(true)
                >
                    <span class="sr-only">Open sidebar</span>
                    <svg class="h-6 w-6 text-gray-700" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"/>
                    </svg>
                </button>

                {/* Sidebar (lg screens inline, sm screens as modal) */}
                <aside
                    class=move || {
                        if sidebar_open.get() {
                            // Mobile modal mode
                            "fixed inset-0 z-30 flex items-center justify-center lg:inset-auto"
                                .to_string()
                        } else {
                            // Hidden on mobile, visible inline on lg
                            "hidden lg:flex lg:w-64".to_string()
                        }
                    }
                >
                    <div class="bg-white rounded-xl border p-6 w-64 shadow-sm">
                        {/* Close button (mobile only) */}
                        <button
                            class="lg:hidden top-4 right-4 text-gray-600"
                            on:click=move |_| set_sidebar_open.set(false)
                        >
                            <svg class="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
                            </svg>
                        </button>
                        <Show when=move|| AuthStateSignal::auth_state().get().is_authenticated()>
                            {   let user = AuthStateSignal::auth_state().get();
                                view!{
                                    <div class="flex flex-col items-center mt-2">
                                        <img
                                            src=user.picture.unwrap_or_else(|| "https://i.pravatar.cc/100".into())
                                            alt="profile"
                                            class="h-20 w-20 rounded-full mb-4"
                                        />
                                        <p class="text-gray-800 font-medium mb-6">{user.email}</p>
                                    </div>
                                }
                            }
                        </Show>

                        <nav class="flex flex-col space-y-4 w-full text-gray-700">
                            {[
                                AccountTabs::PersonalInfo,
                                AccountTabs::Wallet,
                                AccountTabs::Wishlist,
                                AccountTabs::Support,
                                AccountTabs::Terms,
                                AccountTabs::Privacy,
                                AccountTabs::Faq,
                            ]
                                .into_iter()
                                .map(|tab| {
                                    let is_active = move || active_tab() == tab;
                                    if tab.disabled() {
                                        view! {
                                            <span class="flex items-center gap-2 px-2 py-1 rounded-md cursor-not-allowed opacity-50">
                                                {tab.label()}
                                            </span>
                                        }.into_view()
                                    } else {
                                        view! {
                                            <A
                                                clone:tab
                                                href=tab.as_route()
                                                class=move || {
                                                    format!(
                                                        "flex items-center gap-2 px-2 py-1 rounded-md {}",
                                                        if is_active() {
                                                            "text-blue-600 font-medium bg-blue-50"
                                                        } else {
                                                            "hover:text-blue-600"
                                                        }
                                                    )
                                                }
                                            >
                                                {tab.label()}
                                            </A>
                                        }.into_view()
                                    }
                                })
                                .collect_view()}
                        </nav>
                    </div>
                </aside>

                {/* Overlay for mobile */}
                {move || if sidebar_open.get() {
                    view! {
                        <div
                            class="fixed inset-0 bg-black bg-opacity-50 lg:hidden"
                            on:click=move |_| set_sidebar_open.set(false)
                        />
                    }
                } else {
                    view! { <div></div> }
                }}

                {/* Main content */}
                <section class="flex-1 bg-white rounded-xl border p-8 overflow-y-auto">
                    {move || match active_tab() {
                        AccountTabs::PersonalInfo => view! { <PersonalInfoView/> }.into_view(),
                        AccountTabs::Wallet => view! { <WalletView/> }.into_view(),
                        AccountTabs::Wishlist => view! { <WishlistView/> }.into_view(),
                        AccountTabs::Support => view! { <SupportView/> }.into_view(),
                        AccountTabs::Terms => view! { <TermsView/> }.into_view(),
                        AccountTabs::Privacy => view! { <PrivacyView/> }.into_view(),
                        AccountTabs::Faq => view! { <FaqView/> }.into_view(),
                    }}
                </section>
            </main>
            <div class="flex-1 justify-between container mx-auto px-4 py-8 flex gap-6">
                <span class="text-[#45556C] text-sm">"Copyright © 2025 EstateDao. All Rights Reserved."</span>
                <SocialLinks icon_class="text-white bg-blue-600 rounded-full p-1 h-6 w-6" />
            </div>
        </div>
    }
}
