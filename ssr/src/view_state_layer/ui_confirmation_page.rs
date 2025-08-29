use crate::canister::backend;
use crate::component::NotificationData;
use crate::utils::backend_integration_helpers::BookingDisplayInfo;
use crate::view_state_layer::GlobalStateForLeptos;
use leptos::*;

/// **Phase 1: State Management for Confirmation Page V1**
///
/// **Purpose**: Manages the complete state for confirmation page including SSE workflow,
/// booking details, and UI state following established patterns from ui_block_room.rs
///
/// **Integration**: Uses BackendIntegrationHelper for domain struct conversion
/// **Pattern**: Follows GlobalStateForLeptos pattern for reactive state management

#[derive(Debug, Clone, Default)]
pub struct ConfirmationPageUIState {
    // **SSE Booking Workflow States** - Similar to SSEBookingStatusUpdates pattern
    pub payment_confirmed: RwSignal<bool>,
    pub booking_processing: RwSignal<bool>,
    pub booking_completed: RwSignal<bool>,

    // **Booking Data from Domain Structs** - Backend integration via helpers
    pub booking_details: RwSignal<Option<backend::Booking>>,
    pub display_info: RwSignal<Option<BookingDisplayInfo>>,

    // **Payment Processing State**
    pub payment_id: RwSignal<Option<String>>,
    pub app_reference: RwSignal<Option<String>>,

    // **UI State Management** - Following established patterns
    pub loading: RwSignal<bool>,
    pub error: RwSignal<Option<String>>,
    pub current_step: RwSignal<u8>, // 1=payment, 2=booking, 3=complete

    // **SSE Connection State**
    pub sse_connected: RwSignal<bool>,
    pub last_notification: RwSignal<Option<NotificationData>>,

    // **Progress Tracking**
    pub step_messages: RwSignal<Vec<String>>, // For progress display
}

impl GlobalStateForLeptos for ConfirmationPageUIState {}

impl ConfirmationPageUIState {
    /// **Initialize state on page load**
    pub fn initialize() {
        let this = Self::get();
        this.payment_confirmed.set(false);
        this.booking_processing.set(false);
        this.booking_completed.set(false);
        this.loading.set(false);
        this.current_step.set(1);
        this.sse_connected.set(false);
        this.step_messages.set(vec![]);
    }

    /// **Set payment ID from URL params**
    pub fn set_payment_id(payment_id: Option<String>) {
        Self::get().payment_id.set(payment_id);
    }

    /// **Set app reference from local storage**
    pub fn set_app_reference(app_reference: Option<String>) {
        Self::get().app_reference.set(app_reference);
    }

    /// **Update state from SSE notification** - Core integration point
    pub fn update_from_notification(notification: &NotificationData) {
        let this = Self::get();
        this.last_notification.set(Some(notification.clone()));

        // Add step message for progress display
        let message = format!(
            "{}: {}",
            notification.event_type,
            notification.step.as_deref().unwrap_or("Processing")
        );

        this.step_messages.update(|messages| {
            messages.push(message);
            // Keep only last 10 messages
            if messages.len() > 10 {
                messages.remove(0);
            }
        });

        // Update workflow states based on notification step
        match (
            notification.step.as_deref(),
            notification.event_type.as_str(),
        ) {
            (Some("GetPaymentStatusFromPaymentProvider"), "OnStepCompleted") => {
                this.payment_confirmed.set(true);
                this.current_step.set(2);
            }
            (Some("MakeBookingFromBookingProvider"), "OnStepCompleted") => {
                this.booking_processing.set(true);
                this.current_step.set(3);
            }
            (Some("MockStep"), "OnStepCompleted")
            | (Some("GetBookingFromBackend"), "OnStepCompleted") => {
                this.booking_completed.set(true);
                this.loading.set(false);

                // Set booking details if provided
                if let Some(booking) = &notification.backend_booking_details {
                    this.booking_details.set(Some(booking.clone()));

                    // Convert to display info using helper
                    let display_info =
                        crate::utils::BackendIntegrationHelper::get_booking_display_info(booking);
                    this.display_info.set(Some(display_info));
                }
            }
            _ => {}
        }
    }

