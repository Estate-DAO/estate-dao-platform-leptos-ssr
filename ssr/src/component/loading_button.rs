use leptos::prelude::*;
use web_sys::MouseEvent;

/// A flexible button component that supports loading/disabled states
///
/// # Examples
///
/// ```rust
/// // Basic usage with default spinner and loading text
/// <LoadingButton
///     is_loading=is_booking
///     on_click=move |_| block_room_action.dispatch(())
/// >
///     "Book Now"
/// </LoadingButton>
///
/// // With custom loading text and no spinner
/// <LoadingButton
///     is_loading=is_processing
///     loading_text="Processing..."
///     show_spinner=false
///     on_click=move |_| submit_form()
/// >
///     "Submit"
/// </LoadingButton>
///
/// // As a submit button with additional classes
/// <LoadingButton
///     is_loading=is_saving
///     button_type="submit"
///     class="mt-4"
///     on_click=move |_| save_data()
/// >
///     "Save"
/// </LoadingButton>
/// ```
#[component]
pub fn LoadingButton<F>(
    /// Signal indicating whether the button is in a loading state
    is_loading: Signal<bool>,

    /// Click handler for the button
    on_click: F,

    /// Optional custom loading text (defaults to "Loading...")
    #[prop(optional, into)]
    loading_text: Option<String>,

    /// Optional additional classes for the button
    #[prop(optional, into)]
    class: Option<String>,

    /// Type of the button ("button", "submit", "reset")
    #[prop(default = String::from("button"), into)]
    button_type: String,

    /// Whether to show the spinner when loading
    #[prop(default = true)]
    show_spinner: bool,

    /// Optional additional classes for the spinner
    #[prop(optional, into)]
    spinner_class: Option<String>,

    /// Optional additional classes for the content span
    #[prop(optional, into)]
    content_class: Option<String>,

    /// Whether the button is disabled (in addition to loading)
    #[prop(optional, into)]
    disabled: Signal<bool>,

    /// Button content (children)
    children: ChildrenFn,
) -> impl IntoView
where
    F: Fn(MouseEvent) + 'static,
{
    view! {
        <button
            type=button_type
            class={move || {
                let base = "w-full py-3 rounded-full text-white transition-colors duration-150".to_string();
                let additional = class.clone().unwrap_or_default();
                let is_disabled = is_loading.get() || disabled.get();
                if is_disabled {
                    format!("{base} bg-blue-400 cursor-not-allowed opacity-60 {additional}")
                } else {
                    format!("{base} bg-blue-600 hover:bg-blue-800 {additional}")
                }
            }}
            disabled=move || is_loading.get() || disabled.get()
            aria-busy={move || is_loading.get().to_string()}
            aria-disabled={move || (is_loading.get() || disabled.get()).to_string()}
            on:click=on_click
        >
            {move || if is_loading.get() {
                let spinner_class = spinner_class.clone();
                view! {
                    <span class={format!("inline-flex items-center justify-center {}", content_class.clone().unwrap_or_default())}>
                        <Show
                            when=move || show_spinner
                            fallback=|| ()
                        >
                            <span class={format!("animate-spin h-5 w-5 mr-2 border-2 border-white border-t-transparent rounded-full {}", spinner_class.clone().unwrap_or_default())}></span>
                        </Show>
                        {loading_text.clone().unwrap_or_else(|| "Loading...".to_string())}
                    </span>
                }.into_any()
            } else {
                view! {
                    <span class={format!("inline-flex items-center justify-center {}", content_class.clone().unwrap_or_default())}>
                        {children()}
                    </span>
                }.into_any()
            }}
        </button>
    }
}
