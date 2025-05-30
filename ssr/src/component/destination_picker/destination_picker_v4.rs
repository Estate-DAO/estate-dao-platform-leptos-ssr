use super::*;

// Here are the observations about what works and what does not in the expected behaviour of the component
// 1. [not_functional] when user clicks on the input, the dropdown opens
// 2. [not_functional] when user types, the city list is filtered and higlighted
// 3. when user clicks on a city from the dropdown list
//    3.1 [not_functional] But the city is not filled back in the input. The city should fill back into the input when selected.
//    3.2 [not_functional] the city is selected and the dropdown closes.
// 4. [not_functional] when user re-opens the dropdown (by clicking on the input again), The input city (from step 3) should be filled back into the input.
// 5. [not_functional] corollary to 4, if the user deletes a few character from the input after he has reopened the dropdown, the search dropdown should filter the original list accordginly. In other words, filter and highlight should be triggered on each keystroke to the input when user is typing.
// 6. [not_functional] When user click away from the input, the dropdown does not close automatically. it sould close when user clicks outside the input.
// 7. [not_functional] css for the input is as expected. dropdown cannot be seen. hence, no idea how that css looks.

#[component]
pub fn DestinationPickerV4() -> impl IntoView {
    let search_ctx: SearchCtx = expect_context();
    let input_group_ctx = InputGroupState::get();

    let QueryResult {
        data: destinations_resource,
        ..
    } = destinations_query().use_query(|| true);

    let (search_term, set_search_term) = create_signal(String::new());
    let input_ref = create_node_ref::<Input>();
    let component_ref = create_node_ref::<html::Div>(); // Ref for the entire component for click-outside

    // is_open signal derived from InputGroupState
    let is_open = create_memo(move |_| {
        matches!(
            input_group_ctx.open_dialog.get(),
            OpenDialogComponent::CityListComponent
        )
    });

    // Effect to manage search_term when dropdown opens/closes or selection changes
    create_effect(move |_| {
        let current_is_open = is_open.get();
        let current_destination = search_ctx.destination.get();

        if current_is_open {
            // Dropdown is open
            if let Some(active_el) = document().active_element() {
                if input_ref
                    .get()
                    .map_or(false, |ir| ir.is_same_node(Some(&active_el)))
                {
                    // Input is focused
                    if let Some(dest) = &current_destination {
                        // If current search term is the selected destination, clear for new search
                        if search_term.get_untracked()
                            == format!("{}, {}", dest.city, dest.country_name)
                        {
                            set_search_term.set("".to_string());
                        }
                    }
                    // Ensure input text is selected if focused
                    if let Some(input_el) = input_ref.get() {
                        input_el.select();
                    }
                }
            }
        } else {
            // Dropdown is closed
            if let Some(dest) = &current_destination {
                set_search_term.set(format!("{}, {}", dest.city, dest.country_name));
            } else {
                set_search_term.set("".to_string()); // Clear if no destination selected
            }
        }
    });

    let filtered_destinations = create_memo(move |_| {
        let term = search_term.get().to_lowercase();
        destinations_resource
            .get()
            .map_or_else(Vec::new, |dest_opt| {
                dest_opt.map_or_else(Vec::new, |dests| {
                    if term.is_empty() && !is_open.get() {
                        // If not searching and dropdown closed, show nothing
                        return Vec::new();
                    }
                    if term.is_empty() && is_open.get() {
                        // If dropdown open and search term empty, show all
                        return dests;
                    }
                    dests
                        .into_iter()
                        .filter(|dest| {
                            dest.city.to_lowercase().contains(&term)
                                || dest.country_name.to_lowercase().contains(&term)
                        })
                        .collect()
                })
            })
    });

    let display_value_in_input = create_memo(move |_| {
        // When input is focused and dropdown is open, show the live search_term
        if is_open.get()
            && input_ref.get().map_or(false, |el| {
                el.is_same_node(document().active_element().as_ref().map(|v| &**v))
            })
        {
            return search_term.get();
        }
        // Otherwise, show the selected destination's display text or empty if none
        search_ctx
            .destination
            .get()
            .map(|d| format!("{}, {}", d.city, d.country_name))
            .unwrap_or_else(String::new)
    });

    // Click outside handler
    let _ = on_click_outside(component_ref, move |_| {
        if is_open.get_untracked() {
            InputGroupState::toggle_dialog(OpenDialogComponent::None);
        }
    });

    let handle_focus = move |_| {
        InputGroupState::toggle_dialog(OpenDialogComponent::CityListComponent);
        // Effect above handles search_term and selection on focus
    };

    let handle_select_destination = move |dest: Destination| {
        SearchCtx::set_destination(dest.clone());
        // search_term will be updated by the effect when is_open becomes false
        InputGroupState::toggle_dialog(OpenDialogComponent::None); // Close dropdown
    };

    // Helper function to highlight matching text (copied from DestinationSearch)
    let highlight_text = move |text: &str, search: &str| -> View {
        let text_string = text.to_string();
        let search_string = search.to_string();
        if search_string.is_empty() {
            return view! { <span>{text_string.clone()}</span> }.into_view();
        }

        let search_lower = search.to_lowercase();
        let text_lower = text.to_lowercase();

        if let Some(start) = text_lower.find(&search_lower) {
            let end = start + search.len();
            let before = &text[..start];
            let matched = &text[start..end];
            let after = &text[end..];

            view! {
                <span>
                    {before.to_string()}
                    <span class="bg-yellow-200 text-yellow-800">{matched.to_string()}</span>
                    {after.to_string()}
                </span>
            }
            .into_view()
        } else {
            view! { <span>{text_string}</span> }.into_view()
        }
    };

    // Helper function to get icon based on destination type (copied from DestinationSearch)
    let get_destination_icon = |dest: &Destination| -> &str {
        if dest.city.to_lowercase().contains("airport") {
            "✈️"
        } else if dest.city.to_lowercase().contains("railway")
            || dest.city.to_lowercase().contains("station")
        {
            "🏛️" // Changed from train to building for variety, as per original
        } else {
            "📍"
        }
    };

    view! {
        <div class="relative w-full h-full" _ref=component_ref>
            // <!-- Icon positioned absolutely inside the input field's space -->
            <div class="absolute inset-y-0 left-0 flex items-center pl-3 pointer-events-none z-[1]">
                <Icon icon=icondata::BsMap class="text-gray-400 h-5 w-5" />
            </div>

            <input
                type="text"
                _ref=input_ref
                // <!-- pl-10 to make space for the icon. text-base for general alignment. -->
                // <!-- h-full to fill the parent container (e.g., h-[56px] from InputGroup) -->
                class="w-full h-full pl-10 pr-4 py-2 text-[15px] leading-[18px] text-gray-900 bg-transparent focus:outline-none"
                placeholder="Where to?"
                prop:value=display_value_in_input
                on:input=move |ev| set_search_term.set(event_target_value(&ev))
                on:focus=handle_focus
            />

            <Show when=is_open>
                <div
                    // <!-- Dropdown styling: full width on mobile, fixed on desktop, scrollable -->
                    class="absolute top-full left-0 right-0 md:left-auto md:w-[350px] mt-1 bg-white border border-gray-200 rounded-lg shadow-lg z-[98] max-h-[60vh] md:max-h-[280px] overflow-y-auto"
                    // <!-- Prevent mousedown from propagating to component_ref's on_click_outside handler if a scrollbar or empty area in dropdown is clicked -->
                    on:mousedown=move |ev| ev.stop_propagation()
                >
                    <Suspense fallback=move || view! { <div class="p-4 text-gray-500">"Loading..."</div> }>
                        {move || {
                            let current_search_term_for_highlight = search_term.get();
                            filtered_destinations.with(|dests| {
                                if dests.is_empty() {
                                    // Show "No destinations found" only if a search term is entered
                                    if !search_term.get().is_empty() {
                                        view! { <div class="p-4 text-gray-600 text-sm">"No destinations found"</div> }.into_view()
                                    } else {
                                        // Optionally, show a message like "Type to search" or popular destinations
                                        // For now, show nothing if search term is empty and no results (e.g. initial state)
                                        view! { <div class="p-4 text-gray-600 text-sm">"Type to search for a destination"</div> }.into_view()
                                    }
                                } else {
                                    dests.iter().map(|dest| {
                                        let dest_clone = dest.clone();
                                        let dest_clone_for_select = dest.clone();
                                        let icon = get_destination_icon(&dest_clone);
                                        view! {
                                            <div
                                                class="flex items-center p-3 hover:bg-gray-50 cursor-pointer transition-colors border-b border-gray-100 last:border-b-0"
                                                on:click=move |_| handle_select_destination(dest_clone_for_select.clone())
                                            >
                                                <span class="text-xl mr-3 w-5 text-center">{icon}</span>
                                                <div class="flex-1">
                                                    <div class="text-sm font-medium text-gray-800">
                                                        {highlight_text(&dest_clone.city, &current_search_term_for_highlight)}
                                                    </div>
                                                    <div class="text-xs text-gray-500">
                                                        {highlight_text(&dest_clone.country_name, &current_search_term_for_highlight)}
                                                    </div>
                                                </div>
                                            </div>
                                        }
                                    }).collect_view()
                                }
                            })
                        }}
                    </Suspense>
                </div>
            </Show>
        </div>
    }
}
