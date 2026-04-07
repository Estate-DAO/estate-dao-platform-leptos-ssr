use crate::api::client_side_api::{ClientSideApiClient, HotelProviderConfigResponse};
use crate::component::data_table_3::DataTableV3;
use leptos::*;

fn hotel_provider_options(config: &HotelProviderConfigResponse) -> Vec<String> {
    let mut providers = Vec::new();

    for provider in std::iter::once(config.primary_hotel_provider.as_str())
        .chain(config.available_providers.iter().map(String::as_str))
    {
        let trimmed = provider.trim();

        if !trimmed.is_empty() && !providers.iter().any(|existing| existing == trimmed) {
            providers.push(trimmed.to_string());
        }
    }

    providers
}

fn selected_hotel_provider(config: &HotelProviderConfigResponse) -> String {
    let primary = config.primary_hotel_provider.trim();

    if !primary.is_empty() {
        primary.to_string()
    } else {
        hotel_provider_options(config)
            .into_iter()
            .next()
            .unwrap_or_default()
    }
}

fn hotel_provider_label(provider: &str) -> String {
    match provider.trim().to_ascii_lowercase().as_str() {
        "liteapi" => "LiteAPI".to_string(),
        "booking" => "Booking".to_string(),
        "amadeus" => "Amadeus".to_string(),
        other => other.to_string(),
    }
}

/// Server function to send a test error alert
#[server(SendTestErrorAlert, "/api")]
pub async fn send_test_error_alert() -> Result<String, ServerFnError> {
    use crate::init::get_error_alert_service;
    use crate::utils::error_alerts::{CriticalError, ErrorType};

    let service = get_error_alert_service();

    // Create test errors of ALL types to demonstrate the email format

    // 1. JSON Parse Error
    let json_error = CriticalError::new(
        ErrorType::JsonParseFailed {
            json_path: Some("response.data.promotions".to_string()),
            expected_type: Some("array".to_string()),
            actual_type: Some("string".to_string()),
        },
        "Failed to parse LiteAPI response: expected array but got string for promotions field",
    )
    .with_request("GET", "/api/hotels/liteapi/rates?hotel_id=12345")
    .with_source(
        "ssr/src/api/liteapi/l01_get_hotel_info_rates.rs",
        107,
        "deserialize_promotions",
    );

    // 2. HTTP 500 Error
    let http_error = CriticalError::new(
        ErrorType::Http500 {
            status_code: 502,
            response_body: Some(
                "{\"error\": \"Bad Gateway\", \"message\": \"Upstream service unavailable\"}"
                    .to_string(),
            ),
        },
        "External API returned 502 Bad Gateway",
    )
    .with_request("POST", "/api/hotels/search")
    .with_source(
        "ssr/src/adapters/liteapi_adapter/mod.rs",
        156,
        "search_hotels",
    );

    // 3. Payment Failure
    let payment_error = CriticalError::new(
        ErrorType::PaymentFailure {
            payment_id: Some("PAY-TEST-789".to_string()),
            provider: "NowPayments".to_string(),
            failure_reason: Some("Insufficient funds in wallet".to_string()),
        },
        "Payment processing failed for booking NFB-2024-12345",
    )
    .with_request("POST", "/webhook/nowpayments")
    .with_user("user@example.com")
    .with_source("ssr/src/main.rs", 312, "handle_payment_webhook");

    // 4. Booking Provider Failure
    let booking_error = CriticalError::new(
        ErrorType::BookingProviderFailure {
            provider: "liteapi".to_string(),
            hotel_id: Some("HTL-DEMO-456".to_string()),
            operation: "block_room".to_string(),
        },
        "Room blocking failed: Rate no longer available",
    )
    .with_request("POST", "/api/book_room")
    .with_user("customer@test.com")
    .with_source(
        "ssr/src/application_services/booking_service.rs",
        89,
        "block_room",
    );

    // 5. Other / Generic Error
    let other_error = CriticalError::new(
        ErrorType::Other {
            category: "Database".to_string(),
            details: Some("Connection pool exhausted".to_string()),
        },
        "Failed to acquire database connection from pool",
    )
    .with_source("ssr/src/db.rs", 42, "get_connection");

    // Report all errors
    service.report(json_error).await;
    service.report(http_error).await;
    service.report(payment_error).await;
    service.report(booking_error).await;
    service.report(other_error).await;

    // Flush immediately so the test email goes out now
    match service.flush().await {
        Ok(_) => Ok("Test email sent with 5 error types!".to_string()),
        Err(e) => Err(ServerFnError::ServerError(format!("Flush failed: {}", e))),
    }
}

