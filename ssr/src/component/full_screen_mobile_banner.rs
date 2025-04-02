use crate::component::modal::Modal;
use leptos::*;

#[component]
pub fn FullScreenBannerForMobileModeNotReady(children: ChildrenFn) -> impl IntoView {
    let show = create_rw_signal(false);

    // We'll use create_effect to handle client-side mobile detection
    // This ensures the server and client render the same initial state
    create_effect(move |_| {
        let width = window().inner_width().unwrap().as_f64().unwrap();
        show.set(width < 768.0);
    });

    let _cleanup = window_event_listener(ev::resize, move |_| {
        let width = window().inner_width().unwrap().as_f64().unwrap();
        show.set(width < 768.0);
    });

    view! {
        <>
            <Modal show=show>
                <div class="flex flex-col items-center justify-center text-center gap-6 py-8">
                    <h2 class="text-2xl font-semibold text-gray-800">
                        Desktop Experience Recommended
                    </h2>
                    <p class="text-gray-600 max-w-md leading-relaxed">
                        "For the best experience browsing our exclusive properties, please use a desktop. A mobile-optimized version is coming soon!"
                    </p>
                    <p class="text-gray-500 italic">"Thank you for your understanding."</p>
                </div>
            </Modal>
            {children()}
        </>
    }
}
