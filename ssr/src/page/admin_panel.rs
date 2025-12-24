use crate::component::data_table_3::DataTableV3;
use leptos::*;

#[component]
pub fn AdminPanelPage() -> impl IntoView {
    // Action to send test error alert
    let send_test_action = create_action(|_: &()| async {
        let response = reqwest::Client::new()
            .post("/server_fn_api/admin/test_error_alert")
            .send()
            .await;
        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    "✅ Test error email sent!".to_string()
                } else {
                    format!("❌ Failed: {}", resp.status())
                }
            }
            Err(e) => format!("❌ Error: {}", e),
        }
    });

    // Action to flush pending errors
    let flush_action = create_action(|_: &()| async {
        let response = reqwest::Client::new()
            .post("/server_fn_api/admin/flush_error_alerts")
            .send()
            .await;
        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    "✅ Pending errors flushed!".to_string()
                } else {
                    format!("❌ Failed: {}", resp.status())
                }
            }
            Err(e) => format!("❌ Error: {}", e),
        }
    });

    view! {
        <div style="padding: 20px;">
            // Error Alert Controls Section
            <div style="margin-bottom: 24px; padding: 16px; background: #f9fafb; border-radius: 8px; border: 1px solid #e5e7eb;">
                <h3 style="margin: 0 0 12px 0; font-size: 16px; color: #374151;">"🚨" Error Alert Controls</h3>
                <div style="display: flex; gap: 12px; align-items: center; flex-wrap: wrap;">
                    <button
                        on:click=move |_| send_test_action.dispatch(())
                        disabled=move || send_test_action.pending().get()
                        style="padding: 8px 16px; background: #4f46e5; color: white; border: none; border-radius: 6px; cursor: pointer; font-size: 14px;"
                    >
                        {move || if send_test_action.pending().get() { "Sending..." } else { "📧 Send Test Email" }}
                    </button>
                    <button
                        on:click=move |_| flush_action.dispatch(())
                        disabled=move || flush_action.pending().get()
                        style="padding: 8px 16px; background: #059669; color: white; border: none; border-radius: 6px; cursor: pointer; font-size: 14px;"
                    >
                        {move || if flush_action.pending().get() { "Flushing..." } else { "🔄 Flush Pending Errors" }}
                    </button>
                    // Show result messages
                    {move || send_test_action.value().get().map(|msg| view! {
                        <span style="font-size: 14px;">{msg}</span>
                    })}
                    {move || flush_action.value().get().map(|msg| view! {
                        <span style="font-size: 14px;">{msg}</span>
                    })}
                </div>
            </div>
            
            // Existing DataTable
            <DataTableV3 />
        </div>
    }
    .into_view()
}

