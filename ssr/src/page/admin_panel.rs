use crate::component::data_table_3::DataTableV3;
use leptos::*;

/// Server function to send a test error alert
#[server(SendTestErrorAlert, "/api")]
pub async fn send_test_error_alert() -> Result<String, ServerFnError> {
    use crate::init::get_error_alert_service;
    use crate::utils::error_alerts::{CriticalError, ErrorType};

    let service = get_error_alert_service();

    // Create a test error
    let test_error = CriticalError::new(
        ErrorType::BookingProviderFailure {
            provider: "test".to_string(),
            hotel_id: Some("TEST-HOTEL-123".to_string()),
            operation: "admin_test".to_string(),
        },
        "This is a TEST error triggered from the admin panel to verify the error alert system is working correctly.",
    )
    .with_request("POST", "/api/send_test_error_alert")
    .with_source(
        "ssr/src/page/admin_panel.rs",
        6,
        "send_test_error_alert",
    );

    // Report the error
    service.report(test_error).await;

    // Flush immediately so the test email goes out now
    match service.flush().await {
        Ok(_) => Ok("Test error email sent!".to_string()),
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
