use leptos::*;

fn preload_image_sources(
    images: &[String],
    current_index: usize,
    loop_images: bool,
) -> Vec<String> {
    if images.is_empty() {
        return Vec::new();
    }

    let image_len = images.len();
    let mut indexes = vec![current_index];

    for step in [1_usize, 2_usize] {
        let next_index = current_index + step;
        if next_index < image_len {
            indexes.push(next_index);
        } else if loop_images {
            indexes.push((current_index + step) % image_len);
        }

        if current_index >= step {
            indexes.push(current_index - step);
        } else if loop_images {
            indexes.push((image_len + current_index - step) % image_len);
        }
    }

    indexes.sort_unstable();
    indexes.dedup();

    indexes
        .into_iter()
        .filter_map(|index| images.get(index).cloned())
        .collect()
}

#[component]
pub fn ImageLightbox(
    images: Vec<String>,
    #[prop(optional)] initial_index: usize,
    #[prop(optional)] loop_images: bool,
    on_close: Callback<()>,
) -> impl IntoView {
    let images = store_value(images);
    let (current_index, set_current_index) = create_signal(initial_index);
    let image_len = images.with_value(|images| images.len());
    let current_image =
        move || images.with_value(|images| images.get(current_index.get()).cloned());
    let preload_images = move || {
        images.with_value(|images| preload_image_sources(images, current_index.get(), loop_images))
    };

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
        <div class="fixed inset-0 z-[1105] isolate flex items-center justify-center bg-black/95 px-4 py-6">
            <button
                type="button"
                aria-label="Close image viewer"
                class="absolute right-4 top-4 z-[1106] grid h-11 w-11 place-items-center rounded-full bg-black/60 text-white shadow-lg ring-1 ring-white/20 backdrop-blur-sm transition hover:bg-black/75"
                style="top: calc(env(safe-area-inset-top) + 16px); right: calc(env(safe-area-inset-right) + 16px);"
                on:click=move |_| on_close(())
            >
                <span aria-hidden="true" class="text-2xl leading-none">
                    "×"
                </span>
            </button>

            <button
                type="button"
                aria-label="Previous image"
                class="absolute left-4 top-1/2 z-[1106] -translate-y-1/2 rounded-full bg-black/50 p-2 text-4xl text-white shadow-lg ring-1 ring-white/20 backdrop-blur-sm transition hover:bg-black/70 disabled:opacity-30"
                on:click=prev
                disabled=move || !loop_images && current_index.get() == 0
            >
                "‹"
            </button>

            <img
                src=current_image
                alt="Image"
                loading="eager"
                decoding="async"
                fetchpriority="high"
                class="max-h-[85vh] max-w-full rounded-xl shadow-2xl md:max-w-[90vw]"
            />

            <button
                type="button"
                aria-label="Next image"
                class="absolute right-4 top-1/2 z-[1106] -translate-y-1/2 rounded-full bg-black/50 p-2 text-4xl text-white shadow-lg ring-1 ring-white/20 backdrop-blur-sm transition hover:bg-black/70 disabled:opacity-30"
                on:click=next
                disabled=move || !loop_images && current_index.get() == image_len - 1
            >
                "›"
            </button>

            <div aria-hidden="true" class="pointer-events-none absolute h-0 w-0 overflow-hidden opacity-0">
                <For
                    each=preload_images
                    key=|src| src.clone()
                    let:src
                >
                    <img
                        src=src
                        alt=""
                        loading="eager"
                        decoding="async"
                        class="h-0 w-0"
                    />
                </For>
            </div>
        </div>
    }
}
