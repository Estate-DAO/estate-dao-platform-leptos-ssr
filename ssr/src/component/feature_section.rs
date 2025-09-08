use leptos::*;

#[component]
pub fn FeatureCard(
    icon: &'static str,
    title: &'static str,
    description: &'static str,
) -> impl IntoView {
    view! {
        <div class="flex items-center space-x-4">
            <div class="flex items-center justify-center w-12 h-12 rounded-full bg-blue-100 shrink-0">
                <img src=icon alt=title class="w-6 h-6 text-blue-500" />
            </div>

            <div class="flex flex-col items-start space-y-1">
                <h3 class="text-lg font-semibold">{title}</h3>
                <p class="text-gray-600 text-sm text-left">{description}</p>
            </div>
        </div>
    }
}

#[component]
pub fn FeaturesSection() -> impl IntoView {
    view! {
        <section class="bg-gray-50 py-12 px-6">
            <div class="max-w-7xl mx-auto grid grid-cols-1 md:grid-cols-3 gap-8 text-center md:text-left">
                <FeatureCard
                    icon="/icons/wallet.svg"
                    title="Seamless Crypto Payments"
                    description="Pay securely with Bitcoin, Ethereum, and more—no banks, no borders."
                />

                <FeatureCard
                    icon="/icons/like.svg"
                    title="Best Price Guarantee"
                    description="Always get the best deals with zero hidden fees."
                />

                <FeatureCard
                    icon="/icons/globe.svg"
                    title="Curated Stays Worldwide"
                    description="Explore handpicked, crypto-friendly hotels across the globe."
                />
            </div>
        </section>
    }
}
