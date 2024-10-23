use leptos::*;

/// Spinner with constant size
#[component]
pub fn Spinner() -> impl IntoView {
    view! {
        <div class="animate-spin border-solid rounded-full border-t-transparent border-white border-8 h-32 w-32"></div>
    }
}

/// Spinner that fits container
#[component]
pub fn SpinnerFit() -> impl IntoView {
    view! {
        <div class="animate-spin border-solid rounded-full border-t-transparent border-slate-600 border-8 w-full h-full" />
    }
}

/// Spinner that takes up the whole screen with black background
#[component]
pub fn FullScreenSpinner() -> impl IntoView {
    view! {
        <div class="h-screen w-screen grid grid-cols-1 bg-black justify-items-center place-content-center">
            <Spinner />
        </div>
    }
}


/// Spinner with constant size
#[component]
pub fn SpinnerGray() -> impl IntoView {
    view! {
        <div class="animate-spin border-solid rounded-full border-t-transparent border-slate-600 border-8 h-32 w-32"></div>
    }
}

/// Spinner that sits in center of screen, with no background
#[component]
pub fn FullScreenSpinnerGray() -> impl IntoView {
    view! {
        <div class="h-1/2 w-screen grid grid-cols-1 bg-blue justify-items-center place-content-center">
            <SpinnerGray />
        </div>
    }
}
