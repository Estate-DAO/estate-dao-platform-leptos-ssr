use crate::{app::AppRoutes, component::overlay::ShadowOverlay}; // Or super::overlay::ShadowOverlay if in same dir
use leptos::*;
use leptos_icons::Icon; // Specific imports are slightly clearer
use leptos_router::use_navigate;

#[component]
pub fn Modal(#[prop(into)] show: RwSignal<bool>, children: ChildrenFn) -> impl IntoView {
    view! {
        // Use the ShadowOverlay, passing the show signal
        <ShadowOverlay show=show>
            <div class="mx-4 py-4 px-8 max-w-lg max-h-[90vh] items-center cursor-auto flex-col flex justify-around bg-white rounded-lg shadow-lg overflow-y-auto">
                // Header section with the close button
                <div class="flex w-full justify-end items-center p-2 sticky top-0 bg-white z-10"> // Added sticky positioning for close button
                    <button
                        on:click=move |_| show.set(false) // Close button updates the show signal
                        class="text-gray-700 hover:text-red-600 p-1 text-lg md:text-xl rounded-full transition-colors"
                        aria-label="Close modal" // Added aria-label for accessibility
                    >
                        <Icon icon=icondata::ChCross/>
                    </button>
                </div>
                // The main content area provided by the user of the Modal component
                <div class="py-4 w-full">{children()}</div>
            </div>
        </ShadowOverlay>
    }
}

#[component]
pub fn ErrorModal(
    #[prop(into)] show: RwSignal<bool>,
    #[prop(into)] error_title: MaybeSignal<String>, // MaybeSignal<String> is NOT Copy
    #[prop(into)] error_message: MaybeSignal<String>, // MaybeSignal<String> is NOT Copy
    #[prop(optional)] on_retry: Option<Callback<()>>, // Callback is Clone
) -> impl IntoView {
    let error_title = store_value(error_title);
    let error_message = store_value(error_message);

    view! {
        // Use the generic Modal component
        <Modal show=show> // show is RwSignal (Copy)
            <div class="flex flex-col items-center gap-4 p-4">
                // Use the explicit reactive getter pattern here:
                <h2 class="text-xl font-bold text-red-600">{move || error_title.get_value()}</h2>
                // And here:
                <p class="text-gray-700 text-center">{move || error_message.get_value()}</p>
                <div class="flex gap-4 mt-4">
                    // Conditionally render the "Try Again" button
                    {move || { // This inner closure captures `on_retry` (Clone)
                        on_retry.map(|retry| { // `retry` (Callback) is captured (Clone)
                            view! {
                                <button
                                    on:click=move |_| { // This closure captures `retry` (Clone)
                                        // retry.call(());
                                        // Optionally close modal on retry
                                        show.set(false);
                                    }
                                    class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700"
                                >
                                    "Try Again"
                                </button>
                            }
                        })
                    }}
                    <button
                        on:click=move |_| show.set(false) // This closure captures `show` (Copy)
                        class="px-4 py-2 bg-gray-300 text-gray-700 rounded-md hover:bg-gray-400"
                    >
                        "Close"
                    </button>
                </div>
            </div>
        </Modal>
    }
}

// New component with a "Go Home" button
#[component]
pub fn ErrorModalWithHome(
    #[prop(into)] show: RwSignal<bool>,
    #[prop(into)] error_title: MaybeSignal<String>,
    #[prop(into)] error_message: MaybeSignal<String>,
    // #[prop(optional)] on_retry: Option<Callback<()>>,
    /// The route to navigate to when the "Go Home" button is clicked (e.g., "/")
    #[prop(into, default = MaybeSignal::Static("/".to_string()))]
    home_route: MaybeSignal<String>,
    // #[prop(into, default = AppRoutes::Root)] home_route: AppRoutes,
) -> impl IntoView {
    // Hook to get the navigation function
    // This MUST be called unconditionally at the top level of the component
    let navigate = use_navigate();

    // Store non-Copy prop values to ensure they live long enough and can be accessed
    // (Following the pattern from your reference ErrorModal)
    let error_title = store_value(error_title);
    let error_message = store_value(error_message);

    let route_str = home_route.get();

    view! {
        <Modal show=show>
            <div class="flex flex-col items-center gap-4 p-4">
                // Display title and message using stored values
                // .get_value() retrieves the MaybeSignal<String> from StoredValue
                // It implicitly calls .get() when rendered in the view
                <h2 class="text-xl font-bold text-red-600">{move || error_title.get_value()}</h2>
                <p class="text-gray-700 text-center">{move || error_message.get_value()}</p>

                // Container for action buttons
                // Added flex-wrap and justify-center for better layout with multiple buttons
                <div class="flex flex-wrap justify-center gap-4 mt-4">

                    // // Optional "Try Again" button (logic copied from ErrorModal)
                    // {move || {
                    //     on_retry.map(|retry| { // `retry` is Callback (Clone)
                    //         let retry = store_value(retry); // Store if needed inside further nested closures
                    //         view! {
                    //             <button
                    //                 on:click=move |_| {
                    //                     retry.get_value().call(()); // Call the stored callback
                    //                     // Optionally close modal on retry:
                    //                     // show.set(false);
                    //                 }
                    //                 class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700"
                    //             >
                    //                 "Try Again"
                    //             </button>
                    //         }
                    //     })
                    // }}

                    // NEW: Use the <A> component for navigation
                    <a
                        // href takes the route. Pass the stored MaybeSignal directly.
                        // <A> knows how to handle Signals/MaybeSignals.
                        href=route_str.clone()
                        // Apply the same styling as the button
                        class="px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700"
                        // Add an on:click handler to close the modal when the link is clicked.
                        // The navigation will still happen via the href.
                        on:click=move |_| show.set(false)
                    >
                        "Go Home"
                    </a>

                    // "Close" button (logic copied from ErrorModal)
                    <button
                        on:click=move |_| show.set(false) // show is RwSignal (Copy)
                        class="px-4 py-2 bg-gray-300 text-gray-700 rounded-md hover:bg-gray-400"
                    >
                        "Close"
                    </button>
                </div>
            </div>
        </Modal>
    }
}
