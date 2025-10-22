use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    api::{
        client_side_api::{
            CheckPaymentStatusRequest, ClientSideApiClient, GetBackendBookingRequest,
            UpdatePaymentRequest,
        },
        payments::ports::{FailureGetPaymentStatusResponse, GetPaymentStatusResponse},
    },
    canister::backend::{
        BackendPaymentStatus, BePaymentApiResponse, Booking, BookingId as BackendBookingId,
        PaymentDetails,
    },
    send_wrap,
};

#[derive(Clone, Default, Debug)]
pub struct AdminEditPanelState {
    pub payment_id_input: RwSignal<String>,
    pub payment_status_display: RwSignal<Option<GetPaymentStatusResponse>>,
    pub backend_email_input: RwSignal<String>,
    pub backend_app_ref_input: RwSignal<String>,
    pub backend_booking_display: RwSignal<Option<Booking>>,
    pub update_status: RwSignal<String>,
    pub payment_details_form: PaymentDetailsFormState,
}

#[derive(Clone, Default, Debug)]
pub struct PaymentDetailsFormState {
    pub payment_status: RwSignal<String>,
    pub payment_id: RwSignal<String>,
    pub provider: RwSignal<String>,
    pub actually_paid: RwSignal<String>,
    pub invoice_id: RwSignal<String>,
    pub order_description: RwSignal<String>,
    pub pay_amount: RwSignal<String>,
    pub pay_currency: RwSignal<String>,
    pub price_amount: RwSignal<String>,
    pub purchase_id: RwSignal<String>,
    pub order_id: RwSignal<String>,
    pub price_currency: RwSignal<String>,
}

impl AdminEditPanelState {
    pub fn new() -> Self {
        Self::default()
    }
}

// Client API functions
async fn check_payment_status_api(payment_id: u64) -> Option<GetPaymentStatusResponse> {
    let client = ClientSideApiClient::new();
    let request = CheckPaymentStatusRequest { payment_id };
    client.check_payment_status(request).await
}

async fn get_backend_booking_api(email: String, app_reference: String) -> Option<Booking> {
    let client = ClientSideApiClient::new();
    let request = GetBackendBookingRequest {
        email,
        app_reference,
    };
    client.get_backend_booking(request).await
}

async fn update_payment_details_api(
    email: String,
    app_reference: String,
    payment_details: PaymentDetails,
) -> Option<String> {
    let client = ClientSideApiClient::new();
    let request = UpdatePaymentRequest {
        email,
        app_reference,
        payment_details,
    };
    client.update_payment_details(request).await
}

// Helper components
#[component]
fn AdminInput(
    placeholder: String,
    value_signal: RwSignal<String>,
    #[prop(optional)] input_type: Option<String>,
) -> impl IntoView {
    let input_type = input_type.unwrap_or("text".to_string());

    view! {
        <div class="mb-4">
            <input
                type=input_type
                placeholder=placeholder
                class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                prop:value=move || value_signal.get()
                on:input=move |ev| value_signal.set(event_target_value(&ev))
            />
        </div>
    }
}

#[component]
fn StatusBadge(status: String, status_type: &'static str) -> impl IntoView {
    let status_clone = status.clone();

    view! {
        <span class={move || {
            let base_classes = "inline-flex px-2 py-1 text-xs font-semibold rounded-full";
            match status_type {
                "payment" => match status.as_str() {
                    "finished" | "confirmed" => format!("{} bg-green-100 text-green-800", base_classes),
                    "waiting" | "pending" => format!("{} bg-yellow-100 text-yellow-800", base_classes),
                    "failed" | "cancelled" => format!("{} bg-red-100 text-red-800", base_classes),
                    _ => format!("{} bg-gray-100 text-gray-800", base_classes),
                },
                _ => format!("{} bg-gray-100 text-gray-800", base_classes),
            }
        }}>
            {status_clone}
        </span>
    }
}

#[component]
fn PaymentStatusDisplay(payment_response: GetPaymentStatusResponse) -> impl IntoView {
    match payment_response {
        GetPaymentStatusResponse::Success(success) => view! {
            <div class="bg-white p-4 rounded-lg border border-gray-200">
                <h3 class="text-lg font-semibold mb-3">"Payment Status Details"</h3>
                <div class="grid grid-cols-2 gap-4">
                    <div>
                        <span class="font-medium">"Status: "</span>
                        <StatusBadge status=success.payment_status.clone() status_type="payment" />
                    </div>
                    <div>
                        <span class="font-medium">"Payment ID: "</span>
                        <span>{success.payment_id}</span>
                    </div>
                    <div>
                        <span class="font-medium">"Order ID: "</span>
                        <span>{success.order_id}</span>
                    </div>
                    <div>
                        <span class="font-medium">"Amount: "</span>
                        <span>{format!("{} {}", success.pay_amount, success.pay_currency)}</span>
                    </div>
                    <div>
                        <span class="font-medium">"Actually Paid: "</span>
                        <span>{success.actually_paid}</span>
                    </div>
                    <div>
                        <span class="font-medium">"Created: "</span>
                        <span>{success.created_at}</span>
                    </div>
                    <div>
                        <span class="font-medium">"Updated: "</span>
                        <span>{success.updated_at}</span>
                    </div>
                    <div class="col-span-2">
                        <span class="font-medium">"Description: "</span>
                        <span>{success.order_description}</span>
                    </div>
                </div>
            </div>
        }
        .into_any(),
        GetPaymentStatusResponse::Failure(error) => view! {
            <div class="bg-red-50 p-4 rounded-lg border border-red-200">
                <h3 class="text-lg font-semibold text-red-800 mb-2">"Error"</h3>
                <p class="text-red-700">"Payment status check failed"</p>
            </div>
        }
        .into_any(),
    }
}

