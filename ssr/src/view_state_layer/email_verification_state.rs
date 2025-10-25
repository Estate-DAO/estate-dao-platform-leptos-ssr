use crate::api::auth::auth_state::AuthStateSignal;
use crate::api::client_side_api::{SendOtpResponse, VerifyOtpResponse};
use crate::view_state_layer::GlobalStateForLeptos;
use leptos::prelude::*;

#[derive(Debug, Clone, Default)]
pub struct EmailVerificationState {
    // OTP flow state
    pub otp_sent: RwSignal<bool>,
    pub otp_value: RwSignal<String>,
    pub show_otp_input: RwSignal<bool>,
    pub email_verified: RwSignal<bool>,

    // Loading states
    pub send_otp_loading: RwSignal<bool>,
    pub verify_otp_loading: RwSignal<bool>,

    // Error handling
    pub verification_error: RwSignal<Option<String>>,

    // Timer state for resend functionality
    pub resend_timer: RwSignal<u32>, // seconds remaining
    pub can_resend: RwSignal<bool>,
    pub timer_active: RwSignal<bool>,

    // Modal visibility
    pub show_verification_modal: RwSignal<bool>,
}

impl GlobalStateForLeptos for EmailVerificationState {}

impl EmailVerificationState {
    pub fn from_leptos_context() -> Self {
        Self::get()
    }

    // === OTP Flow Control ===

    pub fn start_verification_flow() {
        let this = Self::from_leptos_context();
        this.show_verification_modal.set(true);
        Self::reset_verification_state();
    }

    pub fn cancel_verification() {
        let this = Self::from_leptos_context();
        this.show_verification_modal.set(false);
        Self::reset_verification_state();
    }

    pub fn complete_verification() {
        let this = Self::from_leptos_context();
        this.email_verified.set(true);
        this.show_verification_modal.set(false);
        Self::reset_verification_state();
    }

    fn reset_verification_state() {
        let this = Self::from_leptos_context();
        this.otp_sent.set(false);
        this.otp_value.set(String::new());
        this.show_otp_input.set(false);
        this.send_otp_loading.set(false);
        this.verify_otp_loading.set(false);
        this.verification_error.set(None);
        Self::stop_resend_timer();
    }

    // === Send OTP Handling ===

    pub fn start_send_otp() {
        let this = Self::from_leptos_context();
        this.send_otp_loading.set(true);
        this.verification_error.set(None);
    }

    pub fn handle_send_otp_success(response: SendOtpResponse) {
        let this = Self::from_leptos_context();
        this.send_otp_loading.set(false);

        if response.success {
            this.otp_sent.set(true);
            this.show_otp_input.set(true);
            this.verification_error.set(None);
            Self::start_resend_timer();
        } else {
            this.verification_error.set(Some(response.message));
        }
    }

    pub fn handle_send_otp_error(error: String) {
        let this = Self::from_leptos_context();
        this.send_otp_loading.set(false);
        this.verification_error.set(Some(error));
    }

    // === Verify OTP Handling ===

    pub fn start_verify_otp() {
        let this = Self::from_leptos_context();
        this.verify_otp_loading.set(true);
        this.verification_error.set(None);
    }

    pub fn handle_verify_otp_success(response: VerifyOtpResponse) -> bool {
        let this = Self::from_leptos_context();
        this.verify_otp_loading.set(false);

        if response.success {
            Self::complete_verification();
            true
        } else {
            this.verification_error.set(Some(response.message));
            false
        }
    }

    pub fn handle_verify_otp_error(error: String) {
        let this = Self::from_leptos_context();
        this.verify_otp_loading.set(false);
        this.verification_error.set(Some(error));
    }

    // === Timer Management ===

    pub fn start_resend_timer() {
        let this = Self::from_leptos_context();
        this.resend_timer.set(60); // 1 minute
        this.can_resend.set(false);
        this.timer_active.set(true);
    }

