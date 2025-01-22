use crate::{
    api::{
    	payments::{nowpayments_get_payment_status, ports::GetPaymentStatusRequest},
     	a04_book_room::{RoomDetail, book_room},
      	SuccessBookRoomResponse
    },
    page::create_passenger_details,
    canister::backend::{BackendPaymentStatus::Paid, BePaymentApiResponse, Booking, BookingId},
    state::{
        local_storage::{use_booking_id_store, use_payment_store},
        search_state::{BlockRoomResults, ConfirmationResults},
        view_state::{BlockRoomCtx, HotelInfoCtx},
    },
};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use super::{canister::book_room_details::update_book_room_details_backend, BookRoomResponse};

#[server]
#[cfg(feature = "ssr")]
pub mod ssr_imports {
    pub use broadcaster::BroadcastChannel;
    pub use once_cell::sync::OnceCell;
    pub use std::sync::atomic::{AtomicI32, Ordering};
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use tokio::time::{interval, Duration};

    pub static COUNT: AtomicI32 = AtomicI32::new(0);

    lazy_static::lazy_static! {
        pub static ref COUNT_CHANNEL: BroadcastChannel<usize> = BroadcastChannel::new();
    }

    static LOG_INIT: OnceCell<()> = OnceCell::new();

    pub fn init_logging() {
        LOG_INIT.get_or_init(|| {
            simple_logger::SimpleLogger::new().env().init().unwrap();
        });
    }
}

#[server]
#[cfg(feature = "ssr")]
pub async fn get_server_count() -> Result<usize, ServerFnError> {
    use ssr_imports::*;
    Ok(COUNT.load(Ordering::Relaxed))
}

#[server]
#[cfg(feature = "ssr")]
pub async fn get_booking_details() -> Result<Booking, ServerFnError> {
    use ssr_imports::*;
    Ok(COUNT.load(Ordering::Relaxed))
}

// #[server]
#[cfg(feature = "ssr")]
pub async fn adjust_server_count(delta: i32) -> Result<i32, ServerFnError> {
    use ssr_imports::*;

    let new = COUNT.load(Ordering::Relaxed) as i32 + delta;
    COUNT.store(new, Ordering::Relaxed);
    _ = COUNT_CHANNEL.send(&new).await;
    Ok(new as usize)
}

// #[server]
// #[cfg(feature = "ssr")]
// pub async fn clear_server_count() -> Result<i32, ServerFnError> {
//     use ssr_imports::*;

//     COUNT.store(0, Ordering::Relaxed);
//     _ = COUNT_CHANNEL.send(&0).await;
//     Ok(0)
// }

#[server]
#[cfg(feature = "ssr")]
pub async fn init_booking_state(
    booking: Booking,
    payment_id: u64,
    email: String,
    app_reference_string: String,
) -> Result<(), ServerFnError> {
    use ssr_imports::*;

    COUNT.store(0, Ordering::Relaxed);
    _ = COUNT_CHANNEL.send(&0).await;
    let booking_id = BookingId	{
    	app_reference: app_reference_string,
     	email
    };
    let response = start_payment_polling(payment_id).await?;
    let booking = update_payment_details(booking_id, response).await?;
    let book_room_response = book_room_sse(app_reference_string, booking).await?;
    _ = update_book_room_details_sse(BookRoomResponse::Success(book_room_response), booking_id).await?;
    Ok()
}

#[cfg(feature = "ssr")]
async fn start_payment_polling(payment_id: u64) -> Result<GetPaymentStatusResponse, ServerFnError> {
    use ssr_imports::*;
    let mut interval = interval(Duration::from_millis(4000));

    loop {
        interval.tick().await;

        // Fetch payment status from the API
        let resp = nowpayments_get_payment_status(GetPaymentStatusRequest { payment_id })
            .await
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

        let payment_api_response = BePaymentApiResponse::from((resp, "NOWPayments".to_string()));

        // If the payment status is "finished", return the response
        if resp.get_payment_status() == "finished" {
            adjust_server_count(1);
            return Ok(response);
        }
    }
}