    /// **Set SSE connection status**
    pub fn set_sse_connected(connected: bool) {
        Self::get().sse_connected.set(connected);
    }

    /// **Set loading state**
    pub fn set_loading(loading: bool) {
        Self::get().loading.set(loading);
    }

    /// **Set error state**
    pub fn set_error(error: Option<String>) {
        Self::get().error.set(error);
    }

    /// **Set booking details directly** - For manual updates
    pub fn set_booking_details(booking: Option<backend::Booking>) {
        let this = Self::get();
        this.booking_details.set(booking.clone());

        if let Some(booking) = booking {
            let display_info =
                crate::utils::BackendIntegrationHelper::get_booking_display_info(&booking);
            this.display_info.set(Some(display_info));
        }
    }

    /// **Phase 4: Additional setters for backend workflow integration**

    /// **Set display info directly**
    pub fn set_display_info(display_info: Option<BookingDisplayInfo>) {
        Self::get().display_info.set(display_info);
    }

    /// **Set payment confirmed state**
    pub fn set_payment_confirmed(confirmed: bool) {
        let this = Self::get();
        this.payment_confirmed.set(confirmed);
        if confirmed && this.current_step.get() < 2 {
            this.current_step.set(2);
        }
    }

    /// **Set booking processing state**
    pub fn set_booking_processing(processing: bool) {
        let this = Self::get();
        this.booking_processing.set(processing);
        if processing && this.current_step.get() < 3 {
            this.current_step.set(3);
        }
    }

    /// **Set booking completed state**
    pub fn set_booking_completed(completed: bool) {
        let this = Self::get();
        this.booking_completed.set(completed);
        if completed {
            this.current_step.set(3);
            this.loading.set(false);
        }
    }

    /// **Add step message for progress tracking**
    pub fn add_step_message(message: String) {
        Self::get().step_messages.update(|messages| {
            messages.push(message);
            // Keep only last 10 messages
            if messages.len() > 10 {
                messages.remove(0);
            }
        });
    }

    // **Getters for reactive UI**

    pub fn get_payment_confirmed() -> Signal<bool> {
        Self::get().payment_confirmed.into()
    }

    pub fn get_booking_processing() -> Signal<bool> {
        Self::get().booking_processing.into()
    }

    pub fn get_booking_completed() -> Signal<bool> {
        Self::get().booking_completed.into()
    }

    pub fn get_current_step() -> Signal<u8> {
        Self::get().current_step.into()
    }

    pub fn get_loading() -> Signal<bool> {
        Self::get().loading.into()
    }

    pub fn get_error() -> Signal<Option<String>> {
        Self::get().error.into()
    }

    pub fn get_sse_connected() -> Signal<bool> {
        Self::get().sse_connected.into()
    }

    pub fn get_booking_details() -> Signal<Option<backend::Booking>> {
        Self::get().booking_details.into()
    }

    pub fn get_display_info() -> Signal<Option<BookingDisplayInfo>> {
        Self::get().display_info.into()
    }

    pub fn get_step_messages() -> Signal<Vec<String>> {
        Self::get().step_messages.into()
    }

    /// **Check if all workflow steps are complete**
    pub fn is_workflow_complete() -> Signal<bool> {
        Signal::derive(move || {
            let this = Self::get();
            this.payment_confirmed.get()
                && this.booking_processing.get()
                && this.booking_completed.get()
        })
    }

    /// **Get current step message for display**
    pub fn get_current_step_message() -> Signal<String> {
        Signal::derive(move || {
            let this = Self::get();
            match this.current_step.get() {
                1 => "Confirming your payment".to_string(),
                2 => "Processing your booking".to_string(),
                3 => "Booking confirmed successfully".to_string(),
                _ => "Processing...".to_string(),
            }
        })
    }
}
