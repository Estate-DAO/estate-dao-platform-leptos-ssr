use super::*;
use crate::api::client_side_api::{CitySearchResult, ClientSideApiClient, Place};

use crate::log;
use crate::view_state_layer::ui_search_state::UISearchCtx;
use leptos::{html::Div, NodeRef};
use leptos_use::on_click_outside;
use wasm_bindgen::JsCast;
use web_sys::MouseEvent;

// Helper function to convert CitySearchResult to Destination
fn city_search_result_to_destination(city: CitySearchResult) -> Destination {
    Destination {
        city: city.city_name,
        country_name: city.country_name,
        country_code: city.country_code,
        city_id: city.city_code,
        latitude: Some(city.latitude),
        longitude: Some(city.longitude),
    }
}

#[component]
pub fn DestinationPickerV6() -> impl IntoView {
    let search_ctx: UISearchCtx = expect_context();

    // Simple state management
    let (search_text, set_search_text) = create_signal(String::new());
    let (is_open, set_is_open) = create_signal(false);
    let (active_index, set_active_index) = create_signal(0);
    let (search_results, set_search_results) = create_signal(Vec::<Place>::new());
    let (is_loading, set_is_loading) = create_signal(false);

    // DOM refs
    let container_ref = create_node_ref::<Div>();
    let input_ref = create_node_ref::<leptos::html::Input>();

    // Create API client
    let api_client = ClientSideApiClient::new();

    // Initialize search text with current selection
    create_effect(move |_| {
        if let Some(place) = search_ctx.place.get() {
            if !is_open.get() {
                set_search_text.set(format!(
                    "{}\n{}",
                    place.display_name, place.formatted_address
                ));
            }
        } else if !is_open.get() {
            set_search_text.set(String::new());
        }
    });

    // Reset active index when filtered options change
    create_effect(move |_| {
        let _ = search_results.get();
        set_active_index.set(0);
    });

    // Handle clicks outside to close dropdown
    let _cleanup = on_click_outside(container_ref, move |_| {
        if is_open.get_untracked() {
            set_is_open.set(false);
            // Restore display text
            if let Some(dest) = search_ctx.destination.get() {
                set_search_text.set(format!("{}, {}", dest.city, dest.country_name));
            } else {
                set_search_text.set(String::new());
            }
        }
    });

    // Debounced search function
    let perform_search = create_action(move |prefix: &String| {
        let prefix = prefix.clone();
        let api_client = api_client.clone();
        async move {
            if prefix.trim().is_empty() {
                return Vec::new();
            }

            match api_client.search_places(prefix).await {
                Ok(cities) => cities,
                Err(e) => {
                    log!("City search error: {}", e);
                    Vec::new()
                }
            }
        }
    });

    // Watch for search action completion
    create_effect(move |_| {
        if let Some(results) = perform_search.value().get() {
            set_search_results.set(results);
            set_is_loading.set(false);
        }
    });

    // Watch for search action pending state
    create_effect(move |_| {
        set_is_loading.set(perform_search.pending().get());
    });

    // Event handlers
    let handle_input = move |ev: leptos::ev::Event| {
        let value = event_target_value(&ev);
        set_search_text.set(value.clone());
        set_is_open.set(true);
        set_active_index.set(0);

        // Trigger search with debouncing
        if !value.trim().is_empty() {
            set_is_loading.set(true);
            perform_search.dispatch(value);
        } else {
            set_search_results.set(Vec::new());
            set_is_loading.set(false);
        }
    };

    let handle_focus = move |_: leptos::ev::FocusEvent| {
        log!("Focus event triggered");
        set_is_open.set(true);
    };

    let handle_click = move |ev: MouseEvent| {
        log!(
            "Click event triggered, current is_open: {}",
            is_open.get_untracked()
        );
        ev.stop_propagation();
        set_is_open.set(true); // Always open on click for debugging
    };

    let select_option = move |place: Place| {
        log!(
            "Selecting option: {}, {}",
            place.display_name,
            place.formatted_address
        );
        let _ = UISearchCtx::set_place(place.clone());
        set_search_text.set(format!(
            "{}\n{}",
            place.display_name, place.formatted_address
        ));
        set_is_open.set(false);
        log!(
            "Dropdown should be closed now, is_open: {}",
            is_open.get_untracked()
        );

        // Don't focus input immediately - this was causing the reopen issue
        // The user can click again if they want to search for something else
    };

    let handle_key_down = move |ev: web_sys::KeyboardEvent| {
        match ev.key().as_str() {
            "ArrowDown" => {
                ev.prevent_default();
                if !is_open.get() {
                    return;
                }

                let results = search_results.get();
                if results.is_empty() {
                    return;
                }

                let current = active_index.get();
                let next = if current >= results.len() - 1 {
                    0
                } else {
                    current + 1
                };
                set_active_index.set(next);
            }
            "ArrowUp" => {
                ev.prevent_default();
                if !is_open.get() {
                    return;
                }

                let results = search_results.get();
                if results.is_empty() {
                    return;
                }

                let current = active_index.get();
                let next = if current == 0 {
                    results.len() - 1
                } else {
                    current - 1
                };
                set_active_index.set(next);
            }
            "Enter" => {
                if is_open.get() {
                    ev.prevent_default();
                    let results = search_results.get();
                    let current = active_index.get();

                    if !results.is_empty() && current < results.len() {
                        select_option(results[current].clone());
                    }
                }
            }
            "Escape" => {
                if is_open.get() {
                    ev.prevent_default();
                    set_is_open.set(false);
                    // Restore display text
                    if let Some(dest) = search_ctx.destination.get() {
                        set_search_text.set(format!("{}, {}", dest.city, dest.country_name));
                    } else {
                        set_search_text.set(String::new());
                    }
                }
            }
            "Tab" => {
                if is_open.get() {
                    set_is_open.set(false);
                    // Restore display text
                    if let Some(dest) = search_ctx.destination.get() {
                        set_search_text.set(format!("{}, {}", dest.city, dest.country_name));
                    } else {
                        set_search_text.set(String::new());
                    }
                }
            }
            _ => {}
        }
    };

    // Add this function inside your component, before the view! macro
    let highlight_match = move |text: &str, search: &str| -> View {
        if search.is_empty() {
            return view! {
                {text.to_string()}
            }
            .into_view();
        }

        let search_lower = search.to_lowercase();
        let text_lower = text.to_lowercase();

        if let Some(start) = text_lower.find(&search_lower) {
            let end = start + search.len();
            let before = &text[..start];
            let matched = &text[start..end];
            let after = &text[end..];

            view! {
                {before.to_string()}
                <span class="text-blue-700 font-medium">{matched.to_string()}</span>
                {after.to_string()}
            }
            .into_view()
        } else {
            view! {
                {text.to_string()}
            }
            .into_view()
        }
    };

    view! {
        <div
            class="relative flex w-full md:w-[274px] h-full"
            node_ref=container_ref
        >
            <div class="absolute inset-y-0 left-2 py-6 px-4 text-xl pointer-events-none flex items-center">
                <Icon icon=icondata::BsMap class="text-black font-bold" />
            </div>

            <div class="relative w-full">
                <input
                    type="text"
                    node_ref=input_ref
                    id="destination-live-select"
                    class="w-full h-full pl-14 text-[15px] leading-[18px] text-gray-900 bg-transparent rounded-full transition-colors focus:outline-none py-6"
                    placeholder="Where to?"
                    autocomplete="off"
                    aria-autocomplete="list"
                    aria-controls="destination-dropdown"
                    aria-expanded=move || is_open.get().to_string()
                    role="combobox"
                    prop:value=search_text
                    on:input=handle_input
                    on:focus=handle_focus
                    on:click=handle_click
                    on:keydown=handle_key_down
                />

                // Dropdown - Debug version
                {move || {
                    log!("Dropdown render check - is_open: {}", is_open.get());
                    if is_open.get() {
                        log!("Rendering dropdown");
                        Some(view! {
                            <div
                                id="destination-dropdown"
                                class="absolute z-50 w-full bg-white border border-gray-200 rounded-md shadow-lg max-h-60 overflow-auto mt-2"
                                role="listbox"
                            >
                                {move || {
                                    let results = search_results.get();
                                    let is_loading_val = is_loading.get();
                                    let search_text_val = search_text.get();

                                    log!("Search results count: {}, is_loading: {}", results.len(), is_loading_val);

                                    if is_loading_val {
                                        view! {
                                            <div class="px-3 py-2 text-gray-500">
                                                "Searching..."
                                            </div>
                                        }.into_view()
                                    } else if search_text_val.trim().is_empty() {
                                        view! {
                                            <div class="px-3 py-2 text-gray-500">
                                                "Start typing to search cities..."
                                            </div>
                                        }.into_view()
                                    } else if results.is_empty() {
                                        view! {
                                            <div class="px-3 py-2 text-gray-500">
                                                "No results found"
                                            </div>
                                        }.into_view()
                                    } else {
                                        results.into_iter().enumerate().map(|(i, dest)| {
                                            let dest_clone = dest.clone();
                                            let dest_for_click = dest.clone();
                                            view! {
                                                <div
                                                    class=move || {
                                                        let base = "px-3 py-2 cursor-pointer hover:bg-gray-100";
                                                        if active_index.get() == i {
                                                            format!("{} bg-blue-50 text-blue-600", base)
                                                        } else {
                                                            base.to_string()
                                                        }
                                                    }
                                                    role="option"
                                                    aria-selected=move || (active_index.get() == i).to_string()
                                                    on:click=move |ev| {
                                                        log!("Option clicked");
                                                        ev.stop_propagation();
                                                        select_option(dest_for_click.clone());
                                                    }
                                                    on:mouseenter=move |_| {
                                                        set_active_index.set(i);
                                                    }
                                                >
                                                {highlight_match(&format!("{}\n{}", dest_clone.display_name, dest_clone.formatted_address), &search_text.get())}
                                                </div>
                                            }
                                        }).collect::<Vec<_>>().into_view()
                                    }
                                }}
                            </div>
                        })
                    } else {
                        log!("Not rendering dropdown - closed");
                        None
                    }
                }}
            </div>
        </div>
    }
}
