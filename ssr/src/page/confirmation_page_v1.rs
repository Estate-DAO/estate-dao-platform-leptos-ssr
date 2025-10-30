// use leptos::*;
// use leptos_icons::Icon;
// use leptos_router::hooks::{use_navigate, use_query_map};
// use std::collections::HashMap;

// use crate::api::client_side_api::{ClientSideApiClient, ConfirmationProcessRequest};
// use crate::component::{Divider, Navbar, NotificationData, NotificationListener, SpinnerGray};
// use crate::log;
// use crate::utils::app_reference::BookingId;
// use crate::view_state_layer::{
//     booking_context_state::BookingContextState, ui_confirmation_page::ConfirmationPageUIState,
//     GlobalStateForLeptos,
// };

// /// **Phase 1: Main Confirmation Page V1 Component**
// ///
// /// **Purpose**: Implements the complete confirmation page with SSE integration,
// /// domain struct patterns, and reactive UI following established codebase patterns
// ///
// /// **Integration Points**:
// /// - SSE workflow via NotificationListener
// /// - Domain struct conversion via BackendIntegrationHelper
// /// - State management via ConfirmationPageUIState
// /// - Responsive design using existing CSS patterns

// #[component]
// pub fn ConfirmationPageV1() -> impl IntoView {
//     let confirmation_state: ConfirmationPageUIState = expect_context();
//     let booking_context: BookingContextState = expect_context();
//     let query_map = use_query_map();

//     // **Phase 1: Initialize state on component mount**
//     Effect::new(move |_| {
//         // Initialize confirmation page state
//         ConfirmationPageUIState::initialize();

//         // Extract payment_id from query params
//         let payment_id =
//             query_map.with(|params| params.get("NP_payment_id").map(|p| p.to_string()));

//         // Read booking data from localStorage in parent component
//         let booking_id = BookingId::extract_booking_id_from_local_storage();
//         let app_reference = BookingId::extract_app_reference_from_local_storage();

//         // Initialize booking context state with data from localStorage
//         BookingContextState::initialize_with_data(booking_id.clone(), app_reference.clone());

//         log!(
//             "ConfirmationPageV1 - payment_id: {:?}, app_reference: {:?}",
//             payment_id,
//             app_reference
//         );

//         ConfirmationPageUIState::set_payment_id(payment_id.clone());
//         ConfirmationPageUIState::set_app_reference(app_reference.clone());

//         // **Phase 4: Trigger backend booking workflow via API if payment_id is present**
//         if let (Some(pay_id), Some(app_ref)) = (payment_id.clone(), app_reference.clone()) {
//             ConfirmationPageUIState::set_loading(true);
//             ConfirmationPageUIState::add_step_message(
//                 "Processing payment confirmation...".to_string(),
//             );

//             // Extract query parameters for API call
//             let query_params = query_map.with(|params| {
//                 params
//                     .0
//                     .iter()
//                     .map(|(k, v)| (k.clone(), v.clone()))
//                     .collect::<HashMap<String, String>>()
//             });

//             // Spawn async task to call confirmation API
//             spawn_local(async move {
//                 let api_client = ClientSideApiClient::new();
//                 // Get order_id and email from centralized booking context
//                 let (order_id, email) = BookingContextState::get_order_details_untracked();

//                 let order_id = order_id.unwrap_or_else(|| app_ref.clone()); // fallback to raw app_ref

//                 let request = ConfirmationProcessRequest {
//                     payment_id: Some(pay_id),
//                     app_reference: Some(order_id), // Send order_id format to server
//                     email,
//                     query_params,
//                 };

//                 match api_client.process_confirmation(request).await {
//                     Some(response) => {
//                         if response.success {
//                             log!(
//                                 "Booking workflow triggered successfully: {}",
//                                 response.message
//                             );
//                             ConfirmationPageUIState::add_step_message(
//                                 "Payment confirmed, processing booking...".to_string(),
//                             );
//                             ConfirmationPageUIState::set_payment_confirmed(true);
//                         } else {
//                             log!("Booking workflow failed: {}", response.message);
//                             ConfirmationPageUIState::set_error(Some(response.message));
//                             ConfirmationPageUIState::set_loading(false);
//                         }
//                     }
//                     None => {
//                         let error_msg = "Failed to communicate with booking service";
//                         log!("{}", error_msg);
//                         ConfirmationPageUIState::set_error(Some(error_msg.to_string()));
//                         ConfirmationPageUIState::set_loading(false);
//                     }
//                 }
//             });
//         } else if payment_id.is_some() && app_reference.is_none() {
//             ConfirmationPageUIState::set_error(Some("Payment ID found but booking reference missing. Please ensure you completed the payment process correctly.".to_string()));
//         }
//     });

