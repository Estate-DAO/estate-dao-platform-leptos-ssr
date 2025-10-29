use crate::canister::backend;
use crate::component::NotificationData;
use crate::log;
use crate::utils::backend_integration_helpers::BookingDisplayInfo;
use crate::view_state_layer::GlobalStateForLeptos;
use leptos::*;

#[cfg(feature = "ssr")]
use chrono::{DateTime, Utc};

/// **Simplified Confirmation Page State V2**
///
/// **Purpose**: Clean state management with integrated step tracking,
/// eliminating redundant states and following block_room_v1.rs patterns
///
/// **Key Features**:
/// - Integrated step progression with automatic completion tracking
/// - Real-time progress details from SSE notifications
/// - Centralized error handling
/// - Single source of truth for workflow status

#[derive(Debug, Clone, Default, PartialEq)]
pub enum ConfirmationStep {
    #[default]
    Initializing,
    PaymentConfirmation,
    BookingProcessing,
    EmailSending,
    Completed,
}

impl ConfirmationStep {
    pub fn get_display_info(&self) -> (&'static str, &'static str) {
        match self {
            ConfirmationStep::Initializing => {
                ("Initializing", "Setting up your booking confirmation")
            }
            ConfirmationStep::PaymentConfirmation => ("Payment", "Confirming your payment status"),
            ConfirmationStep::BookingProcessing => ("Booking", "Creating your hotel reservation"),
            ConfirmationStep::EmailSending => ("Email", "Sending confirmation details"),
            ConfirmationStep::Completed => ("Complete", "Your booking is confirmed!"),
        }
    }

