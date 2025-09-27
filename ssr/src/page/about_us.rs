use leptos::*;
use leptos_icons::*;

#[component]
pub fn AboutUsPage() -> impl IntoView {
    view! {
        <main class="min-h-screen bg-white">
            <AboutUsHeroSection />
            <VisionSection />
            <TrustedByTravelersSection />
            <WhatMakesUsDifferentSection />
            <TravelWithoutLimitsSection />
            <WhyChooseUsSection />
        </main>
    }
}

#[component]
fn AboutUsHeroSection() -> impl IntoView {
    view! {
        <section class="relative bg-white px-4 md:px-8 lg:px-16 py-12 md:py-20 overflow-hidden">
            <div class="max-w-7xl mx-auto">
                <div class="grid grid-cols-1 lg:grid-cols-2 gap-8 lg:gap-16 items-center">
                    // <!-- Left side - Text content -->
                    <div class="space-y-6 z-10">
                        <h1 class="text-4xl md:text-5xl lg:text-6xl font-bold leading-tight">
                            "Decentralising Travel"
                            <br/>
                            "for the World"
                        </h1>
                        <p class="text-gray-600 text-lg leading-relaxed">
                            "NoFeeBooking is the world's first crypto-enabled, zero-commission "
                            "hotel booking platform built on the Internet Computer (ICP). "
                            "Covering 2+ million cities globally, we are designed as a truly "
                            "decentralised alternative to traditional travel platforms."
                        </p>
                        <p class="text-gray-600 text-lg leading-relaxed">
                            "By eliminating commission fees for hotels, lowering costs for "
                            "travellers, and ensuring fairness through transparency, we are "
                            "reshaping the way the world books travel."
                        </p>
                    </div>

                    // <!-- Right side - Images and Icons -->
                    <div class="relative h-[600px] lg:h-[700px]">
                        // <!-- Images -->
                        <div class="absolute top-0 right-0 w-48 md:w-56 lg:w-64 rounded-2xl overflow-hidden shadow-xl">
                            <img
                                src="/img/about-us-page/landscape_right_top.png"
                                alt="Travel destination"
                                class="w-full h-full object-cover"
                            />
                            // <!-- Yellow play button -->
                            <div class="absolute top-4 right-4">
                                <img
                                    src="/icons/about_us_yellow_pointer.svg"
                                    alt="Play"
                                    class="w-8 h-8"
                                />
                            </div>
                        </div>

                        <div class="absolute top-32 left-0 w-56 md:w-64 lg:w-72 rounded-2xl overflow-hidden shadow-xl">
                            <img
                                src="/img/about-us-page/square_left_top.png"
                                alt="Coastal view"
                                class="w-full h-full object-cover"
                            />
                        </div>

                        <div class="absolute bottom-32 left-0 w-48 md:w-56 lg:w-64 rounded-2xl overflow-hidden shadow-xl">
                            <img
                                src="/img/about-us-page/landscape_left_bottom.png"
                                alt="Venice canal"
                                class="w-full h-full object-cover"
                            />
                            // <!-- Map pin icon -->
                            <div class="absolute bottom-4 left-4">
                                <img
                                    src="/icons/about_us_Maps.svg"
                                    alt="Location"
                                    class="w-8 h-8"
                                />
                            </div>
                        </div>

                        <div class="absolute bottom-0 right-8 w-40 md:w-48 lg:w-56 rounded-2xl overflow-hidden shadow-xl">
                            <img
                                src="/img/about-us-page/vertical_right_bottom.png"
                                alt="Mosque at sunset"
                                class="w-full h-full object-cover"
                            />
                        </div>

                        // <!-- Decorative arrows and plane -->
                        <div class="absolute top-16 right-56 hidden lg:block">
                            <img
                                src="/icons/about_us_arrow_1.svg"
                                alt="Arrow decoration"
                                class="w-32 h-auto opacity-60"
                            />
                        </div>

                        <div class="absolute bottom-48 right-16 hidden lg:block">
                            <img
                                src="/icons/about_us_arrow_2.svg"
                                alt="Arrow decoration"
                                class="w-24 h-auto opacity-60"
                            />
                        </div>

                        <div class="absolute top-64 right-32 hidden lg:block">
                            <img
                                src="/icons/about_us_plane.svg"
                                alt="Plane icon"
                                class="w-12 h-12"
                            />
                        </div>
                    </div>
                </div>
            </div>
        </section>
    }
}

#[component]
fn VisionSection() -> impl IntoView {
    view! {
        <section class="py-16 md:py-24 bg-gradient-to-br from-blue-50 to-purple-50">
            <div class="max-w-4xl mx-auto px-4 md:px-8 text-center">
                <h2 class="text-3xl md:text-4xl lg:text-5xl font-bold text-blue-600 mb-8">
                    "Vision"
                </h2>
                <p class="text-gray-700 text-lg md:text-xl leading-relaxed">
                    "Our vision is to create a travel ecosystem that is open, efficient, and owned by its users"
                    <br/>
                    "—where technology empowers transparency, efficiency, and community-driven"
                    <br/>
                    "innovation in the global travel industry."
                </p>
            </div>
        </section>
    }
}

