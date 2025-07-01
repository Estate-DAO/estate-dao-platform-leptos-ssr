/// Browser-specific utilities that are only available in hydrated client environments
use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "hydrate")] {
        /// Scrolls the window to the top of the page
        pub fn scroll_to_top() {
            if let Some(window) = web_sys::window() {
                window.scroll_to_with_x_and_y(0.0, 0.0);
            }
        }
    } else {
        /// No-op version for SSR - does nothing
        pub fn scroll_to_top() {
            // No-op on server side
        }
    }
}
