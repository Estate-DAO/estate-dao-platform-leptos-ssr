use leptos::*;

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
