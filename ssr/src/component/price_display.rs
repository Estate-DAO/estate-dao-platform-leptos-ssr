use crate::log;
use accounting::Accounting;
use leptos::prelude::*;

#[component]
pub fn PriceDisplay(
    #[prop(into)] price: f64,
    #[prop(default = "$".into(), into)] currency: String,
    #[prop(default = "".into(), into)] base_class: String,
    #[prop(default = "font-semibold".into(), into)] price_class: String,
    #[prop(default = "font-light text-sm".into(), into)] subtext_class: String,
    #[prop(default = None, into)] appended_text: Option<String>,
) -> impl IntoView {
    let cr = currency.clone();
    let formatted_price = move || {
        let mut ac = Accounting::new_from(&cr, 2);
        ac.set_format("{s} {v}");
        ac.format_money(price)
    };

    let merged_base_class = format!("{} flex items-center space-x-1", base_class);
    let appended_text_clone = appended_text.clone();
    let subtext_class_clone = subtext_class.clone();

    view! {
        <div class=merged_base_class>
            <span class=price_class>{move || formatted_price()}</span>
            <Show
                when=move || !appended_text.clone().is_none()
                fallback=move || view! { <span class=subtext_class.clone()>""</span> }
                // todo (per_night): if multiple nights are selected, then api returns price for all nights at once.
                // fallback=move || view! { <span class=subtext_class.clone()>" / night"</span> }
            >
                <span class=subtext_class_clone.clone()>{appended_text_clone.clone()}</span>
            </Show>
        </div>
    }
}

#[component]
pub fn PriceDisplayV2(
    // #[prop(into]
    price: impl Fn() -> f64 + 'static,
    #[prop(default = "$".into(), into)] currency: String,
    #[prop(default = "".into(), into)] base_class: String,
    #[prop(default = "font-semibold".into(), into)] price_class: String,
    #[prop(default = "font-light text-sm".into(), into)] subtext_class: String,
    #[prop(default = None, into)] appended_text: Option<String>,
) -> impl IntoView {
    let price_clone = || price();

    log!(
        "PriceDisplayV2 -appended_text - {:?} , price  -  {}",
        appended_text,
        price_clone()
    );

    let cr = currency.clone();
    let formatted_price = move || {
        let mut ac = Accounting::new_from(&cr, 2);
        ac.set_format("{s} {v}");
        ac.format_money(price())
    };

    let merged_base_class = format!("{} flex items-center space-x-1", base_class);
    let appended_text_clone = appended_text.clone();
    let subtext_class_clone = subtext_class.clone();

    view! {
        <div class=merged_base_class>
            <span class=price_class>{formatted_price()}</span>
            <Show
                when=move || !appended_text.clone().is_none()
                fallback=move || view! { <span class=subtext_class.clone()>" / night"</span> }
            >
                <span class=subtext_class_clone.clone()>{appended_text_clone.clone()}</span>
            </Show>
        </div>
    }
}
