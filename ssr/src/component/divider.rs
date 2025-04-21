use leptos::*;

#[component]
pub fn Divider() -> impl IntoView {
    view! {
        // <div class="h-px bg-gray-300 my-4"></div>
        <div class="border-b border-gray-100 mt-4 md:mt-3"></div>
    }
}
