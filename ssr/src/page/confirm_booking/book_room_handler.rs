use crate::{
    api::{
        book_room, canister::get_user_booking::get_user_booking_backend, BookRoomRequest,
        RoomDetail,
    },
    canister::backend,
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

#[component]
pub fn BookRoomHandler() -> impl IntoView {
    let (booking_id_signal_read, _, _) = use_booking_id_store();
    let block_room_ctx = expect_context::<BlockRoomCtx>();
    let block_room = expect_context::<BlockRoomResults>();
    let confirmation_ctx = expect_context::<ConfirmationResults>();
    let hotel_info_ctx = expect_context::<HotelInfoCtx>();
    let payment_booking_step_signals: PaymentBookingStatusUpdates = expect_context();

    // payment_booking_step_signals
    //     .p03_call_book_room_api
    //     .set(true);

    let book_room_api_call = create_resource(
        move || payment_booking_step_signals.p03_call_book_room_api.get(),
        move |p03_call_book_room_api| async move {
            if !p03_call_book_room_api {
                return None;
            }
            let hotel_info_ctx: HotelInfoCtx = expect_context();
            let search_list_result: SearchListResults = expect_context();
            let block_room: BlockRoomResults = expect_context();
            let conf_res: ConfirmationResults = expect_context();

            let result_token = search_list_result.get_result_token(hotel_info_ctx.hotel_code.get());

            if block_room.block_room_id.get().is_none() {
                println!(
                    "{}",
                    format!(
                        "block_room_id is not set in context - Got: {:?} ",
                        block_room.block_room_id.get()
                    )
                    .magenta()
                    .bold()
                );
                return None;
            }

            let block_room_id = block_room.block_room_id.get().unwrap();

            let adults = block_room_ctx.adults.get();
            let children = block_room_ctx.children.get();

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
    }
}