    pub fn get_step_number(&self) -> u8 {
        match self {
            ConfirmationStep::Initializing => 0,
            ConfirmationStep::PaymentConfirmation => 1,
            ConfirmationStep::BookingProcessing => 2,
            ConfirmationStep::EmailSending => 3,
            ConfirmationStep::Completed => 4,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StepProgress {
    pub current_message: String,
    pub step_details: Vec<String>,
    pub error_details: Option<String>,
    pub started_at: Option<String>, // ISO timestamp
}

impl Default for StepProgress {
    fn default() -> Self {
        Self {
            current_message: "Initializing...".to_string(),
            step_details: vec![],
            error_details: None,
            started_at: None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ConfirmationPageState {
    // Workflow steps with integrated tracking
    pub current_step: RwSignal<ConfirmationStep>,
    pub completed_steps: RwSignal<Vec<ConfirmationStep>>,

    // Step progress details
    pub step_progress: RwSignal<StepProgress>,

    // API data
    pub payment_id: RwSignal<Option<String>>,
    pub app_reference: RwSignal<Option<String>>,
    pub booking_details: RwSignal<Option<backend::Booking>>,
    pub display_info: RwSignal<Option<BookingDisplayInfo>>,

    // UI state
    pub loading: RwSignal<bool>,
    pub error: RwSignal<Option<String>>,
}

impl GlobalStateForLeptos for ConfirmationPageState {}

impl ConfirmationPageState {
    /// **Initialize state on page load**
    pub fn initialize() {
        let this = Self::get();
        this.current_step.set(ConfirmationStep::Initializing);
        this.completed_steps.set(vec![]);
        this.step_progress.set(StepProgress::default());
        this.loading.set(false);
        this.error.set(None);
        this.booking_details.set(None);
        this.display_info.set(None);
    }

    /// **Advanced to next step with automatic completion tracking**
    pub fn advance_to_step(step: ConfirmationStep, message: String) {
        let this = Self::get();
        let current_step = this.current_step.get_untracked();

        // Mark previous step as completed if advancing
        if step.get_step_number() > current_step.get_step_number() {
            this.completed_steps.update(|completed| {
                if !completed.contains(&current_step) {
                    completed.push(current_step);
                }
            });
        }

        // Update current step and progress
        this.current_step.set(step);
        this.step_progress.update(|progress| {
            progress.current_message = message;
            progress.step_details.clear();
            progress.error_details = None;
            // Simple timestamp for now - could be enhanced with proper date handling
            #[cfg(not(feature = "ssr"))]
            {
                progress.started_at = Some(format!("{}", web_sys::js_sys::Date::now()));
            }
            #[cfg(feature = "ssr")]
            {
                progress.started_at = Some(chrono::Utc::now().to_rfc3339());
            }
        });

        // Clear any previous errors when advancing
        this.error.set(None);
    }

    /// **Add detailed progress information for current step**
    pub fn add_step_detail(detail: String) {
        Self::get().step_progress.update(|progress| {
            progress.step_details.push(detail);
            // Keep last 5 details per step to avoid memory issues
            if progress.step_details.len() > 5 {
                progress.step_details.remove(0);
            }
        });
    }

    /// **Update step message without changing step**
    pub fn update_step_message(message: String) {
        Self::get().step_progress.update(|progress| {
            progress.current_message = message;
        });
    }

    /// **Handle SSE notification and update state accordingly**
    pub fn update_from_sse_notification(notification: &NotificationData) {
        // Add notification as step detail (NotificationData doesn't have message field)
        Self::add_step_detail(format!("{}: Processing step", notification.event_type));

        // Handle step transitions based on pipeline steps
        match (
            notification.step.as_deref(),
            notification.event_type.as_str(),
        ) {
            (Some("GetPaymentStatusFromPaymentProvider"), "OnStepCompleted") => {
                Self::advance_to_step(
                    ConfirmationStep::PaymentConfirmation,
                    "Payment confirmed successfully".to_string(),
                );
            }
            (Some("MakeBookingFromBookingProvider"), "OnStepCompleted") => {
                Self::advance_to_step(
                    ConfirmationStep::BookingProcessing,
                    "Hotel reservation created successfully".to_string(),
                );
            }
            (Some("GetBookingFromBackend"), "OnStepCompleted") => {
                Self::update_step_message("Loading booking details...".to_string());

                // Set booking details if provided and not already loaded from API
                if let Some(booking) = &notification.backend_booking_details {
                    let current_booking = Self::get().booking_details.get_untracked();
                    if current_booking.is_none() {
                        Self::set_booking_details(Some(booking.clone()));
                        Self::add_step_detail(
                            "Booking details retrieved via real-time updates".to_string(),
                        );

                        // Send GA booking completion event when booking details are first retrieved
                    } else {
                        Self::add_step_detail(
                            "Booking details confirmed via real-time updates".to_string(),
                        );
                    }
                }
            }
            (Some("SendEmailAfterSuccessfullBooking"), "OnStepCompleted") => {
                let current_step = Self::get().current_step.get_untracked();
                if current_step != ConfirmationStep::Completed {
                    Self::advance_to_step(
                        ConfirmationStep::EmailSending,
                        "Confirmation booking email is being sent!".to_string(),
                    );
                } else {
                    Self::add_step_detail("Confirmation email sent successfully".to_string());
                }
                Self::send_booking_completed_ga_event();

                Self::get().loading.set(false);
            }
            (Some("MockStep"), "OnStepCompleted") => {
                Self::advance_to_step(
                    ConfirmationStep::Completed,
                    "Your booking is confirmed!".to_string(),
                );
                Self::get().loading.set(false);

                // Send GA booking completion event for mock step completion
                Self::send_booking_completed_ga_event();
            }
            (Some(step), "OnStepFailed") => {
                Self::set_step_error(format!("Failed at step: {}", step));
            }
            _ => {
                // For other notifications, just add as step detail
                if let Some(step) = &notification.step {
                    Self::add_step_detail(format!("Processing {}", step));
                }
            }
        }
    }

    /// **Set error for current step**
    pub fn set_step_error(error: String) {
        let this = Self::get();
        this.error.set(Some(error.clone()));
        this.loading.set(false);

        this.step_progress.update(|progress| {
            progress.error_details = Some(error);
        });
    }

    /// **Batch update for successful API call - following block_room_v1 pattern**
    pub fn batch_update_on_success(booking: Option<backend::Booking>) {
        let this = Self::get();
        this.loading.set(false);
        this.error.set(None);

        if let Some(booking) = booking {
            Self::set_booking_details(Some(booking));
        }

        // Advance to payment confirmation if we haven't started yet
        if this.current_step.get_untracked() == ConfirmationStep::Initializing {
            Self::advance_to_step(
                ConfirmationStep::PaymentConfirmation,
                "Processing payment confirmation...".to_string(),
            );
        }
    }

    /// **Batch update for API error - following block_room_v1 pattern**
    pub fn batch_update_on_error(error: String) {
        let this = Self::get();
        this.loading.set(false);
        this.error.set(Some(error.clone()));

        this.step_progress.update(|progress| {
            progress.error_details = Some(error);
        });
    }

    // **Setters**

    pub fn set_payment_id(payment_id: Option<String>) {
        Self::get().payment_id.set(payment_id);
    }

    pub fn set_app_reference(app_reference: Option<String>) {
        Self::get().app_reference.set(app_reference);
    }

    pub fn set_loading(loading: bool) {
        Self::get().loading.set(loading);
    }

    pub fn set_error(error: Option<String>) {
        Self::get().error.set(error);
    }

    pub fn set_booking_details(booking: Option<backend::Booking>) {
        let this = Self::get();
        this.booking_details.set(booking.clone());

        // Convert to display info using helper
        if let Some(booking) = booking {
            let display_info =
                crate::utils::BackendIntegrationHelper::get_booking_display_info(&booking);
            this.display_info.set(Some(display_info));
        }
    }

    /// **Set booking details from API response JSON data**
    pub fn set_booking_details_from_json(booking_json: Option<serde_json::Value>) {
        if let Some(json_value) = booking_json {
            match serde_json::from_value::<backend::Booking>(json_value) {
                Ok(booking) => {
                    log!("Successfully parsed booking data from API response");
                    Self::set_booking_details(Some(booking));
                    Self::add_step_detail("Booking data received from API".to_string());

                    // Send GA booking completion event when booking details are loaded
                    // Self::send_booking_completed_ga_event();
                }
                Err(e) => {
                    log!("Failed to parse booking data from JSON: {}", e);
                    Self::add_step_detail("Received booking data but failed to parse".to_string());
                }
            }
        }
    }

    /// **Send GA4 booking completion event**
    /// This sends a Google Analytics event when booking is successfully completed
    #[cfg(feature = "ga4")]
    pub fn send_booking_completed_ga_event() {
        use crate::event_streaming::send_booking_completed_event;
        use crate::view_state_layer::cookie_booking_context_state::CookieBookingContextState;

        // Get booking details and user context
        let display_info = Self::get().display_info.get_untracked();
        let app_reference = Self::get().app_reference.get_untracked();
        let user_email = CookieBookingContextState::get_email_untracked();

        if let Some(info) = display_info {
            let booking_ref = app_reference.unwrap_or_else(|| info.booking_reference.clone());

            send_booking_completed_event(
                booking_ref,
                Some(info.hotel_name),
                user_email,
                Some(info.number_of_nights as u32),
                Some(info.number_of_adults as u32),
            );
        } else if let Some(booking_ref) = app_reference {
            // Fallback: send minimal event with just booking reference
            send_booking_completed_event(booking_ref, None, user_email, None, None);
        }
    }

    #[cfg(not(feature = "ga4"))]
    pub fn send_booking_completed_ga_event() {
        // No-op when GA4 feature is not enabled
        log!("GA4 feature not enabled - skipping booking completion event");
    }

    // **Getters for reactive UI**

    pub fn get_current_step() -> Signal<ConfirmationStep> {
        Self::get().current_step.into()
    }

    pub fn get_completed_steps() -> Signal<Vec<ConfirmationStep>> {
        Self::get().completed_steps.into()
    }

    pub fn get_step_progress() -> Signal<StepProgress> {
        Self::get().step_progress.into()
    }

    pub fn get_payment_id() -> Signal<Option<String>> {
        Self::get().payment_id.into()
    }

    pub fn get_app_reference() -> Signal<Option<String>> {
        Self::get().app_reference.into()
    }

    pub fn get_loading() -> Signal<bool> {
        Self::get().loading.into()
    }

    pub fn get_error() -> Signal<Option<String>> {
        Self::get().error.into()
    }

    pub fn get_booking_details() -> Signal<Option<backend::Booking>> {
        Self::get().booking_details.into()
    }

    pub fn get_display_info() -> Signal<Option<BookingDisplayInfo>> {
        Self::get().display_info.into()
    }

    /// **Check if workflow is complete**
    pub fn is_workflow_complete() -> Signal<bool> {
        Signal::derive(move || {
            let current_step = Self::get().current_step.get();
            let display_info = Self::get().display_info.get();

            // Workflow is complete if we reached the final step AND have booking data
            current_step == ConfirmationStep::Completed && display_info.is_some()
        })
    }

    /// **Check if there's an error**
    pub fn has_error() -> Signal<bool> {
        Signal::derive(move || Self::get().error.get().is_some())
    }

    /// **Get current step display message**
    pub fn get_current_step_message() -> Signal<String> {
        Signal::derive(move || Self::get().step_progress.get().current_message)
    }

    /// **Check if step is completed**
    pub fn is_step_completed(step: ConfirmationStep) -> Signal<bool> {
        Signal::derive(move || {
            let completed_steps = Self::get().completed_steps.get();
            completed_steps.contains(&step) || Self::get().current_step.get() == step
        })
    }

    /// **Check if step is current**
    pub fn is_step_current(step: ConfirmationStep) -> Signal<bool> {
        Signal::derive(move || Self::get().current_step.get() == step)
    }
}
