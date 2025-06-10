//! Email handler for post-booking confirmation
//! Implements sending confirmation emails after successful booking and updating backend email_sent status.

use anyhow::anyhow;
use async_trait::async_trait;
use axum::http::{HeaderMap, HeaderValue};
use base64::engine::general_purpose;
use base64::Engine;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time;
use tracing::{debug, error, info, instrument, warn};

use crate::api::canister::book_room_details::call_update_book_room_details_backend;
use crate::api::canister::get_user_booking::get_user_booking_backend;
use crate::api::consts::{ConfigLoader, EmailConfig, EnvVarConfig};
use crate::api::payments::ports::{GetPaymentStatusRequest, GetPaymentStatusResponse};
use crate::api::payments::NowPayments;
// use crate::api::{
//     book_room as book_room_api, create_backend_book_room_response,
//     get_hotel_booking_detail_from_travel_provider_v2, user_details_to_passenger_details,
//     BookRoomRequest, BookRoomResponse, BookingDetails, BookingStatus, HotelBookingDetailRequest,
//     RoomDetail,
// };
use crate::canister::backend::{self, BeBookRoomResponse, Booking, Result1, Result2};
use crate::ssr_booking::pipeline::{PipelineExecutor, PipelineValidator};
use crate::ssr_booking::{PipelineDecision, ServerSideBookingEvent};
use crate::utils::admin::AdminCanisters;
use crate::utils::app_reference::BookingId;
use crate::utils::booking_id::PaymentIdentifiers;

// --- Email Client Implementation ---
#[derive(Debug, Clone)]
pub struct EmailClient(Arc<Mutex<EmailConfig>>);

#[derive(Serialize, Deserialize, Debug)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: u64, // Number of seconds until expiration
}

impl EmailClient {
    pub fn new(config: EmailConfig) -> Self {
        Self(Arc::new(Mutex::new(config)))
    }

    fn get_config(&self) -> anyhow::Result<EmailConfig> {
        let email_config_arc = Arc::clone(&self.0);
        let lock_acquisition_result = email_config_arc.lock();

        match lock_acquisition_result {
            Ok(config_guard) => {
                if config_guard.access_token.is_some() {
                    Ok(config_guard.clone())
                } else {
                    Err(anyhow!("Could not find email config: access token missing"))
                }
            }
            Err(e) => Err(anyhow!("Could not lock email config (poisoned): {e}")),
        }
    }

    async fn refresh_token(&self) -> anyhow::Result<()> {
        match self.get_config() {
            Ok(config) => {
                let client = Client::new();
                let mut params = HashMap::new();
                params.insert("client_secret", &config.client_secret);
                params.insert("client_id", &config.client_id);
                params.insert("refresh_token", &config.refresh_token);
                let grant_type = Some("refresh_token".to_string());
                params.insert("grant_type", &grant_type);

                let response = client
                    .post("https://oauth2.googleapis.com/token")
                    .form(&params)
                    .send()
                    .await?;

                if response.status().is_success() {
                    let token_response: TokenResponse = response.json().await?;
                    let mut email_config = self.0.lock().unwrap();
                    email_config.access_token = Some(token_response.access_token);
                    // Update token_expiry based on the current time
                    let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
                    email_config.token_expiry = current_time + token_response.expires_in;
                    Ok(())
                } else {
                    let error_text = response.text().await?;
                    error!("Error: {}", error_text);
                    Err(anyhow!("Failed to refresh token"))
                }
            }
            Err(_) => Err(anyhow!("Failed to update refresh token")),
        }
    }

