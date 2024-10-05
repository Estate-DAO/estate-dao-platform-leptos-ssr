use leptos::*;
use leptos_icons::*;

#[component]
pub fn DateTimeRangePicker() -> impl IntoView {
    let (is_open, set_is_open) = create_signal(false);
    let selected_range = create_rw_signal("Exact dates");

    view! {
        <div class="absolute inset-y-0 left-2 flex items-center text-2xl z-[15]">
            <Icon icon=icondata::AiCalendarOutlined class="text-black font-light" />
        </div>

        <button
            class="w-full ml-2 py-2 pl-8 text-black bg-transparent border-none focus:outline-none text-sm text-left"
            on:click=move |_| set_is_open.update(|open| *open = !*open)
        >
            Check in - Check out
        </button>

        <Show when=move || is_open()>
            <div class="absolute mt-6 w-full  bg-white border border-gray-200 rounded-xl shadow-lg p-4 z-[15]">
                <div class="grid grid-cols-7 gap-2 justify-items-center">
                    // Placeholder for date cells
                    { (1..=30).map(|day| view! {
                        <div class="border p-2 cursor-pointer hover:bg-gray-100">
                            {day}
                        </div>
                    }).collect::<Vec<_>>() }
                </div>
                <div class="flex justify-between mt-4">
                    { ["Exact dates", "+/- 1 day", "+/- 2 days", "+/- 3 days", "+/- 7 days"].iter().map(|option| {
                        let option_name = option.clone();
                        let is_selected = move || selected_range.get() == option_name;
                        view! {
                            <button
                                class=format!(
                                    "px-4 py-2 rounded {}",
                                    if is_selected() {
                                        "bg-blue-500 text-white"
                                    } else {
                                        "bg-gray-200 text-black"
                                    }
                                )
                                on:click=move |_| selected_range.set(option_name)
                            >
                                {option_name}
                            </button>
                        }
                    }).collect::<Vec<_>>() }
                </div>
            </div>
        </Show>
    }
}