//     view! {
//         <section class="flex flex-col items-center min-h-screen w-full bg-gray-50">
//             <div class="w-full">
//                 <Navbar />
//             </div>

//             // **SSE Connection Status Indicator**
//             <SSEConnectionIndicator />

//             // **Phase 2: SSE Integration - NotificationListener with extracted params**
//             <NotificationListenerWrapper />

//             <div class="flex flex-col items-center w-full max-w-4xl mx-auto px-3 sm:px-4 md:px-6 lg:px-8 pt-4 sm:pt-6 md:pt-8">

//                 // **Phase 3: Progress Stepper Component**
//                 <div class="w-full mb-8 sm:mb-12 md:mb-16">
//                     <ProgressStepper />
//                 </div>

//                 // **Phase 3: Main Content Area - Conditional rendering based on workflow state**
//                 // **Phase 4: Enhanced error handling for backend failures**
//                 <Show
//                     when=move || ConfirmationPageUIState::get_error().get().is_some()
//                     fallback=move || view! {
//                         <Show
//                             when=move || ConfirmationPageUIState::is_workflow_complete().get()
//                             fallback=LoadingView
//                         >
//                             <BookingConfirmationDisplay />
//                         </Show>
//                     }
//                 >
//                     <ErrorView />
//                 </Show>
//             </div>
//         </section>
//     }
// }

// /// **Phase 2: NotificationListener Integration**
// /// Wrapper component using centralized booking context for SSE event handling
// #[component]
// fn NotificationListenerWrapper() -> impl IntoView {
//     let confirmation_state: ConfirmationPageUIState = expect_context();

//     view! {
//         {move || {
//             let app_reference = confirmation_state.app_reference.get();
//             let payment_id = confirmation_state.payment_id.get();

//             if let (Some(_app_ref), Some(_pay_id)) = (app_reference, payment_id) {
//                 // Get order_id and email from centralized booking context
//                 let order_id = BookingContextState::get_order_id_untracked().unwrap_or_default();
//                 let email = BookingContextState::get_email_untracked().unwrap_or_default();

//                 log!("NotificationListener setup - order_id: {}, email: {}", order_id, email);

//                 view! {
//                     <NotificationListener
//                         order_id={order_id.clone()}
//                         email={email.clone()}
//                         event_type="nowpayments".to_string()
//                         on_notification={Box::new(move |notification: NotificationData| {
//                             log!("Received notification: {:#?}", notification);
//                             ConfirmationPageUIState::update_from_notification(&notification);
//                             ConfirmationPageUIState::set_sse_connected(true);
//                         })}
//                     />
//                 }.into_any()
//             } else {
//                 view! {
//                     <div class="text-sm text-yellow-600 mb-4">
//                         "No payment ID found - manual confirmation only"
//                     </div>
//                 }.into_any()
//             }
//         }}
//     }
// }

// /// **Phase 3: Progress Stepper Component**
// /// Visual progress indicator showing payment → booking → completion steps
// #[component]
// fn ProgressStepper() -> impl IntoView {
//     let confirmation_state: ConfirmationPageUIState = expect_context();

//     view! {
//         <div class="flex flex-col items-center justify-center w-full py-4 sm:py-6 md:py-8">
//             <div class="flex flex-row items-center justify-center w-full overflow-x-auto px-2 pb-2">
//                 {move || {
//                     let current_step = confirmation_state.current_step.get();
//                     let payment_confirmed = confirmation_state.payment_confirmed.get();
//                     let booking_processing = confirmation_state.booking_processing.get();
//                     let booking_completed = confirmation_state.booking_completed.get();

//                     let steps = vec![
//                         ("Payment Confirmation", 1, payment_confirmed),
//                         ("Booking Processing", 2, booking_processing),
//                         ("Booking Complete", 3, booking_completed),
//                     ];

//                     steps
//                         .into_iter()
//                         .enumerate()
//                         .map(|(index, (label, step_num, is_completed))| {
//                             let is_current = current_step == step_num;
//                             let is_active = is_completed || is_current;

