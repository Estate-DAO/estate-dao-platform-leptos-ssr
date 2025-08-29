use leptos::*;

#[component]
pub fn Divider(#[prop(optional)] class: MaybeSignal<String>) -> impl IntoView {
    view! {
        // <div class="h-px bg-gray-300 my-4"></div>
        <div class=move || format!("border-b border-gray-100 {}", class.get())></div>
        // <div class="border-b border-gray-100 mt-3 md:mt-2"></div>
    }
}
