use crate::canister::backend::{Booking, PaymentDetails, Result1};
use crate::utils::admin::admin_canister;
use crate::utils::app_reference::BookingId;
use leptos::logging::log;
use leptos::*;

#[server(GreetBackend)]
pub async fn update_payment_details_backend(
    booking_id: (String, String),
    payment_details: PaymentDetails,
) -> Result<Booking, ServerFnError> {
    let adm_cans = admin_canister();

    let backend_cans = adm_cans.backend_canister().await;

    let result = backend_cans
        .update_payment_details(booking_id, payment_details)
        .await;

    log!("{result:#?}");

    match result {
        Ok(Result1::Ok(booking)) => Ok(booking),
        Ok(Result1::Err(e)) => Err(ServerFnError::ServerError(e)),
        Err(e) => {
            log!("Failed to update payment details: {:?}", e);
            Err(ServerFnError::ServerError(e.to_string()))
        }
    }
}
