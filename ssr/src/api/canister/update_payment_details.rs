use crate::canister::backend::{self, Booking, BookingId, PaymentDetails, Result1, Result3};
use crate::utils::admin::admin_canister;
use leptos::logging::log;
use leptos::*;

#[server(GreetBackend)]
pub async fn update_payment_details_backend(
    booking_id: backend::BookingId,
    payment_details: String,
) -> Result<Booking, ServerFnError> {
    let payment_details_struct = serde_json::from_str::<PaymentDetails>(&payment_details)
        .map_err(|er| ServerFnError::new(format!("serverfn - update_payment_details_backend - Could not deserialize Booking: Err = {er:?}")))?;

    call_update_payment_details_backend(booking_id, payment_details_struct)
        .await
        .map_err(ServerFnError::ServerError)
}

pub async fn call_update_payment_details_backend(
    booking_id: backend::BookingId,
    payment_details_struct: PaymentDetails,
) -> Result<Booking, String> {
    let adm_cans = admin_canister();

    let backend_cans = adm_cans.backend_canister().await;
    log!("call_update_payment_details_backend - payment_details_struct - {payment_details_struct:#?}");

    let result = backend_cans
        .update_payment_details(booking_id, payment_details_struct)
        .await
        .map_err(|e| e.to_string())?;

    log!("call_update_payment_details_backend - result - {result:#?}");

    match result {
        Result3::Ok(booking) => Ok(booking),
        Result3::Err(e) => Err(e),
    }
}
