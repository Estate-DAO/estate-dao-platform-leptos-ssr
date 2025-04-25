use crate::log;
use crate::page::{InputGroup, InputGroupMobile};
use leptos::*;

#[component]
pub fn InputGroupContainer(
    #[prop(optional, into)] given_disabled: MaybeSignal<bool>,
    #[prop(optional, into)] default_expanded: MaybeSignal<bool>,
    #[prop(optional, into)] allow_outside_click_collapse: MaybeSignal<bool>, // New prop
) -> impl IntoView {
    // Signal to track if the detailed input group is open on mobile
    let show_full_input = create_rw_signal(default_expanded.get());

    log!(
        "[input_group_container.rs] Initial show_full_input: {}",
        show_full_input.get()
    );

    view! {
            <Show when=move || show_full_input.get()>
                // Mobile: show full InputGroup when expanded
                // <div class="fixed inset-0 z-[9999] flex items-center justify-center bg-black/20 backdrop-blur-sm">
                //     <div class="w-full max-w-xl mx-auto">
                        <InputGroup given_disabled=given_disabled />
                    // </div>
                    <Show when=move || allow_outside_click_collapse.get()>
                        <div
                            class="fixed inset-0"
                            on:click=move |ev| {
                                // Stop propagation to prevent interference with other click handlers
                                ev.stop_propagation();
                                log!("[input_group_container.rs] Overlay clicked, setting show_full_input to false");
                                show_full_input.set(false);
                            }
                        ></div>
                    </Show>
                // </div>
            </Show>
            <Show when=move || !show_full_input.get()>
                // Mobile: show compact InputGroupMobile by default
                <div
                    on:click=move |_| {
                        log!("[input_group_container.rs] Mobile view clicked, setting show_full_input to true");
                        show_full_input.set(true);
                    }
                >
                    <InputGroupMobile />
                </div>
            </Show>
    }
}
