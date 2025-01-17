use crate::canister::backend::{
    BackendPaymentStatus, Booking, HotelDetails, HotelRoomDetails, PaymentDetails,
    SelectedDateRange, UserDetails,
};
use crate::state::search_state::{
    BlockRoomResults, ConfirmationResults, HotelInfoResults, SearchCtx, SearchListResults,
};
use crate::utils::admin::admin_canister;
use leptos::logging::log;
use leptos::*;

#[server]
pub async fn add_booking_backend(email: String, booking: String) -> Result<String, ServerFnError> {
    // log!("\n booking args {:?}", booking);
    let booking_struct = serde_json::from_str::<Booking>(&booking)
        .map_err(|er| ServerFnError::new(format!("Could not deserialize Booking: Err = {er:?}")))?;
    // log!("\n booking_struct{:#?}", booking_struct);

    let adm_cans = admin_canister();
    let backend_cans = adm_cans.backend_canister().await;

    let result = backend_cans.add_booking(email, booking_struct).await;

    log!("{result:?}");

    Ok("Got it!".into())
}