    pub fn tick_timer() {
        let this = Self::from_leptos_context();
        let current = this.resend_timer.get();
        if current > 0 {
            this.resend_timer.set(current - 1);
        } else {
            Self::stop_resend_timer();
            this.can_resend.set(true);
        }
    }

    pub fn stop_resend_timer() {
        let this = Self::from_leptos_context();
        this.timer_active.set(false);
        this.resend_timer.set(0);
        this.can_resend.set(true);
    }

    // === OTP Input Handling ===

    pub fn update_otp_value(value: String) {
        let this = Self::from_leptos_context();
        // Only allow digits and max 6 characters
        let filtered: String = value
            .chars()
            .filter(|c| c.is_ascii_digit())
            .take(6)
            .collect();
        this.otp_value.set(filtered);
    }

    pub fn clear_otp_input() {
        let this = Self::from_leptos_context();
        this.otp_value.set(String::new());
    }

    // === Resend OTP ===

    pub fn resend_otp() {
        let this = Self::from_leptos_context();
        if this.can_resend.get() && !this.send_otp_loading.get() {
            Self::clear_otp_input();
            Self::start_send_otp();
            // The actual send will be handled by the component using send_otp_action
        }
    }

    // === Getters (for reactive signals) ===

    pub fn get_otp_sent() -> bool {
        let this = Self::from_leptos_context();
        this.otp_sent.get()
    }

    pub fn get_otp_value() -> String {
        let this = Self::from_leptos_context();
        this.otp_value.get()
    }

    pub fn get_show_otp_input() -> bool {
        let this = Self::from_leptos_context();
        this.show_otp_input.get()
    }

    pub fn get_email_verified() -> bool {
        AuthStateSignal::auth_state().get().email.is_some() || {
            let this = Self::from_leptos_context();
            this.email_verified.get()
        }
    }

    pub fn get_send_otp_loading() -> bool {
        let this = Self::from_leptos_context();
        this.send_otp_loading.get()
    }

    pub fn get_verify_otp_loading() -> bool {
        let this = Self::from_leptos_context();
        this.verify_otp_loading.get()
    }

    pub fn get_verification_error() -> Option<String> {
        let this = Self::from_leptos_context();
        this.verification_error.get()
    }

    pub fn get_resend_timer() -> u32 {
        let this = Self::from_leptos_context();
        this.resend_timer.get()
    }

    pub fn get_can_resend() -> bool {
        let this = Self::from_leptos_context();
        this.can_resend.get()
    }

    pub fn get_timer_active() -> bool {
        let this = Self::from_leptos_context();
        this.timer_active.get()
    }

    pub fn get_show_verification_modal() -> bool {
        let this = Self::from_leptos_context();
        this.show_verification_modal.get()
    }

    // === Computed Properties ===

    pub fn is_otp_valid() -> bool {
        let this = Self::from_leptos_context();
        this.otp_value.get().len() == 6
    }

    pub fn can_verify_otp() -> bool {
        let this = Self::from_leptos_context();
        Self::is_otp_valid() && !this.verify_otp_loading.get()
    }

    pub fn format_timer() -> String {
        let this = Self::from_leptos_context();
        let seconds = this.resend_timer.get();
        let minutes = seconds / 60;
        let remaining_seconds = seconds % 60;
        format!("{:01}:{:02}", minutes, remaining_seconds)
    }

    // === Debug Helpers ===

    #[cfg(feature = "debug_log")]
    pub fn log_state() {
        use crate::log;
        let this = Self::from_leptos_context();
        log!("EmailVerificationState Debug:");
        log!("  otp_sent: {}", this.otp_sent.get());
        log!("  show_otp_input: {}", this.show_otp_input.get());
        log!("  email_verified: {}", this.email_verified.get());
        log!("  send_otp_loading: {}", this.send_otp_loading.get());
        log!("  verify_otp_loading: {}", this.verify_otp_loading.get());
        log!("  timer_active: {}", this.timer_active.get());
        log!("  resend_timer: {}", this.resend_timer.get());
        log!("  can_resend: {}", this.can_resend.get());
        log!("  otp_value.len(): {}", this.otp_value.get().len());
    }
}
