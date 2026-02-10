use leptos::*;
use leptos_router::use_query_map;

use crate::{
    api::{
        auth::auth_state::AuthStateSignal,
        client_side_api::{ClientSideApiClient, SupportBookingContext, SupportProvider},
    },
    component::{yral_auth_provider::YralAuthProvider, *},
    page::WishlistComponent,
};

#[derive(Clone)]
enum SupportStatus {
    Success(String),
    Error(String),
}

// ---------------- Tabs ----------------

#[component]
pub fn PersonalInfoView() -> impl IntoView {
    view! {
        <div>
            <h1 class="text-xl font-semibold mb-4">"Personal Information"</h1>
            <p class="text-gray-600">"User profile and editable personal details go here."</p>
        </div>
    }
}

#[component]
pub fn WalletView() -> impl IntoView {
    view! {
        <div>
            <h1 class="text-xl font-semibold mb-4">"Wallet"</h1>
            <p class="text-gray-600">"Balance, transactions, and payment methods go here."</p>
        </div>
    }
}

#[component]
pub fn WishlistView() -> impl IntoView {
    let count = move || AuthStateSignal::wishlist_count();
    view! {
        <div>
            <div class="flex justify-between items-center mb-4">
                <h1 class="text-xl font-semibold">"My Favorites"</h1>
                <Show when=move || count().is_some()>
                    <p class="text-gray-600">{move || format!("{} properties liked", count().unwrap_or(0))}</p>
                </Show>
            </div>
            <WishlistComponent />
        </div>
    }
}

