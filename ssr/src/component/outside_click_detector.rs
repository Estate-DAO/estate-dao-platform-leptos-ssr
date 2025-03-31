use crate::log;
use leptos::*;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{Event, Node};

#[component]
pub fn OutsideClickDetector(
    #[prop(into)] on_outside_click: Callback<()>,
    #[prop(optional)] debug: bool,
    children: Children,
) -> impl IntoView {
    let div_ref = create_node_ref::<html::Div>();

    create_effect(move |_| {
        let div = div_ref.get_untracked();
        if let Some(div) = div {
            let div_clone = div.clone();
            let callback = on_outside_click.clone();

            // Create the event handler
            let handler = Closure::wrap(Box::new(move |event: Event| {
                let target = event.target();
                let target_element = target
                    .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
                    .map(|e| e.dyn_into::<Node>().unwrap());

                // Check if the click was outside our div
                let outside_click = match target_element {
                    Some(element) => !div_clone.contains(Some(&element)),
                    None => true,
                };

                if outside_click {
                    if debug {
                        log!("Clicked outside the div");
                    }
                    leptos::Callable::call(&on_outside_click, ());
                }
            }) as Box<dyn FnMut(Event)>);

            // Add event listener to document
            let document = web_sys::window()
                .expect("window should exist")
                .document()
                .expect("document should exist");

            document
                .add_event_listener_with_callback("click", handler.as_ref().unchecked_ref())
                .expect("should add event listener");

            // Store the handler in a closure that will be called during cleanup
            on_cleanup(move || {
                document
                    .remove_event_listener_with_callback("click", handler.as_ref().unchecked_ref())
                    .expect("should remove event listener");
                // Handler will be dropped here, automatically cleaning up the closure
            });
        }
    });

    view! {
        <div _ref=div_ref class="w-full flex justify-center">
            {children()}
        </div>
    }
}
