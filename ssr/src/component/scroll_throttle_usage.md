# Usage of `use_scroll` and `use_throttle_fn` in DateTimeRangePickerCustom

This document explains how to integrate scroll-based month navigation using leptos_use's hooks.

## 1. Enable Features
In your `Cargo.toml`, enable the required crate features:

```toml
leptos-use = { version = "0.13.11", features = ["use_scroll", "use_throttle_fn"] }
```

## 2. Imports
```rust
use leptos::*;
use leptos_use::{use_scroll, UseScrollReturn, use_throttle_fn};
```

## 3. Bind a Scrollable Element
1. Create a `NodeRef` for the scroll container (must match the HTML tag):
   ```rust
   let calendar_ref = create_node_ref::<leptos::html::Div>();
   ```
2. Attach it to your container in the `view!` macro:
   ```html
   <div _ref=calendar_ref class="overflow-y-auto">
     <!-- calendar cells here -->
   </div>
   ```

## 4. Read Scroll Position
Call `use_scroll` with your `NodeRef`:
```rust
let UseScrollReturn { y, .. } = use_scroll(calendar_ref);
```
- `y.get()` yields the current `scrollTop` (in pixels) as an `f64`.

## 5. Throttle the Scroll Handler
1. Track the last position:
   ```rust
   let last_scroll_position = create_rw_signal(0.0);
   ```
2. Create a throttled callback that runs at most once per interval (e.g., 300 ms):
   ```rust
   let throttled_scroll = use_throttle_fn(
       move || {
           let current_y = y.get();
           let delta = current_y - last_scroll_position.get();
           if delta.abs() > 50.0 {
               // Update month here:
               // e.g., set_initial_date(next_or_prev_month(delta));
               last_scroll_position.set(current_y);
           }
       },
       300.0,
   );
   ```
3. Run the throttled callback whenever `y` changes:
   ```rust
   create_effect(move |_| {
       let _ = y.get(); // ensure reactivity
       throttled_scroll();
   });
   ```

## 6. Complete Example Snippet
```rust
let calendar_ref = create_node_ref::<leptos::html::Div>();
let UseScrollReturn { y, .. } = use_scroll(calendar_ref);
let last_scroll_position = create_rw_signal(0.0);
let throttled_scroll = use_throttle_fn(
    move || {
        let current_y = y.get();
        let delta = current_y - last_scroll_position.get();
        if delta.abs() > 50.0 {
            if delta < 0.0 {
                // scroll up → next month
                set_initial_date(next_date(...));
            } else {
                // scroll down → prev month
                set_initial_date(prev_date(...));
            }
            last_scroll_position.set(current_y);
        }
    },
    300.0,
);
create_effect(move |_| {
    let _ = y.get();
    throttled_scroll();
});

view! {
    <div _ref=calendar_ref class="overflow-y-auto">
        <!-- calendar grids -->
    </div>
}
```

---

**Notes:**
- `use_scroll` listens to `scroll` events on the referenced element.
- `use_throttle_fn` ensures your handler runs at most once per specified interval, reducing rapid month flips.
- Adjust the threshold (50 px) and interval (300 ms) to fine-tune UX.
