use leptos::*;
use leptos_icons::Icon;
use leptos::logging::log;

#[component]
pub fn StarRating<T>(rating: T) -> impl IntoView
where 
T:  Fn() -> u8 + 'static
 {
    let derived_rating = Signal::derive(move || rating());
    create_effect(move |_| {
        log!("derived_rating: {}", derived_rating.get());
    });

    view! {
        <div class="flex items-center space-x-2">
            <div class="flex items-center space-x-0.5 ">
                {move || (0..5)
                    .map(|i| {
                        let rating_clone =  derived_rating.get();
                        let icon = {move || {
                            if i <  rating_clone {
                                icondata::BiStarSolid
                            } else {
                                icondata::BiStarRegular
                            }
                        }};
                        {move || view! { <Icon class="w-3 h-3 text-blue-500" icon=icon() /> }}
                    })
                    .collect::<Vec<_>>()}
            </div>
             <span class="inline-block text-xs text-blue-500">{derived_rating}.0</span> 
        </div>
    }
}
