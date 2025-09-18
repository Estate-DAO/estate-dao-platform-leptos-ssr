use leptos::*;

/// Single FAQ item with nicer styling + smooth slide animation
#[component]
fn FaqItem(
    #[prop(into)] question: String,
    #[prop(into)] answer: String,
    #[prop(into)] initially_open: bool,
) -> impl IntoView {
    let (open, set_open) = create_signal(initially_open);

    // button classes adapt to `open`
    let btn_class = move || {
        format!(
            "w-full flex items-start justify-between gap-4 p-5 text-left focus:outline-none transition-colors rounded-lg {}",
            if open.get() { "bg-[#F9F9F9]" } else { "bg-[#F9F9F9]" }
        )
    };

    // circular plus/minus button style
    let icon_wrapper = move || {
        format!(
            "inline-flex h-10 w-10 items-center justify-center  transition-transform duration-300 transition-colors shrink-0 text-blue-600",
        )
    };

    // panel height toggles for smooth slide
    let panel_class = move || {
        format!(
            "bg-[#F9F9F9] overflow-hidden transition-[max-height] duration-300 ease-in-out {}",
            if open.get() { "" } else { "max-h-0" }
        )
    };

    view! {
        <div class="py-2">
            <div class="rounded-lg">
                <button
                    class=btn_class
                    on:click=move |_| set_open.update(|v| *v = !*v)
                    aria-expanded=move || open.get().to_string()
                >
                    <div class="flex-1">
                        <h3 class="text-[#45556C] text-lg font-semibold leading-tight">
                            {question}
                        </h3>
                        <p class="mt-1 text-sm text-gray-600">
                            // short teaser style — hidden when collapsed in the screenshot, so kept subtle
                        </p>
                    </div>

                    <span class=icon_wrapper class=("rotate-180", move || open.get())>
                        { move || if open.get() { "−" } else { "+" } }
                    </span>
                </button>

                <div class=panel_class>
                    <div class="px-6 pb-6 pt-3">
                        <p class="text-[#45556C] text-sm leading-relaxed">
                            {answer}
                        </p>
                    </div>
                </div>
            </div>
        </div>
    }
}

/// Full FAQ view with outer container styling
#[component]
pub fn FaqView() -> impl IntoView {
    view! {
        <div>
            <h2 class="text-xl sm:text-2xl lg:text-3xl font-bold leading-snug tracking-tight text-[#01030A]">"Frequently Asked Questions"</h2>

            // outer rounded panel (light bg like your screenshot)
            <div >
                <FaqItem
                    question="How do I select the cryptocurrency payment option during the booking process?"
                    answer="Once you’ve selected your desired hotel and entered the necessary personal details,
                            you’ll be presented with 300+ cryptocurrency and 75+ fiat payment options.
                            Simply select the cryptocurrency from the dropdown or choose one of our integrated partners.
                            Once the payment is confirmed on the blockchain, your booking will be completed and you’ll receive a confirmation email with reservation details."
                    initially_open=false
                />

                <FaqItem
                    question="Which cryptocurrencies are accepted as payment on NoFeeBooking?"
                    answer="NoFeeBooking accepts 300+ cryptocurrencies including Bitcoin, Ethereum,
                            stablecoins, and many others via our integrated payment partners."
                    initially_open=false
                />

                <FaqItem
                    question="How many payment options does NoFeeBooking currently accept?"
                    answer="Currently, we support 300+ crypto options and over 75 fiat payment methods."
                    initially_open=false
                />

                <FaqItem
                    question="Are there any loyalty programs or rewards for using cryptocurrencies for bookings?"
                    answer="Yes—we offer rewards and special promotions for users who choose to pay with cryptocurrency. Details are shown during checkout and in promotional emails."
                    initially_open=false
                />

                <FaqItem
                    question="What customer support options are available if I encounter issues during my booking process?"
                    answer="You can reach our support team via live chat, email at support@nofeebooking.com, or through the Help Center in your dashboard."
                    initially_open=false
                />
            </div>
        </div>
    }
}
