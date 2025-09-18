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
            "w-full flex items-start justify-between gap-4 p-2 text-left focus:outline-none transition-colors rounded-lg {}",
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
                    <div class="px-6 pb-2">
                        <span class="text-[#45556C] text-sm leading-relaxed">
                            {answer}
                        </span>
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
                    answer="Once you’ve selected your desired hotel and entered the necessary personal details, you’ll be presented with 300+ cryptocurrency and 75+ fiat payment options.\nSimply select the cryptocurrency you wish to use from the dropdown list, or choose to pay with crypto via one of our integrated partners. Once the payment is confirmed on the blockchain, your booking will be completed, and you’ll receive a confirmation email with all reservation details."
                    initially_open=false
                />

                <FaqItem
                    question="Which cryptocurrencies are accepted as payment on NoFeeBooking?"
                    answer="We accept 300+ leading cryptocurrencies. You can view the full list of supported currencies during checkout."
                    initially_open=false
                />

                <FaqItem
                    question="How many payment options does NoFeeBooking currently accept?"
                    answer="NoFeeBooking allows 300+ cryptocurrencies and 75+ fiat currencies, alongside credit/debit cards and popular payment partners. This combination of crypto and traditional methods gives travellers the freedom to book however they prefer."
                    initially_open=false
                />

                <FaqItem
                    question="Are there any loyalty programs or rewards for using cryptocurrencies for bookings?"
                    answer="We are currently working on launching our loyalty and rewards program, designed to provide exclusive benefits for travellers who choose NoFeeBooking. Stay tuned for more details on how you can unlock savings, rewards, and special perks through our upcoming program."
                    initially_open=false
                />

                <FaqItem
                    question="What customer support options are available if I encounter issues during my booking process?"
                    answer="Our 24/7 customer support team is always available to assist with any questions or issues you may face. You can contact us directly via email, and our team will ensure your booking experience is smooth and reliable."
                    initially_open=false
                />
            </div>
        </div>
    }
}
