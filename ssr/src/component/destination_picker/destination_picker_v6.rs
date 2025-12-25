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
pub fn DestinationPickerV6(#[prop(optional, into)] h_class: MaybeSignal<String>) -> impl IntoView {
    let h_class = create_memo(move |_| {
        let class = h_class.get();
        if class.is_empty() {
            "h-full".to_string()
        } else {
            class
        }
    });
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
            crate::log!(
                "[DestinationPickerV6] Place detected: display_name='{}', formatted_address='{}', is_open={}",
                place.display_name,
                place.formatted_address,
                is_open.get()
            );
            if !is_open.get() {
                let search_text = if place.formatted_address.trim().is_empty() {
                    place.display_name.clone()
                } else {
                    format!("{}, {}", place.display_name, place.formatted_address)
                };
                crate::log!(
                    "[DestinationPickerV6] Setting search_text to: '{}'",
                    search_text
                );
                set_search_text.set(search_text);
            }
        } else {
            crate::log!(
                "[DestinationPickerV6] No place in context, is_open={}",
                is_open.get()
            );
            if !is_open.get() {
                set_search_text.set(String::new());
            }
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
            // Restore display text from place (new) or destination (legacy fallback)
            if let Some(place) = search_ctx.place.get() {
                let search_text = if place.formatted_address.trim().is_empty() {
                    place.display_name.clone()
                } else {
                    format!("{}, {}", place.display_name, place.formatted_address)
                };
                crate::log!(
                    "[DestinationPickerV6] Click outside: Restoring from place: '{}'",
                    search_text
                );
                set_search_text.set(search_text);
            } else if let Some(dest) = search_ctx.destination.get() {
                crate::log!(
                    "[DestinationPickerV6] Click outside: Restoring from destination (legacy)"
                );
                set_search_text.set(format!("{}, {}", dest.city, dest.country_name));
            } else {
                crate::log!(
                    "[DestinationPickerV6] Click outside: No place or destination, clearing"
                );
                set_search_text.set(String::new());
            }
        }
    });

    // Debounced search function - returns Option to distinguish success/failure
    let perform_search = create_action(move |prefix: &String| {
        let prefix = prefix.clone();
        let api_client = api_client.clone();
        async move {
            if prefix.trim().is_empty() {
                return Some(Vec::new());
            }

            match api_client.search_places(prefix).await {
                Ok(cities) => Some(cities),
                Err(e) => {
                    log!("City search error (will keep existing results): {}", e);
                    None // Return None on failure to indicate "keep existing"
                }
            }
        }
    });

    // Watch for search action completion - only update if API succeeded
    create_effect(move |prev_results: Option<Vec<Place>>| {
        let current_results = search_results.get_untracked();
        let prev = prev_results.unwrap_or_else(|| current_results.clone());

        if let Some(api_response) = perform_search.value().get() {
            set_is_loading.set(false);

            match api_response {
                Some(new_results) => {
                    // API succeeded - use new results
                    set_search_results.set(new_results.clone());
                    return new_results;
                }
                None => {
                    // API failed - keep existing results, optionally filter them
                    let query = search_text.get_untracked().to_lowercase();
                    if !prev.is_empty() && !query.is_empty() {
                        // Filter existing results to match new query
                        let filtered: Vec<_> = prev
                            .iter()
                            .filter(|p| {
                                p.display_name.to_lowercase().contains(&query)
                                    || p.formatted_address.to_lowercase().contains(&query)
                            })
                            .cloned()
                            .collect();

                        if !filtered.is_empty() {
                            log!(
                                "API failed, showing {} filtered results from cache",
                                filtered.len()
                            );
                            set_search_results.set(filtered.clone());
                            return filtered;
                        }
                    }
                    // Keep existing results as-is
                    log!("API failed, keeping {} existing results", prev.len());
                    return prev;
                }
            }
        }

        current_results
    });

    // Watch for search action pending state
    create_effect(move |_| {
        set_is_loading.set(perform_search.pending().get());
    });

    // Debounce timeout handle
    let debounce_handle = create_rw_signal::<Option<i32>>(None);

    // Minimum characters before searching
    const MIN_SEARCH_CHARS: usize = 3;

    // Event handlers
    let handle_input = move |ev: leptos::ev::Event| {
        let value = event_target_value(&ev);
        set_search_text.set(value.clone());
        set_is_open.set(true);
        set_active_index.set(0);

        // Clear any existing debounce timeout
        if let Some(handle) = debounce_handle.get_untracked() {
            web_sys::window().unwrap().clear_timeout_with_handle(handle);
        }

        let trimmed = value.trim();

        // Check minimum characters
        if trimmed.len() < MIN_SEARCH_CHARS {
            set_search_results.set(Vec::new());
            set_is_loading.set(false);
            debounce_handle.set(None);
            return;
        }

        // Skip if already pending (prevent concurrent requests)
        if perform_search.pending().get_untracked() {
            // Still set up debounce for after the current request completes
            set_is_loading.set(true);
        }

        // Set up debounced search (300ms)
        let search_value = value.clone();
        let callback = wasm_bindgen::prelude::Closure::once(Box::new(move || {
            perform_search.dispatch(search_value);
        }) as Box<dyn FnOnce()>);

        let handle = web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                callback.as_ref().unchecked_ref(),
                600, // 600ms debounce (increased from 300ms)
            )
            .unwrap();

        callback.forget(); // Prevent cleanup until timeout fires
        debounce_handle.set(Some(handle));
        set_is_loading.set(true);
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
        // log!(
        //     "Selecting option: {}, {}",
        //     place.display_name,
        //     place.formatted_address
        // );
        let _ = UISearchCtx::set_place(place.clone());

        let search_text = if place.formatted_address.trim().is_empty() {
            place.display_name.clone()
        } else {
            format!("{}, {}", place.display_name, place.formatted_address)
        };

        set_search_text.set(search_text);
        set_is_open.set(false);
        // log!(
        //     "Dropdown should be closed now, is_open: {}",
        //     is_open.get_untracked()
        // );

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
                    // Restore display text from place (new) or destination (legacy fallback)
                    if let Some(place) = search_ctx.place.get() {
                        let search_text = if place.formatted_address.trim().is_empty() {
                            place.display_name.clone()
                        } else {
                            format!("{}, {}", place.display_name, place.formatted_address)
                        };
                        set_search_text.set(search_text);
                    } else if let Some(dest) = search_ctx.destination.get() {
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

    let highlight_match = move |text: &str, search: &str| -> View {
        if search.is_empty() {
            return text.to_string().into_view();
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
            text.to_string().into_view()
        }
    };

    view! {
        <div class=move || format!("relative flex items-center w-full h-[56px] py-2 {}", h_class()) node_ref=container_ref>
            <div class="absolute inset-y-0 left-2 flex items-center text-xl pointer-events-none">
                <Icon icon=icondata::BsMap class="text-blue-500 font-bold"/>
            </div>

            <div class="relative w-full">
                <input
                    type="text"
                    node_ref=input_ref
                    id="destination-live-select"
                    class=move || {
                        format!(
                            "w-full h-full {} pl-14 text-[15px] leading-[18px] text-gray-900 font-medium bg-transparent rounded-md transition-colors focus:outline-none md:text-ellipsis",
                            h_class(),
                        )
                    }

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
                                class="absolute z-[9999] w-full bg-white border border-gray-200 rounded-md shadow-lg max-h-60 overflow-auto mt-2"
                                role="listbox"
                            >
                                {move || {
                                    let results = search_results.get();
                                    let is_loading_val = is_loading.get();
                                    let search_text_val = search_text.get();

                                    log!(
                                        "Search results count: {}, is_loading: {}",
                                        results.len(),
                                        is_loading_val
                                    );

                                    if is_loading_val {
                                        view! {
                                            <div class="px-3 py-2 text-gray-500">"Searching..."</div>
                                        }
                                            .into_view()
                                    } else if search_text_val.trim().is_empty() {
                                        view! {
                                            <div class="px-3 py-2 text-gray-500">
                                                "Start typing to search cities..."
                                            </div>
                                        }
                                            .into_view()
                                    } else if results.is_empty() {
                                        view! {
                                            <div class="px-3 py-2 text-gray-500">
                                                "No results found"
                                            </div>
                                        }
                                            .into_view()
                                    } else {
                                        results
                                            .into_iter()
                                            .enumerate()
                                            .map(| (i, dest) | {
                                                let dest_clone = dest.clone();
                                                let dest_for_click = dest.clone();
                                                view! {
                                                    <div
                                                        class=move || {
                                                            let base = "px-3 py-2 cursor-pointer hover:bg-gray-100";
                                                            if active_index.get() == i {
                                                                format!(
                                                                    "{} bg-blue-50 text-blue-600",
                                                                    base,
                                                                )
                                                            } else {
                                                                base.to_string()
                                                            }
                                                        }
                                                        role="option"
                                                        aria-selected=move || {
                                                            (active_index.get() == i).to_string()
                                                        }
                                                        on:click=move | ev | {
                                                            log!("Option clicked");
                                                            ev.stop_propagation();
                                                            select_option(dest_for_click.clone());
                                                        }
                                                        on:mouseenter=move | _ | {
                                                            set_active_index.set(i);
                                                        }
                                                    >
                                                        {highlight_match(
                                                            &{
                                                                if dest_clone
                                                                    .formatted_address
                                                                    .trim()
                                                                    .is_empty()
                                                                {
                                                                    dest_clone.display_name.clone()
                                                                } else {
                                                                    format!(
                                                                        "{}, {}",
                                                                        dest_clone.display_name,
                                                                        dest_clone.formatted_address,
                                                                    )
                                                                }
                                                            },
                                                            &search_text.get(),
                                                        )}
                                                    </div>
                                                }
                                            })
                                            .collect::<Vec<_>>()
                                            .into_view()
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
