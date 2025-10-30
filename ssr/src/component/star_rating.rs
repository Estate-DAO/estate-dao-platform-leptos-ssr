// use leptos::logging::log;
use crate::log;
use leptos::prelude::*;
use leptos_icons::Icon;

const MAX_STARS: u8 = 5;

#[component]
pub fn StarRating<T>(rating: T) -> impl IntoView
where
    T: Fn() -> u8 + 'static + Send + Sync,
{
    let derived_rating = Signal::derive(move || rating());
    Effect::new(move |_| {
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
                            let icon_class = move || {
                                if i < rating_clone {
                                    "w-3 h-3 text-yellow-400"
                                } else {
                                    "w-3 h-3 text-gray-300"
                                }
                            };
                            { move || view! { <div class=icon_class()><Icon icon=icon() /></div> } }
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
                <h3 class="text-sm font-medium text-gray-900">
                    "Rating"
                </h3>
            </div>

            <div
                class="mt-4 flex flex-row flex-wrap gap-1.5"
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
                        let is_selected_for_label = is_selected.clone();
                        let label_class = Signal::derive(move || {
                            if is_selected_for_label.get() {
                                "text-xs font-semibold transition-colors duration-150 text-white"
                            } else {
                                "text-xs font-semibold transition-colors duration-150 text-gray-700"
                            }
                        });
                        // <!-- Create a single signal for button class combining all conditional classes -->
                        let button_class = Signal::derive(move || {
                            let base_classes = "group relative flex items-center gap-1 rounded-lg border px-2.5 py-1.5 transition-all duration-150 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 focus-visible:ring-offset-2";
                            if is_selected.get() {
                                format!("{} border-blue-600 bg-blue-600 text-white shadow-sm", base_classes)
                            } else {
                                format!("{} border-gray-200 bg-white text-gray-700 hover:border-gray-300 hover:bg-gray-50", base_classes)
                            }
                        });

                        view! {
                            <button
                                type="button"
                                class={move || button_class.get()}
                                aria-pressed=move || is_selected.get()
                                on:click=move |_| {
                                    let next = if current_value() == Some(rating) {
                                        None
                                    } else {
                                        Some(rating)
                                    };
                                    on_select.run(next);
                                }
                            >
                                <span class={move || {
                                    let base_span_classes = "inline-flex items-center gap-0.5";
                                    format!("{} {}", base_span_classes, label_class.get())
                                }}>
                                    {rating}
                                    <div class=move || {
                                        if is_selected.get() {
                                            "h-3 w-3 transition-colors duration-150 text-white"
                                        } else {
                                            "h-3 w-3 transition-colors duration-150 text-yellow-400"
                                        }
                                    }>
                                        <Icon icon=icondata::BiStarSolid />
                                    </div>
                                </span>

                            </button>
                        }
                    })
                    .collect::<Vec<_>>()
                }
            </div>
        </div>
    }
}
