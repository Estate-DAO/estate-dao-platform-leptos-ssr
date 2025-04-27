use leptos::ev::{FocusEvent, MouseEvent};
use leptos::html::{Div, Input};
use leptos::*;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{Event, KeyboardEvent, Node};

use crate::log;

/// A customizable dropdown select component with search functionality
#[component]
pub fn LiveSelect<T>(
    /// List of options to show in the dropdown
    #[prop(optional)]
    options: MaybeSignal<Vec<T>>,
    /// Currently selected value
    value: Signal<Option<T>>,
    /// Callback to set the selected value
    set_value: Callback<T>,
    /// Function to convert an option to a display label
    label_fn: Callback<T, String>,
    /// Function to convert an option to a unique identifier
    value_fn: Callback<T, String>,
    /// Placeholder text for the input field
    #[prop(optional)]
    placeholder: MaybeSignal<String>,
    /// HTML id for the component
    #[prop(optional)]
    id: MaybeSignal<String>,
    /// Custom CSS classes
    #[prop(optional)]
    class: MaybeSignal<String>,
    /// Enable debug logging
    #[prop(optional)]
    debug: bool,
) -> impl IntoView
where
    T: Clone + PartialEq + 'static,
{
    // Core state signals
    let search_text = create_rw_signal(String::new());
    let is_open = create_rw_signal(false);
    let active_index = create_rw_signal(0);

    // Element references
    let input_ref = create_node_ref::<Input>();
    let dropdown_ref = create_node_ref::<Div>();
    let container_ref = create_node_ref::<Div>();

    // Computed: filtered options based on search text
    let filtered_options = create_memo(move |_| {
        let search = search_text.get().to_lowercase();
        let all_options = options.get();

        if search.is_empty() {
            return all_options;
        }

        all_options
            .into_iter()
            .filter(|opt| label_fn(opt.clone()).to_lowercase().contains(&search))
            .collect::<Vec<T>>()
    });

    // Actions
    let close_dropdown = move || is_open.set(false);
    let open_dropdown = move || is_open.set(true);
    let toggle_dropdown = move || is_open.update(|v| *v = !*v);

    let select_option = move |opt: T| {
        set_value(opt);
        close_dropdown();
        search_text.set(String::new());

        // Focus the input after selection
        if let Some(input) = input_ref.get() {
            let _ = input.focus();
        }
    };

    // Event handlers
    let handle_input = move |ev: Event| {
        let value = event_target_value(&ev);
        search_text.set(value);
        open_dropdown();
        active_index.set(0);
    };

    let handle_focus = move |_: FocusEvent| {
        open_dropdown();
    };

    let handle_toggle_click = move |ev: MouseEvent| {
        ev.prevent_default();
        ev.stop_propagation();
        toggle_dropdown();

        // Focus input when opening
        if !is_open.get() {
            if let Some(input) = input_ref.get() {
                let _ = input.focus();
            }
        }
    };

    // Set up keyboard navigation
    create_effect(move |_| {
        if let Some(input) = input_ref.get_untracked() {
            let handler = Closure::wrap(Box::new(move |ev: KeyboardEvent| {
                match ev.key().as_str() {
                    "ArrowDown" => {
                        ev.prevent_default();
                        if !is_open.get() {
                            open_dropdown();
                        } else {
                            let filtered = filtered_options.get();
                            let max = filtered.len().saturating_sub(1);
                            let next = (active_index.get() + 1).min(max);
                            active_index.set(next);

                            // Scroll to view if needed
                            if let Some(dropdown) = dropdown_ref.get() {
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
                        if is_open.get() {
                            let prev = active_index.get().saturating_sub(1);
                            active_index.set(prev);

                            // Scroll to view if needed
                            if let Some(dropdown) = dropdown_ref.get() {
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
                        close_dropdown();
                    }
                    "Enter" => {
                        if is_open.get() {
                            ev.prevent_default();
                            let filtered = filtered_options.get();
                            if !filtered.is_empty() {
                                let index = active_index.get().min(filtered.len() - 1);
                                select_option(filtered[index].clone());
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
            });
        }
    });

    // Set up outside click detection
    create_effect(move |_| {
        if is_open.get() {
            let container = container_ref.get_untracked();
            if let Some(container) = container {
                let container_clone = container.clone();

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
                        close_dropdown();
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
                });
            }
        }
    });

    // Highlight function to show search matches
    let highlight_text = |text: &str, search: &str| -> Vec<View> {
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

    view! {
        <div
            _ref=container_ref
            class=move || format!("relative w-full {}", class.get())
            id=move || id.get()
        >
            <div class="relative" aria-haspopup="listbox" aria-owns="dropdown-listbox">
                <input
                    type="text"
                    class="w-full p-3 rounded-xl border border-gray-300 focus:outline-none focus:ring-2 focus:ring-blue-300 bg-white text-base placeholder-gray-400 shadow-sm"
                    placeholder=move || placeholder.get()
                    value=move || search_text.get()
                    on:input=handle_input
                    on:focus=handle_focus
                    _ref=input_ref
                    aria-expanded=move || is_open.get().to_string()
                    aria-autocomplete="list"
                    aria-controls="dropdown-listbox"
                    role="combobox"
                />

                <button
                    type="button"
                    class="absolute inset-y-0 right-0 flex items-center pr-3"
                    on:click=handle_toggle_click
                    aria-label="Toggle dropdown"
                >
                    <svg
                        class="w-6 h-6 text-gray-400"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                        aria-hidden="true"
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
                    id="dropdown-listbox"
                    class="absolute z-30 w-full mt-2 bg-white border border-gray-200 rounded-xl shadow-xl max-h-[280px] overflow-auto"
                    _ref=dropdown_ref
                    role="listbox"
                    aria-label="Options"
                >
                    <div class="p-2">
                        <Show
                            when=move || !filtered_options.get().is_empty()
                            fallback=move || view! {
                                <div class="flex justify-center items-center h-20 text-gray-400 text-base" role="status">No results found</div>
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
                                    let is_selected = create_memo(move |_| value.get().as_ref() == Some(&opt_for_compare));
                                    let label = label_fn(opt.clone());
                                    let current_search = create_memo(move |_| search_text.get());

                                    view! {
                                        <div
                                            class=move || format!(
                                                "px-4 py-3 my-1 rounded-lg cursor-pointer transition-all text-base flex items-center gap-2 {} {}",
                                                if is_active.get() { "bg-blue-50 shadow border border-blue-200" } else { "hover:bg-gray-100" },
                                                if is_selected.get() { "font-semibold text-blue-700" } else { "text-gray-800" }
                                            )
                                            on:click=move |_| select_option(opt_for_select.clone())
                                            data-index=index.to_string()
                                            role="option"
                                            id=format!("option-{}", index)
                                            aria-selected=move || is_selected.get().to_string()
                                            tabindex=move || if is_active.get() { "0".to_string() } else { "-1".to_string() }
                                        >
                                            {highlight_text(&label, &current_search.get())}
                                        </div>
                                    }
                                }}
                            </For>
                        </Show>
                    </div>
                </div>
            </Show>
        </div>
    }
}

#[component]
pub fn LiveSelectExample() -> impl IntoView {
    // Example data
    let countries = vec![
        "Afghanistan",
        "Albania",
        "Algeria",
        "Andorra",
        "Angola",
        "Antigua and Barbuda",
        "Argentina",
        "Armenia",
        "Australia",
        "Austria",
        "Azerbaijan",
        "Bahamas",
        "Bahrain",
        "Bangladesh",
        "Barbados",
        "Belarus",
        "Belgium",
        "Belize",
        "Benin",
        "Bhutan",
    ]
    .into_iter()
    .map(String::from)
    .collect::<Vec<String>>();

    let options_sig = create_rw_signal(countries);
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