#[component]
fn BookingDisplay(booking: Booking) -> impl IntoView {
    let payment_status = match &booking.payment_details.payment_status {
        BackendPaymentStatus::Paid(status) => format!("Paid: {}", status),
        BackendPaymentStatus::Unpaid(status) => match status {
            Some(s) => format!("Unpaid: {}", s),
            None => "Unpaid".to_string(),
        },
    };

    view! {
        <div class="bg-white p-4 rounded-lg border border-gray-200">
            <h3 class="text-lg font-semibold mb-3">"Backend Booking Details"</h3>
            <div class="grid grid-cols-2 gap-4">
                <div>
                    <span class="font-medium">"Email: "</span>
                    <span>{booking.booking_id.email}</span>
                </div>
                <div>
                    <span class="font-medium">"App Reference: "</span>
                    <span>{booking.booking_id.app_reference}</span>
                </div>
                <div>
                    <span class="font-medium">"Hotel: "</span>
                    <span>{booking.user_selected_hotel_room_details.hotel_details.hotel_name}</span>
                </div>
                <div>
                    <span class="font-medium">"Payment Status: "</span>
                    <StatusBadge status=payment_status status_type="payment" />
                </div>
                <div>
                    <span class="font-medium">"Payment ID: "</span>
                    <span>{booking.payment_details.payment_api_response.payment_id}</span>
                </div>
                <div>
                    <span class="font-medium">"Order ID: "</span>
                    <span>{booking.payment_details.payment_api_response.order_id}</span>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn AdminEditPanel() -> impl IntoView {
    let state = AdminEditPanelState::new();

    // Payment status lookup
    let check_payment_action = Action::new(move |payment_id: &String| {
        let payment_id = payment_id.parse::<u64>().unwrap_or(0);
        check_payment_status_api(payment_id)
    });

    // Backend booking lookup
    let get_booking_action = Action::new(move |(email, app_ref): &(String, String)| {
        send_wrap(get_backend_booking_api(email.clone(), app_ref.clone()))
    });

    // Payment update action
    let update_payment_action = Action::new(
        move |(email, app_ref, payment_details): &(String, String, PaymentDetails)| {
            send_wrap(update_payment_details_api(
                email.clone(),
                app_ref.clone(),
                payment_details.clone(),
            ))
        },
    );

    // Handle payment status response
    Effect::new(move |_| {
        if let Some(Some(response)) = check_payment_action.value().get() {
            state.payment_status_display.set(Some(response));
        }
    });

    // Handle booking response
    Effect::new(move |_| {
        if let Some(booking_opt) = get_booking_action.value().get() {
            state.backend_booking_display.set(booking_opt);
        }
    });

    // Handle update response
    Effect::new(move |_| {
        if let Some(result) = update_payment_action.value().get() {
            match result {
                Some(message) => state.update_status.set(format!("✅ {}", message)),
                None => state
                    .update_status
                    .set("❌ Error: Failed to update payment details".to_string()),
            }
        }
    });

    view! {
        <div class="container mx-auto px-4 py-8 max-w-6xl">
            <h1 class="text-3xl font-bold mb-8">"Admin Payment Edit Panel"</h1>

            // Payment ID Lookup Section
            <div class="bg-gray-50 p-6 rounded-lg mb-8">
                <h2 class="text-xl font-semibold mb-4">"1. Payment Status Lookup"</h2>
                <div class="flex gap-4 items-end">
                    <div class="flex-1">
                        <AdminInput
                            placeholder="Enter Payment ID".to_string()
                            value_signal=state.payment_id_input
                            input_type="number".to_string()
                        />
                    </div>
                    <button
                        class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700"
                        on:click=move |_| {
                            let payment_id = state.payment_id_input.get();
                            if !payment_id.is_empty() {
                                check_payment_action.dispatch(payment_id);
                            }
                        }
                    >
                        "Check Status"
                    </button>
                </div>

                // Display payment status
                {move || {
                    if let Some(response) = state.payment_status_display.get() {
                        view! {
                            <div class="mt-6">
                                <PaymentStatusDisplay payment_response=response />
                            </div>
                        }.into_any()
                    } else {
                        view! { <div></div> }.into_any()
                    }
                }}
            </div>

            // Backend Booking Lookup Section
            <div class="bg-gray-50 p-6 rounded-lg mb-8">
                <h2 class="text-xl font-semibold mb-4">"2. Backend Booking Lookup"</h2>
                <div class="grid grid-cols-2 gap-4 mb-4">
                    <AdminInput
                        placeholder="Enter Email".to_string()
                        value_signal=state.backend_email_input
                    />
                    <AdminInput
                        placeholder="Enter App Reference".to_string()
                        value_signal=state.backend_app_ref_input
                    />
                </div>
                <div class="flex items-center gap-4">
                    <button
                        class="px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700"
                        on:click=move |_| {
                            let email = state.backend_email_input.get();
                            let app_ref = state.backend_app_ref_input.get();
                            if !email.is_empty() && !app_ref.is_empty() {
                                get_booking_action.dispatch((email, app_ref));
                            }
                        }
                    >
                        "Get Booking"
                    </button>
                    <span class="text-sm text-gray-600">
                        {move || {
                            let email = state.backend_email_input.get();
                            let app_ref = state.backend_app_ref_input.get();
                            if !email.is_empty() && !app_ref.is_empty() {
                                format!("BookingId: {}-{}", email, app_ref)
                            } else {
                                "BookingId: (enter email and app reference)".to_string()
                            }
                        }}
                    </span>
                </div>

                // Display booking details
                {move || {
                    if let Some(booking) = state.backend_booking_display.get() {
                        view! {
                            <div class="mt-6">
                                <BookingDisplay booking=booking />
                            </div>
                        }.into_any()
                    } else {
                        view! { <div></div> }.into_any()
                    }
                }}
            </div>

            // Payment Update Section
            <div class="bg-gray-50 p-6 rounded-lg mb-8">
                <h2 class="text-xl font-semibold mb-4">"3. Update Payment Details"</h2>
                <div class="grid grid-cols-2 gap-4 mb-4">
                    <AdminInput
                        placeholder="Payment Status (finished/waiting/failed)".to_string()
                        value_signal=state.payment_details_form.payment_status
                    />
                    <AdminInput
                        placeholder="Payment ID".to_string()
                        value_signal=state.payment_details_form.payment_id
                        input_type="number".to_string()
                    />
                    <AdminInput
                        placeholder="Provider (nowpayments/stripe)".to_string()
                        value_signal=state.payment_details_form.provider
                    />
                    <AdminInput
                        placeholder="Actually Paid".to_string()
                        value_signal=state.payment_details_form.actually_paid
                        input_type="number".to_string()
                    />
                    <AdminInput
                        placeholder="Order ID".to_string()
                        value_signal=state.payment_details_form.order_id
                    />
                    <AdminInput
                        placeholder="Pay Amount".to_string()
                        value_signal=state.payment_details_form.pay_amount
                        input_type="number".to_string()
                    />
                    <AdminInput
                        placeholder="Pay Currency (USDC)".to_string()
                        value_signal=state.payment_details_form.pay_currency
                    />
                    <AdminInput
                        placeholder="Price Currency (USD)".to_string()
                        value_signal=state.payment_details_form.price_currency
                    />
                </div>
                <div class="mb-4">
                    <AdminInput
                        placeholder="Order Description".to_string()
                        value_signal=state.payment_details_form.order_description
                    />
                </div>

                <button
                    class="px-4 py-2 bg-red-600 text-white rounded-md hover:bg-red-700"
                    on:click={
                        let payment_details_form = state.payment_details_form.clone();
                        let backend_email_input = state.backend_email_input.clone();
                        let backend_app_ref_input = state.backend_app_ref_input.clone();
                        move |_| {
                            let email = backend_email_input.get();
                            let app_ref = backend_app_ref_input.get();
                            let form = &payment_details_form;

                        if !email.is_empty() && !app_ref.is_empty() {
                            let payment_details = PaymentDetails {
                                payment_status: if form.payment_status.get() == "finished" {
                                    BackendPaymentStatus::Paid(form.payment_status.get())
                                } else {
                                    BackendPaymentStatus::Unpaid(Some(form.payment_status.get()))
                                },
                                booking_id: BackendBookingId { email: email.clone(), app_reference: app_ref.clone() },
                                payment_api_response: BePaymentApiResponse {
                                    payment_status: form.payment_status.get(),
                                    payment_id: form.payment_id.get().parse().unwrap_or(0),
                                    payment_id_v2: form.payment_id.get().to_string(),
                                    provider: form.provider.get(),
                                    created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                                    updated_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                                    actually_paid: form.actually_paid.get().parse().unwrap_or(0.0),
                                    invoice_id: 0,
                                    order_description: form.order_description.get(),
                                    pay_amount: form.pay_amount.get().parse().unwrap_or(0.0),
                                    pay_currency: form.pay_currency.get(),
                                    price_amount: 0,
                                    purchase_id: 0,
                                    order_id: form.order_id.get(),
                                    price_currency: form.price_currency.get(),
                                },
                            };

                            update_payment_action.dispatch((email, app_ref, payment_details));
                        }
                    }
                    }
                >
                    "Update Payment Details"
                </button>

                // Display update status
                <div class="mt-4">
                    <p class="text-sm font-medium">{move || state.update_status.get()}</p>
                </div>
            </div>
        </div>
    }
}