#[component]
fn TrustedByTravelersSection() -> impl IntoView {
    view! {
        <section class="py-16 md:py-24 bg-white">
            <div class="max-w-7xl mx-auto px-4 md:px-8">
                <div class="grid grid-cols-1 lg:grid-cols-2 gap-12 lg:gap-16 items-center">
                    // <!-- Left side - Image -->
                    <div class="relative">
                        <div class="relative bg-gradient-to-br from-yellow-100 via-orange-100 to-blue-100 rounded-3xl p-8 overflow-hidden">
                            <img
                                src="/img/about-us-page/section_left.png"
                                alt="Traveler with phone"
                                class="w-full h-auto relative z-10"
                            />
                            // <!-- Decorative circles -->
                            <div class="absolute top-8 right-8 w-16 h-16 bg-blue-400 rounded-full opacity-80"></div>
                            <div class="absolute top-24 right-24 w-8 h-8 bg-teal-400 rounded-full"></div>
                            <div class="absolute bottom-24 left-8 w-12 h-12 bg-blue-300 rounded-full opacity-60"></div>
                            <div class="absolute bottom-8 left-24 w-6 h-6 bg-blue-500 rounded-full"></div>
                        </div>
                    </div>

                    // <!-- Right side - Content -->
                    <div class="space-y-8">
                        <div class="space-y-4">
                            <h2 class="text-3xl md:text-4xl lg:text-5xl font-bold leading-tight">
                                "Trusted by Travelers,"
                                <br/>
                                "Backed by Data"
                            </h2>
                            <p class="text-gray-600 text-lg leading-relaxed">
                                "We believe travel should be open, fair, and accessible to "
                                "everyone. By combining the convenience of modern booking "
                                "systems with the ethos of decentralisation, we deliver a "
                                "platform that benefits both hotels and travellers."
                            </p>
                        </div>

                        // <!-- Statistics Grid -->
                        <div class="grid grid-cols-2 gap-6 md:gap-8">
                            <div class="text-center p-6 bg-blue-50 rounded-2xl">
                                <div class="text-3xl md:text-4xl font-bold text-blue-600 mb-2">
                                    "2 Million +"
                                </div>
                                <div class="text-gray-600 font-medium">
                                    "Active Cities"
                                </div>
                            </div>

                            <div class="text-center p-6 bg-blue-50 rounded-2xl">
                                <div class="text-3xl md:text-4xl font-bold text-blue-600 mb-2">
                                    "100 +"
                                </div>
                                <div class="text-gray-600 font-medium">
                                    "Countries Fiat Ready"
                                </div>
                            </div>

                            <div class="text-center p-6 bg-blue-50 rounded-2xl">
                                <div class="text-3xl md:text-4xl font-bold text-blue-600 mb-2">
                                    "10,000 +"
                                </div>
                                <div class="text-gray-600 font-medium">
                                    "Crypto Partners"
                                </div>
                            </div>

                            <div class="text-center p-6 bg-blue-50 rounded-2xl">
                                <div class="text-3xl md:text-4xl font-bold text-blue-600 mb-2">
                                    "0%"
                                </div>
                                <div class="text-gray-600 font-medium">
                                    "Transaction Fees"
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </section>
    }
}

