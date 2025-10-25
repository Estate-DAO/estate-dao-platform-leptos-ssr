use crate::log;
use leptos::prelude::*;

#[component]
pub fn NumberCounter(
    #[prop(into)] label: String,
    #[prop(default = "".into() , into)] class: String,
    counter: RwSignal<u32>,
    on_increment: impl Fn() + 'static,
) -> impl IntoView {
    let merged_class = format!("flex items-center justify-between {}", class);

    view! {
        <div class=merged_class>
            <p>{label}</p>
            <div class="flex items-center space-x-1">
                <button
                    class="ps-2 py-1 text-2xl"
                    on:click=move |_| counter.update(|n| *n = if *n > 0 { *n - 1 } else { 0 })
                >
                    {"\u{2003}\u{2003}\u{2003}\u{2003}-"}
                </button>
                <input
                    type="number"
                    prop:value=move || counter.get().to_string()
                    on:input=move |ev| {
                        let value = event_target_value(&ev).parse().unwrap_or(0);
                        counter.set(value.max(0));
                    }
                    class=format!(
                        "{} text-center w-6",
                        "[appearance:textfield] [&::-webkit-outer-spin-button]:appearance-none [&::-webkit-inner-spin-button]:appearance-none ",
                    )
                />
                <button class="py-1 text-2xl" on:click=move |_| on_increment()>
                    "+"
                </button>
            </div>
        </div>
    }
}

#[component]
pub fn NumberCounterV2(
    #[prop(into)] label: String,
    #[prop(default = "".into(), into)] class: String,
    counter: RwSignal<u32>,
    on_increment: impl Fn() + 'static,
    #[prop(optional)] on_decrement: Option<impl Fn() + 'static>,
    #[prop(optional, into)] min: Option<u32>,
) -> impl IntoView {
    let merged_class = format!("flex items-center justify-between {}", class);
    let min_value = min.unwrap_or(0);
    let is_min = Memo::new(move |_| {
        let current_value = counter.get();
        let at_min = current_value <= min_value;
        // log!(
        //     "[NumberCounterV2] Checking if at minimum: {} <= {} = {}",
        //     current_value,
        //     min_value,
        //     at_min
        // );
        at_min
    });

    let decrement = {
        let on_decrement = on_decrement;
        move |_| {
            if is_min.get() {
                // log!("[NumberCounterV2] Already at or below minimum, do nothing");
                return; // Already at or below minimum, do nothing
            }

            if let Some(ref decr) = on_decrement {
                // log!("[NumberCounterV2] Decrementing via callback");
                decr();
            } else {
                // log!("[NwumberCounterV2] Decrementing directly");
                counter.update(|n| *n = n.saturating_sub(1));
            }
        }
    };

    view! {
        <div class=merged_class>
            <p>{label}</p>
            <div class="flex items-center space-x-1">
                <button
                    class=move || format!(
                        "ps-2 py-1 text-2xl {}",
                        if is_min.get() { "opacity-50 cursor-not-allowed" } else { "" }
                    )
                    on:click=move |arg| decrement(arg)
                    disabled=move || is_min.get()
                >
                    {"\u{2003}\u{2003}\u{2003}\u{2003}-"}
                </button>
                <input
                    type="number"
                    prop:value=move || counter.get().to_string()
                    min=min_value
                    on:input=move |ev| {
                        let value = event_target_value(&ev).parse().unwrap_or(min_value);
                        counter.set(value.max(min_value));
                    }
                    class=format!(
                        "{} text-center w-6",
                        "[appearance:textfield] [&::-webkit-outer-spin-button]:appearance-none [&::-webkit-inner-spin-button]:appearance-none"
                    )
                />
                <button class="py-1 text-2xl" on:click=move |_| on_increment()>
                    "+"
                </button>
            </div>
        </div>
    }
}
