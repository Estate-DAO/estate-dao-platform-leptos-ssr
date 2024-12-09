use crate::canister::backend::{self, Result_};
use crate::utils::admin::admin_canister;
use crate::utils::app_reference::BookingId;
use colored::Colorize;
use leptos::logging::log;
use leptos::*;

#[server(GreetBackend)]
pub async fn update_book_room_details_backend(
    booking_id: (String, String),
    book_room_details: String,
) -> Result<String, ServerFnError> {
    let book_room_details_struct =
        serde_json::from_str::<backend::BeBookRoomResponse>(&book_room_details).map_err(|er| {
            ServerFnError::new(format!("Could not deserialize Booking: Err = {er:?}"))
        })?;

    let adm_cans = admin_canister();

    let backend_cans = adm_cans.backend_canister().await;
    println!("{:#?}", book_room_details_struct);

    let result = backend_cans
        .update_book_room_response(booking_id, book_room_details_struct)
        .await?;

    println!("{}", format!("{:#?}", result).bright_purple().bold());

    match result {
        Result_::Ok(save_status) => Ok(save_status),
        Result_::Err(e) => Err(ServerFnError::ServerError(e)),
    }
}
