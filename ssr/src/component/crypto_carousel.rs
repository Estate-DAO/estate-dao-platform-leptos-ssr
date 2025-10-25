use leptos::prelude::*;

#[component]
pub fn CryptoCarousel() -> impl IntoView {
    let icons = vec![
        "/icons/btc.svg",
        "/icons/plt.svg",
        "/icons/adk.svg",
        "/icons/soc.svg",
        "/icons/xas.svg",
        "/icons/rep.svg",
        "/icons/krb.svg",
        "/icons/tau.svg",
        "/icons/cru.svg",
    ];

    view! {
        <section class="bg-[#162456] text-white py-16 px-6">
            <div class="max-w-6xl mx-auto text-center space-y-4">
                <h2 class="text-2xl md:text-3xl font-bold">"Travel Without Limits"</h2>
                <p class="max-w-2xl mx-auto text-sm md:text-base text-gray-300">
                    "We accept a wide range of cryptocurrencies, making your hotel booking experience faster, safer, and truly global."
                </p>

                <div class="flex items-center justify-center mx-4 mt-8 space-x-4 md:space-x-6">
                    // <button class="md:inline-flex  hover:bg-white/10 transition">
                    //     <span class="text-xl">"←"</span>
                    // </button>

                    <div class="w-full md:max-w-none">
                        <div
                            class="flex flex-nowrap items-center justify-between gap-6 overflow-x-auto scrollbar-hide px-2 scroll-smooth"
                            style="-webkit-overflow-scrolling: touch;"
                        >
                            {icons.into_iter().map(|icon| view! {
                                <div class="w-14 h-14 md:w-16 md:h-16 flex items-center justify-center rounded-full bg-white shrink-0">
                                    <img src=icon alt="crypto" class="w-8 h-8 md:w-9 md:h-9 object-contain" />
                                </div>
                            }).collect_view()}
                        </div>
                    </div>

                    // <button class="md:inline-flex  hover:bg-white/10 transition">
                    //     <span class="text-xl">"→"</span>
                    // </button>
                </div>
            </div>
        </section>
    }
}