//                             let circle_classes = if is_completed {
//                                 "min-w-[2rem] w-6 h-6 sm:w-8 sm:h-8 rounded-full flex items-center justify-center font-medium transition-colors bg-green-500 text-white"
//                             } else if is_current {
//                                 "min-w-[2rem] w-6 h-6 sm:w-8 sm:h-8 rounded-full flex items-center justify-center font-medium transition-colors bg-blue-500 text-white animate-pulse"
//                             } else {
//                                 "min-w-[2rem] w-6 h-6 sm:w-8 sm:h-8 rounded-full flex items-center justify-center font-medium transition-colors bg-gray-300 text-black"
//                             };

//                             let line_color = if is_completed || (is_current && index > 0) {
//                                 "bg-green-500"
//                             } else if is_current {
//                                 "bg-blue-500"
//                             } else {
//                                 "bg-gray-300"
//                             };

//                             view! {
//                                 <div class="flex items-start shrink-0">
//                                     <div class="flex flex-col items-center">
//                                         <div class=circle_classes>
//                                             {if is_completed {
//                                                 view! {
//                                                     <Icon icon=icondata::AiCheckOutlined />
//                                                 }.into_any()
//                                             } else {
//                                                 view! {
//                                                     <span class="text-xs sm:text-sm">{step_num.to_string()}</span>
//                                                 }.into_any()
//                                             }}
//                                         </div>
//                                         <span class="mt-2 sm:mt-3 md:mt-4 text-[10px] sm:text-xs text-gray-600 text-center break-words max-w-[80px] sm:max-w-[100px] md:max-w-[120px]">
//                                             {label}
//                                         </span>
//                                     </div>
//                                     {if index < 2 {
//                                         view! {
//                                             <div class=format!("h-[1px] w-12 sm:w-16 md:w-24 lg:w-40 transition-colors mt-3 sm:mt-4 mx-1 sm:mx-2 {}", line_color) />
//                                         }.into_any()
//                                     } else {
//                                         view! { <div /> }.into_any()
//                                     }}
//                                 </div>
//                             }
//                         })
//                         .collect::<Vec<_>>()
//                 }}
//             </div>

//             // **Current step message display**
//             <div class="mt-4 text-center">
//                 <p class="text-sm sm:text-base text-gray-700 font-medium">
//                     {move || ConfirmationPageUIState::get_current_step_message().get()}
//                 </p>
//             </div>
//         </div>
//     }
// }

// /// **Phase 3: Loading View** - Displayed while workflow is in progress
// #[component]
// fn LoadingView() -> impl IntoView {
//     let confirmation_state: ConfirmationPageUIState = expect_context();

//     view! {
//         <div class="w-full max-w-full sm:max-w-[450px] md:max-w-[500px] lg:max-w-[600px] border border-blue-100 md:border-blue-200 rounded-xl md:rounded-2xl p-3 sm:p-4 md:p-8 lg:p-10 bg-white shadow-md md:shadow-lg space-y-4 sm:space-y-6 md:space-y-8 mx-auto mt-4 md:mt-10">
//             <div class="text-center">
//                 <SpinnerGray />
//                 <p class="mt-4 text-gray-600">
//                     {move || ConfirmationPageUIState::get_current_step_message().get()}
//                 </p>
//             </div>

//             // **Debug: Show step messages in development**
//             <Show when=move || cfg!(feature = "debug_display")>
//                 <div class="mt-6 p-4 bg-gray-50 rounded-lg">
//                     <h4 class="text-sm font-medium text-gray-700 mb-2">"Step Messages:"</h4>
//                     <div class="space-y-1 text-xs text-gray-600 max-h-32 overflow-y-auto">
//                         {move || {
//                             confirmation_state.step_messages.get()
//                                 .into_iter()
//                                 .map(|msg| view! { <div>{msg}</div> })
//                                 .collect::<Vec<_>>()
//                         }}
//                     </div>
//                 </div>
//             </Show>
//         </div>
//     }
// }

// /// **Phase 4: Error View Component**
// /// Displays comprehensive error information with retry options and support guidance
// #[component]
// fn ErrorView() -> impl IntoView {
//     let confirmation_state: ConfirmationPageUIState = expect_context();

//     view! {
//         <div class="w-full max-w-full sm:max-w-[450px] md:max-w-[500px] lg:max-w-[600px] border border-red-100 md:border-red-200 rounded-xl md:rounded-2xl p-3 sm:p-4 md:p-8 lg:p-10 bg-white shadow-md md:shadow-lg space-y-4 sm:space-y-6 md:space-y-8 mx-auto mt-4 md:mt-10">

