use crate::api::canister::book_room_details::update_book_room_details_backend;
use crate::api::BookingDetails;
use crate::{
    api::{
        book_room, canister::get_user_booking::get_user_booking_backend, BookRoomRequest,
        BookRoomResponse, RoomDetail,
    },
    canister::backend::{self, BeBookRoomResponse},
    page::{
        confirm_booking::{
            booking_handler::read_booking_details_from_local_storage,
            confirmation::PaymentBookingStatusUpdates,
        },
        create_passenger_details,
    },
    state::{
        local_storage::use_booking_id_store,
        search_state::{BlockRoomResults, ConfirmationResults, SearchCtx, SearchListResults},
        view_state::{BlockRoomCtx, HotelInfoCtx},
    },
    utils::app_reference,
};
use candid::types::result;
use colored::Colorize;
use leptos::*;
use leptos::logging::log;

#[component]
pub fn BookRoomHandler() -> impl IntoView {
    let (booking_id_signal_read, _, _) = use_booking_id_store();
    let block_room_ctx = expect_context::<BlockRoomCtx>();
    let block_room_results = expect_context::<BlockRoomResults>();
    let confirmation_results = expect_context::<ConfirmationResults>();
    let hotel_info_ctx = expect_context::<HotelInfoCtx>();
    let payment_booking_step_signals: PaymentBookingStatusUpdates = expect_context();

    // payment_booking_step_signals
    //     .p03_call_book_room_api
    //     .set(true);

    let book_room_api_call = create_resource(
        move || payment_booking_step_signals.p03_call_book_room_api.get(),
        move |p03_call_book_room_api| async move {
            if !p03_call_book_room_api {
                log!("p03_call_book_room_api = {p03_call_book_room_api:?}");
                return None;
            }

            log!("outside first early return p03_call_book_room_api = {p03_call_book_room_api:?}");

            let hotel_info_ctx: HotelInfoCtx = expect_context();
            let search_list_result: SearchListResults = expect_context();
            let block_room: BlockRoomResults = expect_context();
            let conf_res: ConfirmationResults = expect_context();

            let result_token = search_list_result.get_result_token(hotel_info_ctx.hotel_code.get());

            if block_room.block_room_id.get_untracked().is_none() {
                println!(
                    "{}",
                    format!(
                        "block_room_id is not set in context - Got: {:?} ",
                        block_room.block_room_id.get()
                    )
                    .magenta()
                    .bold()
                );
                log!("{:?}", block_room.block_room_id.get());
                return None;
            }

            let block_room_id = block_room.block_room_id.get_untracked().unwrap();
            log!("{block_room_id:?}");

            let adults = block_room_ctx.adults.get_untracked();
            let children = block_room_ctx.children.get_untracked();

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
            let value_for_serverfn: String = serde_json::to_string(&req)
                .expect("serde_json::to_string for BookRoomRequest did not happen");

            log::warn!("REQ FOR BOOK ROOM API >>>{:?}", req);

            let book_room_result = book_room(value_for_serverfn).await.ok();
            log::info!("BOOK_ROOM_API: {book_room_result:?}");
            conf_res.booking_details.set(book_room_result.clone());

            // todo [UAT] - does book_room have a failure response?
            // if yes, model that and make a method 'is_response_success' on BookRoomResponse
            if book_room_result.is_some() {
                log!("p04_update_booking_details_to_backend - with book_room_result = {book_room_result:#?}");
                payment_booking_step_signals
                    .p04_update_booking_details_to_backend
                    .set(true);
                Some(book_room_result.clone())
            } else {
                println!(
                    "{}",
                    format!("book_room_api_cal results - {:?}", book_room_result.clone()).magenta()
                );
                None
            }
        },
    );

    let book_room_canister_call = create_resource(
        move || {
            payment_booking_step_signals
                .p04_update_booking_details_to_backend
                .get()
        },
        move |p04_update_booking_details_to_backend| async move {
            if !p04_update_booking_details_to_backend {
                println!("{}", format!("p04_update_booking_details_to_backend = {p04_update_booking_details_to_backend:?}").red().bold());
                return None;
            }

            if confirmation_results.booking_details.get().is_none() {
                println!(
                    "{}",
                    format!("confirmation_results.booking_details is None")
                        .red()
                        .bold()
                );
                return None;
            }
            let (email, app_reference) = read_booking_details_from_local_storage().unwrap();
            let booking_id = (email.clone(), app_reference.clone());

            let book_room_response = confirmation_results.booking_details.get().unwrap();

            let book_room_backend = create_backend_book_room_response(
                (email, app_reference),
                book_room_response.clone(),
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
        },
    );

    view! {
        <div class="bg-gray-100 p-4 border border-emerald-800">
        <Suspense fallback={move || view!{ " Making the booking ... "}}>
        {move ||
            if let Some(book_room_response) = book_room_api_call.get(){
                view!{
                    <p>
                    "Booking Made!"
                    {format!("details: {book_room_response:#?}")}
                    </p>
                }.into_view()
            }else{
                view!{
                    "Could not make booking..."
                }.into_view()
            }
        }
        </Suspense>

        </div>
        <div class="bg-gray-100 p-4 border border-emerald-800">
        <Suspense fallback={move || view!{ " Making the booking ... "}}>
        {move ||
            if let Some(book_room_response) = book_room_canister_call.get(){
                view!{
                    <p>
                    "Booking saved to database!"
                    {format!("details: {book_room_response:?}")}
                    </p>
                }.into_view()
            }else{
                view!{
                    // todo [UAT] = None can be obtained due to any of the follwoing going wrong
                    // 1. not present in context
                    // 2. signal not set yet
                    // 3. could not save to backend
                    "Could not save booking details to database"
                }.into_view()
            }
        }
        </Suspense>

        </div>
    }
}

fn create_backend_book_room_response(
    (email, app_reference): (String, String),
    book_room_response: BookRoomResponse,
) -> BeBookRoomResponse {
    let fe_booking_details: BookingDetails = book_room_response.commit_booking.into();

    let be_booking_details = backend::BookingDetails {
        booking_id: (email, app_reference),
        travelomatrix_id: fe_booking_details.travelomatrix_id,
        booking_ref_no: fe_booking_details.booking_ref_no,
        booking_status: fe_booking_details.booking_status,
        confirmation_no: fe_booking_details.confirmation_no,
        api_status: book_room_response.status.clone().into(),
    };
    BeBookRoomResponse {
        commit_booking: be_booking_details,
        status: book_room_response.status.to_string(),
        message: book_room_response.message,
    }
}
