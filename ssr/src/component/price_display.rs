use leptos::*;

#[component]
pub fn PriceDisplay(
    #[prop(into)] price: Signal<u32>,
    #[prop(default = "₹".into(), into)] currency: String,
    #[prop(default = "".into(), into)] base_class: String,
    #[prop(default = "font-semibold".into(), into)] price_class: String,
    #[prop(default = "font-light text-sm".into(), into)] subtext_class: String,
    #[prop(default = None, into)] appended_text: Option<String>,
) -> impl IntoView {

    // 10000000 -> 1,00,00,000
    // 1000000 -> 10,00,000
    // 100000 -> 1,00,000
    // 10000 -> 10,000
    // 1000 -> 1,000
 
    let formatted_price = move || {
        let chars: Vec<char> = price().to_string().chars().rev().collect();
        let mut result = String::new();

        for (i, &c) in chars.iter().enumerate() {
            if i > 0 {
                if i == 3 || (i > 3 && (i - 3) % 2 == 0) {
                    result.push(',');
                }
            }
            result.push(c);
        }
        result.chars().rev().collect::<String>()
    };

    let merged_base_class = format!("{} flex items-center space-x-1", base_class);
    let appended_text_clone = appended_text.clone();
    let subtext_class_clone = subtext_class.clone();

    view! {
        <div class=merged_base_class>
            <span class=price_class>{currency}{"\u{00A0}"}{formatted_price}</span>
            <Show
                when=move || !appended_text.clone().is_none()
                fallback=move || view! { <span class=subtext_class.clone()>" / night"</span> }
            >
                <span class=subtext_class_clone.clone()>{appended_text_clone.clone()}</span>
            </Show>
        </div>
    }
}