    // #[tracing::instrument(skip(self, ssb_event))]
    // use tracing to add more logs to this function
    pub async fn send_email_gmail(&self, ssb_event: &ServerSideBookingEvent) -> Result<(), String> {
        // Check if the access token is expired
        if self.is_token_expired().map_err(|f| f.to_string())? {
            self.refresh_token().await.map_err(|e| e.to_string())?;
        }

        let mail_state = self.get_config().ok();

        // get booking details from backend
        let admin_canister = AdminCanisters::from_env();
        let backend = admin_canister.backend_canister().await;
        let booking_id = BookingId::from_order_id(&ssb_event.order_id).ok_or(format!(
            "Failed to parse booking id: {}",
            ssb_event.order_id
        ))?;
        let booking = backend
            .get_booking_by_id(booking_id.into())
            .await
            .map_err(|e| format!("Failed to get booking: Got: Err({})", e))?
            .ok_or("Booking not found")?;

        let username = &booking.get_user_name();
        let to = &booking.get_user_email();
        // todo change this later
        let cc = "abhishek@estatedao.org";
        let booking_id = booking.get_booking_ref();
        let check_in_date = booking.get_check_in_date();
        let check_out_date = booking.get_check_out_date();
        let hotel_name = booking.get_hotel_name();
        let hotel_location = booking.get_hotel_location();
        let number_of_adults = booking.get_number_of_adults();
        let number_of_children = booking.get_number_of_children();
        let amount_paid = booking.get_amount_paid();

        match mail_state {
            Some(state) => {
                let access_token = state.access_token;
                let subject = "Booking Confirmed with Nofeebooking";

                // are you sure that this created a valid good string in email body? in my email, I see some space on the left with this. can you fix that?
                let body = format!(
                    r#"
Hey {username},

The adventure begins! Your hotel booking is confirmed and we'’re just as excited as you are! 👻🎒

Here are your booking details:

🏨 Hotel: {hotel_name}
📍 Location: {hotel_location}
🆔 Booking ID / App Reference: {booking_id}
📅 Stay Dates: {check_in_date} to {check_out_date}
👥 Guests: {number_of_adults} Adults, {number_of_children} Children
💳 Amount Paid: {amount_paid}

⚡ Note: This booking is non-cancellable!! So pack those bags and bring your best vacay mode vibes! 😎

We can't wait for you to check in, chill out, and make awesome memories. 🌴✨  
If you need anything, we’re just an email away.

Sending best wanderlust wishes,  
Team Nofeebooking 🥳
                "#,
                );

                tracing::debug!("Email body: {}", body);

                let url = "https://www.googleapis.com/gmail/v1/users/me/messages/send";

                // Create the email message
                let email_raw = format!(
                    "To: {}\r\nCc: {}\r\nSubject: {}\r\n\r\n{}",
                    to, cc, subject, body
                );
                let encoded_message = general_purpose::STANDARD.encode(email_raw);
                let payload = serde_json::json!({
                    "raw": encoded_message
                });

                let client = Client::new();
                let mut headers = HeaderMap::new();
                headers.insert(
                    "Authorization",
                    HeaderValue::from_str(&format!("Bearer {}", access_token.unwrap())).unwrap(),
                );
                headers.insert(
                    "Content-Type",
                    HeaderValue::from_str("application/json").unwrap(),
                );

                let response = client
                    .post(url)
                    .body(serde_json::to_vec(&payload).unwrap())
                    .headers(headers)
                    .send()
                    .await;

                if response.as_ref().is_ok() && response.as_ref().unwrap().status().is_success() {
                    Ok(())
                } else {
                    let error_text = response.unwrap().text().await.map_err(|f| f.to_string())?;
                    error!("Failed to send email: {:?}", error_text);
                    Err(format!("Failed to send email: {:?}", error_text))
                }
            }
            None => Err("Failed to get mail config".to_string()),
        }
    }

    fn is_token_expired(&self) -> anyhow::Result<bool> {
        let config = self.get_config()?;
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        Ok(current_time >= config.token_expiry)
    }
}

/// Send booking confirmation email if payment is finished, then update backend email_sent status.
pub async fn update_backend_email_sent(
    event: &ServerSideBookingEvent,
    email_client: &EmailClient,
) -> Result<(), String> {
    // Update backend: set email_sent = true
    let admin_canister = AdminCanisters::from_env();
    let backend = admin_canister.backend_canister().await;
    let update_result = backend
        .update_email_sent(
            backend::BookingId {
                app_reference: event.order_id.clone(),
                email: event.user_email.clone(),
            },
            true,
        )
        .await
        .map_err(|e| format!("Failed to update email_sent: {}", e))?;
    match update_result {
        // Replace Result1::Ok/Err with your actual backend response types
        Result1::Ok => {
            info!("Email sent and backend updated with email_sent = true");
            Ok(())
        }
        Result1::Err(e) => {
            error!("Failed to update email_sent in backend: {:?}", e);
            Err(format!("Failed to update email_sent in backend: {:?}", e))
        }
    }
}

