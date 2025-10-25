use leptos::html::Input;
use leptos::prelude::*;

// Expose these so consuming pages can reuse defaults
pub const MIN_PRICE: f64 = 0.0;
pub const MAX_PRICE: f64 = 2_000.0;
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
    #[prop(into)] value: Signal<Option<(f64, f64)>>,
    on_select: Callback<Option<(f64, f64)>>,
) -> impl IntoView {
    let min_value = Signal::derive(move || value().map(|(lo, _)| lo).unwrap_or(MIN_PRICE));
    let max_value = Signal::derive(move || value().map(|(_, hi)| hi).unwrap_or(MAX_PRICE));

    let formatted_value = Signal::derive(move || match value() {
        None => "Any price".to_string(),
        Some((lo, hi)) => {
            let at_min = (lo - MIN_PRICE).abs() < f64::EPSILON;
            let at_max = (hi - MAX_PRICE).abs() < f64::EPSILON;
            match (at_min, at_max) {
                (true, true) => "Any price".to_string(),
                (true, false) => format!("Up to {}", format_price_range_value(hi)),
                (false, true) => format!("From {}", format_price_range_value(lo)),
                (false, false) => format!(
                    "{} - {}",
                    format_price_range_value(lo),
                    format_price_range_value(hi)
                ),
            }
        }
    });

    let on_select_for_clear = on_select.clone();
    let on_select_for_min = on_select.clone();
    let on_select_for_max = on_select.clone();

    let min_slider_ref = create_node_ref::<Input>();
    let max_slider_ref = create_node_ref::<Input>();

    // Keep DOM inputs in sync with external value changes
    Effect::new(move |_| {
        if let Some(input) = min_slider_ref.get() {
            let target = min_value();
            input.set_value(&format!("{:.0}", target));
        }
    });
    Effect::new(move |_| {
        if let Some(input) = max_slider_ref.get() {
            let target = max_value();
            input.set_value(&format!("{:.0}", target));
        }
    });

    // Precompute percentages for the selection bar
    let selection_left_pct = Signal::derive(move || {
        ((min_value() - MIN_PRICE) / (MAX_PRICE - MIN_PRICE) * 100.0).clamp(0.0, 100.0)
    });
    let selection_width_pct = Signal::derive(move || {
        ((max_value() - min_value()) / (MAX_PRICE - MIN_PRICE) * 100.0).clamp(0.0, 100.0)
    });

    view! {
        <div>
            <style>
                "/* Dual range slider styles */
                .range-slider-min {
                    pointer-events: none;
                }
                .range-slider-min::-webkit-slider-thumb {
                    pointer-events: auto;
                }
                .range-slider-min::-moz-range-thumb {
                    pointer-events: auto;
                }
                .range-slider-max {
                    pointer-events: none;
                }
                .range-slider-max::-webkit-slider-thumb {
                    pointer-events: auto;
                }
                .range-slider-max::-moz-range-thumb {
                    pointer-events: auto;
                }"
            </style>
            <div class="flex items-center justify-between">
                <h3 class="text-base font-medium text-gray-900">
                    "Price Range"
                </h3>
                <button
                    type="button"
                    class="text-sm text-blue-600 hover:text-blue-700 disabled:text-gray-400 disabled:hover:text-gray-400"
                    disabled=move || value().is_none()
                    on:click=move |_| {
                        on_select_for_clear.run(None);
                    }
                >
                    "Clear"
                </button>
            </div>

            <div class="mt-4 flex flex-col gap-2">
                <div class="flex items-center justify-between text-xs text-gray-500">
                    <span>{format_price_range_value(MIN_PRICE)}</span>
                    <span>{move || formatted_value() }</span>
                    <span>{format_price_range_value(MAX_PRICE)}</span>
                </div>

                // Slider wrapper with custom selection bar
                <div class="relative h-8">
                    // Base track
                    <div class="absolute top-1/2 -translate-y-1/2 w-full h-2 rounded-lg bg-gray-200"></div>
                    // Selected range bar
                    <div
                        class="absolute top-1/2 -translate-y-1/2 h-2 rounded-lg bg-blue-500"
                        style:left=move || format!("{:.6}%", selection_left_pct())
                        style:width=move || format!("{:.6}%", selection_width_pct())
                    ></div>

                    // Min handle - uses CSS to make only the thumb clickable
                    <input
                        node_ref=min_slider_ref
                        type="range"
                        min=MIN_PRICE
                        max=MAX_PRICE
                        step=SLIDER_STEP
                        prop:value=move || format!("{:.0}", min_value())
                        class="range-slider-min absolute top-0 left-0 w-full h-8 appearance-none bg-transparent accent-blue-600 z-20"
                        on:input=move |ev| {
                            if let Ok(raw) = event_target_value(&ev).parse::<f64>() {
                                let current_max = max_value();
                                let clamped = raw.clamp(MIN_PRICE, current_max);
                                let next = Some((clamped, current_max));
                                on_select_for_min.run(next);
                            }
                        }
                    />

                    // Max handle - uses CSS to make only the thumb clickable
                    <input
                        node_ref=max_slider_ref
                        type="range"
                        min=MIN_PRICE
                        max=MAX_PRICE
                        step=SLIDER_STEP
                        prop:value=move || format!("{:.0}", max_value())
                        class="range-slider-max absolute top-0 left-0 w-full h-8 appearance-none bg-transparent accent-blue-600 z-20"
                        on:input=move |ev| {
                            if let Ok(raw) = event_target_value(&ev).parse::<f64>() {
                                let current_min = min_value();
                                let clamped = raw.clamp(current_min, MAX_PRICE);
                                let next = Some((current_min, clamped));
                                on_select_for_max.run(next);
                            }
                        }
                    />
                </div>

                <p class="text-sm text-gray-600">
                    {move || formatted_value()}
                    <span class="ml-1 text-xs text-gray-500">"per night"</span>
                </p>
            </div>
        </div>
    }
}
