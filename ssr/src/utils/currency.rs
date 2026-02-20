#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SupportedCurrency {
    pub code: &'static str,
    pub name: &'static str,
}

pub const DEFAULT_CURRENCY_CODE: &str = "USD";
pub const CURRENCY_STORAGE_KEY: &str = "estate_selected_currency";
pub const CURRENCY_CHANGE_EVENT: &str = "estate:currency-changed";

pub const SUPPORTED_LITEAPI_CURRENCIES: [SupportedCurrency; 62] = [
    SupportedCurrency {
        code: "AED",
        name: "UAE Dirham",
    },
    SupportedCurrency {
        code: "AMD",
        name: "Armenian Dram",
    },
    SupportedCurrency {
        code: "ARS",
        name: "Argentine Peso",
    },
    SupportedCurrency {
        code: "AUD",
        name: "Australian Dollar",
    },
    SupportedCurrency {
        code: "AZN",
        name: "Azerbaijan Manat",
    },
    SupportedCurrency {
        code: "BHD",
        name: "Bahraini Dinar",
    },
    SupportedCurrency {
        code: "BRL",
        name: "Brazilian Real",
    },
    SupportedCurrency {
        code: "CAD",
        name: "Canadian Dollar",
    },
    SupportedCurrency {
        code: "CHF",
        name: "Swiss Franc",
    },
    SupportedCurrency {
        code: "CLP",
        name: "Chilean Peso",
    },
    SupportedCurrency {
        code: "CNY",
        name: "Yuan Renminbi",
    },
    SupportedCurrency {
        code: "COP",
        name: "Colombian Peso",
    },
    SupportedCurrency {
        code: "CVE",
        name: "Cabo Verde Escudo",
    },
    SupportedCurrency {
        code: "CZK",
        name: "Czech Koruna",
    },
    SupportedCurrency {
        code: "DKK",
        name: "Danish Krone",
    },
    SupportedCurrency {
        code: "DOP",
        name: "Dominican Peso",
    },
    SupportedCurrency {
        code: "EGP",
        name: "Egyptian Pound",
    },
    SupportedCurrency {
        code: "EUR",
        name: "Euro",
    },
    SupportedCurrency {
        code: "FJD",
        name: "Fiji Dollar",
    },
    SupportedCurrency {
        code: "GBP",
        name: "Pound Sterling",
    },
    SupportedCurrency {
        code: "GEL",
        name: "Lari",
    },
    SupportedCurrency {
        code: "GHS",
        name: "Ghana Cedi",
    },
    SupportedCurrency {
        code: "HKD",
        name: "Hong Kong Dollar",
    },
    SupportedCurrency {
        code: "HUF",
        name: "Forint",
    },
    SupportedCurrency {
        code: "IDR",
        name: "Rupiah",
    },
    SupportedCurrency {
        code: "ILS",
        name: "New Israeli Sheqel",
    },
    SupportedCurrency {
        code: "INR",
        name: "Indian Rupee",
    },
    SupportedCurrency {
        code: "ISK",
        name: "Iceland Krona",
    },
    SupportedCurrency {
        code: "JOD",
        name: "Jordanian Dinar",
    },
    SupportedCurrency {
        code: "JPY",
        name: "Yen",
    },
    SupportedCurrency {
        code: "KRW",
        name: "Won",
    },
    SupportedCurrency {
        code: "KWD",
        name: "Kuwaiti Dinar",
    },
    SupportedCurrency {
        code: "KZT",
        name: "Tenge",
    },
    SupportedCurrency {
        code: "LKR",
        name: "Sri Lanka Rupee",
    },
    SupportedCurrency {
        code: "MAD",
        name: "Moroccan Dirham",
    },
    SupportedCurrency {
        code: "MNT",
        name: "Tugrik",
    },
    SupportedCurrency {
        code: "MUR",
        name: "Mauritius Rupee",
    },
    SupportedCurrency {
        code: "MXN",
        name: "Mexican Peso",
    },
    SupportedCurrency {
        code: "MYR",
        name: "Malaysian Ringgit",
    },
    SupportedCurrency {
        code: "NGN",
        name: "Naira",
    },
    SupportedCurrency {
        code: "NOK",
        name: "Norwegian Krone",
    },
    SupportedCurrency {
        code: "NZD",
        name: "New Zealand Dollar",
    },
    SupportedCurrency {
        code: "OMR",
        name: "Rial Omani",
    },
    SupportedCurrency {
        code: "PEN",
        name: "Sol",
    },
    SupportedCurrency {
        code: "PHP",
        name: "Philippine Peso",
    },
    SupportedCurrency {
        code: "PKR",
        name: "Pakistan Rupee",
    },
    SupportedCurrency {
        code: "PLN",
        name: "Zloty",
    },
    SupportedCurrency {
        code: "QAR",
        name: "Qatari Rial",
    },
    SupportedCurrency {
        code: "RON",
        name: "Romanian Leu",
    },
    SupportedCurrency {
        code: "RUB",
        name: "Russian Ruble",
    },
    SupportedCurrency {
        code: "SAR",
        name: "Saudi Riyal",
    },
    SupportedCurrency {
        code: "SEK",
        name: "Swedish Krona",
    },
    SupportedCurrency {
        code: "SGD",
        name: "Singapore Dollar",
    },
    SupportedCurrency {
        code: "THB",
        name: "Baht",
    },
    SupportedCurrency {
        code: "TRY",
        name: "Turkish Lira",
    },
    SupportedCurrency {
        code: "TWD",
        name: "New Taiwan Dollar",
    },
    SupportedCurrency {
        code: "UAH",
        name: "Hryvnia",
    },
    SupportedCurrency {
        code: "USD",
        name: "US Dollar",
    },
    SupportedCurrency {
        code: "VND",
        name: "Dong",
    },
    SupportedCurrency {
        code: "XOF",
        name: "CFA Franc BCEAO",
    },
    SupportedCurrency {
        code: "XPF",
        name: "CFP Franc",
    },
    SupportedCurrency {
        code: "ZAR",
        name: "Rand",
    },
];