//
// INTEGRATION with PIPELINE
//

#[derive(Debug, Clone)]
pub struct SendEmailAfterSuccessfullBooking;
#[async_trait]
impl PipelineValidator for SendEmailAfterSuccessfullBooking {
    #[instrument(
        name = "validate_send_email_after_successfull_booking",
        skip(self, event),
        err(Debug)
    )]
    async fn validate(&self, event: &ServerSideBookingEvent) -> Result<PipelineDecision, String> {
        // Check if all required fields are present
        // if event.order_id.is_empty() {
        //     return Err("Order ID is missing".to_string());
        // }

        if event.payment_id.is_none() {
            return Err("Payment ID is missing".to_string());
        }

        if event.user_email.is_empty() {
            return Err("User email is missing".to_string());
        }

        if event.payment_status.is_none() {
            return Err("Payment status is missing".to_string());
        }

        // if event.backend_payment_status.is_none() {
        //     return Err("Backend payment status is missing".to_string());
        // }

        // Check payment status conditions
        let payment_status = event.payment_status.as_ref().unwrap();
        // let backend_payment_status = event.backend_payment_status.as_ref().unwrap();

        if payment_status != "finished" {
            return Err(format!(
                "Payment status is not finished: {}",
                payment_status
            ));
        }
        // check if email is already sent. if it is sent, then do not run the pipeline
        let email_sent_status = check_email_sent_status_from_canister(event).await;

        if email_sent_status.is_err() {
            return Err(email_sent_status.err().unwrap());
        }

        if email_sent_status.unwrap() {
            return Ok(PipelineDecision::Skip);
        }

        Ok(PipelineDecision::Run)
    }
}

async fn check_email_sent_status_from_canister(
    event: &ServerSideBookingEvent,
) -> Result<bool, String> {
    let admin_canister = AdminCanisters::from_env();
    let backend = admin_canister.backend_canister().await;
    let booking_id = BookingId::from_order_id(&event.order_id)
        .ok_or(format!("Failed to parse booking id: {}", event.order_id))?;
    let email_sent_status = backend
        .get_email_sent(booking_id.into())
        .await
        .map_err(|e| format!("Failed to get email sent status: Got: Err({})", e))?;

    match email_sent_status {
        Result2::Ok(email_sent_status_bool) => Ok(email_sent_status_bool),
        Result2::Err(e) => Err(format!("Failed to get email sent status: Got: Err({})", e)),
    }
}

async fn update_email_sent_status_in_canister(
    event: &ServerSideBookingEvent,
) -> Result<(), String> {
    let admin_canister = AdminCanisters::from_env();
    let backend = admin_canister.backend_canister().await;
    let booking_id = BookingId::from_order_id(&event.order_id)
        .ok_or(format!("Failed to parse booking id: {}", event.order_id))?;

    let update_result = backend
        .update_email_sent(booking_id.into(), true)
        .await
        .map_err(|e| format!("Failed to update email_sent: {}", e))?;

    match update_result {
        // Replace Result1::Ok/Err with your actual backend response types
        Result1::Ok => {
            info!("Email sent and backend updated with email_sent = true");
            Ok(())
        }
        Result1::Err(e) => {
            error!("Failed to update email_sent in backend: {:?}", e);
            Err(format!("Failed to update email_sent in backend: {:?}", e))
        }
    }
}

impl SendEmailAfterSuccessfullBooking {
    async fn run(event: ServerSideBookingEvent) -> Result<ServerSideBookingEvent, String> {
        let config = EnvVarConfig::try_from_env();
        let email_client = EmailClient::new(config.email_client_config);
        let _ = email_client.send_email_gmail(&event).await?;
        update_email_sent_status_in_canister(&event).await?;
        Ok(event)
    }
}

#[async_trait]
impl PipelineExecutor for SendEmailAfterSuccessfullBooking {
    #[instrument(
        name = "execute_send_email_after_successfull_booking",
        skip(event),
        err(Debug)
    )]
    async fn execute(event: ServerSideBookingEvent) -> Result<ServerSideBookingEvent, String> {
        SendEmailAfterSuccessfullBooking::run(event).await
    }
}
