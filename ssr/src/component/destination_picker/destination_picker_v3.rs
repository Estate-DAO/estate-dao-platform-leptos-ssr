use super::*;

// Here are the observations about what works and what does not in the expected behaviour of the component
// 1. [fully_functional] when user clicks on the input, the dropdown opens
// 2. [fully_functional] when user types, the city list is filtered and higlighted
// 3. when user clicks on a city from the dropdown list
//    3.1 [not_functional] the city is not filled back in the input. The city should fill back into the input when selected.
//    3.2 [not_functional] the city is selected and the dropdown closes.
// 4. [not_functional] when user re-opens the dropdown (by clicking on the input again), The input city (from step 3) should be filled back into the input.
// 5. [not_functional] corollary to 4, if the user deletes a few character from the input after he has reopened the dropdown, the search dropdown should filter the original list accordginly. In other words, filter and highlight should be triggered on each keystroke to the input when user is typing.
// 6. [not_functional] When user click away from the input, the dropdown does not close automatically. it sould close when user clicks outside the input.
// 7. [not_functional] css is as expected. so look and feel are as per design. it has one addtional button - 'open destination' to open the search dropdown box. that is not needed. instead, we can just keep the input box and dropdown. fill back the values into the input and close dropdown when value is selected.
// 7.1 [not_functional] not only css is not right, the extra button is not needed.
// 7.2 [not_functional] the dropdown does not render when rendered inside InputGroup. most likely a css issue.

#[component]
pub fn DestinationSearch(#[prop(optional)] on_close: Option<Callback<()>>) -> impl IntoView {
    let QueryResult {
        data: destinations_resource,
        ..
    } = destinations_query().use_query(|| true); // Query runs when component renders

    let destinations = move || {
        // let destinations = create_local_resource(move || {
        log!("destinations_resource: {:?}", destinations_resource.get());
        destinations_resource.get().flatten().unwrap_or_default()
    };

    let (search_term, set_search_term) = create_signal(String::new());
    let input_ref = create_node_ref::<Input>();

    let filtered_destinations = create_memo(move |_| {
        let term = search_term.get().to_lowercase();
        let all_destinations = destinations();

        if term.is_empty() {
            all_destinations
        } else {
            all_destinations
                .into_iter()
                .filter(|dest| {
                    dest.city.to_lowercase().contains(&term)
                        || dest.country_name.to_lowercase().contains(&term)
                })
                .collect()
        }
    });

    // Helper function to highlight matching text
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

    let clear_search = move |_| {
        set_search_term.set(String::new());
        if let Some(input) = input_ref.get() {
            let _ = input.focus();
        }
    };

    let handle_close = move |_| {
        if let Some(callback) = on_close {
            leptos::Callable::call(&callback, ());
        }
    };
    // Helper function to get icon based on destination type
    let get_destination_icon = |dest: &Destination| -> &str {
        // You can customize this logic based on your needs
        if dest.city.to_lowercase().contains("airport") {
            "✈️"
        } else if dest.city.to_lowercase().contains("railway")
            || dest.city.to_lowercase().contains("station")
        {
            "🏛️"
        } else {
            "📍"
        }
    };

    view! {
        <div class="w-full max-w-sm bg-white min-h-screen font-sans">
            // Header
            <div class="flex justify-between items-center p-4 border-b border-gray-200">
                <h2 class="text-lg font-semibold text-gray-900">"Enter destination"</h2>
                <button
                    class="text-gray-500 hover:text-gray-700 text-2xl w-6 h-6 flex items-center justify-center"
                    on:click=handle_close
                >
                    "×"
                </button>
            </div>

            // Search Bar
            <div class="p-4 border-b border-gray-200">
                <div class="flex items-center bg-gray-50 rounded-lg px-4 py-3">
                    <span class="text-gray-500 mr-3 text-base">"🔍"</span>
                    <input
                        type="text"
                        placeholder="new delhi"
                        class="flex-1 bg-transparent border-none outline-none text-base text-gray-900 placeholder-gray-500"
                        node_ref=input_ref
                        prop:value=move || search_term.get()
                        on:input=move |ev| {
                            set_search_term.set(event_target_value(&ev));
                        }
                    />
                    <Show when=move || !search_term.get().is_empty()>
                        <button
                            class="text-blue-500 text-sm px-2 py-1 rounded hover:bg-gray-100"
                            on:click=clear_search
                        >
                            "Clear"
                        </button>
                    </Show>
                </div>
            </div>

            // Results List
            <div class="flex-1">
                <Suspense fallback=move || view! {
                    <div class="py-10 px-5 text-center text-gray-600 text-base">
                        "Loading destinations..."
                    </div>
                }>
                    {move || {
                        let destinations = destinations_resource.get().flatten().unwrap_or_default();
                        let term = search_term.get().to_lowercase();

                        let filtered_destinations: Vec<Destination> = if term.is_empty() {
                            destinations
                        } else {
                            destinations
                                .into_iter()
                                .filter(|dest| {
                                    dest.city.to_lowercase().contains(&term) ||
                                    dest.country_name.to_lowercase().contains(&term)
                                })
                                .collect()
                        };

                        if filtered_destinations.is_empty() && !term.is_empty() {
                            view! {
                                <div class="py-10 px-5 text-center text-gray-600 text-base">
                                    "No destinations found"
                                </div>
                            }.into_view()
                        } else {
                            view! {
                                <For
                                    each=move || filtered_destinations.clone()
                                    key=|dest| dest.city_id.clone()
                                    children=move |dest| {
                                        let search_term_value = search_term.get();
                                        let icon = get_destination_icon(&dest);
                                        view! {
                                            <div class="flex items-center p-4 border-b border-gray-100 hover:bg-gray-50 cursor-pointer transition-colors">
                                                <span class="text-xl mr-4 w-6 flex justify-center">{icon}</span>
                                                <div class="flex-1">
                                                    <div class="text-base font-medium text-gray-900 mb-1">
                                                        {highlight_text(&dest.city, &search_term_value)}
                                                    </div>
                                                    <div class="text-sm text-gray-600">
                                                        {highlight_text(&dest.country_name, &search_term_value)}
                                                    </div>
                                                </div>
                                            </div>
                                        }
                                    }
                                />
                            }
                        }
                    }}
                </Suspense>
            </div>
        </div>
    }
}

// Usage example:
#[component]
pub fn DestinationPickerV3() -> impl IntoView {
    let (show_search, set_show_search) = create_signal(true);

    let close_search = Callback::new(move |_: ()| {
        set_show_search.set(false);
    });

    view! {
        <div>
            <Show when=move || show_search.get()>
                <DestinationSearch on_close=close_search />
            </Show>

            <Show when=move || !show_search.get()>
                <div class="p-4">
                    <h1 class="text-xl font-bold mb-4">"Main App"</h1>
                    <button
                        class="bg-blue-500 text-white px-4 py-2 rounded hover:bg-blue-600"
                        on:click=move |_| set_show_search.set(true)
                    >
                        "Open Destination Search"
                    </button>
                </div>
            </Show>
        </div>
    }
}