//             // **Error Header**
//             <div class="text-center">
//                 <Icon icon=icondata::AiCloseCircleOutlined />
//                 <h2 class="text-lg sm:text-xl md:text-2xl font-semibold text-red-600 mb-2">
//                     "Booking Processing Error"
//                 </h2>
//             </div>

//             <Divider />

//             // **Error Message**
//             <div class="space-y-2">
//                 <h3 class="font-semibold text-gray-800">
//                     "What happened?"
//                 </h3>
//                 <p class="text-sm sm:text-base text-gray-700 bg-red-50 p-3 rounded-lg border border-red-100">
//                     {move || ConfirmationPageUIState::get_error().get().unwrap_or_else(|| "An unknown error occurred".to_string())}
//                 </p>
//             </div>

//             // **Payment Status Information**
//             <div class="space-y-2">
//                 <h3 class="font-semibold text-gray-800">
//                     "Payment Status"
//                 </h3>
//                 <div class="bg-blue-50 p-3 rounded-lg border border-blue-100">
//                     <p class="text-sm text-blue-800 mb-2">
//                         "Your payment may have been processed successfully. Please check:"
//                     </p>
//                     <ul class="text-xs sm:text-sm text-blue-700 space-y-1 ml-4 list-disc">
//                         <li>"Your payment provider account for confirmation"</li>
//                         <li>"Your email for booking confirmation"</li>
//                         <li>"Contact our support team with your payment reference"</li>
//                     </ul>
//                 </div>
//             </div>

//             // **What to do next**
//             <div class="space-y-2">
//                 <h3 class="font-semibold text-gray-800">
//                     "What to do next"
//                 </h3>
//                 <div class="bg-gray-50 p-3 rounded-lg border border-gray-200">
//                     <ol class="text-xs sm:text-sm text-gray-700 space-y-1 ml-4 list-decimal">
//                         <li>"Take a screenshot of this error message"</li>
//                         <li>"Note your payment reference ID if available"</li>
//                         <li>"Contact our support team"</li>
//                         <li>"Do not attempt to make another payment until confirming the status"</li>
//                     </ol>
//                 </div>
//             </div>

//             // **Support Information**
//             <div class="space-y-2 border-t border-gray-200 pt-4">
//                 <h3 class="font-semibold text-gray-800">
//                     "Need Help?"
//                 </h3>
//                 <div class="bg-green-50 p-3 rounded-lg border border-green-200">
//                     <p class="text-sm text-green-800 mb-1 font-medium">
//                         "Our support team is here to help"
//                     </p>
//                     <p class="text-xs text-green-700">
//                         "Please contact support with your payment reference and this error message for quick assistance."
//                     </p>
//                 </div>
//             </div>

//             // **Debug Information** (only in development)
//             <Show when=move || cfg!(feature = "debug_display")>
//                 <div class="border-t border-gray-200 pt-4">
//                     <details class="bg-gray-100 p-3 rounded-lg">
//                         <summary class="text-sm font-medium text-gray-700 cursor-pointer">
//                             "Debug Information"
//                         </summary>
//                         <div class="mt-2 text-xs text-gray-600 font-mono">
//                             <div>"Payment ID: " {move || confirmation_state.payment_id.get().unwrap_or_else(|| "None".to_string())}</div>
//                             <div>"App Reference: " {move || confirmation_state.app_reference.get().unwrap_or_else(|| "None".to_string())}</div>
//                             <div>"Current Step: " {move || confirmation_state.current_step.get().to_string()}</div>
//                         </div>
//                     </details>
//                 </div>
//             </Show>
//         </div>
//     }
// }

// /// **Phase 3: Booking Confirmation Display**
// /// Shows completed booking details using domain struct data
// #[component]
// fn BookingConfirmationDisplay() -> impl IntoView {
//     let confirmation_state: ConfirmationPageUIState = expect_context();

//     view! {
//         <div class="w-full max-w-full sm:max-w-[450px] md:max-w-[500px] lg:max-w-[600px] border border-blue-100 md:border-blue-200 rounded-xl md:rounded-2xl p-3 sm:p-4 md:p-8 lg:p-10 bg-white shadow-md md:shadow-lg space-y-4 sm:space-y-6 md:space-y-8 mx-auto mt-4 md:mt-10">

//             // **Success Header**
//             <div class="text-center text-lg sm:text-xl md:text-2xl font-semibold text-green-600">
//                 <Icon icon=icondata::AiCheckCircleOutlined />
//                 "Your Booking has been confirmed!"
//             </div>

