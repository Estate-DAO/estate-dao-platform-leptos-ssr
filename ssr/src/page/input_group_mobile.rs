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

    // Format date range display (e.g., "04 Nov - 8 Nov")
    let date_range_display = create_memo(move |_| {
        let date_range = search_ctx.date_range.get();
        if date_range.start == (0, 0, 0) || date_range.end == (0, 0, 0) {
            "Add dates".to_string()
        } else {
            let normalized = date_range.normalize();
            normalized.format_dd_month_short()
        }
    });

    // Format guest info display (e.g., "2 Guests")
    let guest_info_display = create_memo(move |_| {
        let guest_selection = &search_ctx.guests;
        let adults = guest_selection.adults.get();
        let children = guest_selection.children.get();
        // let rooms = guest_selection.rooms.get();

        format!(
            "{} {}",
            adults + children,
            if adults + children == 1 {
                "Guest"
            } else {
                "Guests"
            }
        )
    });

    view! {
        // Main wrapper (Card style like the image)
        <div class="w-full max-w-md mx-auto bg-white rounded-xl shadow-md border border-gray-100 p-3 flex items-center justify-between gap-3">

            // Left Content: Destination + Subtitle
            <div class="flex-1 min-w-0 flex flex-col justify-center">
                <div class="text-base font-bold text-gray-900 truncate leading-tight">
                    {move || place_display()}
                </div>
                <div class="text-sm text-gray-500 truncate mt-1 leading-tight">
                    {move || format!("{} • {}", date_range_display.get(), guest_info_display.get())}
                </div>
            </div>

            // Right Content: Search Button
            <button
                type="button"
                class="flex-shrink-0 bg-blue-600 hover:bg-blue-700 text-white rounded-xl w-12 h-12 flex items-center justify-center transition-colors shadow-sm"
                on:click=move |_| {
                    InputGroupState::set_show_full_input(true);
                }
            >
                <Icon icon=icondata::BiSearchRegular class="w-6 h-6" />
            </button>
        </div>
    }
}
