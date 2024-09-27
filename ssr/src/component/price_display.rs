use leptos::*;

#[component]
pub fn PriceComponent(
    #[prop(into)] price: Signal<u32>,
    #[prop(default = "₹".to_string())] currency: String,
    #[prop(default = "font-semibold".to_string())] price_class: String,
    #[prop(default = "font-light text-sm".to_string())] subtext_class: String,
) -> impl IntoView {
    let formatted_price = move || {
        price()
            .to_string()
            .as_bytes()
            .rchunks(3)
            .rev()
            .map(std::str::from_utf8)
            .collect::<Result<Vec<&str>, _>>()
            .unwrap()
            .join(",")
    };

    view! {
        <div class="flex items-center space-x-1">
            <span class={price_class}>{currency}{"\u{00A0}"}{formatted_price}</span>
            <span class={subtext_class}>" /night"</span>
        </div>
    }
}
