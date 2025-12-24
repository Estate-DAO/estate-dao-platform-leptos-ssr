use crate::component::data_table_3::DataTableV3;
use leptos::*;

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

    // Report all errors
    service.report(json_error).await;
    service.report(http_error).await;
    service.report(payment_error).await;
    service.report(booking_error).await;

    // Flush immediately so the test email goes out now
    match service.flush().await {
        Ok(_) => Ok("Test email sent with 4 error types!".to_string()),
        Err(e) => Err(ServerFnError::ServerError(format!("Flush failed: {}", e))),
    }
}

/// Server function to flush pending errors
#[server(FlushPendingErrors, "/api")]
pub async fn flush_pending_errors() -> Result<String, ServerFnError> {
    use crate::init::get_error_alert_service;

    let service = get_error_alert_service();

    match service.flush().await {
        Ok(_) => Ok("Pending errors flushed!".to_string()),
        Err(e) => Err(ServerFnError::ServerError(format!("Flush failed: {}", e))),
    }
}

#[component]
pub fn AdminPanelPage() -> impl IntoView {
    // Server actions using Leptos server functions
    let send_test_action = create_server_action::<SendTestErrorAlert>();
    let flush_action = create_server_action::<FlushPendingErrors>();

    view! {
        <div style="padding: 20px;">
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
                        {move || if flush_action.pending().get() { "Flushing..." } else { "Flush Pending Errors" }}
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