pub const SUGGESTED_CURRENCY_CODES: [&str; 4] = ["USD", "EUR", "GBP", "CAD"];

pub fn normalize_currency_code(code: &str) -> String {
    code.trim().to_ascii_uppercase()
}

pub fn is_supported_currency_code(code: &str) -> bool {
    let normalized = normalize_currency_code(code);
    SUPPORTED_LITEAPI_CURRENCIES
        .iter()
        .any(|currency| currency.code == normalized)
}

pub fn validate_currency_code(code: &str) -> Option<String> {
    let normalized = normalize_currency_code(code);
    if is_supported_currency_code(&normalized) {
        Some(normalized)
    } else {
        None
    }
}

pub fn currency_name_for_code(code: &str) -> Option<&'static str> {
    let normalized = normalize_currency_code(code);
    SUPPORTED_LITEAPI_CURRENCIES
        .iter()
        .find(|currency| currency.code == normalized)
        .map(|currency| currency.name)
}

pub fn currency_symbol_for_code(code: &str) -> String {
    let normalized = normalize_currency_code(code);
    match normalized.as_str() {
        "USD" => "$".to_string(),
        "EUR" => "€".to_string(),
        "GBP" => "£".to_string(),
        "INR" => "₹".to_string(),
        "JPY" | "CNY" => "¥".to_string(),
        _ => format!("{normalized} "),
    }
}

pub fn resolve_currency_code(raw_currency: Option<&str>) -> String {
    raw_currency
        .and_then(validate_currency_code)
        .or_else(get_currency_from_local_storage)
        .unwrap_or_else(|| DEFAULT_CURRENCY_CODE.to_string())
}

#[cfg(not(feature = "ssr"))]
pub fn get_currency_from_local_storage() -> Option<String> {
    let window = web_sys::window()?;
    let storage = window.local_storage().ok().flatten()?;
    let stored_value = storage.get_item(CURRENCY_STORAGE_KEY).ok().flatten()?;
    validate_currency_code(&stored_value)
}

#[cfg(feature = "ssr")]
pub fn get_currency_from_local_storage() -> Option<String> {
    None
}

#[cfg(not(feature = "ssr"))]
pub fn set_currency_in_local_storage(code: &str) {
    if let Some(valid_currency) = validate_currency_code(code) {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let _ = storage.set_item(CURRENCY_STORAGE_KEY, &valid_currency);
            }
        }
    }
}

#[cfg(feature = "ssr")]
pub fn set_currency_in_local_storage(_code: &str) {}

#[cfg(not(feature = "ssr"))]
pub fn notify_currency_change() {
    if let Some(window) = web_sys::window() {
        if let Ok(event) = web_sys::Event::new(CURRENCY_CHANGE_EVENT) {
            let _ = window.dispatch_event(&event);
        }
    }
}

#[cfg(feature = "ssr")]
pub fn notify_currency_change() {}
