use crate::utils::pluralize;
use crate::view_state_layer::ui_search_state::UISearchCtx;
use leptos::*;
use leptos_icons::*;

#[component]
pub fn InputGroupMobile() -> impl IntoView {
    // Get actual data from search context
    let search_ctx: UISearchCtx = expect_context();

    // Create derived signals for display values
    let destination_display = create_memo(move |_| {
        search_ctx
            .destination
            .get()
            .map(|d| format!("{}", d.city))
            .unwrap_or_else(|| "Where to?".to_string())
    });

    // Format date range display
    let date_range_display = create_memo(move |_| {
        let date_range = search_ctx.date_range.get();
        if date_range.start == (0, 0, 0) || date_range.end == (0, 0, 0) {
            "Add dates".to_string()
        } else {
            let normalized = date_range.normalize();
            let nights = normalized.no_of_nights();
            let formatted = normalized.format_mmm_dd();

            if nights > 0 {
                format!("{} ({})", formatted, pluralize(nights, "d", "d"))
            } else {
                formatted
            }
        }
    });

    // Format guest info display
    let guest_info_display = create_memo(move |_| {
        let guest_selection = &search_ctx.guests;
        let adults = guest_selection.adults.get();
        let children = guest_selection.children.get();
        let rooms = guest_selection.rooms.get();

        format!(
            "{}, {}",
            pluralize(rooms, "Room", "Rooms"),
            pluralize(adults + children, "Guest", "Guests")
        )
    });

    view! {
        // Main wrapper (rounded, shadow, border, white bg)
        <div class="flex flex-col mb-4 w-full max-w-xl bg-white rounded-full shadow-md border border-gray-200 px-4 py-3 items-center">
            <div class="flex items-center w-full">
                <span class="text-2xl text-gray-400 mr-3">
                    <Icon icon=icondata::BsSearch />
                </span>
                <div class="flex flex-col flex-1">
                    <span class="text-lg font-medium text-gray-900 leading-tight">
                        {move || destination_display.get()}
                    </span>
                    <span class="text-sm text-gray-500 mt-1">
                        {move || format!("{} - {}", date_range_display.get(), guest_info_display.get())}
                    </span>
                </div>
            </div>
        </div>
    }
}
