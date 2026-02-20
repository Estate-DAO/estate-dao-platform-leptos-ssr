use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_extra::extract::cookie::SignedCookieJar;
use estate_fe::api::auth::types::OidcUser;
use estate_fe::api::consts::EnvVarConfig;
use estate_fe::ssr_booking::email_handler::EmailClient;
use estate_fe::utils::uuidv7;
use estate_fe::view_state_layer::AppState;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{error, info, instrument, warn};

use super::parse_json_request;

const SESSION_COOKIE: &str = "session";
const SUPPORT_RECIPIENTS: &[&str] = &["support@nofeebooking.com"];
const SUPPORT_CC_RECIPIENTS: &[&str] = &[
    "support@estatedao.org",
    "ayushi@estatedao.org",
    "prakash@estatedao.org",
];
const PROD_SUPPORT_CC: &str = "utkarsh@gobazzinga.io";
const MAX_SUBJECT_LEN: usize = 120;
const MAX_QUERY_LEN: usize = 500;
const BOOKING_FALLBACK_IMAGE: &str = "https://nofeebooking.com/img/home.png";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SupportProvider {
    #[serde(rename = "LiteAPI")]
    LiteApi,
}

impl SupportProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            SupportProvider::LiteApi => "LiteAPI",
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SupportBookingContext {
    pub booking_id: Option<String>,
    pub hotel_name: Option<String>,
    pub hotel_location: Option<String>,
    pub hotel_code: Option<String>,
    pub hotel_image_url: Option<String>,
    pub check_in_date: Option<String>,
    pub check_out_date: Option<String>,
    pub adults: Option<u32>,
    pub rooms: Option<u32>,
    pub total_amount: Option<f64>,
    pub currency: Option<String>,
    pub provider: Option<SupportProvider>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SupportRequest {
    pub subject: String,
    pub query: String,
    pub booking_context: Option<SupportBookingContext>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SupportResponse {
    pub success: bool,
    pub message: String,
    pub ticket_id: Option<String>,
}

fn get_current_user(jar: &SignedCookieJar) -> Option<OidcUser> {
    jar.get(SESSION_COOKIE)
        .and_then(|cookie| serde_json::from_str(cookie.value()).ok())
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn is_production_env() -> bool {
    cfg!(feature = "prod-consts")
}

fn is_web_url(value: &str) -> bool {
    value.starts_with("http://") || value.starts_with("https://")
}

fn has_booking_context(ctx: &SupportBookingContext) -> bool {
    ctx.booking_id
        .as_ref()
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false)
        || ctx
            .hotel_name
            .as_ref()
            .map(|v| !v.trim().is_empty())
            .unwrap_or(false)
        || ctx
            .hotel_location
            .as_ref()
            .map(|v| !v.trim().is_empty())
            .unwrap_or(false)
        || ctx
            .hotel_code
            .as_ref()
            .map(|v| !v.trim().is_empty())
            .unwrap_or(false)
        || ctx
            .hotel_image_url
            .as_ref()
            .map(|v| !v.trim().is_empty())
            .unwrap_or(false)
        || ctx
            .check_in_date
            .as_ref()
            .map(|v| !v.trim().is_empty())
            .unwrap_or(false)
        || ctx
            .check_out_date
            .as_ref()
            .map(|v| !v.trim().is_empty())
            .unwrap_or(false)
        || ctx.adults.is_some()
        || ctx.rooms.is_some()
        || ctx.total_amount.is_some()
        || ctx
            .currency
            .as_ref()
            .map(|v| !v.trim().is_empty())
            .unwrap_or(false)
        || ctx.provider.is_some()
}

fn booking_amount_text(ctx: &SupportBookingContext) -> Option<String> {
    ctx.total_amount.map(|amount| {
        let currency = ctx.currency.clone().unwrap_or_default().to_uppercase();
        if currency.trim().is_empty() {
            format!("{:.2}", amount)
        } else {
            format!("{currency} {amount:.2}")
        }
    })
}

fn booking_context_text(ctx: &SupportBookingContext) -> Option<String> {
    if !has_booking_context(ctx) {
        return None;
    }

    let mut lines = Vec::new();
    if let Some(value) = ctx.booking_id.as_ref().filter(|v| !v.trim().is_empty()) {
        lines.push(format!("Booking ID: {value}"));
    }
    if let Some(value) = ctx.hotel_name.as_ref().filter(|v| !v.trim().is_empty()) {
        lines.push(format!("Hotel: {value}"));
    }
    if let Some(value) = ctx.hotel_location.as_ref().filter(|v| !v.trim().is_empty()) {
        lines.push(format!("Location: {value}"));
    }
    if let (Some(check_in), Some(check_out)) = (
        ctx.check_in_date.as_ref().filter(|v| !v.trim().is_empty()),
        ctx.check_out_date.as_ref().filter(|v| !v.trim().is_empty()),
    ) {
        lines.push(format!("Stay: {check_in} to {check_out}"));
    } else if let Some(check_in) = ctx.check_in_date.as_ref().filter(|v| !v.trim().is_empty()) {
        lines.push(format!("Check-in: {check_in}"));
    } else if let Some(check_out) = ctx.check_out_date.as_ref().filter(|v| !v.trim().is_empty()) {
        lines.push(format!("Check-out: {check_out}"));
    }
    if let Some(adults) = ctx.adults {
        lines.push(format!("Adults: {adults}"));
    }
    if let Some(rooms) = ctx.rooms {
        lines.push(format!("Rooms: {rooms}"));
    }
    if let Some(amount_text) = booking_amount_text(ctx) {
        lines.push(format!("Amount Paid: {amount_text}"));
    }
    if let Some(value) = ctx.hotel_code.as_ref().filter(|v| !v.trim().is_empty()) {
        lines.push(format!("Hotel Code: {value}"));
    }
    if let Some(provider) = ctx.provider.as_ref() {
        lines.push(format!("Provider: {}", provider.as_str()));
    }

    Some(format!("Booking Details:\n{}\n", lines.join("\n")))
}

fn booking_context_html(ctx: &SupportBookingContext) -> Option<String> {
    if !has_booking_context(ctx) {
        return None;
    }

    let hotel_name = ctx
        .hotel_name
        .as_ref()
        .filter(|v| !v.trim().is_empty())
        .cloned()
        .unwrap_or_else(|| "Booking Details".to_string());
    let hotel_location = ctx
        .hotel_location
        .as_ref()
        .filter(|v| !v.trim().is_empty())
        .cloned()
        .unwrap_or_default();
    let hero_image = ctx
        .hotel_image_url
        .as_ref()
        .filter(|v| !v.trim().is_empty())
        .cloned()
        .filter(|v| is_web_url(v))
        .unwrap_or_else(|| BOOKING_FALLBACK_IMAGE.to_string());
    let hotel_details_url = ctx
        .hotel_code
        .as_ref()
        .filter(|v| !v.trim().is_empty())
        .map(|code| format!("https://nofeebooking.com/hotel-details?hotelCode={code}"));

    let mut lines = Vec::new();
    if let Some(value) = ctx.booking_id.as_ref().filter(|v| !v.trim().is_empty()) {
        lines.push(format!(
            "<div><strong>Booking ID:</strong> {}</div>",
            html_escape(value)
        ));
    }
    if let (Some(check_in), Some(check_out)) = (
        ctx.check_in_date.as_ref().filter(|v| !v.trim().is_empty()),
        ctx.check_out_date.as_ref().filter(|v| !v.trim().is_empty()),
    ) {
        lines.push(format!(
            "<div><strong>Stay:</strong> {} to {}</div>",
            html_escape(check_in),
            html_escape(check_out)
        ));
    } else if let Some(check_in) = ctx.check_in_date.as_ref().filter(|v| !v.trim().is_empty()) {
        lines.push(format!(
            "<div><strong>Check-in:</strong> {}</div>",
            html_escape(check_in)
        ));
    } else if let Some(check_out) = ctx.check_out_date.as_ref().filter(|v| !v.trim().is_empty()) {
        lines.push(format!(
            "<div><strong>Check-out:</strong> {}</div>",
            html_escape(check_out)
        ));
    }
    if let Some(adults) = ctx.adults {
        lines.push(format!("<div><strong>Adults:</strong> {}</div>", adults));
    }
    if let Some(rooms) = ctx.rooms {
        lines.push(format!("<div><strong>Rooms:</strong> {}</div>", rooms));
    }
    if let Some(amount_text) = booking_amount_text(ctx) {
        lines.push(format!(
            "<div><strong>Amount Paid:</strong> {}</div>",
            html_escape(&amount_text)
        ));
    }
    if let Some(value) = ctx.hotel_code.as_ref().filter(|v| !v.trim().is_empty()) {
        lines.push(format!(
            "<div><strong>Hotel Code:</strong> {}</div>",
            html_escape(value)
        ));
    }
    if let Some(provider) = ctx.provider.as_ref() {
        lines.push(format!(
            "<div><strong>Provider:</strong> {}</div>",
            html_escape(provider.as_str())
        ));
    }

    let hero_image_tag = if let Some(url) = &hotel_details_url {
        format!(
            "<a href=\"{url}\" style=\"display:block;text-decoration:none;\">\
<img src=\"{hero_image}\" alt=\"{hotel_name}\" style=\"width:100%;max-height:220px;object-fit:cover;border-radius:12px;border:1px solid #e5e7eb;\" />\
</a>"
        )
    } else {
        format!(
            "<img src=\"{hero_image}\" alt=\"{hotel_name}\" style=\"width:100%;max-height:220px;object-fit:cover;border-radius:12px;border:1px solid #e5e7eb;\" />"
        )
    };

    Some(format!(
        r#"<table role="presentation" width="100%" cellspacing="0" cellpadding="0" style="margin:16px 0 0;">
  <tr>
    <td style="padding:14px;background:#f9fafb;border:1px solid #e5e7eb;border-radius:12px;">
      <div style="font-size:12px;letter-spacing:0.08em;color:#2563eb;font-weight:700;text-transform:uppercase;margin-bottom:10px;">Booking Preview</div>
      {hero_image_tag}
      <div style="margin-top:12px;">
        <div style="font-size:16px;font-weight:600;color:#111827;margin-bottom:4px;">{hotel_name}</div>
        <div style="font-size:13px;color:#6b7280;margin-bottom:10px;">{hotel_location}</div>
        <div style="font-size:14px;color:#111827;line-height:1.6;">
          {details_html}
        </div>
      </div>
    </td>
  </tr>
</table>"#,
        hero_image_tag = hero_image_tag,
        hotel_name = html_escape(&hotel_name),
        hotel_location = html_escape(&hotel_location),
        details_html = lines.join(""),
    ))
}

#[axum::debug_handler]
#[instrument(skip(_state, jar, body))]
pub async fn support_request_api_server_fn_route(
    State(_state): State<AppState>,
    jar: SignedCookieJar,
    body: String,
) -> Result<Response, Response> {
    info!("Starting support request API (payload_len={})", body.len());

    let request: SupportRequest = parse_json_request(&body)?;
    let subject = request.subject.trim();
    let query = request.query.trim();
    let booking_context = request.booking_context.clone();

    if subject.is_empty() {
        let error_response = json!({ "error": "Subject is required." });
        return Ok((StatusCode::BAD_REQUEST, error_response.to_string()).into_response());
    }

    if query.is_empty() {
        let error_response = json!({ "error": "Query is required." });
        return Ok((StatusCode::BAD_REQUEST, error_response.to_string()).into_response());
    }

    if subject.chars().count() > MAX_SUBJECT_LEN {
        let error_response = json!({ "error": "Subject is too long." });
        return Ok((StatusCode::BAD_REQUEST, error_response.to_string()).into_response());
    }

    if query.chars().count() > MAX_QUERY_LEN {
        let error_response = json!({ "error": "Query is too long." });
        return Ok((StatusCode::BAD_REQUEST, error_response.to_string()).into_response());
    }

    let user = match get_current_user(&jar) {
        Some(user) => user,
        None => {
            let error_response = json!({ "error": "Please sign in to contact support." });
            return Ok((StatusCode::UNAUTHORIZED, error_response.to_string()).into_response());
        }
    };

    let user_email = match user.email.clone() {
        Some(email) if !email.trim().is_empty() => email,
        _ => {
            let error_response = json!({ "error": "Signed-in user email is missing." });
            return Ok((StatusCode::UNAUTHORIZED, error_response.to_string()).into_response());
        }
    };

    let user_name = user.name.clone().unwrap_or_else(|| user_email.clone());
    let ticket_id = uuidv7::create();
    let submitted_at = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");

    let support_subject = format!("[Support] {} ({})", subject, ticket_id);
    let booking_text = booking_context
        .as_ref()
        .and_then(booking_context_text)
        .unwrap_or_default();
    let support_text_body = format!(
        "New support request received.\n\n\
Ticket ID: {ticket_id}\n\
Submitted: {submitted_at}\n\
From: {user_name} <{user_email}>\n\
Subject: {subject}\n\n\
Message:\n{query}\n\n\
{booking_text}"
    );

    let support_html_booking = booking_context
        .as_ref()
        .and_then(booking_context_html)
        .unwrap_or_default();
    let support_html_body = format!(
        r#"<!doctype html>
<html>
  <body style="margin:0;padding:0;background:#f5f7fb;font-family:'Segoe UI',Arial,sans-serif;color:#1f2937;">
    <table role="presentation" width="100%" cellspacing="0" cellpadding="0" style="background:#f5f7fb;padding:24px 0;">
      <tr>
        <td align="center">
          <table role="presentation" width="640" cellspacing="0" cellpadding="0" style="background:#ffffff;border-radius:14px;border:1px solid #e5e7eb;box-shadow:0 10px 30px rgba(31,41,55,0.08);padding:32px;">
            <tr>
              <td style="text-align:left;">
                <div style="font-size:12px;letter-spacing:0.08em;color:#2563eb;font-weight:700;text-transform:uppercase;margin-bottom:10px;">Support Request</div>
                <h1 style="margin:0 0 10px;font-size:22px;color:#111827;">New support request received</h1>
                <p style="margin:0 0 18px;font-size:14px;line-height:1.6;color:#4b5563;">
                  Ticket ID <strong>{ticket_id}</strong> · Submitted {submitted_at}
                </p>
                <table role="presentation" width="100%" cellspacing="0" cellpadding="0" style="margin-bottom:16px;">
                  <tr>
                    <td style="padding:12px;background:#f9fafb;border:1px solid #e5e7eb;border-radius:10px;">
                      <div style="font-size:13px;color:#6b7280;margin-bottom:6px;">From</div>
                      <div style="font-size:15px;font-weight:600;color:#111827;">{user_name}</div>
                      <div style="font-size:13px;color:#6b7280;">{user_email}</div>
                    </td>
                  </tr>
                </table>
                <div style="margin-bottom:12px;">
                  <div style="font-size:13px;color:#6b7280;margin-bottom:6px;">Subject</div>
                  <div style="font-size:16px;font-weight:600;color:#111827;">{subject}</div>
                </div>
                <div style="margin:14px 0 4px;font-size:13px;color:#6b7280;">Message</div>
                <div style="background:#f3f4f6;border-radius:12px;padding:14px;font-size:14px;line-height:1.6;color:#111827;white-space:pre-wrap;">{query}</div>
                {support_html_booking}
              </td>
            </tr>
          </table>
        </td>
      </tr>
    </table>
  </body>
</html>"#,
        ticket_id = html_escape(&ticket_id),
        submitted_at = html_escape(&submitted_at.to_string()),
        user_name = html_escape(&user_name),
        user_email = html_escape(&user_email),
        subject = html_escape(subject),
        query = html_escape(query),
        support_html_booking = support_html_booking,
    );

    let confirmation_subject = format!("Support request received - Ticket {}", ticket_id);
    let confirmation_text_body = format!(
        "Hi {user_name},\n\n\
Thanks for contacting Nofeebooking Support. We've received your request and will get back to you shortly.\n\n\
Ticket ID: {ticket_id}\n\
Subject: {subject}\n\n\
Your message:\n{query}\n\n\
{booking_text}If you need to add more information, just reply to this email.\n\n\
Team Nofeebooking"
    );
    let confirmation_html_booking = booking_context
        .as_ref()
        .and_then(booking_context_html)
        .unwrap_or_default();
    let confirmation_html_body = format!(
        r#"<!doctype html>
<html>
  <body style="margin:0;padding:0;background:#f5f7fb;font-family:'Segoe UI',Arial,sans-serif;color:#1f2937;">
    <table role="presentation" width="100%" cellspacing="0" cellpadding="0" style="background:#f5f7fb;padding:24px 0;">
      <tr>
        <td align="center">
          <table role="presentation" width="640" cellspacing="0" cellpadding="0" style="background:#ffffff;border-radius:14px;border:1px solid #e5e7eb;box-shadow:0 10px 30px rgba(31,41,55,0.08);padding:32px;">
            <tr>
              <td style="text-align:left;">
                <div style="font-size:12px;letter-spacing:0.08em;color:#2563eb;font-weight:700;text-transform:uppercase;margin-bottom:10px;">Support Request</div>
                <h1 style="margin:0 0 10px;font-size:22px;color:#111827;">We received your request</h1>
                <p style="margin:0 0 18px;font-size:14px;line-height:1.6;color:#4b5563;">
                  Ticket ID <strong>{ticket_id}</strong>. Our team will get back to you shortly.
                </p>
                <div style="margin-bottom:12px;">
                  <div style="font-size:13px;color:#6b7280;margin-bottom:6px;">Subject</div>
                  <div style="font-size:16px;font-weight:600;color:#111827;">{subject}</div>
                </div>
                <div style="margin:14px 0 4px;font-size:13px;color:#6b7280;">Your message</div>
                <div style="background:#f3f4f6;border-radius:12px;padding:14px;font-size:14px;line-height:1.6;color:#111827;white-space:pre-wrap;">{query}</div>
                {confirmation_html_booking}
                <div style="margin-top:18px;">
                  <a href="https://nofeebooking.com/account?page=bookings" style="background:#2563eb;color:#ffffff;text-decoration:none;padding:12px 18px;border-radius:10px;font-weight:600;font-size:14px;display:inline-block;">View Bookings</a>
                </div>
                <p style="margin:16px 0 0;font-size:14px;line-height:1.6;color:#4b5563;">
                  If you need to add more information, just reply to this email.
                </p>
                <p style="margin:12px 0 0;font-size:14px;color:#4b5563;">
                  Team Nofeebooking
                </p>
              </td>
            </tr>
          </table>
        </td>
      </tr>
    </table>
  </body>
</html>"#,
        ticket_id = html_escape(&ticket_id),
        subject = html_escape(subject),
        query = html_escape(query),
        confirmation_html_booking = confirmation_html_booking,
    );

    let config = EnvVarConfig::try_from_env();
    let email_client = EmailClient::new(config.email_client_config);

    let mut cc_list: Vec<&str> = SUPPORT_CC_RECIPIENTS.to_vec();
    if is_production_env() {
        cc_list.push(PROD_SUPPORT_CC);
    }
    let support_cc = if cc_list.is_empty() {
        None
    } else {
        Some(cc_list.join(", "))
    };

    for recipient in SUPPORT_RECIPIENTS {
        if let Err(e) = email_client
            .send_multipart_email(
                recipient,
                support_cc.as_deref(),
                &support_subject,
                &support_text_body,
                &support_html_body,
                Some(&user_email),
            )
            .await
        {
            error!("Failed to send support email to {}: {}", recipient, e);
            let error_response =
                json!({ "error": "Failed to send support request. Please try again." });
            return Ok((
                StatusCode::INTERNAL_SERVER_ERROR,
                error_response.to_string(),
            )
                .into_response());
        }
    }

    let mut message = format!(
        "Your support request has been sent. Ticket ID: {}.",
        ticket_id
    );
    if let Err(e) = email_client
        .send_multipart_email(
            &user_email,
            None,
            &confirmation_subject,
            &confirmation_text_body,
            &confirmation_html_body,
            None,
        )
        .await
    {
        warn!(
            "Failed to send support confirmation to {}: {}",
            user_email, e
        );
        message.push_str(
            " We could not send a confirmation email, but our team has received your request.",
        );
    }

    let response = SupportResponse {
        success: true,
        message,
        ticket_id: Some(ticket_id),
    };

    Ok((StatusCode::OK, serde_json::to_string(&response).unwrap()).into_response())
}
