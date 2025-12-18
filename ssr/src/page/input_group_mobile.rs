use crate::component::SelectedDateRange;
use crate::utils::pluralize;
use crate::view_state_layer::input_group_state::InputGroupState;
use crate::view_state_layer::ui_search_state::UISearchCtx;
use leptos::*;
use leptos_icons::*;

#[component]
pub fn InputGroupMobile() -> impl IntoView {
    // Get actual data from search context
    let search_ctx: UISearchCtx = expect_context();

    // Create derived signals for display values
    let place_display = move || {
        search_ctx
            .place
            .get()
            .map(|d| d.display_name)
            .unwrap_or_else(|| "Where to?".to_string())
    };

    // Format date range display (e.g., "04 November - 08 November")
    let date_range_display = create_memo(move |_| {
        let date_range = search_ctx.date_range.get();
        if date_range.start == (0, 0, 0) || date_range.end == (0, 0, 0) {
            "Add dates".to_string()
        } else {
            let normalized = date_range.normalize();
            normalized.format_dd_month_yyyy()
        }
    });

    // Format guest info display (e.g., "1 Room • 2 Adults • 0 Children")
    let guest_info_display = create_memo(move |_| {
        let guest_selection = &search_ctx.guests;
        let adults = guest_selection.adults.get();
        let children = guest_selection.children.get();
        let rooms = guest_selection.rooms.get();

        format!(
            "{} • {} • {}",
            pluralize(rooms, "Room", "Rooms"),
            pluralize(adults, "Adult", "Adults"),
            pluralize(children, "Child", "Children")
        )
    });

    view! {
        // Main wrapper (rounded corners, shadow, white bg)
        <div class="flex flex-col w-full max-w-md mx-auto bg-white rounded-2xl shadow-lg border border-gray-100 overflow-hidden">

            // Destination row
            <div class="flex items-start gap-4 px-5 py-4 border-b border-gray-100">
                <div class="text-blue-500 mt-1">
                    <Icon icon=icondata::BsGeoAlt class="w-6 h-6" />
                </div>
                <div class="flex flex-col flex-1">
                    <span class="text-xs text-gray-500 font-medium">"Destination"</span>
                    <span class="text-base font-semibold text-gray-900 mt-0.5">
                        {move || place_display()}
                    </span>
                </div>
            </div>

            // Date row
            <div class="flex items-start gap-4 px-5 py-4 border-b border-gray-100">
                <div class="text-blue-500 mt-1">
                    <Icon icon=icondata::BiCalendarRegular class="w-6 h-6" />
                </div>
                <div class="flex flex-col flex-1">
                    <span class="text-xs text-gray-500 font-medium">"Date"</span>
                    <span class="text-base font-semibold text-gray-900 mt-0.5">
                        {move || date_range_display.get()}
                    </span>
                </div>
            </div>

            // Guests row
            <div class="flex items-start gap-4 px-5 py-4 border-b border-gray-100">
                <div class="text-blue-500 mt-1">
                    <Icon icon=icondata::BiUserRegular class="w-6 h-6" />
                </div>
                <div class="flex flex-col flex-1">
                    <span class="text-xs text-gray-500 font-medium">"Guests"</span>
                    <span class="text-base font-semibold text-gray-900 mt-0.5">
                        {move || guest_info_display.get()}
                    </span>
                </div>
            </div>

            // See Availability button
            <div class="px-5 py-4">
                <button
                    type="button"
                    class="w-full bg-blue-500 hover:bg-blue-600 text-white font-semibold py-3 px-6 rounded-full transition-colors"
                    on:click=move |_| {
                        InputGroupState::set_show_full_input(true);
                    }
                >
                    "See Availability"
                </button>
            </div>
        </div>
    }
}
