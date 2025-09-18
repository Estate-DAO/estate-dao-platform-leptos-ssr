use leptos::*;

#[component]
pub fn ImageLightbox(
    images: Vec<String>,
    #[prop(optional)] initial_index: usize,
    #[prop(optional)] loop_images: bool,
    on_close: Callback<()>,
) -> impl IntoView {
    let (current_index, set_current_index) = create_signal(initial_index);
    let image_len = images.len();
    let current_image = move || images.get(current_index.get()).cloned();

    // Move to next image
    let next = move |_| {
        set_current_index.update(|i| {
            if *i + 1 < image_len {
                *i += 1;
            } else if loop_images {
                *i = 0;
            }
        });
    };

    // Move to previous image
    let prev = move |_| {
        set_current_index.update(|i| {
            if *i > 0 {
                *i -= 1;
            } else if loop_images {
                *i = image_len - 1;
            }
        });
    };

    view! {
        <div class="fixed inset-0 bg-black bg-opacity-90 z-50 flex items-center justify-center">
            <button
                class="absolute top-4 right-4 text-white text-3xl"
                on:click=move |_| on_close.call(())
            >
                "×"
            </button>

            <button
                class="absolute left-4 text-white text-4xl p-2 bg-black bg-opacity-40 rounded-full hover:bg-opacity-70 disabled:opacity-30"
                on:click=prev
                disabled=move || !loop_images && current_index.get() == 0
            >
                "‹"
            </button>

            <img
                src=current_image
                alt="Image"
                class="max-h-[90vh] max-w-[90vw] rounded-xl shadow-lg"
            />

            <button
                class="absolute right-4 text-white text-4xl p-2 bg-black bg-opacity-40 rounded-full hover:bg-opacity-70 disabled:opacity-30"
                on:click=next
                disabled=move || !loop_images && current_index.get() == image_len - 1
            >
                "›"
            </button>
        </div>
    }
}