/// Server function to flush pending errors
#[server(FlushPendingErrors, "/api")]
pub async fn flush_pending_errors() -> Result<String, ServerFnError> {
    use crate::init::get_error_alert_service;

    let service = get_error_alert_service();
    let count = service.pending_count().await;

    if count == 0 {
        return Ok("No pending errors to flush.".to_string());
    }

    match service.flush().await {
        Ok(_) => Ok(format!("Flushed {} error(s)!", count)),
        Err(e) => Err(ServerFnError::ServerError(format!("Flush failed: {}", e))),
    }
}

/// Server function to get pending error count
#[server(GetPendingErrorCount, "/api")]
pub async fn get_pending_error_count() -> Result<usize, ServerFnError> {
    use crate::init::get_error_alert_service;

    let service = get_error_alert_service();
    Ok(service.pending_count().await)
}

#[component]
pub fn AdminPanelPage() -> impl IntoView {
    // Server actions using Leptos server functions
    let send_test_action = create_server_action::<SendTestErrorAlert>();
    let flush_action = create_server_action::<FlushPendingErrors>();
    let current_hotel_provider = create_rw_signal(String::new());
    let hotel_provider_options_signal = create_rw_signal(Vec::<String>::new());
    let selected_hotel_provider_signal = create_rw_signal(String::new());
    let hotel_provider_status = create_rw_signal(String::new());

    let load_provider_action = create_action(move |_: &()| async move {
        ClientSideApiClient::new().get_hotel_provider_config().await
    });

    let update_provider_action = create_action(move |provider: &String| {
        let provider = provider.clone();
        async move {
            ClientSideApiClient::new()
                .update_hotel_provider_config(provider)
                .await
        }
    });

    // Resource to fetch pending error count (refreshes when flush completes)
    let pending_count = create_resource(
        move || {
            (
                send_test_action.version().get(),
                flush_action.version().get(),
            )
        },
        |_| async move { get_pending_error_count().await.unwrap_or(0) },
    );

    #[cfg(not(feature = "ssr"))]
    {
        let load_provider_action = load_provider_action.clone();
        let initial_provider_load = create_rw_signal(false);

        create_effect(move |_| {
            if !initial_provider_load.get() {
                initial_provider_load.set(true);
                load_provider_action.dispatch(());
            }
        });
    }

    create_effect(move |_| {
        if let Some(result) = load_provider_action.value().get() {
            match result {
                Ok(config) => {
                    let options = hotel_provider_options(&config);
                    let selected = selected_hotel_provider(&config);
                    current_hotel_provider.set(config.primary_hotel_provider.clone());
                    hotel_provider_options_signal.set(options);
                    selected_hotel_provider_signal.set(selected);
                    hotel_provider_status.set(String::new());
                }
                Err(error) => {
                    hotel_provider_status
                        .set(format!("Failed to load hotel provider config: {}", error));
                }
            }
        }
    });

    create_effect(move |_| {
        if let Some(result) = update_provider_action.value().get() {
            match result {
                Ok(config) => {
                    let options = hotel_provider_options(&config);
                    let selected = selected_hotel_provider(&config);
                    current_hotel_provider.set(config.primary_hotel_provider.clone());
                    hotel_provider_options_signal.set(options);
                    selected_hotel_provider_signal.set(selected);
                    hotel_provider_status.set(format!(
                        "Primary provider updated to {}.",
                        hotel_provider_label(&config.primary_hotel_provider)
                    ));
                }
                Err(error) => {
                    hotel_provider_status
                        .set(format!("Failed to update hotel provider config: {}", error));
                }
            }
        }
    });

    view! {
        <div style="padding: 20px;">
            // Hotel Provider Controls Section
            <div style="margin-bottom: 24px; padding: 16px; background: #f9fafb; border-radius: 8px; border: 1px solid #e5e7eb;">
                <h3 style="margin: 0 0 12px 0; font-size: 16px; color: #374151;">"Hotel Provider"</h3>
                <div style="display: flex; gap: 12px; align-items: end; flex-wrap: wrap;">
                    <div style="min-width: 240px;">
                        <label style="display: block; margin-bottom: 6px; font-size: 14px; color: #4b5563;">
                            "Primary provider"
                        </label>
                        <select
                            prop:value=move || selected_hotel_provider_signal.get()
                            on:change=move |ev| selected_hotel_provider_signal.set(event_target_value(&ev))
                            disabled=move || load_provider_action.pending().get() || update_provider_action.pending().get()
                            style="width: 100%; padding: 8px 12px; border: 1px solid #d1d5db; border-radius: 6px; background: white; color: #111827;"
                        >
                            <option value="" disabled=true>
                                {move || {
                                    if load_provider_action.pending().get() && hotel_provider_options_signal.get().is_empty() {
                                        "Loading providers...".to_string()
                                    } else {
                                        "Select provider".to_string()
                                    }
                                }}
                            </option>
                            <For
                                each=move || hotel_provider_options_signal.get()
                                key=|provider| provider.clone()
                                let:provider
                            >
                                <option value=provider.clone()>{hotel_provider_label(&provider)}</option>
                            </For>
                        </select>
                    </div>
                    <button
                        on:click=move |_| {
                            let provider = selected_hotel_provider_signal.get();
                            if !provider.trim().is_empty() {
                                hotel_provider_status.set(String::new());
                                update_provider_action.dispatch(provider);
                            }
                        }
                        disabled=move || {
                            load_provider_action.pending().get()
                                || update_provider_action.pending().get()
                                || selected_hotel_provider_signal.get().trim().is_empty()
                        }
                        style="padding: 8px 16px; background: #2563eb; color: white; border: none; border-radius: 6px; cursor: pointer; font-size: 14px;"
                    >
                        {move || if update_provider_action.pending().get() { "Applying..." } else { "Apply Provider" }}
                    </button>
                    <button
                        on:click=move |_| {
                            hotel_provider_status.set(String::new());
                            load_provider_action.dispatch(());
                        }
                        disabled=move || load_provider_action.pending().get() || update_provider_action.pending().get()
                        style="padding: 8px 16px; background: #e5e7eb; color: #111827; border: none; border-radius: 6px; cursor: pointer; font-size: 14px;"
                    >
                        "Refresh"
                    </button>
                </div>
                <div style="margin-top: 10px; display: flex; gap: 16px; align-items: center; flex-wrap: wrap;">
                    <span style="font-size: 14px; color: #4b5563;">
                        "Current provider: "
                        <strong>{move || {
                            let provider = current_hotel_provider.get();
                            if provider.trim().is_empty() {
                                "Unavailable".to_string()
                            } else {
                                hotel_provider_label(&provider)
                            }
                        }}</strong>
                    </span>
                    <Show
                        when=move || !hotel_provider_status.get().is_empty()
                        fallback=move || view! { <></> }
                    >
                        <span style="font-size: 14px; color: #374151;">
                            {move || hotel_provider_status.get()}
                        </span>
                    </Show>
                </div>
            </div>

            // Error Alert Controls Section
            <div style="margin-bottom: 24px; padding: 16px; background: #f9fafb; border-radius: 8px; border: 1px solid #e5e7eb;">
                <h3 style="margin: 0 0 12px 0; font-size: 16px; color: #374151;">"Error Alert Controls"</h3>
                <div style="display: flex; gap: 12px; align-items: center; flex-wrap: wrap;">
                    <button
                        on:click=move |_| send_test_action.dispatch(SendTestErrorAlert {})
                        disabled=move || send_test_action.pending().get()
                        style="padding: 8px 16px; background: #4f46e5; color: white; border: none; border-radius: 6px; cursor: pointer; font-size: 14px;"
                    >
                        {move || if send_test_action.pending().get() { "Sending..." } else { "Send Test Email" }}
                    </button>
                    <button
                        on:click=move |_| flush_action.dispatch(FlushPendingErrors {})
                        disabled=move || flush_action.pending().get()
                        style="padding: 8px 16px; background: #059669; color: white; border: none; border-radius: 6px; cursor: pointer; font-size: 14px;"
                    >
                        {move || {
                            if flush_action.pending().get() {
                                "Flushing...".to_string()
                            } else {
                                let count = pending_count.get().unwrap_or(0);
                                if count > 0 {
                                    format!("Flush Pending Errors ({})", count)
                                } else {
                                    "Flush Pending Errors".to_string()
                                }
                            }
                        }}
                    </button>
                    // Show result messages
                    {move || send_test_action.value().get().map(|result| {
                        let (msg, color) = match result {
                            Ok(msg) => (msg, "#059669"),
                            Err(e) => (format!("Error: {}", e), "#dc2626"),
                        };
                        view! {
                            <span style=format!("font-size: 14px; margin-left: 8px; color: {};", color)>
                                {msg}
                            </span>
                        }
                    })}
                    {move || flush_action.value().get().map(|result| {
                        let (msg, color) = match result {
                            Ok(msg) => (msg, "#059669"),
                            Err(e) => (format!("Error: {}", e), "#dc2626"),
                        };
                        view! {
                            <span style=format!("font-size: 14px; margin-left: 8px; color: {};", color)>
                                {msg}
                            </span>
                        }
                    })}
                </div>
            </div>

            // Existing DataTable
            <DataTableV3 />
        </div>
    }
    .into_view()
}

