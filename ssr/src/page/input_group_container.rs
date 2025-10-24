use crate::log;
use crate::page::{InputGroup, InputGroupMobile};
use crate::utils::responsive::use_is_desktop;
use crate::view_state_layer::input_group_state::InputGroupState;
use leptos::prelude::*;
use web_sys;

#[component]
pub fn InputGroupContainer(
    #[prop(optional, into)] given_disabled: Signal<bool>,
    #[prop(optional, into)] default_expanded: Signal<bool>,
    #[prop(optional, into)] allow_outside_click_collapse: Signal<bool>, // New prop
) -> impl IntoView {
    // Signal to track if the detailed input group is open on mobile
    let is_desktop = use_is_desktop();

    InputGroupState::set_show_full_input(default_expanded.get());

    let show_full_input = Memo::new(move |_prev| {
        log!(
            "[input_group_container.rs] Derived show_full_input: {}",
            is_desktop.get() || default_expanded.get()
        );
        is_desktop.get() || default_expanded.get() || InputGroupState::is_open_show_full_input()
    });

    view! {
        <Show when=move || show_full_input.get()>
            {view! {
                <InputGroup given_disabled=given_disabled />
                <Show when=move || allow_outside_click_collapse.get()>
                    <div
                        class="fixed inset-0"
                        style="pointer-events: none;"
                        on:click=move |_| {
                            InputGroupState::set_show_full_input(false);
                        }
                    ></div>
                </Show>
            }.into_any()}
        </Show>
        <Show when=move || !show_full_input.get()>
            <div
                on:click=move |_| {
                    InputGroupState::set_show_full_input(true);
                }
            >
                <InputGroupMobile />
            </div>
        </Show>
    }
}
