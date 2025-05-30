use super::*;

// Here are the observations about what works and what does not in the expected behaviour of the component
// 1. [fully_functional] when user clicks on the input, the dropdown opens
// 2. [fully_functional] when user types, the city list is filtered and higlighted
// 3. when user clicks on a city from the dropdown list
//    3.1 [not_functional] But the city is not filled back in the input. The city should fill back into the input when selected.
//    3.2 [fully_functional] the city is selected and the dropdown closes.
// 4. [not_functional] when user re-opens the dropdown (by clicking on the input again), The input city (from step 3) should be filled back into the input.
// 5. [not_functional] corollary to 4, if the user deletes a few character from the input after he has reopened the dropdown, the search dropdown should filter the original list accordginly. In other words, filter and highlight should be triggered on each keystroke to the input when user is typing.
// 6. [not_functional] When user click away from the input, the dropdown does not close automatically. it sould close when user clicks outside the input.
// 7. [fully_functional] css is as expected. so look and feel are as per design.

#[component]
pub fn DestinationPickerV2() -> impl IntoView {
    let search_ctx: SearchCtx = expect_context();

    let QueryResult {
        data: destinations_resource,
        ..
    } = destinations_query().use_query(|| true); // Query runs when component renders

    // Create signals to manage dropdown state
    let is_open = create_rw_signal(false);

    // Create effects to sync LiveSelect state with InputGroupState
    create_effect(move |_| {
        if is_open.get() {
            // When dropdown opens, update InputGroupState
            InputGroupState::set_destination_open();
        } else {
            // When dropdown closes, update InputGroupState if it was already open
            if InputGroupState::is_destination_open() {
                InputGroupState::set_close_dialog();
            }
        }
    });

    // Also create an effect to sync InputGroupState with LiveSelect
    create_effect(move |_| {
        let is_destination_open = InputGroupState::is_destination_open();
        if is_destination_open != is_open.get_untracked() {
            is_open.set(is_destination_open);
        }
    });

    view! {
        <div class="relative flex w-full md:w-[274px] h-full"> // Main container
            <div class="absolute inset-y-0 left-2 py-6  px-6 md:py-4 text-xl pointer-events-none">
                <Icon icon=icondata::BsMap class="text-black font-bold" />
            </div>
            <LiveSelect<Destination>
                options=Signal::derive(move || {
                    destinations_resource.get().flatten().unwrap_or_default()
                })
                value=search_ctx.destination
                set_value=Callback::new(move |dest: Destination| {
                    let _ = SearchCtx::set_destination(dest);
                    // Close the dropdown after selection
                    InputGroupState::toggle_dialog(OpenDialogComponent::None);
                })
                label_fn=Callback::new(|dest: Destination| format!("{}, {}", dest.city, dest.country_name))
                value_fn=Callback::new(|dest: Destination| dest.city_id.clone())
                placeholder="Where to?"
                id="destination-live-select"
                class="w-full h-full items-center"
                input_class="w-full h-full pl-10 text-[15px] leading-[18px] text-gray-900 bg-transparent rounded-full transition-colors focus:outline-none py-6"
                dropdown_class="mt-2"
            />
            // LiveSelect manages its own dropdown, but we sync with InputGroupState
        </div>
    }
}
