use leptos::*;
use leptos::logging::log;

use crate::component::{Divider, HSettingIcon};
use leptos_icons::*;
use wasm_bindgen::JsCast;
use web_sys::MouseEvent;

/// Filter component (button)
#[component]
pub fn Filter() -> impl IntoView {
    let (is_open, set_is_open) = create_signal(false);

    view! {
        <div class="">
            <button
                class="bg-white text-black px-4 py-2 rounded-lg flex items-center border border-gray-300"
                on:click=move |_| set_is_open.update(|open| *open = !*open)
            >
                <Icon class="w-5 h-5 mr-2" icon=HSettingIcon />
                Filters
            </button>

            <Show when=move || is_open()>
                <div class="absolute mt-2 w-52 bg-white borderSortOptions border-gray-300 rounded-xl shadow-lg">
                    <FilterOptions />
                </div>
            </Show>
        </div>
    }
}

#[component]
fn FilterOptions() -> impl IntoView {
    let selected_filter = create_rw_signal("PriceRange".to_string());
    let set_filter = move |filter: String| {
        selected_filter.set(filter);
    };

    let (is_open, set_is_open) = create_signal(true);

    view! {
        <Show when=move || is_open()>
            <div class="fixed inset-0 flex">
                <div class="flex-1 bg-black opacity-80"></div>
                <div class="w-1/2 bg-white shadow-lg flex flex-col">
                    <div class="p-8 border-b bg bg-gray-100 px-2">
                        <h2 class="text-4xl font-semibold">Filters</h2>
                    </div>
                    <div class="flex space-x-40 p-6 px-16">
                        {["PriceRange", "Amenities", "PropertyType", "Ratings"]
                            .iter()
                            .map(|&filter| {
                                let is_selected = move || selected_filter.get() == filter;
                                view! {
                                    <button
                                        class=format!(
                                            "text-lg font-medium relative {}",
                                            if is_selected() {
                                                "font-semibold text-black"
                                            } else {
                                                "text-gray-700"
                                            },
                                        )
                                        on:click=move |_| set_filter(filter.to_string())
                                    >
                                        {filter}
                                        <span
                                            class="absolute left-0 right-0 bottom-[-.5rem] h-0.5 bg-black"
                                            style=if is_selected() {
                                                "display: block;"
                                            } else {
                                                "display: none;"
                                            }
                                        ></span>
                                    </button>
                                }
                            })
                            .collect::<Vec<_>>()}
                    </div>
                    <div class="flex-1 p-4">
                        <Show when=move || selected_filter.get() == "PriceRange">
                            <PriceRange />
                        </Show>
                        <Show when=move || selected_filter.get() == "Amenities">
                            <Amenities />
                        </Show>
                        <Show when=move || selected_filter.get() == "PropertyType">
                            <PropertyType />
                        </Show>
                        <Show when=move || selected_filter.get() == "Ratings">
                            <Ratings />
                        </Show>
                    </div>
                    <div class="p-12 flex justify-start space-x-4">
                        <button
                            type="button"
                            class=" cursor-pointer border border-blue-500 text-black px-4 py-2 rounded-full text-blue-500"
                            on:click={move |_|{
                                log::info!("clear all button");
                                set_is_open.set(false)}}
                        >
                            Clear All
                        </button>
                        <button
                            type="button"
                            class="bg-blue-600 cursor-pointer text-white px-4 py-2 rounded-full"
                            on:click={move |_|{
                                log::info!("apply filter button");
                                set_is_open.set(false)}}
                        >
                            Apply Filters
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}

#[component]
fn PriceRange() -> impl IntoView {
    let (min_value, set_min_value) = create_signal(480);
    let (max_value, set_max_value) = create_signal(20000);
    let (dragging, set_dragging) = create_signal(None);

    let slider_ref = create_node_ref::<html::Div>();

    let on_mouse_down = move |e: MouseEvent, handle: &str| {
        set_dragging.set(Some(handle.to_string()));
        e.prevent_default();
    };

    let on_mouse_move = move |e: MouseEvent| {
        if let Some(slider) = slider_ref.get() {
            let rect = slider.get_bounding_client_rect();
            let x = e.client_x() as f64 - rect.left();
            let percentage = (x / rect.width() * 100.0).clamp(0.0, 100.0);
            let value = (percentage / 100.0 * 19991.0 + 9.0) as i32;

            match dragging.get().as_deref() {
                Some("min") => set_min_value.set(value.min(max_value.get() - 1)),
                Some("max") => set_max_value.set(value.max(min_value.get() + 1)),
                _ => {}
            }
        }
    };

    let on_mouse_up = move |_| set_dragging.set(None);

    view! {
        <div class="flex flex-col items-center space-x-2 cursor-pointer p-8">
            <img src="/img/graph.webp" alt="graph" class="w-full h-full" />
            <div
                class="relative w-full h-12 mt-8 mb-4"
                on:mousemove=on_mouse_move
                on:mouseup=on_mouse_up
                on:mouseleave=on_mouse_up
                ref=slider_ref
            >
                5
                <div class="absolute w-full top-1/2 h-1 bg-gray-300 rounded"></div>
                <div
                    class="absolute top-1/2 h-1 bg-blue-500 rounded"
                    style:left=move || {
                        format!("{}%", (min_value.get() - 9) as f64 / 19991.0 * 100.0)
                    }
                    style:width=move || {
                        format!("{}%", (max_value.get() - min_value.get()) as f64 / 19991.0 * 100.0)
                    }
                ></div>
                <div
                    class="absolute top-1/2 w-6 h-6 -mt-3 -ml-3 bg-white border-2 border-blue-500 rounded-full cursor-pointer"
                    style:left=move || {
                        format!("{}%", (min_value.get() - 9) as f64 / 19991.0 * 100.0)
                    }
                    on:mousedown=move |e| on_mouse_down(e, "min")
                ></div>
                <div
                    class="absolute top-1/2 w-6 h-6 -mt-3 -ml-3 bg-white border-2 border-blue-500 rounded-full cursor-pointer"
                    style:left=move || {
                        format!("{}%", (max_value.get() - 9) as f64 / 19991.0 * 100.0)
                    }
                    on:mousedown=move |e| on_mouse_down(e, "max")
                ></div>
            </div>
            <br />
            <div class="flex flex-row items-center w-full">
                <div class="border border-gray-200 w-80 h-20 rounded-full flex-none px-8 py-2">
                    <p class="text-gray-500">Minimum</p>
                    <p class="text-2xl">{"Rs. "}{min_value}</p>
                </div>
                <div class="h-0.5 w-40 bg-gray-200 mt-4 grow"></div>
                <div class="border border-gray-200 w-80 h-20 rounded-full flex-none px-8 py-2">
                    <p class="text-gray-500">Maximum</p>
                    <p class="text-2xl">{"Rs. "}{max_value}</p>
                </div>
            </div>
        </div>
    }
}

#[component]
fn Amenities() -> impl IntoView {
    view! {
        <div class="flex px-12">
            <div class="w-1/2 flex flex-col space-y-2">
                <label class="flex items-center space-x-2 cursor-pointer py-2">
                    <input type="checkbox" class="form-checkbox" />
                    <span>Wifi</span>
                </label>
                <label class="flex items-center space-x-2 cursor-pointer py-2">
                    <input type="checkbox" class="form-checkbox" />
                    <span>Air Conditioning</span>
                </label>
                <label class="flex items-center space-x-2 cursor-pointer py-2">
                    <input type="checkbox" class="form-checkbox" />
                    <span>Washing Machine</span>
                </label>
            </div>
            <div class="w-1/2 flex flex-col space-y-2">
                <label class="flex items-center space-x-2 cursor-pointer py-2">
                    <input type="checkbox" class="form-checkbox" />
                    <span>Kitchen</span>
                </label>
                <label class="flex items-center space-x-2 cursor-pointer py-2">
                    <input type="checkbox" class="form-checkbox" />
                    <span>Free Parking</span>
                </label>
                <label class="flex items-center space-x-2 cursor-pointer py-2">
                    <input type="checkbox" class="form-checkbox" />
                    <span>TV</span>
                </label>
            </div>
        </div>
    }
}

#[component]
fn PropertyType() -> impl IntoView {
    let selected_type = create_rw_signal(None::<String>);

    let toggle_selection = move |property_type: &str| {
        selected_type.set(Some(property_type.to_string()));
    };

    let is_selected =
        move |property_type: &str| selected_type.get().as_deref() == Some(property_type);
    view! {
        <div class="flex px-12">
            <div class="w-1/2 flex flex-col space-y-4">
                <button
                    class=move || {
                        format!(
                            "border w-80 h-20 rounded-full flex px-8 py-6 {}",
                            if is_selected("House") {
                                "border-blue-500 text-blue-500"
                            } else {
                                "border-gray-200 text-black"
                            },
                        )
                    }
                    on:click=move |_| toggle_selection("House")
                >
                    <Icon icon=icondata::IoHomeOutline class="text-xl" />
                    <span class="ml-2">House</span>
                </button>
                <button
                    class=move || {
                        format!(
                            "border w-80 h-20 rounded-full flex px-8 py-6 {}",
                            if is_selected("Flat") {
                                "border-blue-500 text-blue-500"
                            } else {
                                "border-gray-200 text-black"
                            },
                        )
                    }
                    on:click=move |_| toggle_selection("Flat")
                >
                    <Icon icon=icondata::BsBuilding class="text-xl" />
                    <span class="ml-2">Flat</span>
                </button>
            </div>
            <div class="w-1/2 flex flex-col space-y-4">
                <button
                    class=move || {
                        format!(
                            "border w-80 h-20 rounded-full flex px-8 py-6 {}",
                            if is_selected("Guest house") {
                                "border-blue-500 text-blue-500"
                            } else {
                                "border-gray-200 text-black"
                            },
                        )
                    }
                    on:click=move |_| toggle_selection("Guest house")
                >
                    <Icon icon=icondata::SiHomeassistantcommunitystore class="text-xl" />
                    <span class="ml-2">Guest house</span>
                </button>
                <button
                    class=move || {
                        format!(
                            "border w-80 h-20 rounded-full flex px-8 py-6 {}",
                            if is_selected("Hotel") {
                                "border-blue-500 text-blue-500"
                            } else {
                                "border-gray-200 text-black"
                            },
                        )
                    }
                    on:click=move |_| toggle_selection("Hotel")
                >
                    <Icon icon=icondata::RiHotelBedMapLine class="text-xl" />
                    <span class="ml-2">Hotel</span>
                </button>
            </div>
        </div>
    }
}

#[component]
fn Ratings() -> impl IntoView {
    view! {
        <div class="flex px-12">
            <div class="w-1/2 flex flex-col space-y-2">
                <label class="flex items-center space-x-2 cursor-pointer py-2">
                    <input type="checkbox" class="form-checkbox" />
                    <span>5 star (95)</span>
                </label>
                <label class="flex items-center space-x-2 cursor-pointer py-2">
                    <input type="checkbox" class="form-checkbox" />
                    <span>3 star (1,639)</span>
                </label>
                <label class="flex items-center space-x-2 cursor-pointer py-2">
                    <input type="checkbox" class="form-checkbox" />
                    <span>1 star (71)</span>
                </label>
            </div>
            <div class="w-1/2 flex flex-col space-y-2">
                <label class="flex items-center space-x-2 cursor-pointer py-2">
                    <input type="checkbox" class="form-checkbox" />
                    <span>4 star (249)</span>
                </label>
                <label class="flex items-center space-x-2 cursor-pointer py-2">
                    <input type="checkbox" class="form-checkbox" />
                    <span>2 star (375)</span>
                </label>
                <label class="flex items-center space-x-2 cursor-pointer py-2">
                    <input type="checkbox" class="form-checkbox" />
                    <span>Unrated (647)</span>
                </label>
            </div>
        </div>
    }
}
