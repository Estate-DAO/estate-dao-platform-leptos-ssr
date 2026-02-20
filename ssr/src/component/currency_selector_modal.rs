use leptos::{ev::Event, *};

use crate::utils::currency::{
    currency_name_for_code, get_currency_from_local_storage, notify_currency_change,
    set_currency_in_local_storage, validate_currency_code, SupportedCurrency,
    DEFAULT_CURRENCY_CODE, SUGGESTED_CURRENCY_CODES, SUPPORTED_LITEAPI_CURRENCIES,
};

fn matches_query(currency: &SupportedCurrency, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }

    let query = query.to_ascii_lowercase();
    currency.code.to_ascii_lowercase().contains(&query)
        || currency.name.to_ascii_lowercase().contains(&query)
}

#[component]
pub fn CurrencySelectorModal() -> impl IntoView {
    let show_modal = create_rw_signal(false);
    let search_query = create_rw_signal(String::new());
    let selected_currency = create_rw_signal(DEFAULT_CURRENCY_CODE.to_string());

    create_effect(move |_| {
        #[cfg(not(feature = "ssr"))]
        {
            let selected_or_default = get_currency_from_local_storage()
                .or_else(|| validate_currency_code(DEFAULT_CURRENCY_CODE))
                .unwrap_or_else(|| DEFAULT_CURRENCY_CODE.to_string());

            selected_currency.set(selected_or_default.clone());

            if get_currency_from_local_storage().is_none() {
                set_currency_in_local_storage(&selected_or_default);
                show_modal.set(true);
            }
        }

        #[cfg(feature = "ssr")]
        {
            selected_currency.set(DEFAULT_CURRENCY_CODE.to_string());
        }
    });

    let filtered_currencies = create_memo(move |_| {
        let query = search_query.get();
        SUPPORTED_LITEAPI_CURRENCIES
            .into_iter()
            .filter(|currency| matches_query(currency, &query))
            .collect::<Vec<_>>()
    });

    let suggested_currencies = create_memo(move |_| {
        let query = search_query.get();
        SUGGESTED_CURRENCY_CODES
            .iter()
            .filter_map(|code| {
                SUPPORTED_LITEAPI_CURRENCIES
                    .iter()
                    .find(|currency| currency.code == *code)
                    .copied()
            })
            .filter(|currency| matches_query(currency, &query))
            .collect::<Vec<_>>()
    });

    let select_currency = Callback::new(move |code: String| {
        if let Some(valid_currency) = validate_currency_code(&code) {
            selected_currency.set(valid_currency.clone());
            set_currency_in_local_storage(&valid_currency);
            notify_currency_change();
            show_modal.set(false);
        }
    });

    view! {
        <div class="tooltip-container lang_container">
            <button
                class="item language_currency radius_sm inline-flex h-11 w-11 items-center justify-center rounded-lg border border-gray-300 bg-white text-gray-500 shadow-sm transition hover:bg-gray-50 hover:text-gray-700"
                on:click=move |_| show_modal.set(true)
                title=move || {
                    let code = selected_currency.get();
                    let name = currency_name_for_code(&code).unwrap_or("US Dollar");
                    format!("Selected currency: {} ({})", name, code)
                }
                aria-label=move || {
                    let code = selected_currency.get();
                    let name = currency_name_for_code(&code).unwrap_or("US Dollar");
                    format!("Open currency selector, current currency {} ({})", name, code)
                }
            >
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    width="18"
                    height="18"
                    viewBox="0 0 18 18"
                    fill="none"
                    role="img"
                    class="globe"
                    aria-hidden="true"
                >
                    <g clip-path="url(#clip0_currency_selector)">
                        <path d="M9 0.999767C13.4285 0.999767 17.0001 4.57135 17.0001 8.99983C17.0001 13.4283 13.4285 16.9999 9 16.9999" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"></path>
                        <path d="M9.00006 17C4.57158 17 1 13.4284 1 8.99995C1 4.57147 4.57158 0.999884 9.00006 0.999884" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"></path>
                        <path d="M7.37151 1.94177C4.8026 6.2307 4.8026 11.7694 7.37151 16.0583C8.12351 17.3143 9.8773 17.3143 10.6293 16.0583C13.1982 11.7694 13.1982 6.2307 10.6293 1.94177C9.87642 0.685765 8.12351 0.685765 7.37151 1.94177Z" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"></path>
                        <path d="M1 8.99989H17.0001" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"></path>
                    </g>
                    <defs>
                        <clipPath id="clip0_currency_selector">
                            <rect width="18" height="18" fill="white"></rect>
                        </clipPath>
                    </defs>
                </svg>
            </button>
        </div>

        <Show when=move || show_modal.get()>
            <div class="fixed inset-0 z-[1200] flex items-center justify-center bg-black/50 p-4">
                <div class="flex h-[85vh] w-full max-w-4xl flex-col overflow-hidden rounded-xl bg-white shadow-2xl">
                    <div class="flex items-center justify-between border-b border-gray-200 px-6 py-4">
                        <h2 class="text-xl font-semibold text-gray-900">"Choose your currency"</h2>
                        <button
                            class="rounded-md border border-gray-300 px-3 py-1.5 text-sm font-medium text-gray-700 transition hover:bg-gray-100"
                            on:click=move |_| show_modal.set(false)
                        >
                            "Close"
                        </button>
                    </div>

                    <div class="flex-1 overflow-y-auto p-6">
                        <div class="mb-8 space-y-4">
                            <div class="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
                                <h3 class="text-lg font-semibold text-gray-900">"Suggested currencies"</h3>
                                <input
                                    class="w-full rounded-lg border border-gray-300 px-3 py-2 text-sm outline-none transition focus:border-blue-500 md:w-80"
                                    type="text"
                                    placeholder="Search for a currency"
                                    prop:value=move || search_query.get()
                                    on:input=move |ev: Event| search_query.set(event_target_value(&ev))
                                />
                            </div>

                            <div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
                                <For
                                    each=move || suggested_currencies.get()
                                    key=|currency| currency.code
                                    children=move |currency| {
                                        let code = currency.code;
                                        let name = currency.name;
                                        view! {
                                            <button
                                                class=move || {
                                                    let is_selected = selected_currency.get() == code;
                                                    format!(
                                                        "flex items-center justify-between rounded-lg border px-4 py-3 text-left transition {}",
                                                        if is_selected {
                                                            "border-blue-500 bg-blue-50"
                                                        } else {
                                                            "border-gray-200 hover:border-blue-300 hover:bg-blue-50/40"
                                                        }
                                                    )
                                                }
                                                on:click=move |_| leptos::Callable::call(&select_currency, code.to_string())
                                            >
                                                <span class="font-semibold text-gray-900">{name}</span>
                                                <span class="text-sm font-medium text-gray-600">{code}</span>
                                            </button>
                                        }
                                    }
                                />
                            </div>
                        </div>

                        <div class="space-y-4">
                            <h3 class="text-lg font-semibold text-gray-900">"All currencies"</h3>
                            <div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
                                <For
                                    each=move || filtered_currencies.get()
                                    key=|currency| currency.code
                                    children=move |currency| {
                                        let code = currency.code;
                                        let name = currency.name;
                                        view! {
                                            <button
                                                class=move || {
                                                    let is_selected = selected_currency.get() == code;
                                                    format!(
                                                        "flex items-center justify-between rounded-lg border px-4 py-3 text-left transition {}",
                                                        if is_selected {
                                                            "border-blue-500 bg-blue-50"
                                                        } else {
                                                            "border-gray-200 hover:border-blue-300 hover:bg-blue-50/40"
                                                        }
                                                    )
                                                }
                                                on:click=move |_| leptos::Callable::call(&select_currency, code.to_string())
                                            >
                                                <span class="font-semibold text-gray-900">{name}</span>
                                                <span class="text-sm font-medium text-gray-600">{code}</span>
                                            </button>
                                        }
                                    }
                                />
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </Show>
    }
}
