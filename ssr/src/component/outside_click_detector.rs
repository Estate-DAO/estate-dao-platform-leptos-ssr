// use crate::log;
// use leptos::*;
// use wasm_bindgen::{prelude::*, JsCast};
// use web_sys::{Element, Event, Node};

// #[component]
// pub fn OutsideClickDetector(
//     #[prop(into)] on_outside_click: Callback<()>,
//     #[prop(optional)] debug: bool,
//     #[prop(optional, into)] exclude_selectors: Option<Vec<String>>,
//     children: Children,
// ) -> impl IntoView {
//     let div_ref = create_node_ref::<html::Div>();

//     create_effect(move |_| {
//         let div = div_ref.get_untracked();
//         if let Some(div) = div {
//             let div_clone = div.clone();
//             let callback = on_outside_click.clone();
//             let exclude_selectors = exclude_selectors.clone();

//             // Create the event handler
//             let handler = Closure::wrap(Box::new(move |event: Event| {
//                 log!("[outside_click_detector.rs] Event handler triggered");
//                 let target = event.target();

//                 let target_element = target.and_then(|t| t.dyn_into::<web_sys::Element>().ok());

//                 // First check if the click was on an excluded element
//                 if let Some(element) = &target_element {
//                     if let Some(selectors) = &exclude_selectors {
//                         for selector in selectors {
//                             // Check if the element or any of its parents match the selector
//                             let mut current_element: Option<Element> = Some(element.clone());
//                             while let Some(el) = current_element {
//                                 // Check if element matches the selector (like a class name)
//                                 if selector.starts_with(".")
//                                     && el.class_list().contains(&selector[1..])
//                                 {
//                                     log!("[outside_click_detector.rs] Click on excluded element ({}), ignoring", selector);
//                                     return;
//                                 }
//                                 // Check for element ID
//                                 else if selector.starts_with("#") && el.id() == selector[1..] {
//                                     log!("[outside_click_detector.rs] Click on excluded element by ID ({}), ignoring", selector);
//                                     return;
//                                 }
//                                 // Support for data attributes
//                                 else if selector.starts_with("[data-")
//                                     && el.has_attribute(&selector[1..selector.len() - 1])
//                                 {
//                                     log!("[outside_click_detector.rs] Click on excluded element by attribute ({}), ignoring", selector);
//                                     return;
//                                 }

//                                 // Move up to parent
//                                 current_element = el.parent_element();
//                             }
//                         }
//                     }
//                 }

//                 // Now check if the click was outside our div
//                 let target_node = target_element.map(|e| e.dyn_into::<Node>().unwrap());

//                 let outside_click = match target_node {
//                     Some(element) => {
//                         let is_outside = !div_clone.contains(Some(&element));
//                         log!(
//                             "[outside_click_detector.rs] Click detected, is_outside: {}",
//                             is_outside
//                         );
//                         is_outside
//                     }
//                     None => {
//                         log!("[outside_click_detector.rs] Click detected, no element found");
//                         true
//                     }
//                 };

//                 if outside_click {
//                     if debug {
//                         log!("Clicked outside the div");
//                     }
//                     log!("[outside_click_detector.rs] About to call outside click callback");
//                     leptos::Callable::call(&on_outside_click, ());
//                     log!("[outside_click_detector.rs] After calling outside click callback");
//                 } else {
//                     log!("[outside_click_detector.rs] Click was inside the div, ignoring");
//                 }
//             }) as Box<dyn FnMut(Event)>);

//             // Add event listener to document
//             let document = web_sys::window()
//                 .expect("window should exist")
//                 .document()
//                 .expect("document should exist");

//             document
//                 .add_event_listener_with_callback("click", handler.as_ref().unchecked_ref())
//                 .expect("should add event listener");

//             // Store the handler in a closure that will be called during cleanup
//             on_cleanup(move || {
//                 document
//                     .remove_event_listener_with_callback("click", handler.as_ref().unchecked_ref())
//                     .expect("should remove event listener");
//                 // Handler will be dropped here, automatically cleaning up the closure
//             });
//         }
//     });

//     view! {
//         <div _ref=div_ref class="w-full flex justify-center">
//             {children()}
//         </div>
//     }
// }
