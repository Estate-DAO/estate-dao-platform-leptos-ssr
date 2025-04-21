use crate::log;
use leptos::ev::MouseEvent;
use leptos::html::{Div, Input};
use leptos::*;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{Event, KeyboardEvent, Node};

#[component]
pub fn LiveSelect<T>(
    #[prop(optional)] options: MaybeSignal<Vec<T>>,
    value: Signal<Option<T>>,
    set_value: Callback<T>,
    label_fn: Callback<T, String>,
    value_fn: Callback<T, String>,
    #[prop(optional)] placeholder: MaybeSignal<String>,
    #[prop(optional)] id: MaybeSignal<String>,
    #[prop(optional)] class: MaybeSignal<String>,
    #[prop(optional)] debug: bool,
) -> impl IntoView
where
    T: Clone + PartialEq + 'static,
{
    // State for the component
    let (search_text, set_search_text) = create_signal(String::new());
    let (is_open, set_is_open) = create_signal(false);
    let (active_index, set_active_index) = create_signal(0);
    let input_ref = create_node_ref::<Input>();
    let dropdown_ref = create_node_ref::<Div>();
    let container_ref = create_node_ref::<Div>();

    // Set up outside click detection
    create_effect(move |_| {
        if is_open.get() {
            let container = container_ref.get_untracked();
            if let Some(container) = container {
                let container_clone = container.clone();
                let set_is_open_clone = set_is_open.clone();

                // Create the event handler for outside clicks
                let handler = Closure::wrap(Box::new(move |event: Event| {
                    let target = event.target();
                    let target_element = target
                        .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
                        .map(|e| e.dyn_into::<Node>().unwrap());

                    // Check if the click was outside our container
                    let outside_click = match target_element {
                        Some(element) => !container_clone.contains(Some(&element)),
                        None => true,
                    };

                    if outside_click {
                        if debug {
                            log!("Clicked outside the LiveSelect");
                        }
                        set_is_open_clone.set(false);
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
                        .remove_event_listener_with_callback(
                            "click",
                            handler.as_ref().unchecked_ref(),
                        )
                        .expect("should remove event listener");
                    // Handler will be dropped here, automatically cleaning up the closure
                });
            }
        }
    });

    // Filtered options based on search text
    let filtered_options = create_memo(move |_| {
        let search = search_text.get().to_lowercase();
        if search.is_empty() {
            return options.get();
        }

        options
            .get()
            .into_iter()
            .filter(|opt| label_fn(opt.clone()).to_lowercase().contains(&search))
            .collect::<Vec<T>>()
    });

    // Handle keyboard navigation with properly set up event listener
    create_effect(move |_| {
        if let Some(input) = input_ref.get_untracked() {
            let active_index_clone = active_index.clone();
            let set_active_index_clone = set_active_index.clone();
            let is_open_clone = is_open.clone();
            let set_is_open_clone = set_is_open.clone();
            let filtered_options_clone = filtered_options.clone();
            let set_value_clone = set_value.clone();
            let set_search_text_clone = set_search_text.clone();
            let dropdown_ref_clone = dropdown_ref.clone();

            // Create keyboard event handler
            let handler = Closure::wrap(Box::new(move |ev: KeyboardEvent| {
                match ev.key().as_str() {
                    "ArrowDown" => {
                        ev.prevent_default();
                        if !is_open_clone.get() {
                            set_is_open_clone.set(true);
                        } else {
                            let max = filtered_options_clone.get().len().saturating_sub(1);
                            let next = (active_index_clone.get() + 1).min(max);
                            set_active_index_clone.set(next);

                            // Scroll to view if needed
                            if let Some(dropdown) = dropdown_ref_clone.get() {
                                if let Some(active_item) = dropdown
                                    .query_selector(&format!("[data-index=\"{}\"]", next))
                                    .ok()
                                    .flatten()
                                {
                                    let _ = active_item.scroll_into_view();
                                }
                            }
                        }
                    }
                    "ArrowUp" => {
                        ev.prevent_default();
                        if is_open_clone.get() {
                            let prev = active_index_clone.get().saturating_sub(1);
                            set_active_index_clone.set(prev);

                            // Scroll to view if needed
                            if let Some(dropdown) = dropdown_ref_clone.get() {
                                if let Some(active_item) = dropdown
                                    .query_selector(&format!("[data-index=\"{}\"]", prev))
                                    .ok()
                                    .flatten()
                                {
                                    let _ = active_item.scroll_into_view();
                                }
                            }
                        }
                    }
                    "Escape" => {
                        ev.prevent_default();
                        set_is_open_clone.set(false);
                    }
                    "Enter" => {
                        if is_open_clone.get() {
                            ev.prevent_default();
                            let filtered = filtered_options_clone.get();
                            if !filtered.is_empty() {
                                let index = active_index_clone.get().min(filtered.len() - 1);
                                Callable::call(&set_value_clone, filtered[index].clone());
                                set_is_open_clone.set(false);
                                set_search_text_clone.set(String::new());
                            }
                        }
                    }
                    _ => {}
                }
            }) as Box<dyn FnMut(KeyboardEvent)>);

            // Add event listener
            input
                .add_event_listener_with_callback("keydown", handler.as_ref().unchecked_ref())
                .expect("should add keydown event listener");

            // Clean up on drop
            on_cleanup(move || {
                if let Some(input_elem) = input_ref.get() {
                    input_elem
                        .remove_event_listener_with_callback(
                            "keydown",
                            handler.as_ref().unchecked_ref(),
                        )
                        .expect("should remove keydown event listener");
                }
                // Handler will be dropped here
            });
        }
    });

    // Function to highlight matched text
    let highlight_text = move |text: &str, search: &str| -> Vec<View> {
        if search.is_empty() {
            return vec![text.to_string().into_view()];
        }

        let search_lower = search.to_lowercase();
        let text_lower = text.to_lowercase();

        let mut parts = Vec::new();
        let mut start_index = 0;

        while let Some(match_start) = text_lower[start_index..].find(&search_lower) {
            let absolute_start = start_index + match_start;
            let match_end = absolute_start + search_lower.len();

            // Add text before match
            if absolute_start > start_index {
                parts.push(text[start_index..absolute_start].to_string().into_view());
            }

            // Add highlighted text
            parts.push(view! {
                <strong class="bg-yellow-200">{text[absolute_start..match_end].to_string()}</strong>
            }.into_view());

            start_index = match_end;
        }

        // Add remaining text after matches
        if start_index < text.len() {
            parts.push(text[start_index..].to_string().into_view());
        }

        if parts.is_empty() {
            parts.push(text.to_string().into_view());
        }

        parts
    };

    // Handle selecting an option
    let select_option = move |opt: T| {
        Callable::call(&set_value, opt);
        set_is_open.set(false);
        set_search_text.set(String::new());

        // Focus the input after selection
        if let Some(input) = input_ref.get() {
            let _ = input.focus();
        }
    };

    // Handle input focus
    let handle_focus = move |_| {
        set_is_open.set(true);
    };

    // Handle input change
    let handle_input = move |ev| {
        let value = event_target_value(&ev);
        set_search_text.set(value);
        set_is_open.set(true);
        set_active_index.set(0);
    };

    // Toggle dropdown
    let toggle_dropdown = move |ev: MouseEvent| {
        ev.prevent_default();
        ev.stop_propagation();
        set_is_open.update(|v| *v = !*v);

        // Focus input when opening
        if !is_open.get() {
            if let Some(input) = input_ref.get() {
                let _ = input.focus();
            }
        }
    };

    view! {
        <div
            _ref=container_ref
            class=move || format!("relative w-full {}", class.get())
            id=move || id.get()
        >
            <div class="relative">
                <input
                    type="text"
                    class="w-full p-2 border rounded focus:outline-none focus:ring-2 focus:ring-blue-300"
                    placeholder=move || placeholder.get()
                    value=move || search_text.get()
                    on:input=handle_input
                    on:focus=handle_focus
                    _ref=input_ref
                    aria-expanded=move || is_open.get().to_string()
                    aria-autocomplete="list"
                    role="combobox"
                />
                <button
                    type="button"
                    class="absolute inset-y-0 right-0 flex items-center pr-2"
                    on:click=toggle_dropdown
                    aria-label="Toggle dropdown"
                >
                    <svg
                        class="w-5 h-5 text-gray-400"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d=move || if is_open.get() {
                                "M5 15l7-7 7 7"
                            } else {
                                "M19 9l-7 7-7-7"
                            }
                        />
                    </svg>
                </button>
            </div>

            <Show when=move || is_open.get()>
                <div
                    class="absolute z-10 w-full mt-1 bg-white border border-gray-300 rounded-md shadow-lg max-h-60 overflow-auto"
                    _ref=dropdown_ref
                    role="listbox"
                >
                    <Show
                        when=move || !filtered_options.get().is_empty()
                        fallback=move || view! {
                            <div class="px-3 py-2 text-gray-500">No results found</div>
                        }
                    >
                        <For
                            each=move || filtered_options.get()
                            key=move |opt| value_fn(opt.clone())
                            let:item
                        >
                            {move || {
                                let opt = item.clone();
                                let opt_for_select = opt.clone();
                                let opt_for_compare = opt.clone();
                                let index = filtered_options.get()
                                    .iter()
                                    .position(|x| x == &opt)
                                    .unwrap_or(0);
                                let is_active = create_memo(move |_| active_index.get() == index);
                                let label = label_fn(opt.clone());

                                view! {
                                    <div
                                        class=move || format!(
                                            "px-3 py-2 cursor-pointer hover:bg-gray-100 {} {}",
                                            if is_active.get() { "bg-blue-50" } else { "" },
                                            if value.get().as_ref() == Some(&opt_for_compare) { "font-medium" } else { "" }
                                        )
                                        on:click=move |_| select_option(opt_for_select.clone())
                                        data-index=index.to_string()
                                        role="option"
                                        aria-selected=move || (value.get().as_ref() == Some(&opt)).to_string()
                                    >
                                        {highlight_text(&label, &search_text.get())}
                                    </div>
                                }
                            }}
                        </For>
                    </Show>
                </div>
            </Show>
        </div>
    }
}

// Example usage
#[component]
pub fn LiveSelectExample() -> impl IntoView {
    // Example data
    let options = vec![
        "Afghanistan".to_string(),
        "Albania".to_string(),
        "Algeria".to_string(),
        "Andorra".to_string(),
        "Angola".to_string(),
        "Antigua and Barbuda".to_string(),
        "Argentina".to_string(),
        "Armenia".to_string(),
        "Australia".to_string(),
        "Austria".to_string(),
        "Azerbaijan".to_string(),
        "Bahamas".to_string(),
        "Bahrain".to_string(),
        "Bangladesh".to_string(),
        "Barbados".to_string(),
        "Belarus".to_string(),
        "Belgium".to_string(),
        "Belize".to_string(),
        "Benin".to_string(),
        "Bhutan".to_string(),
    ];

    let options_sig = create_rw_signal(options);
    let (selected_country, set_selected_country) = create_signal(None::<String>);

    view! {
        <div class="p-4 max-w-md mx-auto">
            <h2 class="text-lg font-semibold mb-2">Select a Country</h2>

            <LiveSelect
                options=options_sig.into()
                value=Signal::derive(move || selected_country.get())
                set_value=Callback::new(move |val: String| set_selected_country.set(Some(val)))
                label_fn=Callback::new(|s: String| s)
                value_fn=Callback::new(|s: String| s)
                placeholder="Search countries...".into()
                id="country-select".into()
                class="mb-4".into()
                debug=false
            />

            <div class="mt-4">
                <Show
                    when=move || selected_country.get().is_some()
                    fallback=|| view! { <p>No country selected</p> }
                >
                    <p>Selected country: {move || selected_country.get().unwrap_or_default()}</p>
                </Show>
            </div>
        </div>
    }
}
