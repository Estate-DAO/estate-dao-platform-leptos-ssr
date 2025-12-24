//! Error Alert Service
//!
//! Batches critical errors over 5-minute windows and sends consolidated
//! email alerts to the ops team.

use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, warn};

use crate::api::consts::APP_URL;
use crate::ssr_booking::email_handler::EmailClient;

/// Alert recipients
const ALERT_RECIPIENTS: &[&str] = &["ayushi@estatedao.org", "prakash@estatedao.org"];

/// Flush interval in seconds
const FLUSH_INTERVAL_SECS: u64 = 300; // 5 minutes

// ============================================================================
// Error Types
// ============================================================================

/// Data-carrying error type with variant-specific details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorType {
    JsonParseFailed {
        json_path: Option<String>,
        expected_type: Option<String>,
        actual_type: Option<String>,
    },
    Http500 {
        status_code: u16,
        response_body: Option<String>,
    },
    PaymentFailure {
        payment_id: Option<String>,
        provider: String,
        failure_reason: Option<String>,
    },
    BookingProviderFailure {
        provider: String,
        hotel_id: Option<String>,
        operation: String,
    },
}

impl ErrorType {
    pub fn type_name(&self) -> &'static str {
        match self {
            ErrorType::JsonParseFailed { .. } => "JSON Parse Error",
            ErrorType::Http500 { .. } => "HTTP 500 Error",
            ErrorType::PaymentFailure { .. } => "Payment Failure",
            ErrorType::BookingProviderFailure { .. } => "Booking Provider Failure",
        }
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            ErrorType::JsonParseFailed { .. } => "🔴",
            ErrorType::Http500 { .. } => "🟠",
            ErrorType::PaymentFailure { .. } => "💳",
            ErrorType::BookingProviderFailure { .. } => "🏨",
        }
    }

    /// Render variant-specific details as HTML
    pub fn details_html(&self) -> String {
        match self {
            ErrorType::JsonParseFailed {
                json_path,
                expected_type,
                actual_type,
            } => {
                let mut parts = vec![];
                if let Some(path) = json_path {
                    parts.push(format!(
                        "<strong>Path:</strong> <code>{}</code>",
                        html_escape(path)
                    ));
                }
                if let Some(expected) = expected_type {
                    parts.push(format!(
                        "<strong>Expected:</strong> {}",
                        html_escape(expected)
                    ));
                }
                if let Some(actual) = actual_type {
                    parts.push(format!("<strong>Got:</strong> {}", html_escape(actual)));
                }
                parts.join(" | ")
            }
            ErrorType::Http500 {
                status_code,
                response_body,
            } => {
                let mut html = format!("<strong>Status:</strong> {}", status_code);
                if let Some(body) = response_body {
                    let truncated = if body.len() > 200 {
                        format!("{}...", &body[..200])
                    } else {
                        body.clone()
                    };
                    html.push_str(&format!(
                        "<br><strong>Response:</strong> <code>{}</code>",
                        html_escape(&truncated)
                    ));
                }
                html
            }
            ErrorType::PaymentFailure {
                payment_id,
                provider,
                failure_reason,
            } => {
                let mut parts = vec![format!(
                    "<strong>Provider:</strong> {}",
                    html_escape(provider)
                )];
                if let Some(pid) = payment_id {
                    parts.push(format!("<strong>Payment ID:</strong> {}", html_escape(pid)));
                }
                if let Some(reason) = failure_reason {
                    parts.push(format!("<strong>Reason:</strong> {}", html_escape(reason)));
                }
                parts.join(" | ")
            }
            ErrorType::BookingProviderFailure {
                provider,
                hotel_id,
                operation,
            } => {
                let mut parts = vec![
                    format!("<strong>Provider:</strong> {}", html_escape(provider)),
                    format!("<strong>Operation:</strong> {}", html_escape(operation)),
                ];
                if let Some(hid) = hotel_id {
                    parts.push(format!("<strong>Hotel:</strong> {}", html_escape(hid)));
                }
                parts.join(" | ")
            }
        }
    }
}

// ============================================================================
// Critical Error
// ============================================================================

