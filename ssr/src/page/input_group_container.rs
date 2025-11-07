use crate::log;
use crate::page::{InputGroup, InputGroupMobile};
use crate::utils::responsive::use_is_desktop;
use crate::view_state_layer::input_group_state::InputGroupState;
use leptos::*;
use web_sys;

#[component]
pub fn InputGroupContainer(
    #[prop(optional, into)] given_disabled: MaybeSignal<bool>,
    #[prop(optional, into)] default_expanded: MaybeSignal<bool>,
    #[prop(optional, into)] allow_outside_click_collapse: MaybeSignal<bool>, // New prop
    #[prop(optional, into)] h_class: MaybeSignal<String>,
    #[prop(optional, into)] size: MaybeSignal<String>,
) -> impl IntoView {
    // Signal to track if the detailed input group is open on mobile
    let is_desktop = use_is_desktop();

    InputGroupState::set_show_full_input(default_expanded.get());

    let show_full_input = create_memo(move |_prev| {
        log!(
            "[input_group_container.rs] Derived show_full_input: {}",
            is_desktop.get() || default_expanded.get()
        );
        is_desktop.get() || default_expanded.get() || InputGroupState::is_open_show_full_input()
    });

    view! {
        <Show when=move || show_full_input.get()>
            // Mobile: show full InputGroup when expanded
            // <div class="fixed inset-0 z-[9999] flex items-center justify-center bg-black/20 backdrop-blur-sm">
            //     <div class="w-full max-w-xl mx-auto">
            <InputGroup given_disabled=given_disabled h_class=h_class.clone() size=size.clone() />
            // </div>
            <Show when=move || allow_outside_click_collapse.get()>
                <div
                    class="fixed inset-0"
                    style="pointer-events: none;"
                    on:click=move |ev| {
                        // ev.prevent_default();
                        // ev.stop_propagation();
                        log!("[input_group_container.rs] Overlay clicked");
                        InputGroupState::set_show_full_input(false);
                    }
                ></div>
            </Show>
        // </div>
        </Show>
        <Show when=move || !show_full_input.get()>
            // Mobile: show compact InputGroupMobile by default
            <div
                on:click=move |_| {
                    log!(
                        "[input_group_container.rs] Mobile view clicked, setting show_full_input to true"
                    );
                    InputGroupState::set_show_full_input(true);
                }
            >
                <InputGroupMobile/>
            </div>
        </Show>
    }
}