#[component]
pub fn SupportView() -> impl IntoView {
    let subject = create_rw_signal(String::new());
    let query = create_rw_signal(String::new());
    let (show_validation, set_show_validation) = create_signal(false);
    let (status, set_status) = create_signal::<Option<SupportStatus>>(None);
    let booking_context = create_rw_signal::<Option<SupportBookingContext>>(None);
    let query_map = use_query_map();

    create_effect(move |_| {
        let params = query_map.get();
        let get_value = |key: &str| params.get(key).cloned();
        let parse_provider = |value: &str| match value.to_lowercase().as_str() {
            "liteapi" | "lite_api" | "lite-api" => Some(SupportProvider::LiteApi),
            _ => None,
        };
        let booking_context_candidate = SupportBookingContext {
            booking_id: get_value("booking_id"),
            hotel_name: get_value("hotel_name"),
            hotel_location: get_value("hotel_location"),
            hotel_code: get_value("hotel_code"),
            hotel_image_url: get_value("hotel_image_url"),
            check_in_date: get_value("check_in"),
            check_out_date: get_value("check_out"),
            adults: params.get("adults").and_then(|v| v.parse::<u32>().ok()),
            rooms: params.get("rooms").and_then(|v| v.parse::<u32>().ok()),
            total_amount: params
                .get("total_amount")
                .and_then(|v| v.parse::<f64>().ok()),
            currency: get_value("currency"),
            provider: params.get("provider").and_then(|v| parse_provider(v)),
        };

        let has_context = booking_context_candidate
            .booking_id
            .as_ref()
            .map(|v| !v.trim().is_empty())
            .unwrap_or(false)
            || booking_context_candidate
                .hotel_name
                .as_ref()
                .map(|v| !v.trim().is_empty())
                .unwrap_or(false)
            || booking_context_candidate
                .hotel_location
                .as_ref()
                .map(|v| !v.trim().is_empty())
                .unwrap_or(false)
            || booking_context_candidate
                .hotel_code
                .as_ref()
                .map(|v| !v.trim().is_empty())
                .unwrap_or(false);

        if has_context {
            booking_context.set(Some(booking_context_candidate.clone()));
            if subject.get_untracked().trim().is_empty() {
                let mut fallback = "Support request".to_string();
                if let Some(name) = booking_context_candidate
                    .hotel_name
                    .as_ref()
                    .filter(|v| !v.trim().is_empty())
                {
                    fallback = format!("Support request: {name}");
                }
                if let Some(booking_id) = booking_context_candidate
                    .booking_id
                    .as_ref()
                    .filter(|v| !v.trim().is_empty())
                {
                    if fallback == "Support request" {
                        fallback = format!("Support request for booking {booking_id}");
                    } else {
                        fallback = format!("{fallback} ({booking_id})");
                    }
                }
                subject.set(fallback);
            }
        } else {
            booking_context.set(None);
        }
    });

    let is_authenticated =
        Signal::derive(move || AuthStateSignal::auth_state().get().is_authenticated());
    let user_email = Signal::derive(move || {
        AuthStateSignal::auth_state()
            .get()
            .email
            .unwrap_or_default()
    });

    let subject_error =
        Signal::derive(move || show_validation.get() && subject.get().trim().is_empty());
    let query_error =
        Signal::derive(move || show_validation.get() && query.get().trim().is_empty());
    let is_valid =
        Signal::derive(move || !subject.get().trim().is_empty() && !query.get().trim().is_empty());
    let query_char_count = Signal::derive(move || query.get().chars().count());

    let send_action = create_action(move |_: &()| {
        let subject_value = subject.get_untracked();
        let query_value = query.get_untracked();
        async move {
            set_show_validation.set(true);
            let subject_trimmed = subject_value.trim().to_string();
            let query_trimmed = query_value.trim().to_string();

            if subject_trimmed.is_empty() || query_trimmed.is_empty() {
                set_status.set(Some(SupportStatus::Error(
                    "Please fill in the required fields.".to_string(),
                )));
                return;
            }

            set_status.set(None);
            let client = ClientSideApiClient::new();
            let booking_context_value = booking_context.get_untracked();
            match client
                .send_support_request(subject_trimmed, query_trimmed, booking_context_value)
                .await
            {
                Ok(response) => {
                    set_status.set(Some(SupportStatus::Success(response.message)));
                    subject.set(String::new());
                    query.set(String::new());
                    set_show_validation.set(false);
                }
                Err(e) => {
                    set_status.set(Some(SupportStatus::Error(e)));
                }
            }
        }
    });

    let submit_support_request = move |_| {
        send_action.dispatch(());
    };

    let button_class = move || {
        let base = "w-full rounded-xl px-6 py-3 text-white text-sm font-semibold transition-colors";
        if send_action.pending().get() || !is_valid.get() {
            format!("{base} bg-blue-300 cursor-not-allowed")
        } else {
            format!("{base} bg-blue-600 hover:bg-blue-700")
        }
    };

    view! {
        <div class="space-y-6">
            <Show
                when=move || is_authenticated.get()
                fallback=move || view! {
                    <div class="flex flex-col items-center justify-center py-12">
                        <div class="text-center max-w-md space-y-4">
                            <h2 class="text-xl font-semibold text-gray-900">
                                "Sign in to contact support"
                            </h2>
                            <p class="text-gray-600">
                                "Please sign in to create a support ticket and track updates."
                            </p>
                            // <a
                            //     href="/auth/google"
                            //     class="inline-flex items-center justify-center rounded-xl bg-blue-600 px-6 py-3 text-sm font-semibold text-white hover:bg-blue-700 transition-colors"
                            // >
                            //     "Sign In"
                            // </a>
                            <div class="flex justify-center">
                                <YralAuthProvider />
                            </div>
                        </div>
                    </div>
                }
            >
                <div class="space-y-2">
                    <h1 class="text-2xl font-semibold text-gray-900">
                        "Need Help? We're Here for You!"
                    </h1>
                    <p class="text-gray-600">
                        "Have a question or need support? Send your query using the form below, and our team will get back to you shortly via email."
                    </p>
                </div>

                <div class="rounded-2xl border border-gray-200 bg-white shadow-sm p-6 sm:p-8 space-y-6">
                    <div class="text-sm text-gray-500">
                        "Signed in as "
                        <span class="font-medium text-gray-700">{move || user_email.get()}</span>
                    </div>

                    <Show when=move || booking_context.get().is_some()>
                        {move || {
                            booking_context.get().map(|ctx| {
                                let image_src = ctx
                                    .hotel_image_url
                                    .clone()
                                    .filter(|value| {
                                        value.starts_with("http://")
                                            || value.starts_with("https://")
                                    })
                                    .unwrap_or_else(|| "/img/home.png".to_string());
                                let hotel_name = ctx
                                    .hotel_name
                                    .clone()
                                    .filter(|value| !value.trim().is_empty())
                                    .unwrap_or_else(|| "Booking details".to_string());
                                let hotel_name_heading = hotel_name.clone();
                                let hotel_name_image = hotel_name.clone();
                                let hotel_location = ctx
                                    .hotel_location
                                    .clone()
                                    .unwrap_or_default();
                                let show_location = !hotel_location.trim().is_empty();
                                let hotel_location_text = hotel_location.clone();
                                let booking_id = ctx
                                    .booking_id
                                    .clone()
                                    .unwrap_or_else(|| "-".to_string());
                                let stay_text = match (
                                    ctx.check_in_date.clone(),
                                    ctx.check_out_date.clone(),
                                ) {
                                    (Some(check_in), Some(check_out))
                                        if !check_in.trim().is_empty()
                                            && !check_out.trim().is_empty() =>
                                    {
                                        format!("{check_in} to {check_out}")
                                    }
                                    (Some(check_in), _) if !check_in.trim().is_empty() => {
                                        format!("Check-in {check_in}")
                                    }
                                    (_, Some(check_out)) if !check_out.trim().is_empty() => {
                                        format!("Check-out {check_out}")
                                    }
                                    _ => String::new(),
                                };
                                let guests_text = match (ctx.adults, ctx.rooms) {
                                    (Some(adults), Some(rooms)) => {
                                        format!("{adults} adults · {rooms} rooms")
                                    }
                                    (Some(adults), None) => format!("{adults} adults"),
                                    (None, Some(rooms)) => format!("{rooms} rooms"),
                                    _ => String::new(),
                                };
                                let amount_text = ctx.total_amount.map(|amount| {
                                    let currency = ctx.currency.clone().unwrap_or_default();
                                    if currency.trim().is_empty() {
                                        format!("{amount:.2}")
                                    } else {
                                        format!("{currency} {amount:.2}")
                                    }
                                });
                                let provider_text = ctx
                                    .provider
                                    .as_ref()
                                    .map(|provider| provider.as_str().to_string());
                                let hotel_details_url = ctx.hotel_code.clone().filter(|value| {
                                    !value.trim().is_empty()
                                }).map(|code| format!("/hotel-details?hotelCode={code}"));
                                let view_bookings_url = "/account?page=bookings";
                                let image_src_value = image_src.clone();

                                view! {
                                    <div class="rounded-2xl border border-gray-200 bg-gray-50 p-4 sm:p-5 space-y-4">
                                        <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-2">
                                            <div>
                                                <p class="text-xs font-semibold uppercase tracking-wide text-blue-600">
                                                    "Booking preview"
                                                </p>
                                                <h3 class="text-lg font-semibold text-gray-900">{hotel_name_heading}</h3>
                                                <Show when=move || show_location>
                                                    <p class="text-sm text-gray-600">{hotel_location_text.clone()}</p>
                                                </Show>
                                            </div>
                                            <a
                                                href=view_bookings_url
                                                class="text-sm font-medium text-blue-600 hover:text-blue-700"
                                            >
                                                "View bookings"
                                            </a>
                                        </div>
                                        <div class="grid grid-cols-1 sm:grid-cols-[180px_1fr] gap-4 items-start">
                                            {move || {
                                                if let Some(url) = hotel_details_url.clone() {
                                                    view! {
                                                        <a href=url class="block">
                                                            <img
                                                                src=image_src_value.clone()
                                                                alt=hotel_name_image.clone()
                                                                class="w-full h-32 sm:h-28 object-cover rounded-xl border border-gray-200"
                                                            />
                                                        </a>
                                                    }
                                                    .into_view()
                                                } else {
                                                    view! {
                                                        <img
                                                            src=image_src_value.clone()
                                                            alt=hotel_name_image.clone()
                                                            class="w-full h-32 sm:h-28 object-cover rounded-xl border border-gray-200"
                                                        />
                                                    }
                                                    .into_view()
                                                }
                                            }}
                                            <div class="space-y-2 text-sm text-gray-700">
                                                <div>
                                                    <span class="font-medium text-gray-900">"Booking ID:"</span>
                                                    " "
                                                    {booking_id}
                                                </div>
                                                {if stay_text.is_empty() {
                                                    view! { <></> }.into_view()
                                                } else {
                                                    view! {
                                                        <div>
                                                            <span class="font-medium text-gray-900">"Stay:"</span>
                                                            " "
                                                            {stay_text}
                                                        </div>
                                                    }.into_view()
                                                }}
                                                {if guests_text.is_empty() {
                                                    view! { <></> }.into_view()
                                                } else {
                                                    view! {
                                                        <div>
                                                            <span class="font-medium text-gray-900">"Guests:"</span>
                                                            " "
                                                            {guests_text}
                                                        </div>
                                                    }.into_view()
                                                }}
                                                {if let Some(text) = amount_text {
                                                    view! {
                                                        <div>
                                                            <span class="font-medium text-gray-900">"Amount paid:"</span>
                                                            " "
                                                            {text}
                                                        </div>
                                                    }.into_view()
                                                } else {
                                                    view! { <></> }.into_view()
                                                }}
                                                {if let Some(text) = provider_text {
                                                    view! {
                                                        <div>
                                                            <span class="font-medium text-gray-900">"Provider:"</span>
                                                            " "
                                                            {text}
                                                        </div>
                                                    }.into_view()
                                                } else {
                                                    view! { <></> }.into_view()
                                                }}
                                            </div>
                                        </div>
                                    </div>
                                }
                            })
                        }}
                    </Show>

                    <div class="space-y-5">
                        <div class="space-y-2">
                            <label class="text-sm font-medium text-gray-800">
                                "Subject"
                                <span class="text-red-500">*</span>
                            </label>
                            <input
                                type="text"
                                placeholder="Mention the subject"
                                maxlength="120"
                                class=move || {
                                    let base = "w-full rounded-xl border border-gray-200 bg-gray-50 p-3.5 text-sm text-gray-900 placeholder:text-gray-400 focus:bg-white focus:border-blue-500 focus:ring-2 focus:ring-blue-100 transition-colors";
                                    if subject_error.get() {
                                        format!("{base} border-red-300 focus:border-red-400 focus:ring-red-100")
                                    } else {
                                        base.to_string()
                                    }
                                }
                                value=move || subject.get()
                                on:input=move |ev| {
                                    subject.set(event_target_value(&ev));
                                    set_status.set(None);
                                }
                            />
                            <Show when=move || subject_error.get()>
                                <p class="text-xs text-red-600">"Subject is required."</p>
                            </Show>
                        </div>

                        <div class="space-y-2">
                            <div class="flex items-center justify-between">
                                <label class="text-sm font-medium text-gray-800">
                                    "Query"
                                    <span class="text-red-500">*</span>
                                </label>
                                <span class="text-xs text-gray-400">
                                    {move || format!("{}/500 characters", query_char_count.get())}
                                </span>
                            </div>
                            <textarea
                                rows="6"
                                maxlength="500"
                                placeholder="Enter your message"
                                class=move || {
                                    let base = "w-full rounded-xl border border-gray-200 bg-gray-50 p-3.5 text-sm text-gray-900 placeholder:text-gray-400 focus:bg-white focus:border-blue-500 focus:ring-2 focus:ring-blue-100 transition-colors";
                                    if query_error.get() {
                                        format!("{base} border-red-300 focus:border-red-400 focus:ring-red-100")
                                    } else {
                                        base.to_string()
                                    }
                                }
                                prop:value=move || query.get()
                                on:input=move |ev| {
                                    query.set(event_target_value(&ev));
                                    set_status.set(None);
                                }
                            />
                            <Show when=move || query_error.get()>
                                <p class="text-xs text-red-600">"Query is required."</p>
                            </Show>
                        </div>
                    </div>

                    <Show
                        when=move || status.get().is_some()
                        fallback=|| view! { <></> }
                    >
                        {move || {
                            status.get().map(|message| match message {
                                SupportStatus::Success(text) => view! {
                                    <div class="rounded-xl border border-green-200 bg-green-50 px-4 py-3 text-sm text-green-700">
                                        {text}
                                    </div>
                                }.into_view(),
                                SupportStatus::Error(text) => view! {
                                    <div class="rounded-xl border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
                                        {text}
                                    </div>
                                }.into_view(),
                            })
                        }}
                    </Show>

                    <button
                        type="button"
                        class=button_class
                        disabled=move || send_action.pending().get() || !is_valid.get()
                        aria-busy=move || send_action.pending().get().to_string()
                        on:click=submit_support_request
                    >
                        {move || {
                            if send_action.pending().get() {
                                "Sending..."
                            } else {
                                "Send Query"
                            }
                        }}
                    </button>
                </div>
            </Show>
        </div>
    }
}