#[cfg(feature = "ssr")]
async fn update_payment_details(
    booking_id: BookingId,
    response: GetPaymentStatusResponse,
) -> Result<Booking, ServerFnError> {
    // Convert the response to the backend format
    let payment_api_response = BePaymentApiResponse::from((response, "NOWPayments".to_string()));

    // Create payment details
    let payment_details = PaymentDetails {
        booking_id: booking_id.clone(),
        // payment_status: if response.get_payment_status() == "finished" {
        payment_status: BackendPaymentStatus::Paid(payment_api_response.order_id.clone()),
        // } else {
        //     BackendPaymentStatus::Unpaid(None)
        // },
        payment_api_response,
    };

    // Serialize payment details
    let payment_details_str = serde_json::to_string(&payment_details)
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    // Update payment details on the backend
    let booking = update_payment_details_backend(booking_id, payment_details_str)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    adjust_server_count(1);
    Ok(booking)
}

#[cfg(feature = "ssr")]
async fn book_room_sse(
    app_reference: String,
    booking: Booking,
) -> Option<SuccessBookRoomResponse> {
    let result_token = booking
        .user_selected_hotel_room_details
        .hotel_details
        .hotel_code;

    let block_room_id = booking
        .user_selected_hotel_room_details
        .hotel_details
        .block_room_id;
    
    let booking_guests = booking.guests.clone();
    let booking_guests2 = booking.guests.clone();

    let adults: Vec<crate::state::view_state::AdultDetail> = booking_guests.into();
    let children: Vec<crate::state::view_state::ChildDetail> = booking_guests2.into();

    let room_detail = RoomDetail {
        passenger_details: create_passenger_details(&adults, &children),
    };

    let (email, app_reference) = read_booking_details_from_local_storage().unwrap();
    let req = BookRoomRequest {
        result_token,
        block_room_id,
        app_reference,
        room_details: vec![room_detail],
    };
    log::info!("BOOK_ROOM_API / REQ: {req:?}");

    let value_for_serverfn: String = serde_json::to_string(&req)
        .expect("serde_json::to_string for BookRoomRequest did not happen");
    
    let book_room_result_server = book_room(value_for_serverfn).await;
    
    let book_room_result = match book_room_result_server {
        Ok(book_room_result) => {
            match serde_json::from_str::<SuccessBookRoomResponse>(&book_room_result) {
                Ok(book_room_response_struct) => {
                    book_room_response_struct.clone()
                }
                Err(e) => {
                	return None;
                }
            }
        }
        Err(e) => {
        	return None;
        }
    };

    adjust_server_count(1);
    Some(book_room_result)
}

#[cfg(feature = "ssr")]
async fn update_book_room_details_sse(
    book_room_response: BookRoomResponse,
    booking_id: BookingId,
) -> Option<String> {

    let book_room_response = confirmation_results.booking_details.get().unwrap();

    let book_room_backend = create_backend_book_room_response(
        booking_id,
        book_room_response
    );

    let book_room_backend_str = serde_json::to_string(&book_room_backend)
        .expect("serde_json::to_string for BeBookRoomResponse");

    let book_room_backend_saved_status =
        update_book_room_details_backend(booking_id, book_room_backend_str)
            .await
            .ok();

    if book_room_backend_saved_status.is_none() {
        return None;
    }
    
    match book_room_backend_saved_status
        .unwrap()
        .to_lowercase()
        .as_str()
    {
        "success" => Some("success".to_string()),
        any_other => Some(any_other.to_string()),
    }
}

#[cfg(feature = "ssr")]
fn create_backend_book_room_response(
    booking_id: BookingId,
    book_room_response: BookRoomResponse,
) -> BeBookRoomResponse {
    match book_room_response {
        BookRoomResponse::Failure(fe_booking_details_fail) => BeBookRoomResponse {
            commit_booking: backend::BookingDetails::default(),
            status: fe_booking_details_fail.status.to_string(),
            message: fe_booking_details_fail.message,
        },
        BookRoomResponse::Success(fe_booking_details_success) => {
            let fe_booking_details: BookingDetails =
                fe_booking_details_success.commit_booking.into();

            let be_booking_details = backend::BookingDetails {
                booking_id,
                travelomatrix_id: fe_booking_details.travelomatrix_id,
                booking_ref_no: fe_booking_details.booking_ref_no,
                booking_status: fe_booking_details.booking_status,
                confirmation_no: fe_booking_details.confirmation_no,
                api_status: fe_booking_details_success.status.clone().into(),
            };
            BeBookRoomResponse {
                commit_booking: be_booking_details,
                status: fe_booking_details_success.status.to_string(),
                message: fe_booking_details_success.message,
            }
        }
    }
}