#[component]
fn WhatMakesUsDifferentSection() -> impl IntoView {
    view! {
        <section class="py-16 md:py-24 bg-gray-50">
            <div class="max-w-7xl mx-auto px-4 md:px-8">
                <div class="grid grid-cols-1 lg:grid-cols-2 gap-12 lg:gap-16 items-center">
                    // <!-- Left side - Content -->
                    <div class="space-y-8">
                        <div class="space-y-6">
                            <h2 class="text-3xl md:text-4xl lg:text-5xl font-bold leading-tight text-gray-900">
                                "What Makes Us Different"
                            </h2>
                            <p class="text-gray-600 text-lg leading-relaxed">
                                "NoFeeBooking offers a seamless booking experience with both "
                                "cryptocurrency and fiat payment options, supported by on-chain "
                                "governance that ensures trust and accountability. By running entirely on "
                                "the Internet Computer (ICP), our platform delivers:"
                            </p>
                        </div>

                        // <!-- Features List -->
                        <div class="space-y-4">
                            <div class="flex items-start space-x-3">
                                <div class="w-2 h-2 bg-blue-600 rounded-full mt-3 flex-shrink-0"></div>
                                <span class="text-gray-700 text-lg font-medium">
                                    "Web-speed performance"
                                </span>
                            </div>
                            <div class="flex items-start space-x-3">
                                <div class="w-2 h-2 bg-blue-600 rounded-full mt-3 flex-shrink-0"></div>
                                <span class="text-gray-700 text-lg font-medium">
                                    "Scalable infrastructure"
                                </span>
                            </div>
                            <div class="flex items-start space-x-3">
                                <div class="w-2 h-2 bg-blue-600 rounded-full mt-3 flex-shrink-0"></div>
                                <span class="text-gray-700 text-lg font-medium">
                                    "Community-driven governance"
                                </span>
                            </div>
                        </div>

                        // <!-- Bottom text -->
                        <p class="text-gray-600 text-lg leading-relaxed">
                            "We support "
                            <span class="font-semibold text-gray-800">"300+ cryptocurrencies"</span>
                            " and fiat currencies, enabling broad "
                            "flexibility in payments and making travel bookings accessible to users "
                            "worldwide."
                        </p>
                    </div>

                    // <!-- Right side - Image -->
                    <div class="relative">
                        <div class="relative bg-gradient-to-br from-purple-100 via-blue-100 to-blue-50 rounded-3xl p-8 overflow-hidden">
                            <img
                                src="/img/about-us-page/right_image_section_four.png"
                                alt="Infinity symbol representing infinite possibilities"
                                class="w-full h-auto relative z-10"
                            />
                            // <!-- Decorative circles -->
                            <div class="absolute top-8 left-8 w-12 h-12 bg-purple-300 rounded-full opacity-60"></div>
                            <div class="absolute top-24 left-24 w-6 h-6 bg-blue-400 rounded-full"></div>
                            <div class="absolute bottom-24 right-8 w-16 h-16 bg-blue-300 rounded-full opacity-80"></div>
                            <div class="absolute bottom-8 right-24 w-8 h-8 bg-purple-400 rounded-full opacity-70"></div>
                        </div>
                    </div>
                </div>
            </div>
        </section>
    }
}