/// Rich error context for alerting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriticalError {
    pub error_type: ErrorType,
    pub timestamp: DateTime<Utc>,

    // Deployment context
    pub app_url: String,

    // Request context
    pub request_url: Option<String>,
    pub request_method: Option<String>,
    pub user_context: Option<String>,

    // Source location
    pub file_path: Option<String>,
    pub line_number: Option<u32>,
    pub function_name: Option<String>,

    // Error details
    pub message: String,
    pub stack_trace: Option<String>,
}

impl CriticalError {
    /// Create a new critical error with current app URL
    pub fn new(error_type: ErrorType, message: impl Into<String>) -> Self {
        Self {
            error_type,
            timestamp: Utc::now(),
            app_url: APP_URL.clone(),
            request_url: None,
            request_method: None,
            user_context: None,
            file_path: None,
            line_number: None,
            function_name: None,
            message: message.into(),
            stack_trace: None,
        }
    }

    pub fn with_request(mut self, method: impl Into<String>, url: impl Into<String>) -> Self {
        self.request_method = Some(method.into());
        self.request_url = Some(url.into());
        self
    }

    pub fn with_user(mut self, user: impl Into<String>) -> Self {
        self.user_context = Some(user.into());
        self
    }

    pub fn with_source(
        mut self,
        file: impl Into<String>,
        line: u32,
        function: impl Into<String>,
    ) -> Self {
        self.file_path = Some(file.into());
        self.line_number = Some(line);
        self.function_name = Some(function.into());
        self
    }

    pub fn with_stack_trace(mut self, trace: impl Into<String>) -> Self {
        self.stack_trace = Some(trace.into());
        self
    }
}

// ============================================================================
// Error Alert Service
// ============================================================================

/// Service that batches errors and sends email alerts
#[derive(Clone, Debug)]
pub struct ErrorAlertService {
    errors: Arc<Mutex<Vec<CriticalError>>>,
    email_client: Arc<EmailClient>,
}

impl ErrorAlertService {
    pub fn new(email_client: EmailClient) -> Self {
        Self {
            errors: Arc::new(Mutex::new(Vec::new())),
            email_client: Arc::new(email_client),
        }
    }

    /// Report an error to be batched and sent
    pub async fn report(&self, error: CriticalError) {
        let mut errors = self.errors.lock().await;
        info!(
            error_type = error.error_type.type_name(),
            message = %error.message,
            "Critical error reported"
        );
        errors.push(error);
    }

    /// Start the background flush task
    pub fn start_background_flush(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(FLUSH_INTERVAL_SECS));
            loop {
                ticker.tick().await;
                if let Err(e) = self.flush().await {
                    error!(error = %e, "Failed to flush error alerts");
                }
            }
        });
    }

    /// Flush all batched errors and send email
    pub async fn flush(&self) -> Result<(), String> {
        let errors = {
            let mut lock = self.errors.lock().await;
            if lock.is_empty() {
                return Ok(());
            }
            std::mem::take(&mut *lock)
        };

        let count = errors.len();
        info!(count, "Flushing error alerts");

        let html = build_alert_email_html(&errors);
        let subject = format!(
            "🚨 {} Critical Error{} - Nofeebooking",
            count,
            if count == 1 { "" } else { "s" }
        );

        self.send_alert_email(&subject, &html).await
    }

    async fn send_alert_email(&self, subject: &str, html_body: &str) -> Result<(), String> {
        // Build plain text version
        let text_body = format!(
            "Critical errors detected. Please view the HTML version of this email for details.\n\n\
             Subject: {}\n\
             Time: {}",
            subject,
            Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );

        // Use Gmail API via the existing email client infrastructure
        // We'll need to send to multiple recipients
        for recipient in ALERT_RECIPIENTS {
            if let Err(e) = self
                .send_gmail_alert(recipient, subject, &text_body, html_body)
                .await
            {
                error!(recipient, error = %e, "Failed to send alert email");
            } else {
                info!(recipient, "Alert email sent");
            }
        }

        Ok(())
    }

    async fn send_gmail_alert(
        &self,
        to: &str,
        subject: &str,
        text_body: &str,
        html_body: &str,
    ) -> Result<(), String> {
        use base64::engine::general_purpose;
        use base64::Engine;

        // Get access token from email client (with token refresh)
        let access_token = self
            .email_client
            .get_valid_access_token()
            .await
            .map_err(|e| format!("Failed to get access token: {}", e))?;

        let boundary = "nofeebooking-alert-boundary";
        let email_raw = format!(
            "To: {to}\r\n\
             Subject: {subject}\r\n\
             MIME-Version: 1.0\r\n\
             Content-Type: multipart/alternative; boundary=\"{boundary}\"\r\n\
             \r\n\
             --{boundary}\r\n\
             Content-Type: text/plain; charset=\"UTF-8\"\r\n\r\n\
             {text_body}\r\n\
             \r\n\
             --{boundary}\r\n\
             Content-Type: text/html; charset=\"UTF-8\"\r\n\r\n\
             {html_body}\r\n\
             \r\n\
             --{boundary}--",
        );

        let encoded_message = general_purpose::STANDARD.encode(&email_raw);
        let payload = serde_json::json!({ "raw": encoded_message });

        let client = Client::new();
        let response = client
            .post("https://www.googleapis.com/gmail/v1/users/me/messages/send")
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if response.status().is_success() {
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(format!("Gmail API error: {}", error_text))
        }
    }
}

