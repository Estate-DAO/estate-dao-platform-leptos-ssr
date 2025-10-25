use leptos::prelude::*;
use web_sys::HtmlElement; // Import HtmlElement for type casting

#[component]
pub fn ShadowOverlay(#[prop(into)] show: RwSignal<bool>, children: ChildrenFn) -> impl IntoView {
    let children = StoredValue::new(children); // Store children for potential re-rendering

    // This outer closure handles the conditional rendering based on `show`
    move || {
        // Only render the overlay if show is true
        if show.get() {
            Some(view! {
                <div
                    // Click handler to close the overlay when the background is clicked
                    on:click=move |ev| {
                        // Get the element that was actually clicked
                        let target = event_target::<HtmlElement>(&ev);
                        // Check if the clicked element *is* the background div itself
                        // This prevents closing when clicking on the children elements
                        if target.class_list().contains("modal-bg") {
                            show.set(false); // Update the signal to hide the overlay
                        }
                    }
                    // Styling for the overlay:
                    // - flex layout to center children
                    // - cursor-pointer indicates the background is clickable
                    // - modal-bg class used by the click handler
                    // - fixed position covering the whole screen
                    // - semi-transparent black background
                    // - high z-index to be on top
                    // - overflow-hidden prevents scrolling the overlay itself
                    class="flex cursor-pointer modal-bg w-full h-full fixed left-0 top-0 bg-black/60 z-[99] justify-center items-center overflow-hidden"
                >
                    // Render the children passed into the component
                    // children.with_value ensures we get the latest children if they were reactive
                    {children.with_value(|children| children())}
                </div>
            })
        } else {
            // Render nothing if show is false
            None
        }
    }
}
