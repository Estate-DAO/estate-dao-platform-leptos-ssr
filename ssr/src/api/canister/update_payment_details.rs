use crate::canister::backend::{Booking, PaymentDetails, Result2};
use crate::utils::admin::admin_canister;
use crate::utils::app_reference::BookingId;
use leptos::logging::log;
use leptos::*;

#[server(GreetBackend)]
pub async fn update_payment_details_backend(
    booking_id: (String, String),
    payment_details: String,
) -> Result<Booking, ServerFnError> {
    let payment_details_struct = serde_json::from_str::<PaymentDetails>(&payment_details)
        .map_err(|er| ServerFnError::new(format!("Could not deserialize Booking: Err = {er:?}")))?;

    let adm_cans = admin_canister();

    let backend_cans = adm_cans.backend_canister().await;
    println!("{:#?}", payment_details_struct);

    let result = backend_cans
        .update_payment_details(booking_id, payment_details_struct)
        .await;

    println!("{result:#?}");

    match result {
        Ok(Result2::Ok(booking)) => Ok(booking),
        Ok(Result2::Err(e)) => Err(ServerFnError::ServerError(e)),
        Err(e) => {
            println!("Failed to update payment details: {:?}", e);
            Err(ServerFnError::ServerError(e.to_string()))
        }
    }
}
