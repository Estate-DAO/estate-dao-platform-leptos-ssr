use leptos::*;
use leptos_use::{on_click_outside, use_element_hover};

#[component]
pub fn ProfileDropdown(children: Children) -> impl IntoView {
    let dropdown_ref = create_node_ref::<html::Div>();
    let trigger_ref = create_node_ref::<html::Div>();
    let (is_open, set_is_open) = create_signal(false);

    // <!-- Close dropdown when clicking outside -->
    let _ = on_click_outside(dropdown_ref, move |_| set_is_open.set(false));

    // <!-- Toggle dropdown on click -->
    let toggle_dropdown = move |_| {
        set_is_open.update(|open| *open = !*open);
    };

    view! {
        <div class="relative" node_ref=dropdown_ref>
            // <!-- Profile trigger button -->
            <div
                node_ref=trigger_ref
                class="cursor-pointer"
                on:click=toggle_dropdown
            >
                {children()}
            </div>

            // <!-- Dropdown menu -->
            <div
                class=move || {
                    format!(
                        "absolute right-0 top-full mt-2 w-48 bg-white border border-gray-200 rounded-lg shadow-lg z-50 {}",
                        if is_open.get() { "block" } else { "hidden" }
                    )
                }
            >
                <div class="py-2">
                    // <!-- My Bookings option -->
                    <a
                        href="/my-bookings"
                        class="block px-4 py-2 text-sm text-gray-700 hover:bg-gray-100 transition-colors"
                        on:click=move |_| set_is_open.set(false)
                    >
                        "My Bookings"
                    </a>

                    // <!-- Logout option -->
                    <a
                        href="/logout"
                        class="block px-4 py-2 text-sm text-gray-700 hover:bg-gray-100 transition-colors"
                        on:click=move |_| set_is_open.set(false)
                    >
                        "Logout"
                    </a>
                </div>
            </div>
        </div>
    }
}
