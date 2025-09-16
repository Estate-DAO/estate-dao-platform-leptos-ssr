use leptos::*;

const MIN_PRICE: f64 = 0.0;
const MAX_PRICE: f64 = 10_000.0;
const SLIDER_STEP: f64 = 50.0;

fn format_with_commas(value: i64) -> String {
    let digits = value.abs().to_string();
    let mut formatted = String::with_capacity(digits.len() + digits.len() / 3);

    for (index, ch) in digits.chars().rev().enumerate() {
        if index != 0 && index % 3 == 0 {
            formatted.push(',');
        }
        formatted.push(ch);
    }

    let mut result: String = formatted.chars().rev().collect();
    if value.is_negative() {
        result.insert(0, '-');
    }
    result
}

pub fn format_price_range_value(value: f64) -> String {
    let rounded = value.round().max(0.0) as i64;
    format!("${}", format_with_commas(rounded))
}

#[component]
pub fn PriceRangeFilter(
    #[prop(into)] value: Signal<Option<f64>>,
    on_select: Callback<Option<f64>>,
) -> impl IntoView {
    let slider_value = Signal::derive(move || value().unwrap_or(MAX_PRICE));
    let formatted_value = Signal::derive(move || {
        value()
            .map(|current| format!("Up to {}", format_price_range_value(current)))
            .unwrap_or_else(|| "Any price".to_string())
    });
    let formatted_slider_value = Signal::derive(move || format_price_range_value(slider_value()));

    let on_select_for_clear = on_select.clone();
    let on_select_for_input = on_select.clone();

    view! {
        <div class="bg-white">
            <div class="flex items-center justify-between">
                <h3 class="text-base font-medium text-gray-900">
                    "Price Range"
                </h3>
                <button
                    type="button"
                    class="text-sm text-blue-600 hover:text-blue-700 disabled:text-gray-400 disabled:hover:text-gray-400"
                    disabled=move || value().is_none()
                    on:click=move |_| {
                        leptos::Callable::call(&on_select_for_clear, None);
                    }
                >
                    "Clear"
                </button>
            </div>

            <div class="mt-4 flex flex-col gap-2">
                <div class="flex items-center justify-between text-xs text-gray-500">
                    <span>{format_price_range_value(MIN_PRICE)}</span>
                    <span>{move || formatted_slider_value() }</span>
                    <span>{format_price_range_value(MAX_PRICE)}</span>
                </div>
                <input
                    type="range"
                    min=MIN_PRICE
                    max=MAX_PRICE
                    step=SLIDER_STEP
                    value=move || format!("{:.0}", slider_value())
                    class="w-full h-2 rounded-lg bg-gray-200 accent-blue-600"
                    on:input=move |ev| {
                        if let Ok(value) = event_target_value(&ev).parse::<f64>() {
                            let clamped = value.clamp(MIN_PRICE, MAX_PRICE);
                            leptos::Callable::call(&on_select_for_input, Some(clamped));
                        }
                    }
                />
                <p class="text-sm text-gray-600">
                    {move || formatted_value()}
                    <span class="ml-1 text-xs text-gray-500">"per night"</span>
                </p>
            </div>
        </div>
    }
}
