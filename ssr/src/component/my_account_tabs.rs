use leptos::*;

use crate::component::*;

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
    view! {
        <div>
            <h1 class="text-xl font-semibold mb-4">"Wishlist"</h1>
            <p class="text-gray-600">"Saved items and bookings appear here."</p>
        </div>
    }
}

#[component]
pub fn SupportView() -> impl IntoView {
    view! {
        <div>
            <h1 class="text-xl font-semibold mb-4">"Support"</h1>
            <p class="text-gray-600">"Support tickets, FAQs, and help resources."</p>
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