//             <Divider />

//             // **Booking Details Display** - Using BackendIntegrationHelper data
//             {move || {
//                 match confirmation_state.display_info.get() {
//                     Some(display_info) => {
//                         view! {
//                             <div class="space-y-4 sm:space-y-6">

//                                 // **Hotel Information**
//                                 <div class="space-y-1">
//                                     <h2 class="text-left text-base sm:text-lg md:text-xl font-semibold">
//                                         {display_info.hotel_name}
//                                     </h2>
//                                     <p class="text-left text-gray-600 text-xs md:text-sm lg:text-base">
//                                         {display_info.hotel_location}
//                                     </p>
//                                 </div>

//                                 // **Booking Reference**
//                                 <div class="space-y-1 sm:space-y-1.5 md:space-y-2">
//                                     <Divider />
//                                     <p class="text-left text-gray-600 text-xs md:text-sm lg:text-base">"Reference ID"</p>
//                                     <p class="font-mono text-xs sm:text-sm md:text-base lg:text-lg break-all">
//                                         {display_info.booking_reference}
//                                     </p>
//                                 </div>

//                                 // **Guest Information**
//                                 <div class="space-y-1 sm:space-y-1.5 md:space-y-2">
//                                     <Divider />
//                                     <h3 class="font-semibold mb-1 sm:mb-2 md:mb-3 text-xs sm:text-sm md:text-base lg:text-lg">
//                                         "Guest Information"
//                                     </h3>
//                                     <div class="space-y-0.5 sm:space-y-1">
//                                         <p class="text-[10px] sm:text-xs md:text-sm lg:text-base font-medium">
//                                             {display_info.user_name}
//                                         </p>
//                                         <p class="text-[10px] sm:text-xs md:text-sm lg:text-base text-gray-600">
//                                             {display_info.user_email}
//                                         </p>
//                                         <p class="text-[10px] sm:text-xs md:text-sm lg:text-base text-gray-600">
//                                             {format!("{} Adult{} • {} Children",
//                                                 display_info.number_of_adults,
//                                                 if display_info.number_of_adults > 1 { "s" } else { "" },
//                                                 display_info.number_of_children
//                                             )}
//                                         </p>
//                                     </div>
//                                 </div>

//                                 // **Payment Information**
//                                 <div class="space-y-1 sm:space-y-1.5 md:space-y-2">
//                                     <Divider />
//                                     <p class="text-left text-gray-600 text-xs md:text-sm lg:text-base">"Amount Paid"</p>
//                                     <p class="text-lg sm:text-xl font-semibold text-green-600">
//                                         {format!("${:.2}", display_info.amount_paid)}
//                                     </p>
//                                 </div>
//                             </div>
//                         }.into_any()
//                     }
//                     None => {
//                         view! {
//                             <div class="text-center text-gray-500">
//                                 <SpinnerGray />
//                                 <p class="mt-2">"Loading booking details..."</p>
//                             </div>
//                         }.into_any()
//                     }
//                 }
//             }}

//             // **Footer Message**
//             <div class="text-center text-[10px] sm:text-xs md:text-sm lg:text-base font-medium text-gray-600 pt-2 border-t border-gray-200">
//                 "Please take a screenshot for your reference"
//             </div>
//         </div>
//     }
// }

// /// **SSE Connection Status Indicator**
// /// Shows a small indicator in the top-right to display SSE connection status
// #[component]
// fn SSEConnectionIndicator() -> impl IntoView {
//     view! {
//         <div class="fixed top-4 right-4 z-50">
//             <div class="flex items-center space-x-2 bg-white rounded-full px-3 py-2 shadow-md border">
//                 <div class="flex items-center space-x-1">
//                     {move || {
//                         let is_connected = ConfirmationPageUIState::get_sse_connected().get();
//                         if is_connected {
//                             view! {
//                                 <div class="w-2 h-2 bg-green-500 rounded-full animate-pulse"></div>
//                                 <span class="text-xs text-green-700 font-medium">"Connected"</span>
//                             }.into_any()
//                         } else {
//                             view! {
//                                 <div class="w-2 h-2 bg-red-500 rounded-full"></div>
//                                 <span class="text-xs text-red-700 font-medium">"Disconnected"</span>
//                             }.into_any()
//                         }
//                     }}
//                 </div>
//             </div>
//         </div>
//     }
// }