#[component]
fn TravelWithoutLimitsSection() -> impl IntoView {
    let (current_index, set_current_index) = create_signal(0);

    let crypto_logos = vec![
        "Add_Xyz_Plt_",
        "Asch_Xas_",
        "Augur_Rep_",
        "bitcoin-btc-logo",
        "Crust_Network_CRU_",
        "Karbo_Krb_",
        "Lamden_Tau_",
    ];

    let visible_logos = 5; // Number of logos visible at once
    let total_logos = crypto_logos.len();

    let next_slide = move |_| {
        set_current_index.update(|index| {
            *index = (*index + 1) % total_logos;
        });
    };

    let prev_slide = move |_| {
        set_current_index.update(|index| {
            *index = if *index == 0 {
                total_logos - 1
            } else {
                *index - 1
            };
        });
    };

    view! {
        <section class="py-16 md:py-24 bg-gradient-to-r from-blue-900 to-purple-900">
            <div class="max-w-7xl mx-auto px-4 md:px-8 text-center">
                <div class="mb-12">
                    <h2 class="text-3xl md:text-4xl lg:text-5xl font-bold text-white mb-6">
                        "Travel Without Limits"
                    </h2>
                    <p class="text-blue-100 text-lg md:text-xl max-w-3xl mx-auto">
                        "We accept a wide range of cryptocurrencies, making your hotel booking "
                        "experience faster, safer, and truly global."
                    </p>
                </div>

                // <!-- Crypto Logo Carousel -->
                <div class="relative">
                    <div class="flex items-center justify-center">
                        // <!-- Left Arrow -->
                        <button
                            on:click=prev_slide
                            class="p-3 rounded-full bg-white/10 hover:bg-white/20 transition-colors duration-200 mr-8"
                        >
                            <svg class="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
                            </svg>
                        </button>

                        // <!-- Logo Container -->
                        <div class="overflow-hidden w-full max-w-4xl">
                            <div
                                class="flex transition-transform duration-500 ease-in-out"
                                style=move || format!("transform: translateX(-{}%)", (current_index.get() * 100) / visible_logos)
                            >
                                {crypto_logos.into_iter().enumerate().map(|(i, logo)| {
                                    view! {
                                        <div class="flex-shrink-0 w-1/5 px-4">
                                            <div class="bg-white rounded-full p-4 w-16 h-16 md:w-20 md:h-20 mx-auto flex items-center justify-center shadow-lg hover:shadow-xl transition-shadow duration-200">
                                                <img
                                                    src=format!("/icons/about_us_logo_{}.svg", logo)
                                                    alt=format!("{} logo", logo.replace("_", " "))
                                                    class="w-8 h-8 md:w-10 md:h-10 object-contain"
                                                />
                                            </div>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}

                                // <!-- Duplicate logos for seamless infinite scroll -->
                                {crypto_logos.into_iter().enumerate().map(|(i, logo)| {
                                    view! {
                                        <div class="flex-shrink-0 w-1/5 px-4">
                                            <div class="bg-white rounded-full p-4 w-16 h-16 md:w-20 md:h-20 mx-auto flex items-center justify-center shadow-lg hover:shadow-xl transition-shadow duration-200">
                                                <img
                                                    src=format!("/icons/about_us_logo_{}.svg", logo)
                                                    alt=format!("{} logo", logo.replace("_", " "))
                                                    class="w-8 h-8 md:w-10 md:h-10 object-contain"
                                                />
                                            </div>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        </div>

                        // <!-- Right Arrow -->
                        <button
                            on:click=next_slide
                            class="p-3 rounded-full bg-white/10 hover:bg-white/20 transition-colors duration-200 ml-8"
                        >
                            <svg class="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                            </svg>
                        </button>
                    </div>

                    // <!-- Carousel Indicators -->
                    <div class="flex justify-center mt-8 space-x-2">
                        {(0..total_logos).map(|i| {
                            view! {
                                <button
                                    on:click=move |_| set_current_index.set(i)
                                    class=move || {
                                        if current_index.get() == i {
                                            "w-3 h-3 rounded-full bg-white"
                                        } else {
                                            "w-3 h-3 rounded-full bg-white/30 hover:bg-white/50"
                                        }
                                    }
                                />
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>
            </div>
        </section>
    }
}

#[component]
fn WhyChooseUsSection() -> impl IntoView {
    view! {
        <section class="py-16 md:py-24 bg-white">
            <div class="max-w-7xl mx-auto px-4 md:px-8">
                // <!-- Section Title -->
                <div class="text-center mb-12">
                    <h2 class="text-3xl md:text-4xl lg:text-5xl font-bold text-gray-900 mb-8">
                        "Why Choose Us?"
                    </h2>
                </div>

                // <!-- Features Grid -->
                <div class="grid grid-cols-1 md:grid-cols-3 gap-8 lg:gap-12 mb-12">
                    // <!-- Feature 1: Seamless Crypto Payments -->
                    <div class="text-center space-y-4">
                        <div class="w-16 h-16 mx-auto bg-blue-100 rounded-full flex items-center justify-center mb-4">
                            <svg class="w-8 h-8 text-blue-600" fill="currentColor" viewBox="0 0 24 24">
                                <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z"/>
                            </svg>
                        </div>
                        <h3 class="text-xl font-bold text-gray-900">
                            "Seamless Crypto Payments"
                        </h3>
                        <p class="text-gray-600 leading-relaxed">
                            "Pay securely with Bitcoin, Ethereum, "
                            "and more—no banks, no borders."
                        </p>
                    </div>

                    // <!-- Feature 2: Best Price Guarantee -->
                    <div class="text-center space-y-4">
                        <div class="w-16 h-16 mx-auto bg-blue-100 rounded-full flex items-center justify-center mb-4">
                            <svg class="w-8 h-8 text-blue-600" fill="currentColor" viewBox="0 0 24 24">
                                <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/>
                            </svg>
                        </div>
                        <h3 class="text-xl font-bold text-gray-900">
                            "Best Price Guarantee"
                        </h3>
                        <p class="text-gray-600 leading-relaxed">
                            "Always get the best deals with zero "
                            "hidden fees."
                        </p>
                    </div>

                    // <!-- Feature 3: Curated Stays Worldwide -->
                    <div class="text-center space-y-4">
                        <div class="w-16 h-16 mx-auto bg-blue-100 rounded-full flex items-center justify-center mb-4">
                            <svg class="w-8 h-8 text-blue-600" fill="currentColor" viewBox="0 0 24 24">
                                <path d="M12 2C8.13 2 5 5.13 5 9c0 5.25 7 13 7 13s7-7.75 7-13c0-3.87-3.13-7-7-7zm0 9.5c-1.38 0-2.5-1.12-2.5-2.5s1.12-2.5 2.5-2.5 2.5 1.12 2.5 2.5-1.12 2.5-2.5 2.5z"/>
                            </svg>
                        </div>
                        <h3 class="text-xl font-bold text-gray-900">
                            "Curated Stays Worldwide"
                        </h3>
                        <p class="text-gray-600 leading-relaxed">
                            "Explore handpicked, crypto-friendly "
                            "hotels across the globe."
                        </p>
                    </div>
                </div>

                // <!-- Japan Image -->
                <div class="rounded-3xl overflow-hidden shadow-2xl">
                    <img
                        src="/img/about-us-page/section_6_japan_smaller.webp"
                        alt="Mount Fuji and traditional Japanese pagoda"
                        class="w-full h-64 md:h-80 lg:h-96 object-cover"
                    />
                </div>
            </div>
        </section>
    }
}