#[component]
pub fn TermsView() -> impl IntoView {
    let doc = tnc_doc();
    view! {
        <div>
            <DocView doc />
        </div>
    }
}

#[component]
pub fn PrivacyView() -> impl IntoView {
    let doc = privacy_policy_doc();
    view! {
        <div>
            <DocView doc />
        </div>
    }
}

/// Build the privacy policy doc from the screenshot text (improved, split into paragraphs)
pub fn privacy_policy_doc() -> Doc {
    Doc {
        title: "NFB - Privacy Policy".to_string(),
        intro: vec![
            "This Privacy Policy describes how Nofeebooking (\"we,\" \"us,\" or \"our\") collects, uses, and shares your information when you use our website, www.nofeebooking.com, and related services. By accessing or using our services, you consent to the collection and use of your information as described in this policy.".to_string()
        ],
        sections: vec![
            Section {
                heading: "Information We Collect".to_string(),
                descriptions: vec![
                    "To provide our services and manage your account, we collect several types of information.".to_string()
                ],
                bullets: vec![
                    "Information you provide directly: This includes data you submit when you create an account, make a booking, or contact us. Examples include your name, email address, phone number, physical address, and payment information (e.g., credit card details).".to_string(),
                    "Information collected automatically: When you visit our website, we automatically collect data about your device and usage. This data includes your IP address, browser type, operating system, the pages you view, and the dates and times of your visits. This helps us understand how you interact with our services and improve them.".to_string(),
                    "Information from third parties: We may receive information about you from our partners, such as hotel affiliates, booking providers, and payment processors. This can include confirmation of a booking or updated payment information.".to_string(),
                ],
            },
            Section {
                heading: "How We Use Your Information".to_string(),
                descriptions: vec![
                    "We use the information we collect for the following purposes:".to_string()
                ],
                bullets: vec![
                    "To provide and manage our services: We use your data to process your bookings, manage your account, and provide customer support.".to_string(),
                    "To improve our services: We analyze aggregated data to enhance features and functionality, troubleshoot technical issues, and better understand user preferences.".to_string(),
                    "To communicate with you: We will use your contact information to send booking confirmations, important service updates, or information about our policies.".to_string(),
                    "For marketing and promotions: With your consent, we may send personalized marketing materials, special offers, or updates on new services.".to_string(),
                    "For security and legal compliance: We use information to detect and prevent fraudulent activity, enforce our terms, and comply with legal obligations.".to_string(),
                ],
            },
            Section {
                heading: "Security and Storage of Information".to_string(),
                descriptions: vec![
                    "We take the security of your information seriously and have implemented robust technical and organizational security measures to protect your personal data from unauthorized access, misuse, or loss. Our security measures include encryption, access controls, and secure data storage protocols.".to_string(),
                    "We will retain your data for as long as your account is active or as necessary to provide our services. We may also retain information as needed to comply with legal obligations, resolve disputes, and enforce agreements. Once the retention period ends, your information will be securely deleted or anonymized.".to_string()
                ],
                bullets: vec![],
            },
            Section {
                heading: "Sharing and Disclosure of Information".to_string(),
                descriptions: vec![
                    "We may share your information with our subsidiaries, trusted service providers, and third-party partners who assist us in delivering our services (e.g., payment processors and hotel booking platforms). All partners are required to process your data in accordance with strict confidentiality and security guidelines.".to_string(),
                    "We may also disclose your information for legal reasons, such as in response to a court order or subpoena, to prevent fraud, or in connection with a business transfer like a merger or acquisition. Any international data transfers will be performed with appropriate safeguards to ensure your data is protected.".to_string(),
                ],
                bullets: vec![],
            },
            Section {
                heading: "Your Rights and Choices".to_string(),
                descriptions: vec![
                    "You have several rights regarding your personal data. You can access and update your information, request deletion (subject to legal or business retention needs), and opt out of marketing communications.".to_string()
                ],
                bullets: vec![
                    "Access and Correction: You can access and update your information via your account settings or by contacting us.".to_string(),
                    "Data Deletion: You can request deletion of your account and personal data; however, we may need to retain some information for legal or business purposes.".to_string(),
                    "Opt-Out of Marketing: You can opt out of receiving marketing communications from us at any time by clicking the 'unsubscribe' link in our emails.".to_string(),
                ],
            },
            Section {
                heading: "Contact and Updates".to_string(),
                descriptions: vec![
                    "If you have any questions, concerns, or requests regarding your data, you can contact our support team at support@nofeebooking.com . We will respond promptly to your request.".to_string(),
                    "We may update this Privacy Policy from time to time. We will post any changes on our website and encourage you to review the policy periodically to stay informed about how we handle personal information.".to_string()
                ],
                bullets: vec![],
            },
        ],
    }
}
pub fn tnc_doc() -> Doc {
    Doc {
        title: "NFB – Terms & Conditions and Use Policy".to_string(),
        intro: vec![
            "By using Nofeebooking’s Website, Mobile App, or related services (collectively, the \"Services\"), you agree to use them solely for personal, non-commercial purposes. You are strictly prohibited from engaging in unauthorized activities such as decompiling, reverse engineering, or using automated data collection methods. To use our Services, you must be at least 18 years old or have reached the legal minimum age in your jurisdiction. Nofeebooking reserves the right to remove any content or information you post at our discretion, without prior notice. Additionally, you acknowledge that all communications between you and Nofeebooking may be recorded and used as needed, in accordance with our policies.".to_string()
        ],
        sections: vec![
            Section {
                heading: "Use of Services and Account Responsibilities".to_string(),
                descriptions: vec![
                    "You agree to use Nofeebooking’s Services for lawful, personal use only. Unauthorized activities such as reverse engineering, automated scraping, or misuse of the platform are strictly prohibited. You must be at least 18 years old or of legal age in your jurisdiction to use our Services.".to_string(),
                ],
                bullets: vec![],
            },
            Section {
                heading: "User-Generated Content and Intellectual Property Rights".to_string(),
                descriptions: vec![
                    "You are solely responsible for any content you post, upload, or submit to the Services, including reviews, ratings, photos, or other information.".to_string(),
                    "You agree not to post or transmit any content that is defamatory, obscene, harassing, fraudulent, or otherwise objectionable, or content that infringes on third-party intellectual property rights.".to_string(),
                ],
                bullets: vec![
                    "By posting content, you grant Nofeebooking a non-exclusive, worldwide, royalty-free, perpetual, and transferable license to use, reproduce, modify, adapt, publish, and distribute such content in any media.".to_string(),
                    "You must not upload or use any trademarks, copyrights, or other intellectual property of a third party without express permission.".to_string(),
                    "Nofeebooking reserves the right to moderate, remove, or refuse to publish any content that violates these policies and may suspend or terminate your account for violations.".to_string(),
                ],
            },
            Section {
                heading: "Booking, Cancellations, and Refunds".to_string(),
                descriptions: vec![
                    "Nofeebooking acts as an intermediary between you and service providers (e.g., hotels, rental companies). All bookings made through our platform are subject to the terms and conditions of the respective provider.".to_string(),
                    "Requests for cancellations, refunds, or booking changes are not guaranteed and depend entirely on the provider’s policies.".to_string(),
                ],
                bullets: vec![
                    "Nofeebooking may charge service fees in addition to provider fees.".to_string(),
                    "Refunds may also be subject to compliance requirements or delays, particularly for payments made using digital assets or cryptocurrencies.".to_string(),
                ],
            },
            Section {
                heading: "Promotions, Credits, and Usage Restrictions".to_string(),
                descriptions: vec![
                    "Promotions and credits offered by Nofeebooking are non-transferable and cannot be redeemed for cash. Abuse of promotions or credits may result in cancellation or denial of services.".to_string(),
                ],
                bullets: vec![
                    "You must not misuse discounts, submit fraudulent bookings, or attempt to bypass platform rules.".to_string(),
                    "Nofeebooking reserves the right to revoke credits, cancel bookings, or take necessary actions to protect platform integrity.".to_string(),
                ],
            },
            Section {
                heading: "Liability, Indemnification, and Risks".to_string(),
                descriptions: vec![
                    "Nofeebooking is not liable for any direct, indirect, or consequential damages resulting from use of our Services.".to_string(),
                    "Using digital assets or cryptocurrencies carries inherent risks, and you accept full responsibility for any losses incurred.".to_string(),
                ],
                bullets: vec![
                    "You agree to defend and indemnify Nofeebooking and its affiliates against any claims, damages, or costs arising from your misuse of the Services or violations of applicable laws.".to_string(),
                ],
            },
            Section {
                heading: "Termination of Accounts".to_string(),
                descriptions: vec![
                    "Nofeebooking may suspend or terminate your account and access to Services at any time, with or without cause, and without prior notice.".to_string(),
                ],
                bullets: vec![
                    "Termination grounds may include violations of these Terms, fraudulent activity, or harmful behavior.".to_string(),
                    "Upon termination, your right to use the Services will immediately cease.".to_string(),
                ],
            },
            Section {
                heading: "Amendments, Privacy, and General Provisions".to_string(),
                descriptions: vec![
                    "Nofeebooking reserves the right to update or modify these Terms at any time without prior notice. Your continued use of the Services constitutes agreement to the updated Terms.".to_string(),
                    "It is your responsibility to review the Terms regularly to stay informed about changes.".to_string(),
                ],
                bullets: vec![
                    "Personal data will be handled in accordance with our Privacy Policy.".to_string(),
                    "While Nofeebooking strives to provide reliable services, you accept that unforeseen events or disruptions may occur.".to_string(),
                ],
            },
            Section {
                heading: "User Information, Security, and Compliance".to_string(),
                descriptions: vec![
                    "You agree to provide accurate and complete information when using Nofeebooking’s Services.".to_string(),
                    "You are responsible for safeguarding your account credentials and payment details.".to_string(),
                    "You agree to comply with all applicable laws and regulations while using our platform.".to_string(),
                ],
                bullets: vec![
                    "Nofeebooking is not responsible for disruptions caused by external events such as natural disasters or service outages.".to_string(),
                    "We reserve the right to take necessary actions to maintain the security and integrity of our Services.".to_string(),
                ],
            },
        ],
    }
}
