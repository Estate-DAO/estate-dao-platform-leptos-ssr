// use leptos::logging::log;
use crate::log;
use leptos::*;
use leptos_icons::Icon;

const MAX_STARS: u8 = 5;

#[component]
pub fn StarRating<T>(rating: T) -> impl IntoView
where
    T: Fn() -> u8 + 'static,
{
    let derived_rating = Signal::derive(move || rating());
    create_effect(move |_| {
        // log!("derived_rating: {}", derived_rating.get());
    });

    view! {
        <div class="flex items-center space-x-2">
            <div class="flex items-center space-x-0.5 ">
                {move || {
                    (0..MAX_STARS)
                        .map(|i| {
                            let rating_clone = derived_rating.get();
                            let icon = {
                                move || {
                                    if i < rating_clone {
                                        icondata::BiStarSolid
                                    } else {
                                        icondata::BiStarRegular
                                    }
                                }
                            };
                            { move || view! { <Icon class="w-3 h-3 text-blue-500" icon=icon() /> } }
                        })
                        .collect::<Vec<_>>()
                }}
            </div>
            <span class="inline-block text-xs text-blue-500">{derived_rating}.0</span>
        </div>
    }
}

#[component]
pub fn StarRatingFilter(
    #[prop(into)] value: Signal<Option<u8>>,
    on_select: Callback<Option<u8>>,
) -> impl IntoView {
    view! {
        <div class="">
            <div class="flex flex-col gap-1">
                <h3 class="text-base font-medium text-gray-900">
                    "Rating"
                </h3>
            </div>

            <div
                class="mt-4 flex flex-row flex-wrap gap-2"
                role="group"
                aria-label="Filter hotels by minimum star rating"
            >
                {(1..=MAX_STARS)
                    .map(|rating| {
                        let on_select = on_select.clone();
                        let current_value = value.clone();
                        let is_selected = Signal::derive(move || {
                            current_value().map_or(false, |selected| selected == rating)
                        });

                        view! {
                            <button
                                type="button"
                                class="group relative flex items-center gap-1.5 rounded-lg border px-2 py-1 transition-all duration-150 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 focus-visible:ring-offset-2"
                                class=("border-blue-500 bg-blue-50", move || {
                                    is_selected.get()
                                })
                                class=("border-gray-200 bg-white hover:border-gray-300 hover:bg-gray-50", move || {
                                    !is_selected.get()
                                })
                                aria-pressed=move || is_selected.get()
                                on:click=move |_| {
                                    let next = if current_value() == Some(rating) {
                                        None
                                    } else {
                                        Some(rating)
                                    };
                                    leptos::Callable::call(&on_select, next);
                                }
                            >
                                <span
                                    class="text-sm font-medium"
                                    class=("text-blue-700", move || is_selected.get())
                                    class=("text-gray-700", move || !is_selected.get())
                                >
                                    {rating}
                                </span>
                                <Icon
                                    icon=icondata::BiStarSolid
                                    class="h-4 w-4 text-yellow-400"
                                />
                            </button>
                        }
                    })
                    .collect::<Vec<_>>()
                }
            </div>
        </div>
    }
}
