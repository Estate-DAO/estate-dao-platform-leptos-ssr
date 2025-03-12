use crate::canister::backend::{self, Booking, BookingId, PaymentDetails, Result1};
use crate::utils::admin::admin_canister;
use leptos::logging::log;
use leptos::*;

// write the following function in idiomatic rust with await.map_err AI!
#[server(GreetBackend)]
pub async fn update_payment_details_backend(
    booking_id: backend::BookingId,
    payment_details: String,
) -> Result<Booking, ServerFnError> {
    let payment_details_struct = serde_json::from_str::<PaymentDetails>(&payment_details)
        .map_err(|er| ServerFnError::new(format!("Could not deserialize Booking: Err = {er:?}")))?;

    let result = call_update_payment_details_backend(booking_id, payment_details_struct).await;

    match result {
        Ok(booking) => Ok(booking),
        Err(e) => Err(ServerFnError::ServerError(e)),
    }
}


pub async fn call_update_payment_details_backend(
    booking_id: backend::BookingId,
    payment_details_struct: PaymentDetails,
) -> Result<Booking, String> {
    let adm_cans = admin_canister();

    let backend_cans = adm_cans.backend_canister().await;
    println!("{:#?}", payment_details_struct);

    let result = backend_cans
        .update_payment_details(booking_id, payment_details_struct)
        .await
        .map_err(|e| e.to_string())?;

    println!("{result:#?}");

    match result {
        Result1::Ok(booking) => Ok(booking),
        Result1::Err(e) => Err(e),
    }
}