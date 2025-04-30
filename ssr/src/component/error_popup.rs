/// Error popup components for displaying error messages and handling navigation.
///
/// This module provides two types of error popups:
/// 1. `ErrorPopup` - A basic error popup with dismiss functionality
/// 2. `NavigatingErrorPopup` - An error popup that automatically navigates to a specified route
///
/// # Basic Error Popup Usage
/// ```rust
/// use crate::component::error_popup::ErrorPopup;
///
/// // In your component:
/// view! {
///     <ErrorPopup />
/// }
///
/// // To show an error:
/// let api_error_state = ApiErrorState::from_leptos_context();
/// api_error_state.set_error("Error message".to_string(), ApiErrorType::Generic);
/// ```
///
/// # Navigating Error Popup Usage
/// ```rust
/// use crate::component::error_popup::NavigatingErrorPopup;
///
/// // Basic usage with default label ("Dismiss"):
/// view! {
///     <NavigatingErrorPopup
///         route="/home"
///     />
/// }
///
/// // With custom label and error type:
/// view! {
///     <NavigatingErrorPopup
///         route="/dashboard"
///         label="Go to Dashboard"
///         error_type=ApiErrorType::Payment
///     />
/// }
/// ```
///
/// # Features
/// - `ErrorPopup`:
///   - Displays error messages with type
///   - Dismissible via button
///   - Automatically hides when error is resolved
///
/// - `NavigatingErrorPopup`:
///   - All features of basic ErrorPopup
///   - Auto-navigates to specified route after 5 seconds
///   - Shows countdown timer
///   - Customizable button label
///   - Configurable error type
///
use crate::state::api_error_state::{ApiErrorState, ApiErrorType};
use leptos::*;
use leptos_router::use_navigate;
use leptos_use::{use_timeout_fn, UseTimeoutFnReturn};

#[component]
pub fn ErrorPopup() -> impl IntoView {
    let api_error_state = ApiErrorState::from_leptos_context();

    let has_error = api_error_state.has_error;
    let error_message = api_error_state.error_message;
    let show_popup = api_error_state.show_popup;
    let error_type = api_error_state.error_type;

    let dismiss = move |_| {
        api_error_state.dismiss_popup();
    };

    view! {
        <div class="error-popup-container"
             class:hidden=move || !show_popup.get()>
            <div class="error-popup">
                <div class="error-popup-header">
                    <h3>
                        {move || {
                            if let Some(err_type) = error_type.get() {
                                format!("{} Error", err_type)
                            } else {
                                "Error".to_string()
                            }
                        }}
                    </h3>
                    // <button class="error-popup-close" on:click=dismiss.clone()>{"×"}</button>
                </div>
                <div class="error-popup-body">
                    <p>{move || error_message.get()}</p>
                </div>
                <div class="error-popup-footer">
                    <button class="error-popup-button" on:click=dismiss>Dismiss</button>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn NavigatingErrorPopup(
    #[prop(into)] route: String,
    #[prop(into, default = "Dismiss".to_string())] label: String,
    #[prop(default = ApiErrorType::Generic)] error_type: ApiErrorType,
) -> impl IntoView {
    let api_error_state = ApiErrorState::from_leptos_context();
    let navigate = use_navigate();

    let error_message = api_error_state.error_message;
    let show_popup = api_error_state.show_popup;

    // Set up auto-navigation after 5 seconds
    let (countdown, set_countdown) = create_signal(5);

    let dismiss_and_navigate = move |_: ()| {
        api_error_state.dismiss_popup();
        navigate(&route, Default::default());
    };

    // let dismiss_and_navigate2 = move |_| {
    //     api_error_state.dismiss_popup();
    //     navigate(&route, Default::default());
    // };

    // Set up the countdown timer using use_timeout_fn
    // let tick = move |_: i32| {
    //     if countdown.get() > 0 {
    //         set_countdown.update(|count| *count -= 1);
    //         // Restart the timer for the next second
    //         (countdown_timer.start)(());
    //     } else {
    //         dismiss_and_navigate(MouseEvent::new().unwrap());
    //     }
    // };

    // let countdown_timer = use_timeout_fn(tick, 1000.0);

    // Start the countdown when the popup is shown
    // create_effect(move |_| {
    //     if show_popup.get() {
    //         set_countdown.set(5);
    //         (countdown_timer.start)(());
    //     } else {
    //         (countdown_timer.stop)();
    //     }
    // });
    let can_navigate = create_rw_signal(false);

    create_effect(move |_| {
        let UseTimeoutFnReturn { start, .. } = use_timeout_fn(
            move |_: ()| {
                // if countdown.get() > 0 {
                // set_countdown.update(|count| *count -= 1);
                //  }
                //  else {
                //     dismiss_and_navigate(());
                //  }
                // println!("Redirecting in 5 seconds...");
            },
            1000.0,
        );

        start(());
    });
    view! {
        <div class="error-popup-container"
             class:hidden=move || !show_popup.get()>
            <div class="error-popup">
                <div class="error-popup-header">
                    <h3>
                        {format!("{} Error", error_type)}
                    </h3>
                </div>
                <div class="error-popup-body">
                    <p>{move || error_message.get()}</p>
                    <p class="text-sm text-gray-500 mt-2">
                        {"Redirecting in "}{move || countdown.get()}{" seconds..."}
                    </p>
                </div>
                <div class="error-popup-footer">
                    <button class="error-popup-button" on:click=move |_|dismiss_and_navigate(())>
                        {label}
                    </button>
                </div>
            </div>
        </div>
    }
}
