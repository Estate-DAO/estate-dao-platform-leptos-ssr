use leptos::*;
// Import the state and the specific modal component
use crate::component::modal::ErrorModalWithHome;
use crate::log;
use crate::state::api_error_state::{ApiErrorState, ApiErrorType};

/// A component that listens to the global ApiErrorState from context
/// and displays an ErrorModalWithHome when an error is set to be shown.
#[component]
pub fn GlobalApiErrorPopup(
    /// The route to navigate to when the "Go Home" button is clicked.
    /// Defaults to "/".
    #[prop(into, default = MaybeSignal::Static("/".to_string()))]
    home_route: MaybeSignal<String>,
    /// Optional: A callback to execute when the user clicks "Try Again".
    /// Note: The global popup might not know the context for a retry.
    /// Consider if retry logic should be handled differently.
    #[prop(optional)]
    on_retry: Option<Callback<()>>,
) -> impl IntoView {
    // --- 1. Get the Global State ---
    // Attempt to retrieve the ApiErrorState from the Leptos context.
    // This expects the state to have been provided by a parent component.
    let error_state = match use_context::<ApiErrorState>() {
        Some(state) => state,
        None => {
            // Log an error if the context is missing in development builds
            // #[cfg(debug_assertions)]
            log!("GlobalApiErrorPopup: ApiErrorState context not found! Make sure it's provided higher up in the component tree.");
            // Render nothing if the context is missing
            return view! { <></> }.into_view();
        }
    };

    // --- 2. Prepare Props for ErrorModalWithHome ---

    // The 'show' signal for the modal is directly the 'show_popup' signal from the state.
    // The modal component itself will handle setting this to `false` when closed.
    let show_signal = error_state.show_popup;

    // Create a derived signal for the error title based on the error type.
    let title_signal = Signal::derive(move || {
        error_state
            .error_type
            .get()
            .map(|t| format!("{} Error", t)) // Use the Display impl of ApiErrorType
            .unwrap_or_else(|| "Error Occurred".to_string()) // Default title if type is None
    });

    // Create a derived signal for the error message.
    let message_signal = Signal::derive(move || error_state.error_message.get());

    // --- 3. Render the Modal ---
    // Render the ErrorModalWithHome, passing the signals derived from the global state.
    view! {
        <ErrorModalWithHome
            show=show_signal          // Controlled by global state's show_popup
            error_title=title_signal  // Derived from global state's error_type
            error_message=message_signal // Derived from global state's error_message
            home_route=home_route     // Passed through from this component's props
            // on_retry=on_retry         // Pass optional retry callback through
        />
    }
}

/*
// --- How to Provide Context and Use ---

// In your main App component or a layout component:
use crate::state::api_error_state::ApiErrorState;
use crate::component::global_error_popup::GlobalApiErrorPopup; // Import the new component

#[component]
fn App() -> impl IntoView {
    // 1. Create an instance of the state.
    //    ApiErrorState::default() provides RwSignals initialized appropriately.
    let global_error_state = ApiErrorState::default();

    // 2. Provide the state into context for descendant components.
    provide_context(global_error_state.clone()); // Clone is cheap (RwSignals are Copy)

    // Example: A button somewhere else in the app that triggers an error
    let trigger_err_action = create_action(move |_: &()| {
        let state = global_error_state.clone(); // Clone state for the action
        async move {
            // Simulate an API call failure
            state.set_error(ApiErrorType::Generic, "Something went wrong!".to_string());
        }
    });


    view! {
        <leptos_router::Router> // Assuming you have router setup
            <main>
                // Your application routes and content
                // <Routes> ... </Routes>

                <button on:click=move |_| trigger_err_action.dispatch(())> "Trigger Global Error" </button>

                // 3. Mount the GlobalApiErrorPopup component.
                //    It should be placed somewhere stable in your layout
                //    so it's always rendered and listening to the context.
                <GlobalApiErrorPopup
                    home_route="/dashboard" // Example: Set the home route
                    // Optional: Define what "Try Again" does in the global context,
                    // maybe it just dismisses the popup?
                    on_retry=Callback::new(move |_| {
                        log!("Global 'Try Again' clicked - dismissing popup.");
                        global_error_state.dismiss_popup();
                    })
                />
            </main>
        </leptos_router::Router>
    }
}

*/
