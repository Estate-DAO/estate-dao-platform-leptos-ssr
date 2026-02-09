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
const MAX_SUBJECT_LEN: usize = 120;
const MAX_QUERY_LEN: usize = 500;

#[derive(Serialize, Deserialize, Debug)]
pub struct SupportRequest {
    pub subject: String,
    pub query: String,
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
    let support_body = format!(
        "New support request received.\n\n\
Ticket ID: {ticket_id}\n\
Submitted: {submitted_at}\n\
From: {user_name} <{user_email}>\n\
Subject: {subject}\n\n\
Query:\n{query}\n"
    );

    let confirmation_subject = format!("We received your support request ({})", ticket_id);
    let confirmation_body = format!(
        "Hi {user_name},\n\n\
Thanks for contacting Nofeebooking Support. We've received your request and will get back to you shortly.\n\n\
Ticket ID: {ticket_id}\n\
Subject: {subject}\n\n\
Your message:\n{query}\n\n\
If you need to add more information, just reply to this email.\n\n\
Team Nofeebooking"
    );

    let config = EnvVarConfig::try_from_env();
    let email_client = EmailClient::new(config.email_client_config);

    for recipient in SUPPORT_RECIPIENTS {
        if let Err(e) = email_client
            .send_plain_text_email(
                recipient,
                &support_subject,
                &support_body,
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
        .send_plain_text_email(&user_email, &confirmation_subject, &confirmation_body, None)
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
