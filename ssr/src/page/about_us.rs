use crate::component::{CryptoCarousel, FeaturesSection, Footer, Navbar};
use leptos::*;

#[component]
pub fn AboutUsPage() -> impl IntoView {
    view! {
        <Navbar />
        <section class="font-figtree text-[#0F172A]">
            <div class="container mx-auto px-6 lg:px-12 py-20">
                <div class="grid lg:grid-cols-2 gap-12 items-center">
                    <div>
                        <h1 class="font-figtree font-semibold text-[40px] lg:text-[64px] leading-[1.4] tracking-[-0.03em] mb-8">
                            "Decentralising Travel for the World"
                        </h1>
                        <p class="font-figtree font-normal text-[18px] lg:text-[20px] leading-[1.4] text-[#45556C] mb-6">
                            "NoFeeBooking is the world’s first crypto-enabled, zero-commission hotel booking platform built on the Internet Computer (ICP). Covering 2+ million cities globally, we are designed as a truly decentralised alternative to traditional travel platforms."
                        </p>
                        <p class="font-figtree font-normal text-[18px] lg:text-[20px] leading-[1.4] text-[#45556C]">
                            "By eliminating commission fees for hotels, lowering costs for travellers, and ensuring fairness through transparency, we are reshaping the way the world books travel."
                        </p>
                    </div>
                    <img src="/img/decentralize_travel.png" alt="Decentralising Travel" class="rounded-2xl object-cover w-full" />
                </div>
            </div>

            <div class="bg-[#EFF6FF] py-24 text-center">
                <div class="container mx-auto px-6 lg:px-12">
                    <h3 class="text-[#145EF0] font-figtree font-semibold text-[40px] mb-4">Vision</h3>
                    <p class="max-w-3xl mx-auto text-[#45556C] font-figtree text-[24px] leading-[1.4] tracking-[-0.02em]">
                        "Our vision is to create a travel ecosystem that is open, efficient, and owned by its users—where technology empowers transparency, efficiency, and community-driven innovation in the global travel industry."
                    </p>
                </div>
            </div>

            <div class="py-20">
                <div class="grid lg:grid-cols-2 gap-12 items-center">
                    <div class="flex justify-center lg:justify-start">
                        <img
                            src="/img/trusted_traveller.png"
                            alt="Traveller"
                            class="object-cover w-full h-auto max-w-[480px] lg:ml-0"
                        />
                    </div>

                    <div class="flex flex-col items-center lg:items-start text-center lg:text-left mr-4">
                        <h2 class="font-figtree font-semibold text-[40px] lg:text-[64px] leading-[1.4] tracking-[-0.03em] mb-6">
                            "Trusted by Travelers, Backed by Data"
                        </h2>
                        <p class="font-figtree text-[20px] leading-[1.4] text-[#45556C] mb-10 max-w-[520px]">
                            "We believe travel should be open, fair, and accessible to everyone. By combining the convenience of modern booking systems with the ethos of decentralisation, we deliver a platform that benefits both hotels and travellers."
                        </p>

                        <div class="grid sm:grid-cols-2 gap-6 justify-items-center">
                            <div class="flex flex-col justify-center items-center w-[265px] h-[150px] bg-white border border-[#CAD5E2] rounded-[30px]">
                                <div class="font-figtree font-bold text-[32px] leading-[1.4] tracking-[-0.03em] text-[#145EF0]">2 Million +</div>
                                <div class="font-figtree font-medium text-[20px] leading-[1.4] text-[#45556C]">Active Cities</div>
                            </div>
                            <div class="flex flex-col justify-center items-center w-[265px] h-[150px] bg-white border border-[#CAD5E2] rounded-[30px]">
                                <div class="font-figtree font-bold text-[32px] leading-[1.4] tracking-[-0.03em] text-[#145EF0]">100 +</div>
                                <div class="font-figtree font-medium text-[20px] leading-[1.4] text-[#45556C]">Countries Fiat Ready</div>
                            </div>
                            <div class="flex flex-col justify-center items-center w-[265px] h-[150px] bg-white border border-[#CAD5E2] rounded-[30px]">
                                <div class="font-figtree font-bold text-[32px] leading-[1.4] tracking-[-0.03em] text-[#145EF0]">10,000 +</div>
                                <div class="font-figtree font-medium text-[20px] leading-[1.4] text-[#45556C]">Crypto Partners</div>
                            </div>
                            <div class="flex flex-col justify-center items-center w-[265px] h-[150px] bg-white border border-[#CAD5E2] rounded-[30px]">
                                <div class="font-figtree font-bold text-[32px] leading-[1.4] tracking-[-0.03em] text-[#145EF0]">0%</div>
                                <div class="font-figtree font-medium text-[20px] leading-[1.4] text-[#45556C]">Transaction Fees</div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>


            <div class="ml-6 py-6 lg:ml-12">
                <div class="grid lg:grid-cols-2 gap-12 items-center">
                    <div class="container mx-auto px-12 lg:px-12 lg:mr-0 mr-12">
                        <h3 class="font-figtree font-semibold text-[40px] lg:text-[64px] leading-[1.4] tracking-[-0.03em] mb-4">What Makes Us Different</h3>
                        <p class="font-figtree text-[20px] leading-[1.4] text-[#45556C] mb-4">
                            "NoFeeBooking offers a seamless booking experience with both cryptocurrency and fiat payment options, supported by on-chain governance that ensures trust and accountability. By running entirely on the Internet Computer (ICP), our platform delivers:"
                        </p>
                        <ul class="list-disc list-inside text-[#45556C] text-[20px] leading-[1.4] px-4 space-y-2 mb-4">
                            <li>Web-speed performance</li>
                            <li>Scalable infrastructure</li>
                            <li>Community-driven governance</li>
                        </ul>
                        <p class="font-figtree text-[20px] leading-[1.4] text-[#45556C]">
                            "We support 300+ cryptocurrencies and fiat currencies, enabling broad flexibility in payments and making travel bookings accessible to users worldwide."
                        </p>
                    </div>

                    <div class="flex justify-center lg:justify-end">
                        <img src="/img/about_icp.png" alt="ICP Logo" class="h-auto object-contain" />
                    </div>
                </div>
            </div>


        </section>

        <CryptoCarousel />
        <div class="bg-blue-50 px-16">
                <FeaturesSection show_why_choose_us=true />
        </div>
        <div class="container mx-auto px-6 lg:px-12">
            <img src="/img/japan.jpg" alt="Japan" class="w-full rounded-md h-56 object-cover" />
        </div>
        <div class="my-16" />
        <Footer />

    }
}
