use leptos::prelude::*;
use std::fmt::Debug;

/// A component to display the debug representation of a value using a custom formatting function
///
/// # Examples
///
///     // Debug using a closure that returns a formatted string
///     <DebugDisplay
///         label="Destination Context"
///         value=|| format!("{:#?}", search_ctx.destination.get())
///     />
///     
///     // Debug with a static value
///     <DebugDisplay
///         label="Static Data"
///         value=|| format!("{:#?}", some_static_data)
///     />
///     
///     // Complex formatting
///     <DebugDisplay
///         label="Computation Result"
///         value=|| {
///             let value = complex_computation();
///             format!("Computed Value: {:#?}", value)
///         }
///     />
#[component]
pub fn DebugDisplay(
    /// A function that returns the string to be displayed
    #[prop(into)]
    value: Callback<(), String>,

    /// A label to display above the debug output
    #[prop(into)]
    label: String,
) -> impl IntoView {
    cfg_if::cfg_if! {
        // this is a safety valve - just in we forget to remove any debug component, it should not be visible in production
        if #[cfg(feature = "debug_display")] {
            view! {
                // Container with relative positioning to create label effect
                <div class="relative m-2">
                    // Label positioned absolutely over the border
                    <div class="absolute -top-2 left-3 bg-white px-2 text-xs text-indigo-600 font-semibold">
                        {label}
                    </div>

            // The preformatted block for the debug output
            <pre class="border border-dashed border-indigo-400 bg-indigo-50 p-3 text-xs text-indigo-900 overflow-auto rounded shadow-sm font-mono">
                // Call the value function to get the debug string
                {move || value.run(()) }
            </pre>
        </div>
            }
        }

        //  else {
        //     view! {}
        // }
    }
}
