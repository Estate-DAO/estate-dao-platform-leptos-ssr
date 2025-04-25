use crate::page::{InputGroup, InputGroupMobile};
use leptos::*;

#[component]
pub fn InputGroupContainer(
    #[prop(optional, into)] given_disabled: MaybeSignal<bool>,
    #[prop(optional, into)] default_expanded: MaybeSignal<bool>,
) -> impl IntoView {
    // Signal to track if the detailed input group is open on mobile
    let show_full_input = create_rw_signal(default_expanded.get());

    view! {
            <Show when=move || show_full_input.get()>
                // Mobile: show full InputGroup when expanded
                // <div class="fixed inset-0 z-[9999] flex items-center justify-center bg-black/20 backdrop-blur-sm">
                //     <div class="w-full max-w-xl mx-auto">
                        <InputGroup given_disabled=given_disabled />
                    // </div>
                    <div class="fixed inset-0" on:click=move |_| show_full_input.set(false)></div>
                // </div>
            </Show>
            <Show when=move || !show_full_input.get()>
                // Mobile: show compact InputGroupMobile by default
                <div on:click=move |_| show_full_input.set(true)>
                    <InputGroupMobile />
                </div>
            </Show>
    }
}
