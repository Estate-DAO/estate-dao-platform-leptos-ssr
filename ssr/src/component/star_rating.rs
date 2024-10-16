use leptos::*;
use leptos_icons::Icon;

#[component]
pub fn StarRating(#[prop(into)] rating: u8) -> impl IntoView {
    view! {
        <div class="flex items-center space-x-2">
            <div class="flex items-center space-x-0.5 ">
                {(0..5)
                    .map(|i| {
                        let icon = {
                            if i < rating {
                                icondata::BiStarSolid
                            } else {
                                icondata::BiStarRegular
                            }
                        };
                        view! { <Icon class="w-3 h-3 text-blue-500" icon=icon /> }
                    })
                    .collect::<Vec<_>>()}
            </div>
            <span class="inline-block text-xs text-blue-500">{rating}.0</span>
        </div>
    }
}
