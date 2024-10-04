use crate::component::Divider;
// use leptos::logging::log;
use leptos::*;
use leptos_icons::*;
use web_sys::*;
 
#[component]
pub fn SortBy() -> impl IntoView {
    let (is_open, set_is_open) = create_signal(false);
    let icon = create_memo(move |_| {
        if is_open() {
            icondata::BiChevronUpRegular
        } else {
            icondata::BiChevronDownRegular
        }
    });

    view! {
        <div class="relative">
            <button
                class="bg-white text-black px-4 py-2 rounded-lg flex items-center border border-gray-300"
                // on:blur=move |_| set_is_open.set(false)
                on:click=move |_| set_is_open.update(|open| *open = !*open)
            >
            "Sort by" <Icon icon=icon
            class="w-6 h-6 ml-2"
            />
            </button>

            <Show when=move || is_open()>
                <div class="absolute mt-2 w-52 bg-white borderSortOptions border-gray-300 rounded-xl shadow-lg">
                    <SortOptions/>
                </div>
            </Show>
        </div>
    }
}

#[component]
fn SortOptions() -> impl IntoView {
    let selected = create_rw_signal("Name");

    let apply_selection = move |_| {
        log::info!("Sort By: {}", selected());
        web_sys::console::log_1(&format!(
            "Sort By: {}",
            selected()
        )
        .into());
    };

    view! {
        <form class="p-4">
            <div class="space-y-4">
                <SortOption name="sort" value="Prices high to low" selected=selected/>
        <Divider />

                <SortOption name="sort" value="Prices low to high" selected=selected/>
        <Divider />

                <SortOption name="sort" value="Rating high to low" selected=selected/>
            </div>
            <button type="button" class="w-full mt-4 bg-white border border-black-2 text-black px-4 py-2 rounded-full" on:click=apply_selection>
                "Apply"
            </button>
        </form>
    }
}

#[component]
fn SortOption(
    name: &'static str,
    value: &'static str,
    selected: RwSignal<&'static str>,
) -> impl IntoView {
    view! {
        <label class="flex items-center space-x-2 cursor-pointer">
            <input
                type="radio"
                name=name
                value=value
                checked=move || selected.get() == value
                on:change=move |_| selected.set(value)
            />
            <span>{value}</span>
        </label>
    }
}
