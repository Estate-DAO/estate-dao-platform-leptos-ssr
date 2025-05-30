use super::*;

#[component]
pub fn DestinationPicker() -> impl IntoView {
    let is_open = create_memo(move |_| {
        // log!("is_open called");
        InputGroupState::is_destination_open()
    });
    let search_ctx: SearchCtx = expect_context();

    let QueryResult {
        data: destinations_resource,
        state,
        // is_loading,
        // is_fetching,
        // is_invalid,
        ..
    } = destinations_query().use_query(move || is_open.get());

    let display_value = create_memo(move |_| {
        search_ctx
            .destination
            .get()
            .map(|d| format!("{}, {}", d.city, d.country_name))
            .unwrap_or_else(|| "Where to?".to_string())
    });

    view! {
        // !<-- Main wrapper with relative positioning -->
        <div class="relative w-full h-full">
            // !<-- Input slot with consistent height -->
            <div class="w-full h-full px-4">
                <div class="absolute inset-y-0 left-2 flex items-center text-xl pl-6">
                    <Icon icon=icondata::BsMap class="text-black" />
                </div>

                <button
                    class="w-full h-full flex items-center pl-12 text-black bg-transparent rounded-full transition-colors text-sm"
                    on:click=move |_| {
                        log!("clicked CityListComponent");
                        InputGroupState::toggle_dialog(OpenDialogComponent::CityListComponent);
                    }
                >
                    {display_value}
                </button>
            </div>

            <Show when=move || is_open()>
                // !<-- Main Modal Container -->
                <div
                    class="fixed inset-0 z-[97]"
                    on:click=move |_| InputGroupState::toggle_dialog(OpenDialogComponent::CityListComponent)
                >
                    // !<-- Content Container -->
                    <div
                        class="fixed bottom-0 left-0 right-0 top-auto md:absolute md:top-full md:left-0 md:bottom-auto md:max-w-[33%] md:w-1/3 z-[98] max-h-[90vh] overflow-y-auto rounded-t-lg md:rounded-lg box-border"
                        on:click=|e| e.stop_propagation()
                    >
                        // <div class="bg-gray-300 md:mt-1 md:rounded-lg md:border md:border-gray-200 md:shadow-lg">
                        <div class="bg-white md:mt-1 md:rounded-lg md:border md:border-gray-200 md:shadow-lg">
                            // !<-- Mobile Header -->
                            <div class="flex items-center justify-between p-4 border-b border-gray-200 sticky top-0 bg-white z-10 rounded-t-lg md:hidden">
                                <button
                                    class="text-gray-800 hover:bg-gray-100 p-2 rounded-full transition-colors"
                                    on:click=move |_| InputGroupState::toggle_dialog(OpenDialogComponent::CityListComponent)
                                >
                                    <Icon icon=icondata::BiXRegular class="text-2xl" />
                                </button>
                                <h2 class="text-lg font-medium">Select Destination</h2>
                                <div class="w-10"></div>
                            </div>

                            // !<-- Dropdown content -->
                            <div class="md:max-h-[280px] md:overflow-auto">
                                <div class="p-2">
                                    <Suspense fallback=move || {
                                        view! {
                                            <div class="flex justify-center items-center h-32">
                                                <span class="text-gray-500">"Loading..."</span>
                                            </div>
                                        }
                                    }>
                                        {move || {
                                            destinations_resource
                                                .get()
                                                .map(|dest_vec| {
                                                    view! {
                                                        <ShowDestinations
                                                            dest_vec=dest_vec.unwrap_or_default()
                                                        />
                                                    }
                                                })
                                        }}
                                    </Suspense>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}

#[component]
fn ShowDestinations(dest_vec: Vec<Destination>) -> impl IntoView {
    view! {
        // !<-- Scrollable container -->
        <div class="h-[calc(100vh-8rem)] md:h-auto overflow-y-auto hide-scrollbar">
            {move || {
                dest_vec
                    .clone()
                    .into_iter()
                    .map(|dest| {
                        let country = dest.country_name.clone();
                        let city = dest.city.clone();
                        view! {
                            <div
                                class="cursor-pointer hover:bg-gray-50 active:bg-gray-100 py-3 px-3 transition-colors"
                                on:click=move |_| {
                                    SearchCtx::set_destination(dest.clone());
                                    InputGroupState::toggle_dialog(OpenDialogComponent::CityListComponent);
                                }
                            >
                                <div class="flex flex-col">
                                    <span class="text-gray-900 text-sm font-medium">
                                        {&city}
                                    </span>
                                    <span class="text-gray-500 text-xs">
                                        {&country}
                                    </span>
                                </div>
                                <Divider />
                            </div>
                        }
                    })
                    .collect_view()
            }}
        </div>
    }
}