// ============================================================================
// Email HTML Builder
// ============================================================================

fn build_alert_email_html(errors: &[CriticalError]) -> String {
    use std::collections::HashMap;

    // Group errors by type
    let mut grouped: HashMap<&'static str, Vec<&CriticalError>> = HashMap::new();
    for error in errors {
        grouped
            .entry(error.error_type.type_name())
            .or_default()
            .push(error);
    }

    // Sort groups by count (most errors first)
    let mut sorted_groups: Vec<_> = grouped.into_iter().collect();
    sorted_groups.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

    let total_count = errors.len();
    let timestamp = Utc::now().format("%b %d, %Y at %I:%M %p UTC").to_string();

    // Get app URL from first error (they should all be the same)
    let app_url = errors
        .first()
        .map(|e| e.app_url.as_str())
        .unwrap_or("unknown");

    let mut sections_html = String::new();
    for (type_name, type_errors) in sorted_groups {
        let emoji = type_errors
            .first()
            .map(|e| e.error_type.emoji())
            .unwrap_or("❌");
        let count = type_errors.len();

        let mut errors_html = String::new();
        for (i, error) in type_errors.iter().enumerate() {
            let time = error.timestamp.format("%I:%M:%S %p").to_string();
            let url = error.request_url.as_deref().unwrap_or("-");
            let method = error.request_method.as_deref().unwrap_or("");
            let user = error.user_context.as_deref().unwrap_or("anonymous");

            let source_info = match (&error.file_path, error.line_number, &error.function_name) {
                (Some(file), Some(line), Some(func)) => {
                    format!(
                        "<div style=\"font-size:11px;color:#6b7280;margin-top:4px;\">\
                         📍 <code>{}:{}</code> in <code>{}</code></div>",
                        html_escape(file),
                        line,
                        html_escape(func)
                    )
                }
                (Some(file), Some(line), None) => {
                    format!(
                        "<div style=\"font-size:11px;color:#6b7280;margin-top:4px;\">\
                         📍 <code>{}:{}</code></div>",
                        html_escape(file),
                        line
                    )
                }
                _ => String::new(),
            };

            let stack_trace_html = if let Some(trace) = &error.stack_trace {
                format!(
                    "<details style=\"margin-top:8px;\">\
                     <summary style=\"cursor:pointer;color:#6366f1;font-size:12px;\">Stack Trace</summary>\
                     <pre style=\"font-size:11px;background:#f3f4f6;padding:8px;border-radius:4px;overflow-x:auto;margin-top:4px;\">{}</pre>\
                     </details>",
                    html_escape(trace)
                )
            } else {
                String::new()
            };

            let type_details = error.error_type.details_html();

            errors_html.push_str(&format!(
                r#"<div style="padding:12px;background:#f9fafb;border-radius:8px;margin-bottom:8px;border-left:3px solid #6366f1;">
                    <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:6px;">
                        <span style="font-weight:600;color:#111827;">{method} {url}</span>
                        <span style="font-size:12px;color:#6b7280;">{time}</span>
                    </div>
                    <div style="font-size:13px;color:#374151;margin-bottom:4px;">
                        <strong>User:</strong> {user}
                    </div>
                    <div style="font-size:13px;color:#374151;margin-bottom:4px;">
                        {type_details}
                    </div>
                    <div style="font-size:13px;color:#dc2626;margin-top:6px;">
                        {message}
                    </div>
                    {source_info}
                    {stack_trace_html}
                </div>"#,
                method = html_escape(method),
                url = html_escape(url),
                time = time,
                user = html_escape(user),
                type_details = type_details,
                message = html_escape(&error.message),
                source_info = source_info,
                stack_trace_html = stack_trace_html,
            ));
        }

        sections_html.push_str(&format!(
            r#"<details open style="margin-bottom:16px;">
                <summary style="cursor:pointer;padding:12px;background:#e0e7ff;border-radius:8px;font-weight:600;color:#3730a3;">
                    {emoji} {type_name} ({count})
                </summary>
                <div style="padding:12px 0;">
                    {errors_html}
                </div>
            </details>"#,
            emoji = emoji,
            type_name = type_name,
            count = count,
            errors_html = errors_html,
        ));
    }

    format!(
        r#"<!doctype html>
<html>
<body style="margin:0;padding:0;background:#f5f7fb;font-family:'Segoe UI',Arial,sans-serif;color:#1f2937;">
<table role="presentation" width="100%" cellspacing="0" cellpadding="0" style="background:#f5f7fb;padding:24px 0;">
<tr><td align="center">
<table role="presentation" width="680" cellspacing="0" cellpadding="0" style="background:#ffffff;border-radius:14px;border:1px solid #e5e7eb;box-shadow:0 10px 30px rgba(31,41,55,0.08);padding:32px;">
<tr><td>
    <div style="text-align:center;margin-bottom:24px;">
        <div style="font-size:32px;margin-bottom:8px;">🚨</div>
        <h1 style="margin:0;font-size:22px;color:#111827;">Critical Error Alert</h1>
        <p style="margin:8px 0 0;font-size:14px;color:#6b7280;">
            {total_count} error{plural} collected • {timestamp}
        </p>
        <p style="margin:4px 0 0;font-size:12px;color:#9ca3af;">
            Environment: <code style="background:#f3f4f6;padding:2px 6px;border-radius:4px;">{app_url}</code>
        </p>
    </div>
    <div>
        {sections_html}
    </div>
    <div style="text-align:center;margin-top:24px;padding-top:16px;border-top:1px solid #e5e7eb;">
        <p style="margin:0;font-size:12px;color:#9ca3af;">
            This is an automated alert from Nofeebooking Error Monitoring
        </p>
    </div>
</td></tr>
</table>
</td></tr>
</table>
</body>
</html>"#,
        total_count = total_count,
        plural = if total_count == 1 { "" } else { "s" },
        timestamp = timestamp,
        app_url = html_escape(app_url),
        sections_html = sections_html,
    )
}

/// Simple HTML escaping
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

// ============================================================================
// Convenience macros
// ============================================================================

/// Report a JSON parse error
#[macro_export]
macro_rules! report_json_error {
    ($service:expr, $message:expr, $json_path:expr) => {
        $service
            .report($crate::utils::error_alerts::CriticalError::new(
                $crate::utils::error_alerts::ErrorType::JsonParseFailed {
                    json_path: Some($json_path.to_string()),
                    expected_type: None,
                    actual_type: None,
                },
                $message,
            ))
            .await
    };
}

/// Report an HTTP 500 error
#[macro_export]
macro_rules! report_http500 {
    ($service:expr, $message:expr, $status:expr) => {
        $service
            .report($crate::utils::error_alerts::CriticalError::new(
                $crate::utils::error_alerts::ErrorType::Http500 {
                    status_code: $status,
                    response_body: None,
                },
                $message,
            ))
            .await
    };
}