#[cfg(test)]
mod tests {
    use super::{hotel_provider_options, selected_hotel_provider};
    use crate::api::client_side_api::HotelProviderConfigResponse;

    #[test]
    fn provider_options_include_primary_when_missing_from_available_list() {
        let config = HotelProviderConfigResponse {
            primary_hotel_provider: "amadeus".to_string(),
            available_providers: vec!["liteapi".to_string(), "booking".to_string()],
        };

        assert_eq!(
            hotel_provider_options(&config),
            vec![
                "amadeus".to_string(),
                "liteapi".to_string(),
                "booking".to_string(),
            ]
        );
    }

    #[test]
    fn provider_options_drop_duplicates_and_blank_values() {
        let config = HotelProviderConfigResponse {
            primary_hotel_provider: "liteapi".to_string(),
            available_providers: vec![
                "".to_string(),
                "liteapi".to_string(),
                "amadeus".to_string(),
                "amadeus".to_string(),
                "booking".to_string(),
            ],
        };

        assert_eq!(
            hotel_provider_options(&config),
            vec![
                "liteapi".to_string(),
                "amadeus".to_string(),
                "booking".to_string(),
            ]
        );
    }

    #[test]
    fn selected_provider_uses_primary_provider_from_config() {
        let config = HotelProviderConfigResponse {
            primary_hotel_provider: "booking".to_string(),
            available_providers: vec!["liteapi".to_string(), "amadeus".to_string()],
        };

        assert_eq!(selected_hotel_provider(&config), "booking".to_string());
    }
}
