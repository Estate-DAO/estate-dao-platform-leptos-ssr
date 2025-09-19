use leptos::*;

#[component]
pub fn PropertyTypeFilter(
    #[prop(into)] options: Signal<Vec<String>>,
    #[prop(into)] selected: Signal<Vec<String>>,
    on_toggle: Callback<String>,
    on_clear: Callback<()>,
) -> impl IntoView {
    let has_selection = Signal::derive(move || !selected().is_empty());

    view! {
        <div>
            <div class="flex items-center justify-between">
                <h3 class="text-base font-medium text-gray-900">"Property Type"</h3>
                <button
                    type="button"
                    class="text-sm text-blue-600 hover:text-blue-700 disabled:text-gray-400 disabled:hover:text-gray-400"
                    disabled=move || !has_selection()
                    on:click=move |_| { leptos::Callable::call(&on_clear, ()); }
                >
                    "Clear"
                </button>
            </div>

            <div class="mt-3 flex flex-col gap-2">
                {move || {
                    let opts = options();
                    let sel = selected();
                    opts.into_iter()
                        .map(|label| {
                            let checked = sel.iter().any(|v| v.eq_ignore_ascii_case(&label));
                            let lbl = label.clone();
                            view! {
                                <label class="flex items-center gap-2 text-sm cursor-pointer">
                                    <input
                                        type="checkbox"
                                        class="form-checkbox h-4 w-4 text-blue-600"
                                        prop:checked=checked
                                        on:change=move |_| {
                                            leptos::Callable::call(&on_toggle, lbl.clone());
                                        }
                                    />
                                    <span class="text-gray-700">{label}</span>
                                </label>
                            }
                        })
                        .collect_view()
                }}
            </div>
        </div>
    }
}
