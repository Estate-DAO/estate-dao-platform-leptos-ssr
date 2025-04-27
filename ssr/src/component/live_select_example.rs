use crate::component::LiveSelect;
use leptos::*; // Assuming LiveSelect is defined in the same crate

#[component]
pub fn LiveSelectExample() -> impl IntoView {
    // Define options
    let options = vec![
        "Apple".to_string(),
        "Banana".to_string(),
        "Cherry".to_string(),
        "Date".to_string(),
        "Elderberry".to_string(),
    ];

    // Create signal for selected value
    let (selected, set_selected) = create_signal(None::<String>);

    // Define set_value callback
    let set_value = Callback::new(move |fruit: String| set_selected.set(Some(fruit)));

    // Define label_fn and value_fn
    let label_fn = Callback::new(|fruit: String| fruit.clone());
    let value_fn = Callback::new(|fruit: String| fruit.clone());

    // Placeholder
    let placeholder = "Select a fruit".to_string();

    // id
    let id = "fruit-select".to_string();

    // Render
    view! {
        <div>
            <h2>"Live Select Example"</h2>
            <LiveSelect
                options=options
                value=selected
                set_value=set_value
                label_fn=label_fn
                value_fn=value_fn
                placeholder=placeholder
                id=id
                debug=true
            />
            {move || {
                match selected.get() {
                    Some(fruit) => view! { <p>"Selected: " {fruit}</p> }.into_view(),
                    None => view! { <p>"No fruit selected"</p> }.into_view(),
                }
            }}
        </div>
    }
}
