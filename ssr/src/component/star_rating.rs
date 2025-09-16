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
    #[prop(optional)] selection: Option<RwSignal<Option<u8>>>,
    #[prop(optional)] on_select: Option<Callback<Option<u8>>>,
) -> impl IntoView {
    let selection = selection.unwrap_or_else(|| create_rw_signal(None));

    view! {
        <div class="rounded-xl border border-slate-200 bg-white p-4 shadow-sm">
            <div class="flex flex-col gap-1">
                <h3 class="text-sm font-semibold uppercase tracking-wide text-slate-800">
                    "Star Rating"
                </h3>
                <p class="text-xs text-slate-500">
                    "Select a minimum hotel rating. Tap the same rating again to clear."
                </p>
            </div>

            <div
                class="mt-4 flex flex-col gap-2"
                role="group"
                aria-label="Filter hotels by minimum star rating"
            >
                {((1..=MAX_STARS).rev())
                    .map(|rating| {
                        let selection_for_state = selection.clone();
                        let selection_for_click = selection.clone();
                        let on_select = on_select.clone();
                        let is_selected = Signal::derive(move || {
                            selection_for_state
                                .get()
                                .map_or(false, |value| value == rating)
                        });
                        let label_text = format!(
                            "{rating}+ star{}",
                            if rating > 1 { "s" } else { "" }
                        );

                        view! {
                            <button
                                type="button"
                                class="group w-full rounded-lg border px-3 py-2 text-left transition-colors duration-150 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 focus-visible:ring-offset-2 focus-visible:ring-offset-white"
                                class=("border-blue-600 bg-blue-600 text-white shadow-sm", move || {
                                    is_selected.get()
                                })
                                class=(
                                    "border-slate-200 bg-white text-slate-700 hover:border-blue-400 hover:bg-blue-50",
                                    move || !is_selected.get()
                                )
                                aria-pressed=move || is_selected.get()
                                on:click=move |_| {
                                    let next = {
                                        let current = selection_for_click.get_untracked();
                                        if current == Some(rating) {
                                            None
                                        } else {
                                            Some(rating)
                                        }
                                    };
                                    selection_for_click.set(next);
                                    if let Some(cb) = on_select.clone() {
                                        leptos::Callable::call(&cb, next);
                                    }
                                }
                            >
                                <div class="flex items-center justify-between gap-3">
                                    <div class="flex items-center gap-2">
                                        <div class="flex items-center gap-1">
                                            {(1..=MAX_STARS)
                                                .map(|position| {
                                                    let icon = if position <= rating {
                                                        icondata::BiStarSolid
                                                    } else {
                                                        icondata::BiStarRegular
                                                    };
                                                    let is_selected_signal = is_selected.clone();
                                                    view! {
                                                        <span
                                                            class="inline-flex h-4 w-4 items-center justify-center transition-colors duration-150"
                                                            class=(
                                                                "text-white",
                                                                move || {
                                                                    is_selected_signal.get()
                                                                        && position <= rating
                                                                },
                                                            )
                                                            class=(
                                                                "text-blue-200",
                                                                move || {
                                                                    is_selected_signal.get()
                                                                        && position > rating
                                                                },
                                                            )
                                                            class=(
                                                                "text-blue-500 group-hover:text-blue-600",
                                                                move || {
                                                                    !is_selected_signal.get()
                                                                        && position <= rating
                                                                },
                                                            )
                                                            class=(
                                                                "text-slate-300 group-hover:text-blue-400",
                                                                move || {
                                                                    !is_selected_signal.get()
                                                                        && position > rating
                                                                },
                                                            )
                                                        >
                                                            <Icon icon=icon class="h-4 w-4" />
                                                        </span>
                                                    }
                                                })
                                                .collect::<Vec<_>>()
                                            }
                                        </div>
                                        <span class="text-sm font-medium">{label_text}</span>
                                    </div>
                                    <span class="text-xs uppercase tracking-wide text-slate-400">
                                        "Minimum"
                                    </span>
                                </div>
                            </button>
                        }
                    })
                    .collect::<Vec<_>>()
                }
            </div>

            <p
                class="mt-3 text-xs transition-colors duration-150"
                class=("text-blue-600 font-semibold", move || selection.get().is_some())
                class=("text-slate-400", move || selection.get().is_none())
            >
                {move || match selection.get() {
                    Some(rating) => format!(
                        "Filtering by {rating}+ star{}",
                        if rating > 1 { "s" } else { "" }
                    ),
                    None => "No rating filter applied".to_string(),
                }}
            </p>
        </div>
    }
}
